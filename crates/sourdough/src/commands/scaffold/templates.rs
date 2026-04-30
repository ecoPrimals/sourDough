//! Inlined primal DNA templates — the offspring is self-contained after budding.
//!
//! These templates are the genetic material that sourDough passes to new primals.
//! Each scaffolded primal receives its own copy of core traits, types, and patterns
//! with zero runtime dependency on sourDough.

/// Generate the core crate `Cargo.toml` for a scaffolded primal.
pub(super) fn core_cargo_toml(core_crate_name: &str, name: &str) -> String {
    format!(
        r#"[package]
name = "{core_crate_name}"
description = "Core library for {name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[lints]
workspace = true

[dependencies]
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
thiserror = {{ workspace = true }}
tracing = {{ workspace = true }}

[dev-dependencies]
tokio = {{ workspace = true, features = ["test-util"] }}
"#,
    )
}

/// Generate the core `lib.rs` with a starter primal implementation.
pub(super) fn lib_rs(name: &str) -> String {
    let type_name = super::primal_rust_type_name(name);
    format!(
        r#"//! # {name} Core
//!
//! Core library for the {name} primal.
//!
//! Self-contained: all primal DNA (traits, types, patterns) is defined here.
//! This primal discovers other primals at runtime via JSON-RPC 2.0 IPC.

pub mod error;
pub mod health;
pub mod lifecycle;

pub use error::{{PrimalError, PrimalResult}};
pub use health::{{HealthReport, HealthStatus, PrimalHealth}};
pub use lifecycle::{{PrimalLifecycle, PrimalState}};

/// The {name} primal.
pub struct {type_name}Primal {{
    state: PrimalState,
}}

impl {type_name}Primal {{
    /// Create a new primal instance.
    #[must_use]
    pub fn new() -> Self {{
        Self {{
            state: PrimalState::Created,
        }}
    }}
}}

impl Default for {type_name}Primal {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl PrimalLifecycle for {type_name}Primal {{
    fn state(&self) -> PrimalState {{
        self.state
    }}

    async fn start(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_start() {{
            return Err(PrimalError::lifecycle("cannot start from current state"));
        }}
        self.state = PrimalState::Running;
        Ok(())
    }}

    async fn stop(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_stop() {{
            return Err(PrimalError::lifecycle("cannot stop from current state"));
        }}
        self.state = PrimalState::Stopped;
        Ok(())
    }}
}}

impl PrimalHealth for {type_name}Primal {{
    fn health_status(&self) -> HealthStatus {{
        if self.state.is_running() {{
            HealthStatus::Healthy
        }} else {{
            HealthStatus::Unknown
        }}
    }}

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {{
        Ok(HealthReport::new("{name}", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_lifecycle() {{
        let mut primal = {type_name}Primal::new();
        assert_eq!(primal.state(), PrimalState::Created);

        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);

        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
    }}

    #[tokio::test]
    async fn test_health() {{
        let mut primal = {type_name}Primal::new();
        primal.start().await.unwrap();

        assert!(primal.health_status().is_healthy());

        let report = primal.health_check().await.unwrap();
        assert_eq!(report.name, "{name}");
    }}
}}
"#,
    )
}

pub(super) const ERROR_RS: &str = r#"//! Common error types for this primal.
//!
//! Extend this enum with domain-specific variants as your primal evolves.

use thiserror::Error;

/// Result type alias for primal operations.
pub type PrimalResult<T> = Result<T, PrimalError>;

/// Common errors that any primal might encounter.
#[derive(Debug, Error)]
pub enum PrimalError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Lifecycle error (start/stop/reload).
    #[error("lifecycle error: {0}")]
    Lifecycle(String),

    /// Health check error.
    #[error("health error: {0}")]
    Health(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Timeout.
    #[error("operation timed out: {0}")]
    Timeout(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Dependency error (upstream service failed).
    #[error("dependency error: {service}: {message}")]
    Dependency {
        /// Name of the dependency that failed.
        service: String,
        /// Error message.
        message: String,
    },
}

impl PrimalError {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a lifecycle error.
    pub fn lifecycle(msg: impl Into<String>) -> Self {
        Self::Lifecycle(msg.into())
    }

    /// Create a dependency error.
    pub fn dependency(service: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Dependency {
            service: service.into(),
            message: msg.into(),
        }
    }

    /// Check if this is a retryable error.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::Dependency { .. }
        )
    }
}
"#;

pub(super) const LIFECYCLE_RS: &str = r#"//! Primal lifecycle management.
//!
//! Every primal has a lifecycle: created, running, stopped.
//! This module provides the state machine and trait for managing it.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// State of a primal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalState {
    /// Not yet started.
    Created,
    /// Starting up.
    Starting,
    /// Running normally.
    Running,
    /// Stopping.
    Stopping,
    /// Stopped.
    Stopped,
    /// Failed.
    Failed,
}

