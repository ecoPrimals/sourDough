//! G68 Platform Substrate compliance validation.
//!
//! Checks that a primal properly abstracts platform-specific filesystem
//! operations through the platform substrate layer rather than using raw
//! Unix/Windows APIs directly in business logic.
//!
//! Three layers checked:
//! - **L1 Links**: raw `symlink` calls → should use `platform_link()`
//! - **L2 Permissions**: raw `set_mode()` / `PermissionsExt` → should use `PlatformAccess`
//! - **L3 Device backends**: raw `rustix`/`libc`/`nix` → should use backend traits
//!
//! The scanner distinguishes production violations from test assertions:
//! - `set_mode()` / `from_mode()` / `set_permissions()` = write operations (violations)
//! - `mode()` read-only queries in test files = assertions verifying correctness (not violations)
//! - Files in `tests/`, `*_test.rs`, or `#[cfg(test)]` blocks are tracked separately

use anyhow::Result;
use std::path::Path;

/// L1: Link operations that should use `platform_link()` instead.
const L1_DEISM_PATTERNS: &[(&str, &str)] = &[
    (
        "std::os::unix::fs::symlink",
        "raw Unix symlink (use platform_link())",
    ),
    (
        "unix::fs::symlink(",
        "raw Unix symlink (use platform_link())",
    ),
    (
        "std::os::windows::fs::symlink_file",
        "raw Windows symlink (use platform_link())",
    ),
    (
        "std::os::windows::fs::symlink_dir",
        "raw Windows symlink (use platform_link())",
    ),
];

/// L2 WRITE operations — real violations (mutating filesystem permissions).
///
/// `set_mode(` is only a violation when `PermissionsExt` is also in the file,
/// indicating it's a filesystem permission mutation rather than a tar header
/// or other unrelated API (e.g., `tar::Header::set_mode()`).
const L2_WRITE_PATTERNS: &[(&str, &str)] = &[
    (
        "Permissions::from_mode(",
        "raw Permissions::from_mode() (use PlatformAccess.apply())",
    ),
    (
        "set_permissions(",
        "raw set_permissions() (use PlatformAccess.apply())",
    ),
];

/// L2 WRITE patterns that require `PermissionsExt` context to be violations.
const L2_CONTEXTUAL_WRITE_PATTERNS: &[(&str, &str)] = &[(
    ".set_mode(",
    "raw PermissionsExt::set_mode() (use PlatformAccess.apply())",
)];

/// L2 patterns that are violations in production but acceptable in tests
/// (reading mode to verify correctness is a valid test assertion).
const L2_READ_PATTERNS: &[(&str, &str)] = &[
    (
        "PermissionsExt",
        "PermissionsExt import (use PlatformAccess)",
    ),
    (".mode()", "mode() read query (use query_access())"),
];

/// L3: Device/OS backends that should use trait abstractions.
const L3_DEISM_PATTERNS: &[(&str, &str)] = &[
    (
        "use rustix::",
        "raw rustix import (use platform-gated backend trait)",
    ),
    (
        "use libc::",
        "raw libc import (use platform-gated backend trait)",
    ),
    (
        "use nix::",
        "raw nix import (use platform-gated backend trait)",
    ),
    (
        "extern crate libc",
        "raw libc extern (use platform-gated backend trait)",
    ),
];

/// Positive patterns indicating proper platform abstraction.
const ABSTRACTION_PATTERNS: &[(&str, &str)] = &[
    ("platform_link(", "uses platform_link()"),
    ("PlatformAccess", "uses PlatformAccess enum"),
    ("platform_substrate", "has platform_substrate module"),
    ("query_access(", "uses query_access()"),
    ("ensure_dir_with_access(", "uses ensure_dir_with_access()"),
    ("ensure_secure_parent(", "uses ensure_secure_parent()"),
    ("is_symlink(", "uses platform-aware is_symlink()"),
];

struct Violation {
    location: String,
    description: String,
    is_test: bool,
}

