//! Sovereign CI composite command.
//!
//! Runs all relevant validators in sequence against a primal directory and
//! produces a unified pass/fail exit code. Designed for Forgejo post-receive
//! hooks and sovereign CI pipelines.
//!
//! Static checks (source analysis — run on every push):
//! - `platform-substrate` — G68 L1/L2/L3 violations
//! - `platform-paths` — hardcoded path silicon deism
//! - `neural-api` — Neural API routing compliance
//! - `tarpc` — G64 dual-protocol compliance
//! - `transport` — transport abstraction compliance
//!
//! Live checks (optional — run post-deploy):
//! - `convergence` — probe all running primals
//! - `rpc-surface` — per-primal method surface audit

use anyhow::Result;
use std::path::{Path, PathBuf};

use clap::Args;

#[derive(Args)]
pub(crate) struct CiArgs {
    /// Path to the primal directory to validate
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Also run live checks (requires running primals)
    #[arg(long)]
    pub live: bool,

    /// Socket directory for live checks
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

    // Static checks
    if !args.skip.contains(&"platform-substrate".to_owned()) {
        results.push(run_check("platform-substrate", path));
    }
    if !args.skip.contains(&"platform-paths".to_owned()) {
        results.push(run_check("platform-paths", path));
    }
    if !args.skip.contains(&"neural-api".to_owned()) {
        results.push(run_check("neural-api", path));
    }
    if !args.skip.contains(&"tarpc".to_owned()) {
        results.push(run_check("tarpc", path));
    }
    if !args.skip.contains(&"transport".to_owned()) {
        results.push(run_check("transport", path));
    }
    if !args.skip.contains(&"deps".to_owned()) {
        results.push(run_check("deps", path));
    }

    // Live checks
    if args.live {
        let socket_dir = args.socket_dir.clone().unwrap_or_else(default_socket_dir);
        results.push(run_live_convergence(&socket_dir));
    }

    // Report
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

fn run_check(name: &'static str, path: &Path) -> CheckResult {
    let supports_json = matches!(
        name,
        "platform-substrate" | "platform-paths" | "neural-api" | "tarpc" | "depot" | "ribocipher" | "deps"
    );

    let mut cmd = std::process::Command::new(
        std::env::current_exe().unwrap_or_else(|_| "sourdough".into()),
    );
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

fn run_live_convergence(socket_dir: &Path) -> CheckResult {
    let output = std::process::Command::new(std::env::current_exe().unwrap_or_else(|_| "sourdough".into()))
        .args([
            "validate",
            "convergence",
            "--socket-dir",
            &socket_dir.display().to_string(),
            "--json",
        ])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let passed = out.status.success()
                && (stdout.contains("\"CONVERGED\"") || stdout.contains("\"PARTIAL\""));
            CheckResult {
                name: "convergence",
                passed,
            }
        }
        Err(_) => CheckResult {
            name: "convergence",
            passed: false,
        },
    }
}

fn contains_prod_failures(json_output: &str) -> bool {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_output) {
        // Check platform-substrate / platform-paths style
        if let Some(prod) = v.get("production").and_then(|p| p.get("count")) {
            if prod.as_u64().unwrap_or(0) > 0 {
                return true;
            }
        }
        // Check rpc-surface / neural-api style
        if let Some(failures) = v.get("failures").and_then(|f| f.as_array()) {
            if !failures.is_empty() {
                return true;
            }
        }
        // Check compliance field
        if let Some(compliance) = v.get("compliance").and_then(|c| c.as_str()) {
            return matches!(compliance, "NONE" | "STUB" | "DIVERGED" | "BROKEN");
        }
    }
    false
}

fn default_socket_dir() -> PathBuf {
    std::env::var("BIOMEOS_SOCKET_DIR")
        .or_else(|_| std::env::var("XDG_RUNTIME_DIR").map(|d| format!("{d}/biomeos")))
        .map_or_else(|_| PathBuf::from("/tmp/biomeos"), PathBuf::from)
}

fn print_json(results: &[CheckResult], total: usize, passed: usize, failed: usize) {
    println!("{{");
    println!("  \"ci\": \"{}\",", if failed == 0 { "PASS" } else { "FAIL" });
    println!("  \"total\": {total},");
    println!("  \"passed\": {passed},");
    println!("  \"failed\": {failed},");
    println!("  \"checks\": [");
    for (i, r) in results.iter().enumerate() {
        let comma = if i + 1 < results.len() { "," } else { "" };
        let status = if r.passed { "PASS" } else { "FAIL" };
        println!(
            "    {{\"name\": \"{}\", \"status\": \"{status}\"}}{}",
            r.name, comma
        );
    }
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
