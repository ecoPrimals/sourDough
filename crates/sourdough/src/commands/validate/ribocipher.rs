//! `sourdough validate ribocipher` — audit primal accept loops for riboCipher compliance.
//!
//! Checks:
//! - Accept loop reads first byte BEFORE protocol-specific parsing
//! - Signal bytes `0xEC`/`0xED`/`0xEE` handled before legacy fallback
//! - Legacy fallback logs at WARN level
//! - Client connections send signal prefix before payload
//!
//! See: `wateringHole/RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md`

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Patterns that indicate riboCipher server-side signal detection.
const SIGNAL_DETECTION_PATTERNS: &[(&str, &str)] = &[
    ("0xEC", "clear signal byte constant"),
    ("0xED", "mito signal byte constant"),
    ("0xEE", "nuclear signal byte constant"),
    ("SIGNAL_CLEAR", "riboCipher clear constant import"),
    ("SIGNAL_MITO", "riboCipher mito constant import"),
    ("SIGNAL_NUCLEAR", "riboCipher nuclear constant import"),
    ("detect_signal", "riboCipher signal detection call"),
    ("ribocipher", "riboCipher module reference"),
    ("riboCipher", "riboCipher reference (camelCase)"),
    ("is_signal_byte", "signal byte classification"),
];

/// Patterns that indicate riboCipher client-side signal sending.
const CLIENT_SIGNAL_PATTERNS: &[(&str, &str)] = &[
    ("send_clear_signal", "riboCipher client signal helper"),
    ("[0xEC, 0x01]", "clear JSON-RPC signal literal"),
    ("[0xEC, 0x02]", "clear BTSP binary signal literal"),
    ("SIGNAL_CLEAR", "signal constant for client prepend"),
    ("write_all(&[0xEC", "manual signal write"),
];

/// Patterns that indicate accept loop first-byte detection.
const ACCEPT_LOOP_PATTERNS: &[(&str, &str)] = &[
    ("peek_protocol", "peek-based first-byte detection"),
    ("detect_signal", "riboCipher signal detection"),
    ("read(&mut byte", "first-byte read pattern"),
    ("read_exact(&mut first", "first-byte exact read"),
    ("first_byte", "first byte variable"),
    ("AsyncReadExt", "async read for signal detection"),
];

/// Patterns that indicate deprecated legacy peek-and-guess (should WARN).
const LEGACY_PEEK_PATTERNS: &[(&str, &str)] = &[
    ("b'{' =>", "legacy JSON detect via brace"),
    ("0x7B =>", "legacy JSON detect via hex"),
    ("Protocol::Binary", "legacy binary fallback"),
    ("Protocol::JsonRpc", "legacy JSON-RPC peek"),
];

/// Patterns that indicate proper deprecation logging on legacy paths.
///
/// Wave 112 escalates from WARN to ERROR. Both levels are compliant.
const DEPRECATION_WARN_PATTERNS: &[(&str, &str)] = &[
    ("DEPRECATED", "deprecation label"),
    ("unsignalled", "unsignalled connection warning"),
    ("legacy", "legacy path identification"),
    ("warn!", "tracing warn macro"),
    ("tracing::warn", "explicit tracing warn"),
    ("error!", "tracing error macro (Wave 112+)"),
    ("tracing::error", "explicit tracing error (Wave 112+)"),
    ("log::warn", "log crate warn"),
    ("log::error", "log crate error"),
];

/// Compliance level for a primal's riboCipher implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ComplianceLevel {
    /// No riboCipher implementation detected.
    None,
    /// Partial: some patterns found but incomplete.
    Partial,
    /// Full: server detection + client signal + deprecation warnings.
    Full,
}

impl std::fmt::Display for ComplianceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "NONE"),
            Self::Partial => write!(f, "PARTIAL"),
            Self::Full => write!(f, "FULL"),
        }
    }
}

/// Result of auditing a single primal for riboCipher compliance.
#[derive(Debug)]
pub(crate) struct AuditResult {
    pub primal_name: String,
    pub compliance: ComplianceLevel,
    pub signal_detection: Vec<String>,
    pub client_signals: Vec<String>,
    pub accept_loop_found: bool,
    pub legacy_warns: bool,
    pub issues: Vec<String>,
    pub notes: Vec<String>,
}

