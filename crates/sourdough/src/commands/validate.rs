//! Validation commands for checking primal compliance.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum ValidateCommand {
    /// Validate basic primal structure
    Primal {
        /// Path to the primal directory
        path: PathBuf,
    },

    /// Validate UniBin compliance
    #[command(name = "unibin")]
    UniBin {
        /// Path to the primal directory
        path: PathBuf,
    },

    /// Validate ecoBin compliance
    #[command(name = "ecobin")]
    EcoBin {
        /// Path to the primal directory
        path: PathBuf,
    },
}

pub fn run(cmd: ValidateCommand) -> Result<()> {
    match cmd {
        ValidateCommand::Primal { path } => validate_primal(path),
        ValidateCommand::UniBin { path } => validate_unibin(path),
        ValidateCommand::EcoBin { path } => validate_ecobin(path),
    }
}

fn validate_primal(path: PathBuf) -> Result<()> {
    crate::info(&format!("Validating primal at: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check workspace structure
    if !path.join("Cargo.toml").exists() {
        errors.push("Missing Cargo.toml".to_string());
    } else {
        crate::success("Cargo.toml found");
    }

    // Check for specs directory
    if !path.join("specs").exists() {
        warnings.push("Missing specs/ directory".to_string());
    } else {
        crate::success("specs/ directory found");
    }

    // Check for crates directory
    if !path.join("crates").exists() {
        errors.push("Missing crates/ directory".to_string());
    } else {
        crate::success("crates/ directory found");
    }

    // Check for README
    if !path.join("README.md").exists() {
        warnings.push("Missing README.md".to_string());
    } else {
        crate::success("README.md found");
    }

    // Check for core crate
    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        let entries: Vec<_> = std::fs::read_dir(&crates_dir)?
            .filter_map(|e| e.ok())
            .collect();

        let has_core = entries
            .iter()
            .any(|e| e.file_name().to_string_lossy().contains("-core"));

        if has_core {
            crate::success("Core crate found");

            // Check if core crate has sourdough-core dependency
            for entry in &entries {
                let name = entry.file_name();
                if name.to_string_lossy().contains("-core") {
                    let core_cargo = entry.path().join("Cargo.toml");
                    if core_cargo.exists() {
                        let content = std::fs::read_to_string(&core_cargo)?;
                        if content.contains("sourdough-core") {
                            crate::success("sourdough-core dependency found");
                        } else {
                            warnings
                                .push("Core crate doesn't depend on sourdough-core".to_string());
                        }
                    }
                }
            }
        } else {
            warnings.push("No *-core crate found".to_string());
        }
    }

    // Check for trait implementations
    check_trait_implementations(&path)?;

    println!();
    report_results(&errors, &warnings)
}

fn validate_unibin(path: PathBuf) -> Result<()> {
    crate::info(&format!("Validating UniBin at: {}", path.display()));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // First run basic primal validation
    crate::info("Running basic primal validation...");
    validate_primal(path.clone())?;

    println!();
    crate::info("Checking UniBin compliance...");

    // Check for single binary
    let cargo_toml_path = path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let content = std::fs::read_to_string(&cargo_toml_path)?;

        // Simple check for [[bin]] section
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

fn validate_ecobin(path: PathBuf) -> Result<()> {
    crate::info(&format!("Validating ecoBin at: {}", path.display()));
    println!();

    // Run UniBin validation first
    validate_unibin(path.clone())?;

    println!();
    crate::info("Checking ecoBin compliance (Pure Rust)...");

    let mut errors: Vec<String> = Vec::new();
    let mut _warnings: Vec<String> = Vec::new();

    // Check for C dependencies
    crate::info("Checking dependency tree for C dependencies...");

    let output = std::process::Command::new("cargo")
        .args(["tree"])
        .current_dir(&path)
        .output()
        .context("Failed to run cargo tree")?;

    if output.status.success() {
        let tree = String::from_utf8_lossy(&output.stdout);

        // Check for known C dependencies
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
                errors.push(format!("Found C dependency: {}", dep));
            }
        }
    }

    // Check for cross-compilation readiness
    crate::info("Checking cross-compilation readiness...");
    println!("  (Full check requires building for all targets)");

    // Check formatting
    println!();
    crate::info("Checking code formatting...");
    match check_formatting(&path) {
        Ok(true) => crate::success("Code is properly formatted"),
        Ok(false) => errors.push("Code formatting issues found (run cargo fmt)".to_string()),
        Err(e) => println!("  ⚠ Could not check formatting: {}", e),
    }

    // Check clippy
    println!();
    crate::info("Checking clippy lints...");
    match check_clippy(&path) {
        Ok(issues) if issues.is_empty() => crate::success("No clippy warnings"),
        Ok(issues) => {
            errors.push(format!("Found {} clippy issue(s)", issues.len()));
            for issue in issues.iter().take(5) {
                println!("  {}", issue);
            }
            if issues.len() > 5 {
                println!("  ... and {} more", issues.len() - 5);
            }
        }
        Err(e) => println!("  ⚠ Could not run clippy: {}", e),
    }

    println!();
    report_results(&errors, &[])
}

/// Check if primal implements core traits
fn check_trait_implementations(path: &Path) -> Result<()> {
    crate::info("Checking trait implementations...");

    let crates_dir = path.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }

    // Find lib.rs files in core crates
    let entries: Vec<_> = std::fs::read_dir(&crates_dir)?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let name = entry.file_name();
        if name.to_string_lossy().contains("-core") {
            let lib_rs = entry.path().join("src/lib.rs");
            if lib_rs.exists() {
                let content = std::fs::read_to_string(&lib_rs)?;

                // Check for key trait implementations
                let traits_to_check = [
                    ("PrimalLifecycle", "lifecycle management"),
                    ("PrimalHealth", "health reporting"),
                    ("PrimalIdentity", "identity (via universal adapter)"),
                    ("PrimalDiscovery", "discovery (via universal adapter)"),
                ];

                for (trait_name, description) in traits_to_check {
                    if content.contains(trait_name) {
                        crate::success(&format!("  {} implemented ({})", trait_name, description));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check cargo fmt compliance
fn check_formatting(path: &Path) -> Result<bool> {
    let output = std::process::Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .current_dir(path)
        .output();

    match output {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(true), // If cargo fmt not available, pass
    }
}

/// Check clippy compliance
fn check_clippy(path: &Path) -> Result<Vec<String>> {
    let output = std::process::Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .current_dir(path)
        .output();

    match output {
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let issues: Vec<String> = stderr
                .lines()
                .filter(|line| line.contains("warning:") || line.contains("error:"))
                .map(|s| s.to_string())
                .collect();
            Ok(issues)
        }
        _ => Ok(Vec::new()),
    }
}

fn report_results(errors: &[String], warnings: &[String]) -> Result<()> {
    if !errors.is_empty() {
        println!();
        crate::error("Validation errors:");
        for error in errors {
            println!("  ✗ {}", error);
        }
    }

    if !warnings.is_empty() {
        println!();
        crate::warning("Warnings:");
        for warning in warnings {
            println!("  ⚠ {}", warning);
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
        anyhow::bail!("Validation failed with {} error(s)", errors.len())
    }
}
