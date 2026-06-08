//! Migration commands for evolving existing primals toward ecosystem standards.
//!
//! Currently supports:
//! - `transport` — migrates a primal from self-binding to transport injection

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(crate) enum MigrateCommand {
    /// Migrate to transport-injected architecture
    #[command(name = "transport")]
    Transport {
        /// Path to the primal directory
        path: PathBuf,

        /// Apply changes (default: dry-run report only)
        #[arg(long)]
        apply: bool,
    },
}

pub(crate) fn run(cmd: MigrateCommand) -> Result<()> {
    match cmd {
        MigrateCommand::Transport { path, apply } => migrate_transport(&path, apply),
    }
}

/// Self-binding patterns to detect and replace.
struct BindPattern {
    detect: &'static str,
    description: &'static str,
    suggestion: &'static str,
}

const BIND_PATTERNS: &[BindPattern] = &[
    BindPattern {
        detect: "TcpListener::bind(",
        description: "hardcoded TCP listener binding",
        suggestion: "Accept TransportEndpoint and bind via transport layer",
    },
    BindPattern {
        detect: "UnixListener::bind(",
        description: "hardcoded UDS listener binding",
        suggestion: "Accept TransportEndpoint and bind via transport layer",
    },
    BindPattern {
        detect: ".bind(\"0.0.0.0",
        description: "hardcoded bind-all address",
        suggestion: "Inject address via TRANSPORT_ENDPOINT env var",
    },
    BindPattern {
        detect: ".bind(\"127.0.0.1",
        description: "hardcoded localhost binding",
        suggestion: "Inject address via TRANSPORT_ENDPOINT env var",
    },
];

/// Files to generate when applying migration.
struct MigrationPlan {
    violations: Vec<Violation>,
    missing_env_handling: bool,
    missing_transport_dep: bool,
    target_server_crate: Option<PathBuf>,
}

struct Violation {
    file: PathBuf,
    line: usize,
    pattern: &'static str,
    description: &'static str,
    suggestion: &'static str,
}

fn migrate_transport(path: &Path, apply: bool) -> Result<()> {
    crate::info(&format!(
        "Transport migration {} — {}",
        if apply { "(apply)" } else { "(dry-run)" },
        path.display()
    ));
    println!();

    let plan = analyze(path)?;

    if plan.violations.is_empty() && !plan.missing_env_handling && !plan.missing_transport_dep {
        crate::success("Primal is already transport-compliant — no migration needed");
        return Ok(());
    }

    report_plan(&plan, path);

    if !apply {
        println!();
        crate::info("Run with --apply to execute this migration plan");
        return Ok(());
    }

    execute_plan(&plan, path)?;
    println!();
    crate::success("Migration applied — run `sourdough validate transport` to verify");
    Ok(())
}

fn analyze(path: &Path) -> Result<MigrationPlan> {
    let rs_files = collect_source_files(path);

    let mut violations = Vec::new();
    let mut has_transport_endpoint = false;
    let mut has_env_handling = false;
    let mut target_server_crate: Option<PathBuf> = None;

    for file in &rs_files {
        let content =
            std::fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
        let rel = file.strip_prefix(path).unwrap_or(file);

        let rel_str = rel.to_string_lossy();
        let in_test = rel_str.contains("/tests/")
            || rel_str.starts_with("tests/")
            || rel_str.ends_with("_test.rs");

        if content.contains("TransportEndpoint") {
            has_transport_endpoint = true;
        }
        if content.contains("TRANSPORT_ENDPOINT") {
            has_env_handling = true;
        }

        if in_test {
            continue;
        }

        if (content.contains("fn main") || content.contains("tokio::main"))
            && target_server_crate.is_none()
        {
            target_server_crate = file.parent().map(Path::to_path_buf);
        }

        for (line_num, line) in content.lines().enumerate() {
            for pattern in BIND_PATTERNS {
                if line.contains(pattern.detect) {
                    violations.push(Violation {
                        file: rel.to_path_buf(),
                        line: line_num + 1,
                        pattern: pattern.detect,
                        description: pattern.description,
                        suggestion: pattern.suggestion,
                    });
                }
            }
        }
    }

    Ok(MigrationPlan {
        violations,
        missing_env_handling: !has_env_handling,
        missing_transport_dep: !has_transport_endpoint,
        target_server_crate,
    })
}

fn report_plan(plan: &MigrationPlan, root: &Path) {
    if !plan.violations.is_empty() {
        crate::warning("Self-binding violations to resolve:");
        for v in &plan.violations {
            println!(
                "  {}:{} — {} (`{}`)",
                v.file.display(),
                v.line,
                v.description,
                v.pattern
            );
            println!("    → {}", v.suggestion);
        }
        println!();
    }

    if plan.missing_transport_dep {
        crate::info("Missing: sourdough-core TransportEndpoint dependency");
        println!("    → Add `sourdough-core` to Cargo.toml or copy transport types");
        println!();
    }

    if plan.missing_env_handling {
        crate::info("Missing: TRANSPORT_ENDPOINT env var / CLI handling");
        if let Some(server) = &plan.target_server_crate {
            let target = root.join(server).join("transport_config.rs");
            println!("    → Will generate: {}", target.display());
        } else {
            println!("    → Will generate transport_config.rs in server crate");
        }
        println!();
    }

    let total = plan.violations.len()
        + usize::from(plan.missing_env_handling)
        + usize::from(plan.missing_transport_dep);
    crate::info(&format!("Migration plan: {total} action(s)"));
}

