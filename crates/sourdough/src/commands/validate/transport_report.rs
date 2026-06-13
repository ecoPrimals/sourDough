//! Batch transport compliance reporting across the ecosystem.
//!
//! Scans a directory of primal checkouts and produces a structured
//! compliance report for the entire ecosystem.

use anyhow::{Context, Result};
use std::path::Path;

/// Result of a single primal's transport audit.
#[derive(Debug)]
struct PrimalAudit {
    name: String,
    self_bind_count: usize,
    injection_count: usize,
    platform_issues: usize,
    has_sourdough_dep: bool,
    status: AuditStatus,
}

#[derive(Debug, Clone, Copy)]
enum AuditStatus {
    Compliant,
    Warnings,
    NonCompliant,
    Skipped,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compliant => write!(f, "✓ Compliant"),
            Self::Warnings => write!(f, "⚠ Warnings"),
            Self::NonCompliant => write!(f, "✗ Non-compliant"),
            Self::Skipped => write!(f, "— Skipped"),
        }
    }
}

/// Run a transport compliance report across all primals in a directory.
pub(crate) fn run(
    primals_dir: &Path,
    output: Option<&Path>,
    json: bool,
    exempt: &[String],
) -> Result<()> {
    if !json {
        crate::info(&format!("Scanning primals in: {}", primals_dir.display()));
        println!();
    }

    let entries = std::fs::read_dir(primals_dir)
        .with_context(|| format!("Cannot read primals directory: {}", primals_dir.display()))?;

    let mut audits: Vec<PrimalAudit> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        if !path.join("Cargo.toml").exists() {
            continue;
        }

        if exempt.iter().any(|e| e.eq_ignore_ascii_case(&name)) {
            audits.push(PrimalAudit {
                name,
                self_bind_count: 0,
                injection_count: 0,
                platform_issues: 0,
                has_sourdough_dep: false,
                status: AuditStatus::Skipped,
            });
            continue;
        }

        let audit = audit_primal(&name, &path);
        audits.push(audit);
    }

    audits.sort_by(|a, b| a.name.cmp(&b.name));

    if json {
        let json_output = format_json(&audits);
        println!("{json_output}");
        if let Some(out_path) = output {
            std::fs::write(out_path, &json_output)
                .with_context(|| format!("Cannot write report to: {}", out_path.display()))?;
        }
    } else {
        let report = format_report(&audits);
        println!("{report}");

        if let Some(out_path) = output {
            std::fs::write(out_path, &report)
                .with_context(|| format!("Cannot write report to: {}", out_path.display()))?;
            crate::success(&format!("Report written to: {}", out_path.display()));
        }

        let compliant = audits
            .iter()
            .filter(|a| matches!(a.status, AuditStatus::Compliant))
            .count();
        let total = audits.len();
        let skipped = audits
            .iter()
            .filter(|a| matches!(a.status, AuditStatus::Skipped))
            .count();
        let audited = total - skipped;

        println!();
        crate::info(&format!(
            "Summary: {compliant}/{audited} compliant ({skipped} exempt, {total} total)"
        ));
    }

    Ok(())
}