pub(crate) fn validate(path: &Path, json: bool) -> Result<()> {
    crate::info(&format!(
        "Validating G68 platform substrate compliance: {}",
        path.display()
    ));
    println!();

    let src_dir = super::find_source_dir(path);
    let Some(src_dir) = src_dir else {
        anyhow::bail!(
            "No source directory found at {} — expected crates/*/src/ or src/",
            path.display()
        );
    };

    let rs_files = super::collect_rs_files(&src_dir);
    if rs_files.is_empty() {
        anyhow::bail!("No .rs source files found");
    }

    let mut l1_violations: Vec<Violation> = Vec::new();
    let mut l2_violations: Vec<Violation> = Vec::new();
    let mut l3_violations: Vec<Violation> = Vec::new();
    let mut abstractions_found = Vec::new();

    for file in &rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        let is_test_file = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || rel_str.ends_with("_test.rs")
            || rel_str.ends_with("_tests.rs");

        let has_cfg_test = content.contains("#[cfg(test)]");

        let in_platform_module = rel_str.contains("platform_substrate")
            || rel_str.contains("platform/")
            || rel_str.contains("transport");

        let has_cfg_guard = content.contains("#[cfg(unix)]")
            || content.contains("#[cfg(windows)]")
            || content.contains("#[cfg(target_os")
            || content.contains("cfg!(unix)")
            || content.contains("cfg!(windows)");

        if in_platform_module {
            // Platform modules are always exempt
        } else if has_cfg_guard {
            // Properly guarded code is exempt
        } else {
            let file_is_test = is_test_file;

            // L1: link violations (always violations regardless of test status)
            for &(pattern, desc) in L1_DEISM_PATTERNS {
                if content.contains(pattern) {
                    l1_violations.push(Violation {
                        location: format!("  {}", rel.display()),
                        description: desc.to_owned(),
                        is_test: file_is_test,
                    });
                }
            }

            // L2 WRITE operations: violations in both prod and test
            for &(pattern, desc) in L2_WRITE_PATTERNS {
                if content.contains(pattern) {
                    l2_violations.push(Violation {
                        location: format!("  {}", rel.display()),
                        description: desc.to_owned(),
                        is_test: file_is_test,
                    });
                }
            }

            // L2 contextual writes: only a violation when PermissionsExt is in scope
            // (distinguishes filesystem set_mode from tar::Header::set_mode etc.)
            let has_permissions_ext = content.contains("PermissionsExt");
            for &(pattern, desc) in L2_CONTEXTUAL_WRITE_PATTERNS {
                if content.contains(pattern) && has_permissions_ext {
                    l2_violations.push(Violation {
                        location: format!("  {}", rel.display()),
                        description: desc.to_owned(),
                        is_test: file_is_test,
                    });
                }
            }

            // L2 READ operations: only violations in production code.
            // In test files or #[cfg(test)] modules, reading mode() to assert
            // correctness is a valid test pattern — not silicon deism.
            for &(pattern, desc) in L2_READ_PATTERNS {
                if content.contains(pattern) {
                    let is_test_context = file_is_test || has_cfg_test;
                    l2_violations.push(Violation {
                        location: format!("  {}", rel.display()),
                        description: desc.to_owned(),
                        is_test: is_test_context,
                    });
                }
            }

            // L3: device backend violations
            for &(pattern, desc) in L3_DEISM_PATTERNS {
                if content.contains(pattern) {
                    l3_violations.push(Violation {
                        location: format!("  {}", rel.display()),
                        description: desc.to_owned(),
                        is_test: file_is_test,
                    });
                }
            }
        }

        for &(pattern, desc) in ABSTRACTION_PATTERNS {
            if content.contains(pattern) {
                abstractions_found.push(format!("  {}: {desc}", rel.display()));
            }
        }
    }

    if json {
        return print_json(
            &l1_violations,
            &l2_violations,
            &l3_violations,
            &abstractions_found,
        );
    }

    print_human_report(
        &l1_violations,
        &l2_violations,
        &l3_violations,
        &abstractions_found,
    )
}

fn count_prod(violations: &[Violation]) -> usize {
    violations.iter().filter(|v| !v.is_test).count()
}

fn count_test(violations: &[Violation]) -> usize {
    violations.iter().filter(|v| v.is_test).count()
}

