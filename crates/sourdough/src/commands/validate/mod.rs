//! Validation commands for checking primal compliance.

mod composition;
mod depot;
mod transport_report;

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

    /// Validate `ecoBin` compliance (project dir or compiled binary)
    #[command(name = "ecobin")]
    EcoBin {
        /// Path to the primal directory or compiled binary
        path: PathBuf,
    },

    /// Validate a primal composition (check binary presence)
    #[command(name = "composition")]
    Composition {
        /// Composition name (tower, node, nest, nucleus, full) or comma-separated primal list
        composition: String,

        /// Path to the primals binary directory
        #[arg(long, default_value = "primals")]
        primals_dir: PathBuf,

        /// Use triple-first layout (`primals/{triple}/{name}`)
        #[arg(long)]
        triple_first: bool,

        /// Path to a custom compositions manifest (TOML)
        #[arg(long)]
        manifest: Option<PathBuf>,
    },

    /// Validate transport abstraction compliance
    #[command(name = "transport")]
    Transport {
        /// Path to the primal directory
        path: PathBuf,
    },

    /// Check depot binary freshness (detect stale binaries)
    #[command(name = "depot")]
    Depot {
        /// Path to the depot directory (primals binary tree)
        #[arg(default_value = "primals")]
        depot_dir: PathBuf,

        /// Source directory to compare against (uses latest file mtime)
        #[arg(long)]
        source: Option<PathBuf>,

        /// Hours after which a binary is considered stale (default: 48)
        #[arg(long, default_value = "48")]
        stale_hours: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run transport compliance audit across all primals in a directory
    #[command(name = "transport-report")]
    TransportReport {
        /// Parent directory containing primal checkouts
        #[arg(long, default_value = "..")]
        primals_dir: PathBuf,

        /// Write report to file (in addition to stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Output as JSON (machine-readable for CI/depot automation)
        #[arg(long)]
        json: bool,

        /// Exempt primals (comma-separated, e.g. "biomeOS,songBird,sourDough")
        #[arg(
            long,
            value_delimiter = ',',
            default_value = "biomeOS,songBird,sourDough"
        )]
        exempt: Vec<String>,
    },
}

pub(crate) fn run(cmd: ValidateCommand) -> Result<()> {
    match cmd {
        ValidateCommand::Primal { path } => validate_primal(&path),
        ValidateCommand::UniBin { path } => validate_unibin(&path),
        ValidateCommand::EcoBin { path } => validate_ecobin(&path),
        ValidateCommand::Composition {
            composition,
            primals_dir,
            triple_first,
            manifest,
        } => composition::validate(
            &composition,
            &primals_dir,
            triple_first,
            manifest.as_deref(),
        ),
        ValidateCommand::Transport { path } => validate_transport(&path),
        ValidateCommand::Depot {
            depot_dir,
            source,
            stale_hours,
            json,
        } => depot::run(&depot_dir, source.as_deref(), stale_hours, json),
        ValidateCommand::TransportReport {
            primals_dir,
            output,
            json,
            exempt,
        } => transport_report::run(&primals_dir, output.as_deref(), json, &exempt),
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
    if path.is_file() {
        return validate_ecobin_binary(path);
    }
    validate_ecobin_project(path)
}

fn validate_ecobin_binary(path: &Path) -> Result<()> {
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

    println!();
    report_results(&errors, &warnings)
}

fn validate_ecobin_project(path: &Path) -> Result<()> {
    crate::info(&format!("Validating ecoBin project: {}", path.display()));
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

// --- Transport compliance checks ---

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
    ("TRANSPORT_ENDPOINT", "accepts injected endpoint env var"),
    ("ipc.resolve", "resolves endpoints via Songbird"),
    ("ipc.register", "registers capabilities with Songbird"),
];

fn validate_transport(path: &Path) -> Result<()> {
    crate::info(&format!(
        "Validating transport abstraction compliance: {}",
        path.display()
    ));
    println!();

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut compliant: Vec<String> = Vec::new();

    let src_dir = find_source_dir(path);
    let Some(src_dir) = src_dir else {
        anyhow::bail!(
            "No source directory found at {} — expected crates/*/src/ or src/",
            path.display()
        );
    };

    crate::info("Scanning for self-binding anti-patterns...");
    let rs_files = collect_rs_files(&src_dir);

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
    crate::info("Checking platform-specific socket API usage...");
    let mut platform_issues = Vec::new();
    for file in &rs_files {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        let rel = file.strip_prefix(path).unwrap_or(file);

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

    println!();
    for c in &compliant {
        crate::success(c);
    }

    println!();
    report_results(&errors, &warnings)
}

pub(super) fn find_source_dir(path: &Path) -> Option<PathBuf> {
    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        for entry in std::fs::read_dir(&crates_dir).ok()?.flatten() {
            let src = entry.path().join("src");
            if src.exists() {
                return Some(crates_dir);
            }
        }
    }
    let src = path.join("src");
    if src.exists() {
        return Some(src);
    }
    None
}

pub(super) fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files_recursive(dir, &mut files);
    files
}

fn collect_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
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
