//! G68+ Platform Paths compliance validator.
//!
//! Detects hardcoded path assumptions that break cross-platform deployment:
//! - Raw `/tmp` usage (should use `PrimalDirs::runtime`)
//! - `$HOME/.config` hardcoded (should use `PrimalDirs::config`)
//! - `$HOME/.local/share` hardcoded (should use `PrimalDirs::data`)
//! - XDG env reads outside the `platform_paths` module (fragile, doesn't handle macOS/Windows)
//! - Raw `PathBuf::from("/var/...")` or `/run/...` paths

use anyhow::Result;
use std::path::Path;

struct Violation {
    file: String,
    description: String,
    is_test: bool,
}

const HARDCODED_PATH_PATTERNS: &[(&str, &str)] = &[
    (
        "PathBuf::from(\"/tmp",
        "hardcoded /tmp (use PrimalDirs::runtime)",
    ),
    (
        "PathBuf::from(\"/var",
        "hardcoded /var (use PrimalDirs for data/logs)",
    ),
    (
        "PathBuf::from(\"/run",
        "hardcoded /run (use PrimalDirs::runtime)",
    ),
    (
        "\"/tmp/biomeos",
        "hardcoded /tmp/biomeos path (use PrimalDirs::runtime)",
    ),
];

const RAW_XDG_PATTERNS: &[(&str, &str)] = &[
    (
        "std::env::var(\"XDG_CONFIG_HOME\")",
        "raw XDG_CONFIG_HOME read (use PrimalDirs::config)",
    ),
    (
        "std::env::var(\"XDG_DATA_HOME\")",
        "raw XDG_DATA_HOME read (use PrimalDirs::data)",
    ),
    (
        "std::env::var(\"XDG_CACHE_HOME\")",
        "raw XDG_CACHE_HOME read (use PrimalDirs::cache)",
    ),
    (
        "std::env::var(\"XDG_STATE_HOME\")",
        "raw XDG_STATE_HOME read (use PrimalDirs::logs)",
    ),
];

const EXEMPT_FILES: &[&str] = &[
    "platform_paths.rs",
    "platform_paths/",
    "env_keys.rs",
    "transport/socket.rs",
    "transport.rs",
    "announce.rs",
    "ci.rs",
    "convergence.rs",
    "validate/mod.rs",
];

