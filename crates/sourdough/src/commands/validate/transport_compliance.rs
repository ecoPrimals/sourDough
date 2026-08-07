//! Single-primal transport abstraction compliance validation.
//!
//! Checks that a primal uses transport injection (`TransportEndpoint`)
//! rather than self-binding, and that platform-specific APIs are
//! properly gated.

use anyhow::Result;
use std::path::Path;

/// Anti-patterns that indicate self-binding transport (primals should not do this).
pub(super) const SELF_BIND_PATTERNS: &[(&str, &str)] = &[
    ("TcpListener::bind", "hardcoded TCP self-binding"),
    ("UnixListener::bind", "hardcoded UDS self-binding"),
    (".bind(\"0.0.0.0", "hardcoded bind-all address"),
    (".bind(\"127.0.0.1", "hardcoded localhost bind"),
    ("--port", "CLI port flag (transport should be injected)"),
    ("--socket", "CLI socket flag (transport should be injected)"),
];

/// Positive patterns that indicate transport injection compliance.
pub(super) const INJECTION_PATTERNS: &[(&str, &str)] = &[
    ("TransportEndpoint", "uses TransportEndpoint enum"),
    ("connect_transport", "uses connect_transport()"),
    ("TransportListener", "uses G66 TransportListener"),
    ("bind_transport", "uses G66 bind_transport()"),
    ("TransportStream", "uses G66 TransportStream"),
    ("TRANSPORT_ENDPOINT", "accepts injected endpoint env var"),
    ("PRIMAL_BIND_MODE", "respects bind mode (Android/SELinux)"),
    ("platform_default", "uses platform_default() resolution"),
    (
        "from_env_or_default",
        "uses from_env_or_default() injection",
    ),
];

/// Patterns indicating silicon deism (unconditional Unix assumptions).
const SILICON_DEISM_PATTERNS: &[(&str, &str)] = &[
    (
        "use tokio::net::UnixStream",
        "unconditional UnixStream import",
    ),
    (
        "use tokio::net::UnixListener",
        "unconditional UnixListener import",
    ),
    ("use std::os::unix", "unconditional std::os::unix import"),
    (
        "use rustix::",
        "unconditional rustix import outside transport",
    ),
];

pub(crate) fn validate(path: &Path) -> Result<()> {
    crate::info(&format!(
        "Validating transport abstraction compliance: {}",
        path.display()
    ));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut compliant: Vec<String> = Vec::new();

    let src_dir = super::find_source_dir(path);
    let Some(src_dir) = src_dir else {
        anyhow::bail!(
            "No source directory found at {} — expected crates/*/src/ or src/",
            path.display()
        );
    };

    crate::info("Scanning for self-binding anti-patterns...");
    let rs_files = super::collect_rs_files(&src_dir);

    if rs_files.is_empty() {
        warnings.push("No .rs source files found".to_string());
    }

    let mut self_bind_violations = Vec::new();
    let mut injection_found = Vec::new();

    for file in &rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(path).unwrap_or(file);

        let rel_str = rel.to_string_lossy();
        let in_test = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || rel_str.ends_with("_test.rs")
            || content.contains("#[cfg(test)]");

        for &(pattern, description) in SELF_BIND_PATTERNS {
            if content.contains(pattern) && !in_test {
                self_bind_violations
                    .push(format!("  {}: {description} (`{pattern}`)", rel.display()));
            }
        }

        for &(pattern, description) in INJECTION_PATTERNS {
            if content.contains(pattern) {
                injection_found.push(format!("  {}: {description}", rel.display()));
            }
        }
    }

    if self_bind_violations.is_empty() {
        compliant.push("No self-binding anti-patterns found in business logic".to_string());
    } else {
        crate::warning("Self-binding patterns found (should use transport injection):");
        for v in &self_bind_violations {
            println!("{v}");
        }
        let n = self_bind_violations.len();
        warnings.push(format!("{n} self-binding pattern(s) found"));
    }

    println!();
    if injection_found.is_empty() {
        errors.push(
            "No transport injection patterns found (TransportEndpoint, connect_transport, etc.)"
                .to_string(),
        );
    } else {
        crate::info("Transport injection patterns detected:");
        for p in &injection_found {
            println!("{p}");
        }
        let n = injection_found.len();
        compliant.push(format!("{n} transport injection pattern(s) found"));
    }

    println!();
    check_platform_guards(&rs_files, path, &mut compliant, &mut warnings);

    println!();
    check_silicon_deism(&rs_files, path, &mut compliant, &mut warnings);

    println!();
    for c in &compliant {
        crate::success(c);
    }

    println!();
    super::report_results(&errors, &warnings)
}

fn check_silicon_deism(
    rs_files: &[std::path::PathBuf],
    base_path: &Path,
    compliant: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    crate::info("Checking for silicon deism (G66)...");
    let mut deism_issues = Vec::new();

    for file in rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(base_path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        let in_transport = rel_str.contains("transport")
            || rel_str.contains("socket")
            || rel_str.contains("stream");
        let in_test = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || content.contains("#[cfg(test)]");
        let has_cfg_guard = content.contains("#[cfg(unix)]") || content.contains("cfg!(unix)");

        if in_test || in_transport {
            continue;
        }

        for &(pattern, description) in SILICON_DEISM_PATTERNS {
            if content.contains(pattern) && !has_cfg_guard {
                deism_issues.push(format!(
                    "  {}: {description} (move to transport layer)",
                    rel.display()
                ));
            }
        }
    }

    if deism_issues.is_empty() {
        compliant
            .push("No silicon deism detected — Unix APIs confined to transport layer".to_owned());
    } else {
        crate::warning(
            "Silicon deism detected (G66 violation — Unix APIs outside transport layer):",
        );
        for issue in &deism_issues {
            println!("{issue}");
        }
        let n = deism_issues.len();
        warnings.push(format!("{n} silicon deism violation(s)"));
    }
}

fn check_platform_guards(
    rs_files: &[std::path::PathBuf],
    base_path: &Path,
    compliant: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    crate::info("Checking platform-specific socket API usage...");
    let mut platform_issues = Vec::new();
    for file in rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(base_path).unwrap_or(file);

        let has_cfg_guard = content.contains("#[cfg(unix)]")
            || content.contains("#[cfg(target_os")
            || content.contains("cfg!(unix)");
        let uses_unix_api = content.contains("tokio::net::UnixStream")
            || content.contains("tokio::net::UnixListener")
            || content.contains("std::os::unix");

        if uses_unix_api && !has_cfg_guard {
            platform_issues.push(format!(
                "  {}: Unix-only APIs without #[cfg(unix)] guard",
                rel.display()
            ));
        }
    }

    if platform_issues.is_empty() {
        compliant.push("Platform-specific APIs properly gated or not used".to_string());
    } else {
        for p in &platform_issues {
            println!("{p}");
        }
        let n = platform_issues.len();
        warnings.push(format!("{n} file(s) use Unix APIs without platform guards"));
    }
}