impl PrimalState {
    /// Check if the primal is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal can be started.
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(self, Self::Created | Self::Stopped | Self::Failed)
    }

    /// Check if the primal can be stopped.
    #[must_use]
    pub const fn can_stop(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for PrimalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Lifecycle trait for primals.
///
/// Implement this to define how your primal starts, stops, and reloads.
pub trait PrimalLifecycle: Send + Sync {
    /// Get the current state.
    fn state(&self) -> PrimalState;

    /// Start the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if startup fails.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Reload configuration (default: stop then start).
    ///
    /// # Errors
    ///
    /// Returns an error if reload fails.
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async {
            self.stop().await?;
            self.start().await
        }
    }

    /// Handle a shutdown signal (default: calls stop).
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async { self.stop().await }
    }
}
"#;

/// Generate the server crate `Cargo.toml`.
pub(super) fn server_cargo_toml(
    server_crate_name: &str,
    core_crate_name: &str,
    name: &str,
) -> String {
    format!(
        r#"[package]
name = "{server_crate_name}"
description = "JSON-RPC server for {name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[lints]
workspace = true

[[bin]]
name = "{name_lower}"
path = "src/main.rs"

[dependencies]
{core_crate_name} = {{ path = "../{core_crate_name}" }}
tokio = {{ workspace = true, features = ["fs"] }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
anyhow = {{ workspace = true }}
tracing = {{ workspace = true }}
tracing-subscriber = {{ workspace = true }}
clap = {{ workspace = true }}
"#,
        name_lower = name.to_lowercase(),
    )
}

/// Generate the server `main.rs` with CLI entry point.
pub(super) fn server_main_rs(name: &str) -> String {
    let type_name = super::primal_rust_type_name(name);
    let name_lower = name.to_lowercase();
    let core_ident = format!("{}_core", name_lower.replace('-', "_"));
    format!(
        r#"//! {name} server binary.
//!
//! JSON-RPC 2.0 server with capability wire standard handlers.

mod dispatch;
mod server;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "{name_lower}", about = "{name} primal server")]
struct Cli {{
    /// Family ID for socket naming (production mode).
    #[arg(long, env = "FAMILY_ID")]
    family_id: Option<String>,
}}

#[tokio::main]
async fn main() -> Result<()> {{
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let mut primal = {core_ident}::{type_name}Primal::new();
    {core_ident}::PrimalLifecycle::start(&mut primal)
        .await
        .map_err(|e| anyhow::anyhow!("startup failed: {{e}}"))?;

    tracing::info!("{name} started");

    server::run("{name_lower}", cli.family_id.as_deref(), &primal).await
}}
"#,
    )
}

