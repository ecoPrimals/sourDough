//! Validation commands for checking primal compliance.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(crate) enum ValidateCommand {
    /// Validate basic primal structure
    Primal {
        /// Path to the primal directory
        path: PathBuf,
    },

    /// Validate `UniBin` compliance
    #[command(name = "unibin")]
    UniBin {
        /// Path to the primal directory
        path: PathBuf,
    },

    /// Validate `ecoBin` compliance
    #[command(name = "ecobin")]
    EcoBin {
        /// Path to the primal directory
        path: PathBuf,
    },
}

pub(crate) fn run(cmd: ValidateCommand) -> Result<()> {
    match cmd {
        ValidateCommand::Primal { path } => validate_primal(&path),
        ValidateCommand::UniBin { path } => validate_unibin(&path),
        ValidateCommand::EcoBin { path } => validate_ecobin(&path),
    }
}

fn validate_primal(path: &Path) -> Result<()> {
    crate::info(&format!("Validating primal at: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if path.join("Cargo.toml").exists() {
        crate::success("Cargo.toml found");
    } else {
        errors.push("Missing Cargo.toml".to_string());
    }

    if path.join("specs").exists() {
        crate::success("specs/ directory found");
    } else {
        warnings.push("Missing specs/ directory".to_string());
    }

    if path.join("crates").exists() {
        crate::success("crates/ directory found");
    } else {
        errors.push("Missing crates/ directory".to_string());
    }

    if path.join("README.md").exists() {
        crate::success("README.md found");
    } else {
        warnings.push("Missing README.md".to_string());
    }

    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        let entries: Vec<_> = std::fs::read_dir(&crates_dir)?
            .filter_map(std::result::Result::ok)
            .collect();

        let has_core = entries
            .iter()
            .any(|e| e.file_name().to_string_lossy().contains("-core"));

        if has_core {
            crate::success("Core crate found");
        } else {
            warnings.push("No *-core crate found".to_string());
        }
    }

    check_trait_implementations(path)?;

    println!();
    report_results(&errors, &warnings)
}

fn validate_unibin(path: &Path) -> Result<()> {
    crate::info(&format!("Validating UniBin at: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    crate::info("Running basic primal validation...");
    validate_primal(path)?;

    println!();
    crate::info("Checking UniBin compliance...");

    let cargo_toml_path = path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let content = std::fs::read_to_string(&cargo_toml_path)?;

        if content.contains("[[bin]]") {
            let bin_count = content.matches("[[bin]]").count();
            if bin_count == 1 {
                crate::success("Single binary defined");
            } else {
                errors.push("Multiple binaries defined (should be one UniBin)".to_string());
            }
        } else {
            warnings.push("No [[bin]] section found".to_string());
        }
    }

    println!();
    report_results(&errors, &warnings)
}

fn validate_ecobin(path: &Path) -> Result<()> {
    crate::info(&format!("Validating ecoBin at: {}", path.display()));
    println!();

    validate_unibin(path)?;

    println!();
    crate::info("Checking ecoBin compliance (Pure Rust)...");

    let mut errors: Vec<String> = Vec::new();

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

    crate::info("Checking cross-compilation readiness...");
    println!("  (Full check requires building for all targets)");

    println!();
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

    println!();
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

    println!();
    report_results(&errors, &[])
}

fn check_trait_implementations(path: &Path) -> Result<()> {
    crate::info("Checking trait implementations...");

    let crates_dir = path.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&crates_dir)?
        .filter_map(std::result::Result::ok)
        .collect();

    for entry in entries {
        let name = entry.file_name();
        if name.to_string_lossy().contains("-core") {
            let lib_rs = entry.path().join("src/lib.rs");
            if lib_rs.exists() {
                let content = std::fs::read_to_string(&lib_rs)?;

                let traits_to_check = [
                    ("PrimalLifecycle", "lifecycle management"),
                    ("PrimalHealth", "health reporting"),
                    ("PrimalIdentity", "identity (via universal adapter)"),
                    ("PrimalDiscovery", "discovery (via universal adapter)"),
                ];

                for (trait_name, description) in traits_to_check {
                    if content.contains(trait_name) {
                        crate::success(&format!("  {trait_name} implemented ({description})"));
                    }
                }
            }
        }
    }

    Ok(())
}

fn report_results(errors: &[String], warnings: &[String]) -> Result<()> {
    if !errors.is_empty() {
        println!();
        crate::error("Validation errors:");
        for error in errors {
            println!("  ✗ {error}");
        }
    }

    if !warnings.is_empty() {
        println!();
        crate::warning("Warnings:");
        for warning in warnings {
            println!("  ⚠ {warning}");
        }
    }

    if errors.is_empty() {
        println!();
        if warnings.is_empty() {
            crate::success("All checks passed!");
        } else {
            crate::success("Validation passed (with warnings)");
        }
        Ok(())
    } else {
        let n = errors.len();
        anyhow::bail!("Validation failed with {n} error(s)");
    }
}
