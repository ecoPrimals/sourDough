//! Sovereign CI composite command.
//!
//! Runs all relevant validators in sequence against a primal directory and
//! produces a unified pass/fail exit code. Designed for Forgejo post-receive
//! hooks and sovereign CI pipelines.
//!
//! ## Static checks (source analysis — run on every push)
//!
//! - `platform-substrate` — G68 L1/L2/L3 violations
//! - `platform-paths` — hardcoded path silicon deism
//! - `neural-api` — Neural API routing compliance
//! - `tarpc` — G64 dual-protocol compliance
//! - `transport` — transport abstraction compliance
//! - `deps` — G72 dependency pandemic audit
//!
//! ## Live checks (optional — run post-deploy with `--live`)
//!
//! - `convergence` — probe all running primals for Neural API convergence
//! - `rpc-surface` — per-primal method surface audit via socket probing

use anyhow::Result;
use std::path::{Path, PathBuf};

use clap::Args;

/// Static validators and whether they support `--json` output.
const STATIC_CHECKS: &[(&str, bool)] = &[
    ("platform-substrate", true),
    ("platform-paths", true),
    ("neural-api", true),
    ("tarpc", true),
    ("transport", false),
    ("deps", true),
];

#[derive(Args)]
pub(crate) struct CiArgs {
    /// Path to the primal directory to validate
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Also run live checks (requires running primals with active sockets)
    #[arg(long)]
    pub live: bool,

    /// Socket directory for live checks (auto-discovered if not set)
    #[arg(long)]
    pub socket_dir: Option<PathBuf>,

    /// Output as JSON (machine-readable for CI pipelines)
    #[arg(long)]
    pub json: bool,

    /// Fail on warnings (strict mode for gated merges)
    #[arg(long)]
    pub strict: bool,

    /// Skip specific validators (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,
}

struct CheckResult {
    name: &'static str,
    passed: bool,
}

#[expect(clippy::unnecessary_wraps, reason = "CLI dispatch requires Result")]
pub(crate) fn run(args: &CiArgs) -> Result<()> {
    let path = &args.path;
    let mut results = Vec::new();

    if !args.json {
        crate::info(&format!("sourDough CI — {}", path.display()));
        println!();
    }

    for &(name, supports_json) in STATIC_CHECKS {
        if !args.skip.iter().any(|s| s == name) {
            results.push(run_check(name, supports_json, path));
        }
    }

    if args.live {
        let socket_dir = args.socket_dir.clone().unwrap_or_else(default_socket_dir);
        if !args.skip.iter().any(|s| s == "convergence") {
            results.push(run_live_check("convergence", &socket_dir));
        }
        if !args.skip.iter().any(|s| s == "rpc-surface") {
            results.push(run_live_check("rpc-surface", &socket_dir));
        }
    }

    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    if args.json {
        print_json(&results, total, passed, failed);
    } else {
        print_human(&results, total, passed, failed);
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn sourdough_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("sourdough"))
}

fn run_check(name: &'static str, supports_json: bool, path: &Path) -> CheckResult {
    let mut cmd = std::process::Command::new(sourdough_exe());
    cmd.args(["validate", name, &path.display().to_string()]);
    if supports_json {
        cmd.arg("--json");
    }

    match cmd.output() {
        Ok(out) => {
            let passed = if supports_json {
                let stdout = String::from_utf8_lossy(&out.stdout);
                out.status.success() && !contains_prod_failures(&stdout)
            } else {
                out.status.success()
            };
            CheckResult { name, passed }
        }
        Err(_) => CheckResult {
            name,
            passed: false,
        },
    }
}

fn run_live_check(name: &'static str, socket_dir: &Path) -> CheckResult {
    let output = std::process::Command::new(sourdough_exe())
        .args([
            "validate",
            name,
            "--socket-dir",
            &socket_dir.display().to_string(),
            "--json",
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let passed = out.status.success() && !contains_live_failures(&stdout);
            CheckResult { name, passed }
        }
        Err(_) => CheckResult {
            name,
            passed: false,
        },
    }
}

fn contains_prod_failures(json_output: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json_output) else {
        return false;
    };
    if let Some(prod) = v.get("production").and_then(|p| p.get("count")) {
        if prod.as_u64().unwrap_or(0) > 0 {
            return true;
        }
    }
    if let Some(failures) = v.get("failures").and_then(|f| f.as_array()) {
        if !failures.is_empty() {
            return true;
        }
    }
    if let Some(compliance) = v.get("compliance").and_then(|c| c.as_str()) {
        return matches!(compliance, "NONE" | "STUB" | "DIVERGED" | "BROKEN");
    }
    false
}

fn contains_live_failures(json_output: &str) -> bool {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json_output) else {
        return true;
    };
    // convergence: pass if CONVERGED or PARTIAL
    if let Some(status) = v.get("status").and_then(|s| s.as_str()) {
        return !matches!(status, "CONVERGED" | "PARTIAL");
    }
    // rpc-surface: pass if no "divergence" entries
    if let Some(divergences) = v.get("divergences").and_then(|d| d.as_array()) {
        return !divergences.is_empty();
    }
    // Generic: trust exit code (already checked by caller)
    false
}

fn default_socket_dir() -> PathBuf {
    std::env::var("BIOMEOS_SOCKET_DIR")
        .or_else(|_| std::env::var("XDG_RUNTIME_DIR").map(|d| format!("{d}/biomeos")))
        .map_or_else(|_| PathBuf::from("/tmp/biomeos"), PathBuf::from)
}

fn print_json(results: &[CheckResult], total: usize, passed: usize, failed: usize) {
    let status = if failed == 0 { "PASS" } else { "FAIL" };
    let checks: Vec<String> = results
        .iter()
        .map(|r| {
            let s = if r.passed { "PASS" } else { "FAIL" };
            format!("    {{\"name\": \"{}\", \"status\": \"{s}\"}}", r.name)
        })
        .collect();
    println!("{{");
    println!("  \"ci\": \"{status}\",");
    println!("  \"total\": {total},");
    println!("  \"passed\": {passed},");
    println!("  \"failed\": {failed},");
    println!("  \"checks\": [");
    println!("{}", checks.join(",\n"));
    println!("  ]");
    println!("}}");
}

fn print_human(results: &[CheckResult], total: usize, passed: usize, failed: usize) {
    for r in results {
        if r.passed {
            crate::success(r.name);
        } else {
            crate::error(r.name);
        }
    }

    println!();
    if failed == 0 {
        crate::success(&format!("CI PASS — {passed}/{total} checks passed"));
    } else {
        crate::error(&format!("CI FAIL — {failed}/{total} checks failed"));
    }
}
