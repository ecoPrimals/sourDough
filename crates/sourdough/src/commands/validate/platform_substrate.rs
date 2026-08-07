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

/// L2: Permission operations that should use `PlatformAccess` instead.
const L2_DEISM_PATTERNS: &[(&str, &str)] = &[
    (
        "PermissionsExt",
        "raw PermissionsExt (use PlatformAccess)",
    ),
    ("set_mode(", "raw set_mode() (use PlatformAccess.apply())"),
    (
        "from_mode(",
        "raw Permissions::from_mode() (use PlatformAccess.apply())",
    ),
    ("mode()", "raw mode() query (use query_access())"),
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

    let mut l1_violations = Vec::new();
    let mut l2_violations = Vec::new();
    let mut l3_violations = Vec::new();
    let mut abstractions_found = Vec::new();

    for file in &rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(path).unwrap_or(file);
        let rel_str = rel.to_string_lossy();

        let in_test = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || rel_str.ends_with("_test.rs")
            || content.contains("#[cfg(test)]");

        let in_platform_module = rel_str.contains("platform_substrate")
            || rel_str.contains("platform/")
            || rel_str.contains("transport");

        let has_cfg_guard = content.contains("#[cfg(unix)]")
            || content.contains("#[cfg(windows)]")
            || content.contains("#[cfg(target_os")
            || content.contains("cfg!(unix)")
            || content.contains("cfg!(windows)");

        if !in_test && !in_platform_module {
            for &(pattern, desc) in L1_DEISM_PATTERNS {
                if content.contains(pattern) && !has_cfg_guard {
                    l1_violations.push(format!("  {}: {desc}", rel.display()));
                }
            }
            for &(pattern, desc) in L2_DEISM_PATTERNS {
                if content.contains(pattern) && !has_cfg_guard {
                    l2_violations.push(format!("  {}: {desc}", rel.display()));
                }
            }
            for &(pattern, desc) in L3_DEISM_PATTERNS {
                if content.contains(pattern) && !has_cfg_guard {
                    l3_violations.push(format!("  {}: {desc}", rel.display()));
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

    let total_violations = l1_violations.len() + l2_violations.len() + l3_violations.len();

    if !l1_violations.is_empty() {
        crate::warning("L1 Link violations (raw symlink instead of platform_link()):");
        for v in &l1_violations {
            println!("{v}");
        }
        println!();
    }

    if !l2_violations.is_empty() {
        crate::warning("L2 Permission violations (raw mode bits instead of PlatformAccess):");
        for v in &l2_violations {
            println!("{v}");
        }
        println!();
    }

    if !l3_violations.is_empty() {
        crate::warning("L3 Device backend violations (raw FFI instead of backend traits):");
        for v in &l3_violations {
            println!("{v}");
        }
        println!();
    }

    if total_violations == 0 {
        crate::success("No platform substrate violations found (G68 compliant)");
    } else {
        println!(
            "  {} total violation(s): {} L1, {} L2, {} L3",
            total_violations,
            l1_violations.len(),
            l2_violations.len(),
            l3_violations.len()
        );
    }

    println!();
    if abstractions_found.is_empty() {
        crate::info("No platform abstraction patterns detected (module may not need any).");
    } else {
        crate::info("Platform abstraction patterns found:");
        for a in &abstractions_found {
            println!("{a}");
        }
    }

    println!();
    let compliance = if total_violations == 0 {
        "G68"
    } else if !abstractions_found.is_empty() {
        "partial"
    } else {
        "none"
    };
    crate::info(&format!("Compliance level: {compliance}"));

    if total_violations > 0 {
        anyhow::bail!("{total_violations} G68 platform substrate violation(s) found");
    }

    Ok(())
}

fn print_json(
    l1: &[String],
    l2: &[String],
    l3: &[String],
    abstractions: &[String],
) -> Result<()> {
    let report = serde_json::json!({
        "g68_compliance": l1.is_empty() && l2.is_empty() && l3.is_empty(),
        "violations": {
            "l1_links": l1.len(),
            "l2_permissions": l2.len(),
            "l3_device_backends": l3.len(),
            "total": l1.len() + l2.len() + l3.len(),
        },
        "abstractions_detected": abstractions.len(),
        "details": {
            "l1_violations": l1,
            "l2_violations": l2,
            "l3_violations": l3,
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
    fn detects_l2_violation() {
        let (_dir, path) = temp_primal(&[(
            "lib.rs",
            "use std::os::unix::fs::PermissionsExt;\nfn perms() { p.set_mode(0o755); }",
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
    fn platform_module_is_exempt() {
        let dir = tempfile::tempdir().unwrap();
        let crate_src = dir.path().join("crates/test-core/src/platform_substrate");
        std::fs::create_dir_all(&crate_src).unwrap();
        std::fs::write(
            crate_src.join("mod.rs"),
            "use std::os::unix::fs::symlink;\nuse std::os::unix::fs::PermissionsExt;",
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
    fn json_output_works() {
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
}
