//! ecoBin validation — binary and project compliance checks.
//!
//! Validates that a primal binary or project meets ecoBin requirements:
//! statically linked, no C dependencies, properly formatted, lint-free.

use anyhow::{Context, Result};
use std::path::Path;

/// Validate an ecoBin artifact — routes to binary or project validation.
pub(crate) fn validate(path: &Path) -> Result<()> {
    if path.is_file() {
        validate_binary(path)
    } else {
        validate_project(path)
    }
}

fn validate_binary(path: &Path) -> Result<()> {
    crate::info(&format!("Validating ecoBin binary: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let metadata = std::fs::metadata(path).context("Cannot read file metadata")?;
    let size_bytes = metadata.len();
    let size_kb = size_bytes / 1024;
    let budget_mb: u64 = 50;

    crate::info(&format!("Size: {size_kb} KB"));
    if size_bytes > budget_mb * 1024 * 1024 {
        errors.push(format!(
            "Binary too large: {size_kb} KB (budget: {budget_mb} MB)"
        ));
    } else {
        crate::success(&format!(
            "Size within budget ({size_kb} KB < {budget_mb} MB)"
        ));
    }

    check_linking(path, &mut errors, &mut warnings);
    check_ldd(path, &mut errors, &mut warnings);

    println!();
    super::report_results(&errors, &warnings)
}

fn check_linking(path: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    match std::process::Command::new("file").arg(path).output() {
        Ok(out) if out.status.success() => {
            let desc = String::from_utf8_lossy(&out.stdout);
            if desc.contains("statically linked") {
                crate::success("Statically linked");
            } else if desc.contains("dynamically linked") {
                errors.push("Dynamically linked (ecoBin requires static)".into());
            } else {
                warnings.push("Could not determine linking type from `file` output".into());
            }

            if desc.contains("stripped") {
                crate::success("Stripped");
            } else if desc.contains("not stripped") {
                warnings.push("Binary is not stripped (release builds should strip)".into());
            }

            if desc.contains("ELF") {
                crate::success("ELF binary detected");
            }
        }
        Ok(_) => warnings.push("`file` command failed".into()),
        Err(_) => warnings.push("`file` command not available".into()),
    }
}

fn check_ldd(path: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
    match std::process::Command::new("ldd").arg(path).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stdout.contains("not a dynamic executable")
                || stderr.contains("not a dynamic executable")
                || stdout.contains("statically linked")
            {
                crate::success("ldd confirms static binary (no dynamic deps)");
            } else if out.status.success() {
                let deps: Vec<&str> = stdout.lines().collect();
                let n = deps.len();
                errors.push(format!(
                    "ldd found {n} dynamic dependencies (ecoBin must be static)"
                ));
                for dep in deps.iter().take(5) {
                    println!("    {dep}");
                }
            }
        }
        Err(_) => warnings.push("`ldd` not available (cannot verify static linking)".into()),
    }
}

fn validate_project(path: &Path) -> Result<()> {
    crate::info(&format!("Validating ecoBin project: {}", path.display()));
    println!();

    super::validate_unibin(path)?;

    println!();
    crate::info("Checking ecoBin compliance (Pure Rust)...");

    let mut errors: Vec<String> = Vec::new();

    check_dependency_tree(path, &mut errors)?;
    check_deny_toml(path, &mut errors);

    crate::info("Checking cross-compilation readiness...");
    println!("  (Full check requires building for all targets)");

    println!();
    check_formatting(path, &mut errors);

    println!();
    check_clippy(path, &mut errors);

    println!();
    super::report_results(&errors, &[])
}

fn check_dependency_tree(path: &Path, errors: &mut Vec<String>) -> Result<()> {
    crate::info("Checking dependency tree for C dependencies...");

    let output = std::process::Command::new("cargo")
        .args(["tree"])
        .current_dir(path)
        .output()
        .context("Failed to run cargo tree")?;

    if output.status.success() {
        let tree = String::from_utf8_lossy(&output.stdout);

        let c_deps = ["ring", "openssl", "libsqlite"];
        let mut found_c_deps = Vec::new();

        for dep in c_deps {
            if tree.contains(dep) {
                found_c_deps.push(dep);
            }
        }

        if found_c_deps.is_empty() {
            crate::success("No known C dependencies found");
        } else {
            for dep in found_c_deps {
                errors.push(format!("Found C dependency: {dep}"));
            }
        }
    }
    Ok(())
}

fn check_deny_toml(path: &Path, errors: &mut Vec<String>) {
    crate::info("Checking cargo-deny configuration...");
    let deny_path = path.join("deny.toml");
    if deny_path.exists() {
        let deny_content = std::fs::read_to_string(&deny_path).unwrap_or_default();
        if deny_content.contains("ring") {
            crate::success("deny.toml present with ring ban");
        } else {
            errors.push("deny.toml exists but does not ban `ring`".to_string());
        }
        if deny_content.contains("openssl-sys") || deny_content.contains("openssl") {
            crate::success("deny.toml bans OpenSSL");
        } else {
            errors.push("deny.toml does not ban OpenSSL".to_string());
        }
    } else {
        errors.push("Missing deny.toml (required for ecoBin compliance)".to_string());
    }
}

fn check_formatting(path: &Path, errors: &mut Vec<String>) {
    crate::info("Checking code formatting...");
    match std::process::Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .current_dir(path)
        .output()
    {
        Ok(out) if out.status.success() => crate::success("Code is properly formatted"),
        Ok(_) => errors.push("Code formatting issues found (run cargo fmt)".to_string()),
        Err(e) => println!("  ⚠ Could not check formatting: {e}"),
    }
}

fn check_clippy(path: &Path, errors: &mut Vec<String>) {
    crate::info("Checking clippy lints...");
    match std::process::Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(path)
        .output()
    {
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let issues: Vec<String> = stderr
                .lines()
                .filter(|line| line.contains("warning:") || line.contains("error:"))
                .map(std::string::ToString::to_string)
                .collect();
            let n = issues.len();
            errors.push(format!("Found {n} clippy issue(s)"));
            for issue in issues.iter().take(5) {
                println!("  {issue}");
            }
            if issues.len() > 5 {
                let more = issues.len() - 5;
                println!("  ... and {more} more");
            }
        }
        Ok(_) => crate::success("No clippy warnings"),
        Err(e) => println!("  ⚠ Could not run clippy: {e}"),
    }
}
