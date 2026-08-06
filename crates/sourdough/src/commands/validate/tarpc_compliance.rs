//! tarpc service compliance validator (G64 Cephalization).
//!
//! Audits a primal's source code for dual-protocol readiness:
//! - tarpc service trait defined in core crate
//! - tarpc listener in server crate
//! - Dual-socket endpoint convention (`.sock` + `.tarpc.sock`)

use anyhow::{Context, Result};
use std::path::Path;

/// Patterns indicating tarpc service definition.
const SERVICE_PATTERNS: &[&str] = &[
    "#[tarpc::service]",
    "tarpc::service",
    "impl tarpc::server",
    "BaseChannel::with_defaults",
];

/// Patterns indicating tarpc transport wiring.
const TRANSPORT_PATTERNS: &[&str] = &[
    "tarpc::serde_transport",
    "serde_transport::unix",
    "tokio_serde::formats::Bincode",
    ".tarpc.sock",
];

/// Patterns indicating tarpc dependency declaration.
const CARGO_PATTERNS: &[&str] = &["tarpc", "tokio-serde"];

/// Result of a tarpc compliance audit.
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "audit result fields are semantically distinct"
)]
struct TarpcAudit {
    primal_name: String,
    has_tarpc_dep: bool,
    has_service_trait: bool,
    has_transport_wiring: bool,
    has_dual_socket: bool,
    issues: Vec<String>,
}

impl TarpcAudit {
    /// Overall compliance level.
    const fn compliance_level(&self) -> &'static str {
        if self.has_service_trait && self.has_transport_wiring && self.has_dual_socket {
            "FULL"
        } else if self.has_tarpc_dep && self.has_service_trait {
            "PARTIAL"
        } else if self.has_tarpc_dep {
            "DEP_ONLY"
        } else {
            "NONE"
        }
    }
}

/// Run a tarpc compliance audit on a primal directory.
pub(super) fn validate(path: &Path, json: bool) -> Result<()> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let primal_name = path.file_name().map_or_else(
        || "unknown".to_string(),
        |n| n.to_string_lossy().to_string(),
    );

    crate::info(&format!("Auditing tarpc compliance: {primal_name}"));

    let audit = run_audit(&path, &primal_name)?;

    if json {
        print_json(&audit);
    } else {
        print_report(&audit);
    }

    Ok(())
}

fn run_audit(path: &Path, primal_name: &str) -> Result<TarpcAudit> {
    let mut audit = TarpcAudit {
        primal_name: primal_name.to_owned(),
        has_tarpc_dep: false,
        has_service_trait: false,
        has_transport_wiring: false,
        has_dual_socket: false,
        issues: Vec::new(),
    };

    check_cargo_deps(path, &mut audit)?;
    check_source_patterns(path, &mut audit)?;

    Ok(audit)
}

fn check_cargo_deps(path: &Path, audit: &mut TarpcAudit) -> Result<()> {
    let cargo_paths = find_cargo_tomls(path);

    for cargo_path in cargo_paths {
        let content = std::fs::read_to_string(&cargo_path)
            .with_context(|| format!("reading {}", cargo_path.display()))?;

        for pattern in CARGO_PATTERNS {
            if content.contains(pattern) {
                audit.has_tarpc_dep = true;
                break;
            }
        }
    }

    if !audit.has_tarpc_dep {
        audit
            .issues
            .push("No tarpc dependency found in any Cargo.toml".to_owned());
    }

    Ok(())
}

fn check_source_patterns(path: &Path, audit: &mut TarpcAudit) -> Result<()> {
    let crates_dir = path.join("crates");
    let src_dir = if crates_dir.exists() {
        crates_dir
    } else {
        path.join("src")
    };

    if !src_dir.exists() {
        audit.issues.push(format!(
            "No source directory found at {}",
            src_dir.display()
        ));
        return Ok(());
    }

    let rs_files = find_rs_files(&src_dir);

    for file_path in &rs_files {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("reading {}", file_path.display()))?;

        for pattern in SERVICE_PATTERNS {
            if content.contains(pattern) {
                audit.has_service_trait = true;
                break;
            }
        }

        for pattern in TRANSPORT_PATTERNS {
            if content.contains(pattern) {
                audit.has_transport_wiring = true;
            }
            if pattern == &".tarpc.sock" && content.contains(pattern) {
                audit.has_dual_socket = true;
            }
        }
    }

    if !audit.has_service_trait && audit.has_tarpc_dep {
        audit
            .issues
            .push("tarpc dependency present but no #[tarpc::service] trait defined".to_owned());
    }
    if audit.has_service_trait && !audit.has_transport_wiring {
        audit.issues.push(
            "Service trait defined but no transport wiring (serde_transport) found".to_owned(),
        );
    }
    if audit.has_transport_wiring && !audit.has_dual_socket {
        audit.issues.push(
            "Transport wired but no `.tarpc.sock` convention found (expected dual-socket)"
                .to_owned(),
        );
    }

    Ok(())
}

fn find_cargo_tomls(path: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();

    let root_cargo = path.join("Cargo.toml");
    if root_cargo.exists() {
        results.push(root_cargo);
    }

    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&crates_dir) {
            for entry in entries.flatten() {
                let cargo = entry.path().join("Cargo.toml");
                if cargo.exists() {
                    results.push(cargo);
                }
            }
        }
    }

    results
}

fn find_rs_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    collect_rs_files(dir, &mut results);
    results
}

fn collect_rs_files(dir: &Path, results: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, results);
        } else if path.extension().is_some_and(|e| e == "rs") {
            results.push(path);
        }
    }
}

fn print_report(audit: &TarpcAudit) {
    println!();
    crate::info(&format!(
        "tarpc Compliance: {} [{}]",
        audit.primal_name,
        audit.compliance_level()
    ));
    println!();

    let check = |label: &str, ok: bool| {
        if ok {
            crate::success(&format!("  {label}"));
        } else {
            crate::warning(&format!("  {label}"));
        }
    };

    check("tarpc dependency in Cargo.toml", audit.has_tarpc_dep);
    check("#[tarpc::service] trait defined", audit.has_service_trait);
    check(
        "serde_transport wiring (listener/connect)",
        audit.has_transport_wiring,
    );
    check(
        "Dual-socket convention (.tarpc.sock)",
        audit.has_dual_socket,
    );

    if !audit.issues.is_empty() {
        println!();
        crate::info("Issues:");
        for issue in &audit.issues {
            println!("  - {issue}");
        }
    }

    println!();
    match audit.compliance_level() {
        "FULL" => crate::success("Fully compliant with G64 cephalization dual-protocol pattern"),
        "PARTIAL" => crate::info("Partially compliant — transport wiring or dual-socket missing"),
        "DEP_ONLY" => {
            crate::warning("tarpc dependency only — service trait and wiring not yet implemented");
        }
        _ => crate::warning("No tarpc integration detected"),
    }
}

fn print_json(audit: &TarpcAudit) {
    let output = serde_json::json!({
        "primal": audit.primal_name,
        "compliance_level": audit.compliance_level(),
        "has_tarpc_dep": audit.has_tarpc_dep,
        "has_service_trait": audit.has_service_trait,
        "has_transport_wiring": audit.has_transport_wiring,
        "has_dual_socket": audit.has_dual_socket,
        "issues": audit.issues,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}