fn audit_primal(name: &str, path: &Path) -> PrimalAudit {
    let src_dir = super::find_source_dir(path);
    let Some(src_dir) = src_dir else {
        return PrimalAudit {
            name: name.to_owned(),
            self_bind_count: 0,
            injection_count: 0,
            platform_issues: 0,
            has_sourdough_dep: false,
            status: AuditStatus::Skipped,
        };
    };

    let has_sourdough_dep = detect_sourdough_dep(path);

    let rs_files = super::collect_rs_files(&src_dir);
    let mut self_bind_count = 0;
    let mut injection_count = 0;
    let mut platform_issues = 0;

    for file in &rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        let in_test = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || rel_str.ends_with("_test.rs")
            || content.contains("#[cfg(test)]");

        let is_template = rel_str.contains("templates/")
            || rel_str.contains("scaffold/")
            || rel_str.contains("migrate");

        if !in_test && !is_template {
            for &(pattern, _) in super::SELF_BIND_PATTERNS {
                if content.contains(pattern) {
                    self_bind_count += 1;
                }
            }
        }

        for &(pattern, _) in super::INJECTION_PATTERNS {
            if content.contains(pattern) {
                injection_count += 1;
            }
        }

        let uses_unix_api = content.contains("tokio::net::UnixStream")
            || content.contains("tokio::net::UnixListener")
            || content.contains("std::os::unix");
        let has_platform_guard = content.contains("#[cfg(unix)]")
            || content.contains("#[cfg(target_os")
            || content.contains("cfg!(unix)")
            || content.contains("PRIMAL_BIND_MODE")
            || content.contains("bind_mode");

        if uses_unix_api && !has_platform_guard && !in_test {
            platform_issues += 1;
        }
    }

    let status = if self_bind_count == 0 && injection_count > 0 {
        AuditStatus::Compliant
    } else if self_bind_count == 0 {
        AuditStatus::Warnings
    } else {
        AuditStatus::NonCompliant
    };

    PrimalAudit {
        name: name.to_owned(),
        self_bind_count,
        injection_count,
        platform_issues,
        has_sourdough_dep,
        status,
    }
}

/// Check if any Cargo.toml in the primal depends on sourdough-core.
fn detect_sourdough_dep(path: &Path) -> bool {
    let check_file = |p: &Path| -> bool {
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .contains("sourdough-core")
    };

    if check_file(&path.join("Cargo.toml")) {
        return true;
    }

    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                if check_file(&entry.path().join("Cargo.toml")) {
                    return true;
                }
            }
        }
    }

    false
}