fn print_human_report(
    l1: &[Violation],
    l2: &[Violation],
    l3: &[Violation],
    abstractions: &[String],
) -> Result<()> {
    let prod_l1 = count_prod(l1);
    let prod_l2 = count_prod(l2);
    let prod_l3 = count_prod(l3);
    let test_l1 = count_test(l1);
    let test_l2 = count_test(l2);
    let test_l3 = count_test(l3);
    let prod_total = prod_l1 + prod_l2 + prod_l3;
    let test_total = test_l1 + test_l2 + test_l3;

    // Production violations
    let prod_violations: Vec<&Violation> = l1
        .iter()
        .chain(l2.iter())
        .chain(l3.iter())
        .filter(|v| !v.is_test)
        .collect();

    if !prod_violations.is_empty() {
        crate::warning(&format!("Production violations ({prod_total}):"));
        for v in &prod_violations {
            println!("{}: {}", v.location, v.description);
        }
        println!();
    }

    // Test-only violations (informational, not blocking)
    let test_violations: Vec<&Violation> = l1
        .iter()
        .chain(l2.iter())
        .chain(l3.iter())
        .filter(|v| v.is_test)
        .collect();

    if !test_violations.is_empty() {
        crate::info(&format!(
            "Test-only violations ({test_total}) — assertions, not silicon deism:"
        ));
        for v in &test_violations {
            println!("{}: {}", v.location, v.description);
        }
        println!();
    }

    // Summary line
    if prod_total == 0 && test_total == 0 {
        crate::success("No platform substrate violations found (G68 compliant)");
    } else if prod_total == 0 {
        crate::success(&format!(
            "No production violations (G68-prod compliant) — {test_total} test-only"
        ));
    } else {
        println!("  Production: {prod_total} violation(s) ({prod_l1} L1, {prod_l2} L2, {prod_l3} L3)");
        println!("  Test-only:  {test_total} (not blocking)");
    }

    println!();
    if abstractions.is_empty() {
        crate::info("No platform abstraction patterns detected (module may not need any).");
    } else {
        crate::info("Platform abstraction patterns found:");
        for a in abstractions {
            println!("{a}");
        }
    }

    // Compliance level
    println!();
    let compliance = if prod_total == 0 && test_total == 0 {
        "G68"
    } else if prod_total == 0 {
        "G68-prod"
    } else if !abstractions.is_empty() {
        "partial"
    } else {
        "none"
    };
    crate::info(&format!("Compliance level: {compliance}"));

    if prod_total > 0 {
        anyhow::bail!("{prod_total} production G68 violation(s) found");
    }

    Ok(())
}