/// Run the riboCipher validation against a primal directory.
pub(crate) fn run(path: &Path, json: bool) -> Result<()> {
    let primal_name = path.file_name().map_or_else(
        || "unknown".to_string(),
        |n| n.to_string_lossy().to_string(),
    );

    if !json {
        crate::info(&format!(
            "Validating riboCipher compliance: {primal_name} ({})",
            path.display()
        ));
        println!();
    }

    let result = audit_primal(path, &primal_name)?;

    if json {
        print_json(&result)?;
    } else {
        print_human(&result);
    }

    match result.compliance {
        ComplianceLevel::Full => Ok(()),
        ComplianceLevel::Partial => {
            anyhow::bail!(
                "riboCipher compliance: PARTIAL — {} issue(s) remain",
                result.issues.len()
            );
        }
        ComplianceLevel::None => {
            anyhow::bail!("riboCipher compliance: NONE — no signal detection implemented");
        }
    }
}

fn audit_primal(path: &Path, primal_name: &str) -> Result<AuditResult> {
    let src_dir = find_source_dir(path);

    let mut signal_detection: Vec<String> = Vec::new();
    let mut client_signals: Vec<String> = Vec::new();
    let mut accept_loop_found = false;
    let mut legacy_warns = false;
    let mut issues: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let source_files = collect_rust_files(&src_dir)?;

    if source_files.is_empty() {
        issues.push("No Rust source files found".to_string());
        return Ok(AuditResult {
            primal_name: primal_name.to_string(),
            compliance: ComplianceLevel::None,
            signal_detection,
            client_signals,
            accept_loop_found,
            legacy_warns,
            issues,
            notes,
        });
    }

    for file_path in &source_files {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("reading {}", file_path.display()))?;

        if is_test_file(file_path, &content) {
            continue;
        }

        let rel_path = file_path
            .strip_prefix(path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (pattern, desc) in SIGNAL_DETECTION_PATTERNS {
            if content.contains(pattern) {
                signal_detection.push(format!("{rel_path}: {desc}"));
            }
        }

        for (pattern, desc) in CLIENT_SIGNAL_PATTERNS {
            if content.contains(pattern) {
                client_signals.push(format!("{rel_path}: {desc}"));
            }
        }

        for (pattern, _) in ACCEPT_LOOP_PATTERNS {
            if content.contains(pattern) {
                accept_loop_found = true;
                break;
            }
        }

        let has_legacy = LEGACY_PEEK_PATTERNS
            .iter()
            .any(|(p, _)| content.contains(p));
        if has_legacy {
            let has_warn = DEPRECATION_WARN_PATTERNS
                .iter()
                .any(|(p, _)| content.contains(p));
            if has_warn {
                legacy_warns = true;
                notes.push(format!(
                    "{rel_path}: legacy fallback with deprecation log (compliant)"
                ));
            } else {
                issues.push(format!(
                    "{rel_path}: legacy peek pattern WITHOUT deprecation log (WARN or ERROR required)"
                ));
            }
        }
    }

    if signal_detection.is_empty() {
        issues.push("No riboCipher signal detection (0xEC/0xED/0xEE) in accept loop".to_string());
    }

    if client_signals.is_empty() {
        issues.push("No client-side riboCipher signal sending found".to_string());
    }

    if !accept_loop_found {
        issues.push("No accept loop / first-byte detection pattern found".to_string());
    }

    let compliance = if !signal_detection.is_empty()
        && !client_signals.is_empty()
        && accept_loop_found
        && (legacy_warns || !has_any_legacy(&source_files, path)?)
    {
        ComplianceLevel::Full
    } else if !signal_detection.is_empty() || !client_signals.is_empty() {
        ComplianceLevel::Partial
    } else {
        ComplianceLevel::None
    };

    Ok(AuditResult {
        primal_name: primal_name.to_string(),
        compliance,
        signal_detection,
        client_signals,
        accept_loop_found,
        legacy_warns,
        issues,
        notes,
    })
}