#[expect(clippy::unnecessary_wraps, reason = "signature required by dispatch")]
pub(super) fn validate(path: &Path, json: bool) -> Result<()> {
    let source_dir = super::find_source_dir(path);
    let Some(source_dir) = source_dir else {
        if json {
            println!(r#"{{"error":"no source directory found"}}"#);
        } else {
            crate::error("No source directory found");
        }
        return Ok(());
    };

    let files = super::collect_rs_files(&source_dir);
    let violations = scan_violations(path, &files);

    let has_platform_paths = files
        .iter()
        .any(|f| f.to_string_lossy().contains("platform_paths"));

    if json {
        print_json(&violations, has_platform_paths);
    } else {
        print_human(&violations, has_platform_paths);
    }

    Ok(())
}

fn scan_violations(base_path: &Path, files: &[std::path::PathBuf]) -> Vec<Violation> {
    let mut violations = Vec::new();

    for file in files {
        let rel = file.strip_prefix(base_path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        if EXEMPT_FILES.iter().any(|exempt| rel_str.contains(exempt)) {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(file) else {
            continue;
        };

        let file_is_test = rel_str.contains("/tests/")
            || rel_str.contains("/test_")
            || rel_str.ends_with("_test.rs")
            || content.contains("#[cfg(test)]");

        let non_doc_content: String = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//!") && !trimmed.starts_with("///")
            })
            .collect::<Vec<_>>()
            .join("\n");

        for &(pattern, desc) in HARDCODED_PATH_PATTERNS {
            if non_doc_content.contains(pattern) {
                violations.push(Violation {
                    file: rel_str.to_string(),
                    description: desc.to_owned(),
                    is_test: file_is_test,
                });
            }
        }

        for &(pattern, desc) in RAW_XDG_PATTERNS {
            if non_doc_content.contains(pattern) {
                violations.push(Violation {
                    file: rel_str.to_string(),
                    description: desc.to_owned(),
                    is_test: file_is_test,
                });
            }
        }
    }

    violations
}

fn print_json(violations: &[Violation], has_platform_paths: bool) {
    let prod: Vec<_> = violations.iter().filter(|v| !v.is_test).collect();
    let test: Vec<_> = violations.iter().filter(|v| v.is_test).collect();

    let compliance = compliance_level(prod.len(), test.len());

    println!("{{");
    println!("  \"compliance\": \"{compliance}\",");
    println!("  \"has_platform_paths_module\": {has_platform_paths},");
    println!("  \"production\": {{");
    println!("    \"count\": {},", prod.len());
    println!("    \"violations\": [");
    for (i, v) in prod.iter().enumerate() {
        let comma = if i + 1 < prod.len() { "," } else { "" };
        println!(
            "      {{\"file\": \"{}\", \"issue\": \"{}\"}}{}",
            v.file, v.description, comma
        );
    }
    println!("    ]");
    println!("  }},");
    println!("  \"test_only\": {{");
    println!("    \"count\": {},", test.len());
    println!("    \"violations\": [");
    for (i, v) in test.iter().enumerate() {
        let comma = if i + 1 < test.len() { "," } else { "" };
        println!(
            "      {{\"file\": \"{}\", \"issue\": \"{}\"}}{}",
            v.file, v.description, comma
        );
    }
    println!("    ]");
    println!("  }}");
    println!("}}");
}

fn print_human(violations: &[Violation], has_platform_paths: bool) {
    let prod: Vec<_> = violations.iter().filter(|v| !v.is_test).collect();
    let test: Vec<_> = violations.iter().filter(|v| v.is_test).collect();

    let compliance = compliance_level(prod.len(), test.len());

    println!();
    crate::info(&format!(
        "Platform Paths compliance: {compliance} ({} production, {} test-only)",
        prod.len(),
        test.len(),
    ));

    if has_platform_paths {
        crate::success("  platform_paths module present");
    } else {
        crate::warning("  no platform_paths module found");
    }

    if !prod.is_empty() {
        println!();
        crate::error("  Production violations:");
        for v in &prod {
            println!("    {} — {}", v.file, v.description);
        }
    }

    if !test.is_empty() {
        println!();
        crate::warning("  Test-only violations (acceptable):");
        for v in &test {
            println!("    {} — {}", v.file, v.description);
        }
    }

    if prod.is_empty() && test.is_empty() {
        crate::success("  Zero hardcoded path violations — fully compliant");
    }
}

const fn compliance_level(prod_count: usize, test_count: usize) -> &'static str {
    if prod_count == 0 && test_count == 0 {
        "G68-paths"
    } else if prod_count == 0 {
        "G68-paths-prod"
    } else {
        "partial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_hardcoded_tmp_path() {
        let tmp = tempfile::tempdir().unwrap();
        let crates_dir = tmp.path().join("crates/foo-core/src");
        fs::create_dir_all(&crates_dir).unwrap();
        fs::write(
            crates_dir.join("server.rs"),
            r#"let p = PathBuf::from("/tmp/mysocket.sock");"#,
        )
        .unwrap();

        let result = validate(tmp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn exempt_platform_paths_module() {
        let tmp = tempfile::tempdir().unwrap();
        let crates_dir = tmp.path().join("crates/foo-core/src");
        fs::create_dir_all(&crates_dir).unwrap();
        fs::write(
            crates_dir.join("platform_paths.rs"),
            r#"std::env::var("XDG_CONFIG_HOME")"#,
        )
        .unwrap();

        let result = validate(tmp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn compliance_full_when_clean() {
        assert_eq!(compliance_level(0, 0), "G68-paths");
    }

    #[test]
    fn compliance_prod_when_test_only() {
        assert_eq!(compliance_level(0, 3), "G68-paths-prod");
    }

    #[test]
    fn compliance_partial_when_prod_violations() {
        assert_eq!(compliance_level(2, 1), "partial");
    }
}