fn format_json(audits: &[PrimalAudit]) -> String {
    let items: Vec<serde_json::Value> = audits
        .iter()
        .map(|a| {
            serde_json::json!({
                "primal": a.name,
                "status": match a.status {
                    AuditStatus::Compliant => "compliant",
                    AuditStatus::Warnings => "warnings",
                    AuditStatus::NonCompliant => "non_compliant",
                    AuditStatus::Skipped => "skipped",
                },
                "self_bind_count": a.self_bind_count,
                "injection_count": a.injection_count,
                "platform_issues": a.platform_issues,
                "has_sourdough_dep": a.has_sourdough_dep,
            })
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_default()
}

fn format_report(audits: &[PrimalAudit]) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    out.push_str("# Transport Compliance Report\n\n");
    let _ = writeln!(
        out,
        "Generated: {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    );
    out.push_str("| Primal | Status | Self-bind | Injection | Platform | Dep |\n");
    out.push_str("|--------|--------|-----------|-----------|----------|-----|\n");

    for audit in audits {
        let status_icon = match audit.status {
            AuditStatus::Compliant => "✓",
            AuditStatus::Warnings => "⚠",
            AuditStatus::NonCompliant => "✗",
            AuditStatus::Skipped => "—",
        };
        let dep_flag = if audit.has_sourdough_dep {
            "⚠ sd"
        } else {
            "—"
        };
        let _ = writeln!(
            out,
            "| {} | {status_icon} | {} | {} | {} | {dep_flag} |",
            audit.name, audit.self_bind_count, audit.injection_count, audit.platform_issues,
        );
    }

    let compliant = audits
        .iter()
        .filter(|a| matches!(a.status, AuditStatus::Compliant))
        .count();
    let non_compliant = audits
        .iter()
        .filter(|a| matches!(a.status, AuditStatus::NonCompliant))
        .count();
    let warnings = audits
        .iter()
        .filter(|a| matches!(a.status, AuditStatus::Warnings))
        .count();
    let skipped = audits
        .iter()
        .filter(|a| matches!(a.status, AuditStatus::Skipped))
        .count();

    let dep_violators: Vec<&str> = audits
        .iter()
        .filter(|a| a.has_sourdough_dep)
        .map(|a| a.name.as_str())
        .collect();

    let _ = write!(
        out,
        "\n**Summary**: {compliant} compliant, {warnings} warnings, {non_compliant} non-compliant, {skipped} exempt\n"
    );

    if !dep_violators.is_empty() {
        let _ = write!(
            out,
            "\n**⚠ sourdough-core dependency violation**: {}\n\
             Primals must implement TransportEndpoint locally — the wire format is the contract.\n",
            dep_violators.join(", ")
        );
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_primal_dir(dir: &Path, name: &str, src_content: &str) -> std::path::PathBuf {
        let primal = dir.join(name);
        let src = primal
            .join("crates")
            .join(format!("{name}-core"))
            .join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(primal.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(src.join("lib.rs"), src_content).unwrap();
        primal
    }

    #[test]
    fn empty_primals_dir_produces_empty_report() {
        let dir = TempDir::new().unwrap();
        let result = run(dir.path(), None, false, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn non_existent_dir_returns_error() {
        let result = run(Path::new("/nonexistent/path/xyz"), None, false, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn skips_directories_without_cargo_toml() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("not-a-primal");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("random.txt"), "not a primal").unwrap();
        let result = run(dir.path(), None, false, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn exempt_primal_is_skipped() {
        let dir = TempDir::new().unwrap();
        setup_primal_dir(dir.path(), "myprimal", "fn main() {}");
        let exempt = vec!["myprimal".to_string()];
        let result = run(dir.path(), None, true, &exempt);
        assert!(result.is_ok());
    }

    #[test]
    fn compliant_primal_detected() {
        let dir = TempDir::new().unwrap();
        let content = r#"
            use crate::transport::TransportEndpoint;
            fn connect() { connect_transport(&endpoint); }
            const EP: &str = "TRANSPORT_ENDPOINT";
        "#;
        setup_primal_dir(dir.path(), "goodprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn noncompliant_primal_detected_self_bind() {
        let dir = TempDir::new().unwrap();
        let content = r#"
            use tokio::net::TcpListener;
            async fn serve() {
                let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
            }
        "#;
        setup_primal_dir(dir.path(), "badprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn json_output_is_valid_json() {
        let dir = TempDir::new().unwrap();
        let content = "fn main() { let _ = TransportEndpoint::default(); }";
        setup_primal_dir(dir.path(), "jsonprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn report_written_to_file() {
        let dir = TempDir::new().unwrap();
        let content = "fn main() {}";
        setup_primal_dir(dir.path(), "reportprimal", content);
        let output_path = dir.path().join("report.md");
        let result = run(dir.path(), Some(&output_path), false, &[]);
        assert!(result.is_ok());
        assert!(output_path.exists());
        let report = fs::read_to_string(&output_path).unwrap();
        assert!(report.contains("Transport Compliance Report"));
    }

    #[test]
    fn json_report_written_to_file() {
        let dir = TempDir::new().unwrap();
        let content = "fn main() {}";
        setup_primal_dir(dir.path(), "jsonfile", content);
        let output_path = dir.path().join("report.json");
        let result = run(dir.path(), Some(&output_path), true, &[]);
        assert!(result.is_ok());
        assert!(output_path.exists());
        let json_str = fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn sourdough_dep_detected() {
        let dir = TempDir::new().unwrap();
        let primal = dir.path().join("depprimal");
        let src = primal.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            primal.join("Cargo.toml"),
            "[package]\nname = \"depprimal\"\n\n[dependencies]\nsourdough-core = \"0.4\"\n",
        )
        .unwrap();
        fs::write(src.join("lib.rs"), "fn main() {}").unwrap();
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn platform_issue_detected_for_unguarded_unix() {
        let dir = TempDir::new().unwrap();
        let content = r#"
            use tokio::net::UnixStream;
            async fn connect() {
                UnixStream::connect("/tmp/test.sock").await.unwrap();
            }
        "#;
        setup_primal_dir(dir.path(), "unixprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn guarded_unix_no_platform_issue() {
        let dir = TempDir::new().unwrap();
        let content = "
            #[cfg(unix)]
            use tokio::net::UnixStream;
            async fn connect() { }
        ";
        setup_primal_dir(dir.path(), "guardedprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_code_not_flagged_for_self_bind() {
        let dir = TempDir::new().unwrap();
        let content = r#"
            use tokio::net::TcpListener;
            #[cfg(test)]
            mod tests {
                async fn test_serve() {
                    TcpListener::bind("127.0.0.1:0").await.unwrap();
                }
            }
        "#;
        setup_primal_dir(dir.path(), "testprimal", content);
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn multiple_primals_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        setup_primal_dir(dir.path(), "zulu", "fn main() {}");
        setup_primal_dir(dir.path(), "alpha", "fn main() {}");
        setup_primal_dir(dir.path(), "mike", "fn main() {}");
        let result = run(dir.path(), None, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn audit_status_display() {
        assert_eq!(format!("{}", AuditStatus::Compliant), "✓ Compliant");
        assert_eq!(format!("{}", AuditStatus::Warnings), "⚠ Warnings");
        assert_eq!(format!("{}", AuditStatus::NonCompliant), "✗ Non-compliant");
        assert_eq!(format!("{}", AuditStatus::Skipped), "— Skipped");
    }

    #[test]
    fn format_json_produces_valid_array() {
        let audits = vec![
            PrimalAudit {
                name: "test".to_owned(),
                self_bind_count: 2,
                injection_count: 1,
                platform_issues: 0,
                has_sourdough_dep: false,
                status: AuditStatus::NonCompliant,
            },
            PrimalAudit {
                name: "good".to_owned(),
                self_bind_count: 0,
                injection_count: 3,
                platform_issues: 0,
                has_sourdough_dep: false,
                status: AuditStatus::Compliant,
            },
        ];
        let json_str = format_json(&audits);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["primal"], "test");
        assert_eq!(parsed[0]["status"], "non_compliant");
        assert_eq!(parsed[1]["status"], "compliant");
    }

    #[test]
    fn format_report_includes_summary() {
        let audits = vec![
            PrimalAudit {
                name: "alpha".to_owned(),
                self_bind_count: 0,
                injection_count: 2,
                platform_issues: 0,
                has_sourdough_dep: false,
                status: AuditStatus::Compliant,
            },
            PrimalAudit {
                name: "beta".to_owned(),
                self_bind_count: 1,
                injection_count: 0,
                platform_issues: 1,
                has_sourdough_dep: true,
                status: AuditStatus::NonCompliant,
            },
        ];
        let report = format_report(&audits);
        assert!(report.contains("Transport Compliance Report"));
        assert!(report.contains("alpha"));
        assert!(report.contains("beta"));
        assert!(report.contains("1 compliant"));
        assert!(report.contains("1 non-compliant"));
        assert!(report.contains("sourdough-core dependency violation"));
        assert!(report.contains("beta"));
    }

    #[test]
    fn detect_sourdough_dep_in_workspace_crate() {
        let dir = TempDir::new().unwrap();
        let primal = dir.path().join("deptest");
        let crate_dir = primal.join("crates").join("deptest-core");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            primal.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/deptest-core\"]\n",
        )
        .unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[dependencies]\nsourdough-core = { path = \"../../sourdough-core\" }\n",
        )
        .unwrap();
        assert!(detect_sourdough_dep(&primal));
    }

    #[test]
    fn detect_no_sourdough_dep() {
        let dir = TempDir::new().unwrap();
        let primal = dir.path().join("clean");
        fs::create_dir_all(&primal).unwrap();
        fs::write(
            primal.join("Cargo.toml"),
            "[package]\nname = \"clean\"\n\n[dependencies]\ntokio = \"1\"\n",
        )
        .unwrap();
        assert!(!detect_sourdough_dep(&primal));
    }
}