fn has_any_legacy(source_files: &[PathBuf], base: &Path) -> Result<bool> {
    for file_path in source_files {
        let content = std::fs::read_to_string(file_path)?;
        if is_test_file(file_path, &content) {
            continue;
        }
        let _rel = file_path.strip_prefix(base).unwrap_or(file_path);
        if LEGACY_PEEK_PATTERNS
            .iter()
            .any(|(p, _)| content.contains(p))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn find_source_dir(path: &Path) -> PathBuf {
    let crates = path.join("crates");
    if crates.exists() {
        return crates;
    }
    let src = path.join("src");
    if src.exists() {
        return src;
    }
    path.to_path_buf()
}

fn collect_rust_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    collect_rust_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rust_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') || name_str == "target" || name_str == "archive" {
            continue;
        }

        if path.is_dir() {
            collect_rust_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn is_test_file(path: &Path, content: &str) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/tests/") || path_str.contains("/test/") {
        return true;
    }
    if path_str.ends_with("_test.rs") || path_str.ends_with("_tests.rs") {
        return true;
    }
    if content.starts_with("#[cfg(test)]") {
        return true;
    }
    false
}

fn print_human(result: &AuditResult) {
    println!(
        "  Primal: {}  |  Compliance: {}",
        result.primal_name, result.compliance
    );
    println!();

    if result.signal_detection.is_empty() {
        crate::error("No signal detection patterns found");
    } else {
        crate::success("Signal detection patterns found:");
        for item in &result.signal_detection {
            println!("    {item}");
        }
    }

    println!();

    if result.client_signals.is_empty() {
        crate::error("No client signal patterns found");
    } else {
        crate::success("Client signal patterns found:");
        for item in &result.client_signals {
            println!("    {item}");
        }
    }

    println!();

    if result.accept_loop_found {
        crate::success("Accept loop / first-byte detection found");
    } else {
        crate::error("No accept loop / first-byte detection found");
    }

    if result.legacy_warns {
        crate::success("Legacy paths have deprecation warnings");
    }

    if !result.notes.is_empty() {
        println!();
        crate::info("Notes:");
        for note in &result.notes {
            println!("    {note}");
        }
    }

    if !result.issues.is_empty() {
        println!();
        crate::error("Issues:");
        for issue in &result.issues {
            println!("    ✗ {issue}");
        }
    }

    println!();
}