/// Generate the server `server.rs` with UDS listener + first-byte peek.
pub(super) fn server_rs(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::primal_rust_type_name(name);
    format!(
        r#"//! Unix domain socket server with first-byte protocol detection.

use anyhow::Result;
use tokio::io::{{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::net::UnixListener;
use tracing::{{info, warn}};

/// Run the JSON-RPC server on a Unix domain socket.
pub async fn run(
    primal_name: &str,
    family_id: Option<&str>,
    primal: &{core_ident}::{type_name}Primal,
) -> Result<()> {{
    let socket_dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {{
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
        format!("{{runtime_dir}}/biomeos")
    }});
    tokio::fs::create_dir_all(&socket_dir).await?;

    let filename = match family_id.filter(|id| !id.is_empty() && *id != "default") {{
        Some(fid) => format!("{{primal_name}}-{{fid}}.sock"),
        None => format!("{{primal_name}}.sock"),
    }};
    let socket_path = std::path::PathBuf::from(&socket_dir).join(&filename);

    // Clean up stale socket
    let _ = tokio::fs::remove_file(&socket_path).await;

    let listener = UnixListener::bind(&socket_path)?;
    info!("Listening on {{}}", socket_path.display());

    loop {{
        let (stream, _addr) = listener.accept().await?;

        let mut reader = BufReader::new(stream);
        let first_byte = match reader.fill_buf().await {{
            Ok(buf) if !buf.is_empty() => buf[0],
            Ok(_) => continue,
            Err(e) => {{
                warn!("Connection error: {{e}}");
                continue;
            }}
        }};

        if first_byte == b'{{' {{
            // JSON-RPC 2.0
            handle_jsonrpc(reader, primal).await;
        }} else {{
            // BTSP binary framing (not yet implemented)
            warn!("BTSP connection detected — not yet implemented");
        }}
    }}
}}

async fn handle_jsonrpc(
    mut reader: BufReader<tokio::net::UnixStream>,
    primal: &{core_ident}::{type_name}Primal,
) {{
    let mut line = String::new();
    loop {{
        line.clear();
        match reader.read_line(&mut line).await {{
            Ok(0) => return,
            Err(e) => {{
                warn!("Read error: {{e}}");
                return;
            }}
            Ok(_) => {{}}
        }}

        let response = crate::dispatch::handle_request(line.trim(), primal);
        let writer = reader.get_mut();
        if let Err(e) = writer.write_all(response.as_bytes()).await {{
            warn!("Write error: {{e}}");
            return;
        }}
        if let Err(e) = writer.write_all(b"\n").await {{
            warn!("Write error: {{e}}");
            return;
        }}
    }}
}}
"#,
    )
}

/// Generate the server `dispatch.rs` with capability wire handlers.
pub(super) fn dispatch_rs(name: &str) -> String {
    format!("{}{}", dispatch_rs_core(name), dispatch_rs_tests(name),)
}

fn dispatch_rs_core(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::primal_rust_type_name(name);
    format!(
        r#"//! JSON-RPC 2.0 method dispatch with capability wire standard handlers.

use {core_ident}::PrimalHealth;

const PRIMAL_NAME: &str = "{name}";
const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

const METHODS: &[&str] = &[
    "health.liveness",
    "health.readiness",
    "health.check",
    "capabilities.list",
];

/// Dispatch a JSON-RPC request and return the response string.
pub fn handle_request(
    raw: &str,
    primal: &{core_ident}::{type_name}Primal,
) -> String {{
    let req: serde_json::Value = match serde_json::from_str(raw) {{
        Ok(v) => v,
        Err(_) => return error_response(serde_json::Value::Null, -32700, "Parse error"),
    }};

    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = req
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    let result = match method {{
        "health.liveness" => serde_json::json!({{ "alive": true }}),
        "health.readiness" => {{
            let ready = primal.is_ready();
            serde_json::json!({{
                "ready": ready,
                "capabilities": METHODS,
            }})
        }}
        "health.check" => {{
            let status = primal.health_status();
            serde_json::json!({{
                "status": format!("{{status:?}}"),
                "liveness": primal.is_live(),
                "readiness": primal.is_ready(),
            }})
        }}
        "capabilities.list" | "capability.list" => {{
            serde_json::json!({{
                "primal": PRIMAL_NAME,
                "version": PRIMAL_VERSION,
                "methods": METHODS,
                "protocol": "jsonrpc-2.0",
                "transport": ["uds"],
            }})
        }}
        _ => return error_response(id, -32601, "Method not found"),
    }};

    serde_json::json!({{
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }})
    .to_string()
}}

fn error_response(id: serde_json::Value, code: i32, message: &str) -> String {{
    serde_json::json!({{
        "jsonrpc": "2.0",
        "id": id,
        "error": {{ "code": code, "message": message }},
    }})
    .to_string()
}}
"#,
    )
}

fn dispatch_rs_tests(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::primal_rust_type_name(name);
    format!(
        r##"
#[cfg(test)]
mod tests {{
    use super::*;

    fn make_primal() -> {core_ident}::{type_name}Primal {{
        {core_ident}::{type_name}Primal::new()
    }}

    #[test]
    fn liveness_returns_alive() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":1,"method":"health.liveness"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["result"]["alive"], true);
    }}

    #[test]
    fn capabilities_list_includes_primal_and_methods() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":2,"method":"capabilities.list"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["result"]["primal"], PRIMAL_NAME);
        assert!(resp["result"]["methods"].is_array());
    }}

    #[test]
    fn unknown_method_returns_error() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":3,"method":"unknown.method"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["error"]["code"], -32601);
    }}

    #[test]
    fn invalid_json_returns_parse_error() {{
        let primal = make_primal();
        let resp: serde_json::Value =
            serde_json::from_str(&handle_request("not json", &primal)).unwrap();
        assert_eq!(resp["error"]["code"], -32700);
    }}
}}
"##,
    )
}

/// Generate `deny.toml` for a scaffolded primal.
pub(super) const DENY_TOML: &str = r#"# cargo-deny configuration
# https://embarkstudios.github.io/cargo-deny/

[graph]
targets = []
all-features = true

[advisories]
yanked = "deny"
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "Unicode-3.0",
    "Zlib",
    "BSL-1.0",
    "AGPL-3.0-or-later",
    "CC0-1.0",
]
confidence-threshold = 0.8
exceptions = []

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "warn"
highlight = "all"
allow-wildcard-paths = true