fn print_json(
    l1: &[Violation],
    l2: &[Violation],
    l3: &[Violation],
    abstractions: &[String],
) -> Result<()> {
    let prod_l1 = count_prod(l1);
    let prod_l2 = count_prod(l2);
    let prod_l3 = count_prod(l3);
    let test_l1 = count_test(l1);
    let test_l2 = count_test(l2);
    let test_l3 = count_test(l3);
    let prod_total = prod_l1 + prod_l2 + prod_l3;
    let test_total = test_l1 + test_l2 + test_l3;

    let prod_details: Vec<String> = l1
        .iter()
        .chain(l2.iter())
        .chain(l3.iter())
        .filter(|v| !v.is_test)
        .map(|v| format!("{}: {}", v.location.trim(), v.description))
        .collect();

    let test_details: Vec<String> = l1
        .iter()
        .chain(l2.iter())
        .chain(l3.iter())
        .filter(|v| v.is_test)
        .map(|v| format!("{}: {}", v.location.trim(), v.description))
        .collect();

    let compliance = if prod_total == 0 && test_total == 0 {
        "G68"
    } else if prod_total == 0 {
        "G68-prod"
    } else {
        "partial"
    };

    let report = serde_json::json!({
        "g68_compliance": prod_total == 0,
        "compliance_level": compliance,
        "violations": {
            "production": {
                "l1_links": prod_l1,
                "l2_permissions": prod_l2,
                "l3_device_backends": prod_l3,
                "total": prod_total,
            },
            "test_only": {
                "l1_links": test_l1,
                "l2_permissions": test_l2,
                "l3_device_backends": test_l3,
                "total": test_total,
            },
            "combined_total": prod_total + test_total,
        },
        "abstractions_detected": abstractions.len(),
        "details": {
            "production_violations": prod_details,
            "test_violations": test_details,
            "abstractions": abstractions,
        }
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_primal(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let crate_src = dir.path().join("crates/test-core/src");
        std::fs::create_dir_all(&crate_src).unwrap();
        for (name, content) in files {
            std::fs::write(crate_src.join(name), content).unwrap();
        }
        let p = dir.path().to_path_buf();
        (dir, p)
    }

    fn temp_primal_with_tests(
        prod_files: &[(&str, &str)],
        test_files: &[(&str, &str)],
    ) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let crate_src = dir.path().join("crates/test-core/src");
        std::fs::create_dir_all(&crate_src).unwrap();
        for (name, content) in prod_files {
            std::fs::write(crate_src.join(name), content).unwrap();
        }
        let test_dir = dir.path().join("crates/test-core/tests");
        std::fs::create_dir_all(&test_dir).unwrap();
        for (name, content) in test_files {
            std::fs::write(test_dir.join(name), content).unwrap();
        }
        let p = dir.path().to_path_buf();
        (dir, p)
    }

    #[test]
    fn clean_primal_passes() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "pub fn run() { println!(\"hello\"); }",
        )]);
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn detects_l1_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use std::os::unix::fs::symlink;\nfn link() { symlink(a, b); }",
        )]);
        assert!(validate(&path, false).is_err());
    }

    #[test]
    fn detects_l2_write_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use std::os::unix::fs::PermissionsExt;\nfn perms() { p.set_mode(0o755); }",
        )]);
        assert!(validate(&path, false).is_err());
    }

    #[test]
    fn tar_header_set_mode_is_not_violation() {
        let (_dir, path) = temp_primal(&[(
            "archive.rs",
            "use tar;\nfn pack() { header.set_mode(0o755); header.set_cksum(); }",
        )]);
        // tar::Header::set_mode without PermissionsExt is not a filesystem violation
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn detects_l2_from_mode_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use std::fs::Permissions;\nfn secure() { let p = Permissions::from_mode(0o600); }",
        )]);
        assert!(validate(&path, false).is_err());
    }

    #[test]
    fn detects_l3_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use rustix::io;\nfn raw() {}",
        )]);
        assert!(validate(&path, false).is_err());
    }

    #[test]
    fn mode_read_in_test_file_is_not_prod_violation() {
        let (_dir, path) = temp_primal_with_tests(
            &[("lib.rs", "pub fn run() {}")],
            &[(
                "permission_tests.rs",
                "use std::os::unix::fs::PermissionsExt;\nfn test_perms() { let m = p.mode(); assert_eq!(m, 0o600); }",
            )],
        );
        // Should pass because mode() read is in a test file
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn mode_read_in_cfg_test_module_is_not_prod_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "pub fn run() {}\n\n#[cfg(test)]\nmod tests {\n    use std::os::unix::fs::PermissionsExt;\n    fn check() { let m = p.mode(); }\n}",
        )]);
        // mode() + PermissionsExt in #[cfg(test)] → test-only, not prod violation
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn set_permissions_in_test_file_is_still_tracked() {
        let (_dir, path) = temp_primal_with_tests(
            &[("lib.rs", "pub fn run() {}")],
            &[(
                "integration_test.rs",
                "fn setup() { std::fs::set_permissions(p, perms); }",
            )],
        );
        // set_permissions in test file is tracked as test violation but doesn't block
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn set_mode_in_production_code_fails() {
        let (_dir, path) = temp_primal(&[(
            "server.rs",
            "use std::os::unix::fs::PermissionsExt;\nfn secure_socket() { p.set_mode(0o600); }",
        )]);
        assert!(validate(&path, false).is_err());
    }

    #[test]
    fn platform_module_is_exempt() {
        let dir = tempfile::tempdir().unwrap();
        let crate_src = dir.path().join("crates/test-core/src/platform_substrate");
        std::fs::create_dir_all(&crate_src).unwrap();
        std::fs::write(
            crate_src.join("mod.rs"),
            "use std::os::unix::fs::symlink;\nuse std::os::unix::fs::PermissionsExt;\nfn apply() { p.set_mode(0o600); }",
        )
        .unwrap();
        let lib = dir.path().join("crates/test-core/src/lib.rs");
        std::fs::write(lib, "pub mod platform_substrate;").unwrap();
        assert!(validate(dir.path(), false).is_ok());
    }

    #[test]
    fn cfg_guarded_code_is_exempt() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "#[cfg(unix)]\nuse std::os::unix::fs::symlink;\n#[cfg(unix)]\nfn link() { symlink(a, b); }",
        )]);
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn json_output_includes_prod_test_split() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "pub fn run() {}",
        )]);
        assert!(validate(&path, true).is_ok());
    }

    #[test]
    fn detects_abstractions() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use crate::platform_substrate::PlatformAccess;\nfn secure() { platform_link(a, b).unwrap(); }",
        )]);
        assert!(validate(&path, false).is_ok());
    }

    #[test]
    fn compliance_level_g68_prod_when_only_test_violations() {
        let (_dir, path) = temp_primal_with_tests(
            &[("lib.rs", "pub fn run() {}")],
            &[(
                "perm_test.rs",
                "use std::os::unix::fs::PermissionsExt;\nfn check() { assert_eq!(p.mode(), 0o755); }",
            )],
        );
        // Should pass (G68-prod level)
        assert!(validate(&path, false).is_ok());
    }
}