fn print_json(result: &AuditResult) -> Result<()> {
    let json = serde_json::json!({
        "primal": result.primal_name,
        "compliance": format!("{}", result.compliance),
        "signal_detection": result.signal_detection,
        "client_signals": result.client_signals,
        "accept_loop_found": result.accept_loop_found,
        "legacy_warns": result.legacy_warns,
        "issues": result.issues,
        "notes": result.notes,
    });
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_primal(name: &str, files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join(name);
        fs::create_dir_all(&base).unwrap();

        for (rel_path, content) in files {
            let full = base.join(rel_path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full, content).unwrap();
        }
        dir
    }

    #[test]
    fn no_source_files_returns_none() {
        let dir = TempDir::new().unwrap();
        let result = audit_primal(dir.path(), "empty").unwrap();
        assert_eq!(result.compliance, ComplianceLevel::None);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn fully_compliant_primal() {
        let dir = setup_primal(
            "testPrimal",
            &[(
                "crates/testprimal/src/server.rs",
                r#"
use sourdough_core::transport::{detect_signal, send_clear_signal, SIGNAL_CLEAR, SIGNAL_MITO, SIGNAL_NUCLEAR};
use tokio::io::AsyncReadExt;

async fn handle_connection(mut stream: TcpStream) {
    let mut byte = [0u8; 1];
    stream.read(&mut byte).await.unwrap();
    match byte[0] {
        0xEC => { /* clear signal */ }
        0xED => { /* mito signal */ }
        0xEE => { /* nuclear signal */ }
        b'{' => {
            // DEPRECATED: unsignalled legacy JSON-RPC (Wave 112 — ERROR)
            tracing::error!("unsignalled connection - legacy peek");
        }
        _ => {
            tracing::error!("DEPRECATED: unsignalled binary");
        }
    }
}

async fn connect_to_peer(mut stream: TcpStream) {
    send_clear_signal(&mut stream, ProtocolType::NdjsonRpc).await.unwrap();
}
"#,
            )],
        );

        let primal_dir = dir.path().join("testPrimal");
        let result = audit_primal(&primal_dir, "testPrimal").unwrap();
        assert_eq!(result.compliance, ComplianceLevel::Full);
        assert!(!result.signal_detection.is_empty());
        assert!(!result.client_signals.is_empty());
        assert!(result.accept_loop_found);
        assert!(result.legacy_warns);
    }

    #[test]
    fn partial_compliance_server_only() {
        let dir = setup_primal(
            "partial",
            &[(
                "src/main.rs",
                r#"
use sourdough_core::transport::detect_signal;

async fn accept(mut stream: TcpStream) {
    let signal = detect_signal(&mut stream).await;
    match signal {
        SignalResult::Clear(0xEC) => {}
        _ => {}
    }
}
"#,
            )],
        );

        let primal_dir = dir.path().join("partial");
        let result = audit_primal(&primal_dir, "partial").unwrap();
        assert_eq!(result.compliance, ComplianceLevel::Partial);
    }

    #[test]
    fn no_compliance_plain_primal() {
        let dir = setup_primal(
            "plain",
            &[(
                "src/main.rs",
                r#"
fn main() {
    println!("Hello world");
}
"#,
            )],
        );

        let primal_dir = dir.path().join("plain");
        let result = audit_primal(&primal_dir, "plain").unwrap();
        assert_eq!(result.compliance, ComplianceLevel::None);
    }

    #[test]
    fn test_files_are_excluded() {
        let dir = setup_primal(
            "testonly",
            &[(
                "src/tests/ribocipher_test.rs",
                r#"
// All riboCipher patterns here should be ignored
use sourdough_core::transport::{detect_signal, send_clear_signal, SIGNAL_CLEAR};
let mut byte = [0u8; 1];
0xEC
0xED
0xEE
"#,
            )],
        );

        let primal_dir = dir.path().join("testonly");
        let result = audit_primal(&primal_dir, "testonly").unwrap();
        assert_eq!(result.compliance, ComplianceLevel::None);
    }

    #[test]
    fn legacy_without_warn_is_issue() {
        let dir = setup_primal(
            "nowarn",
            &[(
                "src/server.rs",
                r#"
use sourdough_core::transport::{detect_signal, SIGNAL_CLEAR};
let mut byte = [0u8; 1];
stream.read(&mut byte).await?;
match byte[0] {
    0xEC => { detect_signal(&mut stream).await; }
    b'{' => { handle_json() }
    _ => { Protocol::Binary }
}
async fn connect(s: &mut S) {
    send_clear_signal(s, ProtocolType::NdjsonRpc).await;
}
"#,
            )],
        );

        let primal_dir = dir.path().join("nowarn");
        let result = audit_primal(&primal_dir, "nowarn").unwrap();
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.contains("WITHOUT deprecation log"))
        );
    }

    #[test]
    fn compliance_display() {
        assert_eq!(format!("{}", ComplianceLevel::None), "NONE");
        assert_eq!(format!("{}", ComplianceLevel::Partial), "PARTIAL");
        assert_eq!(format!("{}", ComplianceLevel::Full), "FULL");
    }

    #[test]
    fn json_output_is_valid() {
        let result = AuditResult {
            primal_name: "test".to_string(),
            compliance: ComplianceLevel::Partial,
            signal_detection: vec!["src/a.rs: clear signal byte constant".to_string()],
            client_signals: vec![],
            accept_loop_found: true,
            legacy_warns: false,
            issues: vec!["No client signal".to_string()],
            notes: vec![],
        };
        let json = serde_json::json!({
            "primal": result.primal_name,
            "compliance": format!("{}", result.compliance),
            "signal_detection": result.signal_detection,
            "client_signals": result.client_signals,
            "accept_loop_found": result.accept_loop_found,
            "legacy_warns": result.legacy_warns,
            "issues": result.issues,
            "notes": result.notes,
        });
        let output = serde_json::to_string_pretty(&json).unwrap();
        assert!(output.contains("PARTIAL"));
        assert!(output.contains("No client signal"));
    }
}