# ecoBin v3.0: C-backed crates banned from application builds.
deny = [
    { crate = "openssl-sys", wrappers = [] },
    { crate = "openssl-src", wrappers = [] },
    { crate = "native-tls", wrappers = [] },
    { crate = "aws-lc-sys", wrappers = [] },
    { crate = "cmake", wrappers = [] },
    { crate = "cc", wrappers = [] },
    { crate = "bindgen", wrappers = [] },
    { crate = "bzip2-sys", wrappers = [] },
    { crate = "curl-sys", wrappers = [] },
    { crate = "libz-sys", wrappers = [] },
    { crate = "pkg-config", wrappers = [] },
    { crate = "vcpkg", wrappers = [] },
    { crate = "zstd-sys", wrappers = [] },
    { crate = "lz4-sys", wrappers = [] },
    { crate = "libsqlite3-sys", wrappers = [] },
    { crate = "cryptoki-sys", wrappers = [] },
]
skip = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
"#;

/// Generate `ci.yml` GitHub Actions workflow.
pub(super) fn ci_yml(_name: &str) -> String {
    "name: CI\n\
     on: [push, pull_request]\n\
     concurrency:\n\
     \x20 group: ci-${{ github.ref }}\n\
     \x20 cancel-in-progress: true\n\
     jobs:\n\
     \x20 check:\n\
     \x20\x20\x20 runs-on: ubuntu-latest\n\
     \x20\x20\x20 timeout-minutes: 20\n\
     \x20\x20\x20 steps:\n\
     \x20\x20\x20\x20\x20 - uses: actions/checkout@v4\n\
     \x20\x20\x20\x20\x20 - uses: dtolnay/rust-toolchain@stable\n\
     \x20\x20\x20\x20\x20 - uses: Swatinem/rust-cache@v2\n\
     \x20\x20\x20\x20\x20 - run: cargo fmt --all -- --check\n\
     \x20\x20\x20\x20\x20 - run: cargo clippy --workspace --all-targets -- -D warnings\n\
     \x20\x20\x20\x20\x20 - run: cargo test --workspace\n"
        .to_owned()
}

/// Generate `notify-plasmidbin.yml` workflow.
pub(super) const NOTIFY_PLASMIDBIN_YML: &str = r#"name: Notify plasmidBin
on:
  push:
    branches: [main]
jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - name: Dispatch rebuild to plasmidBin
        uses: peter-evans/repository-dispatch@v3
        with:
          token: ${{ secrets.PLASMIDBIN_DISPATCH_TOKEN }}
          repository: ecoPrimals/plasmidBin
          event-type: primal-updated
          client-payload: '{"primal": "${{ github.event.repository.name }}", "sha": "${{ github.sha }}"}'
"#;

pub(super) const HEALTH_RS: &str = r"//! Health check traits for observability.
//!
//! Every primal needs to be observable. This module provides health check
//! traits usable by orchestrators, load balancers, and monitoring systems.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// Overall health status of a primal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy and ready to serve requests.
    Healthy,
    /// Unhealthy but may recover.
    Degraded {
        /// Reason for degraded status.
        reason: String,
    },
    /// Unhealthy and not serving requests.
    Unhealthy {
        /// Reason for unhealthy status.
        reason: String,
    },
    /// Health unknown (e.g., startup in progress).
    Unknown,
}

impl HealthStatus {
    /// Check if the status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status allows serving requests.
    #[must_use]
    pub const fn is_serving(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }
}

/// Health report for a primal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    /// Primal name.
    pub name: String,
    /// Primal version.
    pub version: String,
    /// Overall status.
    pub status: HealthStatus,
    /// Liveness (is the process alive?).
    pub liveness: bool,
    /// Readiness (can it serve requests?).
    pub readiness: bool,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Unknown,
            liveness: true,
            readiness: false,
        }
    }

    /// Set status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.readiness = status.is_serving();
        self.status = status;
        self
    }
}

/// Health check trait for primals.
///
/// Implement this to provide health information about your primal.
pub trait PrimalHealth: Send + Sync {
    /// Get the current health status (quick check).
    fn health_status(&self) -> HealthStatus;

    /// Perform a full health check (may be expensive).
    ///
    /// # Errors
    ///
    /// Returns an error if the health check itself fails.
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;

    /// Check liveness (is the process alive?).
    fn is_live(&self) -> bool {
        true
    }

    /// Check readiness (can it serve requests?).
    fn is_ready(&self) -> bool {
        self.health_status().is_serving()
    }
}
";