fn execute_plan(plan: &MigrationPlan, path: &Path) -> Result<()> {
    if plan.missing_env_handling {
        let target_dir = plan
            .target_server_crate
            .as_ref()
            .map_or_else(|| path.join("src"), |p| path.join(p));

        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("create dir: {}", target_dir.display()))?;

        let transport_config = target_dir.join("transport_config.rs");
        std::fs::write(&transport_config, TRANSPORT_CONFIG_TEMPLATE)
            .with_context(|| format!("write {}", transport_config.display()))?;
        crate::success(&format!("Generated: {}", transport_config.display()));

        let main_path = target_dir.join("main.rs");
        if main_path.exists() {
            let main_content = std::fs::read_to_string(&main_path)?;
            if !main_content.contains("mod transport_config") {
                let patched = format!("mod transport_config;\n{main_content}");
                std::fs::write(&main_path, patched)?;
                crate::success("Added `mod transport_config;` to main.rs");
            }
        }
    }

    if !plan.violations.is_empty() {
        crate::info("Self-binding violations require manual review:");
        for v in &plan.violations {
            println!(
                "  {}:{} — replace `{}` with transport-injected binding",
                v.file.display(),
                v.line,
                v.pattern
            );
        }
        println!();
        crate::info("See transport_config.rs for the injection pattern to adopt");
    }

    Ok(())
}

fn collect_source_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let crates_dir = path.join("crates");
    if crates_dir.exists() {
        walk_rs(&crates_dir, &mut files);
    } else {
        let src = path.join("src");
        if src.exists() {
            walk_rs(&src, &mut files);
        }
    }
    files
}

fn walk_rs(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_rs(&p, files);
        } else if p.extension().is_some_and(|e| e == "rs") {
            files.push(p);
        }
    }
}

const TRANSPORT_CONFIG_TEMPLATE: &str = r#"//! Transport configuration — injected endpoint binding.
//!
//! Primals do not choose their transport. The launcher or deployment layer
//! injects a `TRANSPORT_ENDPOINT` env var (JSON) or the primal reads it
//! from its capability registry.
//!
//! Generated by `sourdough migrate transport`.

use std::net::SocketAddr;

/// Resolved bind target for this primal.
pub enum BindTarget {
    /// Bind a Unix domain socket at this path.
    Uds(std::path::PathBuf),
    /// Bind a TCP listener at this address.
    Tcp(SocketAddr),
}

/// Parse the injected transport endpoint.
///
/// Resolution order:
/// 1. `TRANSPORT_ENDPOINT` env var (JSON: `{"transport":"uds","path":"..."}` or
///    `{"transport":"tcp","host":"...","port":...}`)
/// 2. CLI `--transport` argument (if provided)
/// 3. Platform default (UDS on Unix, TCP localhost on other platforms)
pub fn resolve_bind_target(cli_transport: Option<&str>) -> BindTarget {
    if let Ok(json) = std::env::var("TRANSPORT_ENDPOINT") {
        if let Ok(ep) = serde_json::from_str::<serde_json::Value>(&json) {
            match ep.get("transport").and_then(|t| t.as_str()) {
                Some("uds") => {
                    if let Some(path) = ep.get("path").and_then(|p| p.as_str()) {
                        return BindTarget::Uds(std::path::PathBuf::from(path));
                    }
                }
                Some("tcp") => {
                    let host = ep.get("host").and_then(|h| h.as_str()).unwrap_or("127.0.0.1");
                    let port = ep.get("port").and_then(|p| p.as_u64()).unwrap_or(0) as u16;
                    if let Ok(addr) = format!("{host}:{port}").parse() {
                        return BindTarget::Tcp(addr);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(json) = cli_transport {
        if let Ok(ep) = serde_json::from_str::<serde_json::Value>(json) {
            match ep.get("transport").and_then(|t| t.as_str()) {
                Some("uds") => {
                    if let Some(path) = ep.get("path").and_then(|p| p.as_str()) {
                        return BindTarget::Uds(std::path::PathBuf::from(path));
                    }
                }
                Some("tcp") => {
                    let host = ep.get("host").and_then(|h| h.as_str()).unwrap_or("127.0.0.1");
                    let port = ep.get("port").and_then(|p| p.as_u64()).unwrap_or(0) as u16;
                    if let Ok(addr) = format!("{host}:{port}").parse() {
                        return BindTarget::Tcp(addr);
                    }
                }
                _ => {}
            }
        }
    }

    // Platform default
    if cfg!(unix) {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        BindTarget::Uds(std::path::PathBuf::from(format!(
            "{runtime_dir}/biomeos/PRIMAL_NAME.sock"
        )))
    } else {
        BindTarget::Tcp(SocketAddr::from(([127, 0, 0, 1], 0)))
    }
}
"#;
