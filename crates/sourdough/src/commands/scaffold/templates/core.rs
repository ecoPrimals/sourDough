//! Core crate templates: Cargo.toml, lib.rs, error.rs, lifecycle.rs, health.rs.
//!
//! These templates define the primal DNA — lifecycle, health, and error traits
//! that every primal inherits at scaffold time.

/// Generate the core crate `Cargo.toml` for a scaffolded primal.
pub(in crate::commands::scaffold) fn core_cargo_toml(core_crate_name: &str, name: &str) -> String {
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
tarpc = {{ workspace = true }}
tokio-serde = {{ workspace = true }}

[dev-dependencies]
tokio = {{ workspace = true, features = ["test-util"] }}
"#,
    )
}

/// Generate the core `lib.rs` with a starter primal implementation.
pub(in crate::commands::scaffold) fn lib_rs(name: &str) -> String {
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! # {name} Core
//!
//! Core library for the {name} primal.
//!
//! Self-contained: all primal DNA (traits, types, patterns) is defined here.
//! This primal discovers other primals at runtime via JSON-RPC 2.0 IPC.
//! High-performance intra-gate callers use the tarpc service (G64 cephalization).

pub mod env_keys;
pub mod error;
pub mod health;
pub mod lifecycle;
pub mod platform_paths;
pub mod platform_signal;
pub mod platform_substrate;
pub mod protocol_negotiation;
pub mod tarpc_service;
pub mod transport;

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

pub(in crate::commands::scaffold) const TRANSPORT_RS: &str = r#"//! G66 Transport Abstraction — silicon-agnostic IPC.
//!
//! Eliminates silicon deism: primals express *what* they connect to (a service,
//! a capability) without encoding *how* bytes move. `#[cfg(unix)]` lives here —
//! not scattered across business logic.
//!
//! Components:
//! - `TransportEndpoint` — where to connect (UDS / TCP / MeshRelay)
//! - `TransportStream` — the connected byte pipe (AsyncRead + AsyncWrite)
//! - `TransportListener` — server-side accept loop
//! - `connect_transport()` / `bind_transport()` — the bridges

use serde::{Deserialize, Serialize};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Where to connect — platform-neutral endpoint descriptor.
///
/// The primal never constructs this directly in production. The launcher,
/// biomeOS, or songBird injects via `TRANSPORT_ENDPOINT` env var.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    #[serde(rename = "uds")]
    Uds { path: String },
    #[serde(rename = "tcp")]
    Tcp { host: String, port: u16 },
    #[serde(rename = "mesh_relay")]
    MeshRelay { peer_id: String, capability: String },
}

impl TransportEndpoint {
    /// Platform default: UDS on Unix, TCP localhost elsewhere.
    #[must_use]
    pub fn platform_default(primal_name: &str, family_id: Option<&str>) -> Self {
        if cfg!(unix) {
            let runtime_dir = std::env::var(crate::env_keys::XDG_RUNTIME_DIR)
                .unwrap_or_else(|_| "/tmp".to_owned());
            let filename = match family_id.filter(|id| !id.is_empty() && *id != "default") {
                Some(fid) => format!("{primal_name}-{fid}.sock"),
                None => format!("{primal_name}.sock"),
            };
            Self::Uds { path: format!("{runtime_dir}/biomeos/{filename}") }
        } else {
            Self::Tcp { host: "127.0.0.1".to_owned(), port: 0 }
        }
    }

    /// Parse from `TRANSPORT_ENDPOINT` env var, falling back to platform default.
    #[must_use]
    pub fn from_env_or_default(primal_name: &str, family_id: Option<&str>) -> Self {
        std::env::var("TRANSPORT_ENDPOINT")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Self::platform_default(primal_name, family_id))
    }

    /// Whether the endpoint is local (same-host).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds { .. } => true,
            Self::Tcp { host, .. } => host == "127.0.0.1" || host == "::1" || host == "localhost",
            Self::MeshRelay { .. } => false,
        }
    }
}

/// Transport-agnostic connected stream.
#[derive(Debug)]
pub enum TransportStream {
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    Tcp(tokio::net::TcpStream),
}

impl TransportStream {
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => "uds",
            Self::Tcp(_) => "tcp",
        }
    }
}

impl AsyncRead for TransportStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a service via its endpoint.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> io::Result<TransportStream> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let stream = tokio::net::TcpStream::connect(format!("{host}:{port}")).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay { .. } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "mesh relay requires songBird routing",
        )),
    }
}

/// Transport-agnostic listener.
#[derive(Debug)]
pub enum TransportListener {
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Accept the next connection.
    pub async fn accept(&self) -> io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Unix(stream))
            }
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Tcp(stream))
            }
        }
    }

    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => "uds",
            Self::Tcp(_) => "tcp",
        }
    }

    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => true,
            Self::Tcp(l) => l.local_addr().map(|a| a.ip().is_loopback()).unwrap_or(false),
        }
    }
}

/// Bind a listener on the given endpoint.
pub async fn bind_transport(endpoint: &TransportEndpoint) -> io::Result<TransportListener> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let socket_path = std::path::PathBuf::from(path);
            if let Some(parent) = socket_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::remove_file(&socket_path);
            let listener = tokio::net::UnixListener::bind(&socket_path)?;
            Ok(TransportListener::Unix(listener))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
            Ok(TransportListener::Tcp(listener))
        }
        TransportEndpoint::MeshRelay { .. } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "mesh relay cannot be bound — register with songBird",
        )),
    }
}
"#;

pub(in crate::commands::scaffold) const PROTOCOL_NEGOTIATION_RS: &str = r#"//! G65 Protocol Negotiation — single-socket protocol selection.
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc at connection time.
//! Phase 3 of cephalization: one socket serves both protocols via negotiation.
//!
//! Wire format:
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! ```
//!
//! No `PROTOCOLS:` line = legacy client → assume JSON-RPC.

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{info, warn};

/// RPC protocol variants supported by the ecoPrimals ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 — text-based, backward-compatible default.
    #[default]
    JsonRpc,
    /// tarpc — binary, type-safe, high-performance.
    Tarpc,
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

impl IpcProtocol {
    /// Wire name used in negotiation lines.
    #[must_use]
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }

    /// Parse from wire name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" | "binary" => Some(Self::Tarpc),
            _ => None,
        }
    }

    /// All supported protocols (tarpc preferred).
    #[must_use]
    pub fn all_supported() -> Vec<Self> {
        vec![Self::Tarpc, Self::JsonRpc]
    }
}

/// Select best protocol from client's preference list.
#[must_use]
pub fn select_protocol(client_prefs: &[IpcProtocol], server_supports: &[IpcProtocol]) -> IpcProtocol {
    for proto in client_prefs {
        if server_supports.contains(proto) {
            return *proto;
        }
    }
    IpcProtocol::JsonRpc
}

/// Server-side negotiation: read client request, select best, respond.
///
/// Returns `None` if no `PROTOCOLS:` line received (backward compat).
pub async fn negotiate_server<T>(
    transport: &mut T,
    server_supported: &[IpcProtocol],
    timeout_ms: u64,
) -> Result<Option<IpcProtocol>, std::io::Error>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(transport);
    let mut line = String::new();

    let read_result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        reader.read_line(&mut line),
    )
    .await;

    match read_result {
        Ok(Ok(n)) if n > 0 => {
            if line.trim().starts_with("PROTOCOLS: ") {
                let protocols: Vec<IpcProtocol> = line
                    .trim()
                    .strip_prefix("PROTOCOLS: ")
                    .unwrap_or("")
                    .split(',')
                    .filter_map(|s| IpcProtocol::parse(s.trim()))
                    .collect();

                if protocols.is_empty() {
                    return Ok(None);
                }

                let selected = select_protocol(&protocols, server_supported);
                let response = format!("PROTOCOL: {}\n", selected.wire_name());
                reader
                    .get_mut()
                    .write_all(response.as_bytes())
                    .await?;
                reader.get_mut().flush().await?;
                info!("server negotiated: {selected}");
                Ok(Some(selected))
            } else {
                warn!("no protocol negotiation, assuming JSON-RPC");
                Ok(None)
            }
        }
        Ok(Err(e)) => {
            warn!("negotiation read error: {e}");
            Ok(None)
        }
        _ => Ok(None),
    }
}
"#;

pub(in crate::commands::scaffold) const PLATFORM_SUBSTRATE_RS: &str = r#"//! G68 Platform Substrate Abstraction — eliminate silicon deism beyond transport.
//!
//! Two abstraction layers provided here (L3 device backends are domain-specific):
//! - **L1 Links**: `platform_link()` — symlink on Unix, junction/hard-link on Windows
//! - **L2 Permissions**: `PlatformAccess` — POSIX mode bits on Unix, ACL-compatible on Windows
//!
//! The test: "Does this primal do *less* on Windows, or the *same thing differently*?"
//! If less → silicon deism. If differently → platform abstraction.

use std::io;
use std::path::Path;

// ─── L1: Links ─────────────────────────────────────────────────────────────

/// Create a platform-appropriate link from `original` to `link`.
///
/// - **Unix**: Symbolic link.
/// - **Windows**: Hard link (file) or symlink (directory).
/// - **Other**: Hard link.
pub fn platform_link(original: &Path, link: &Path) -> io::Result<()> {
    platform_link_impl(original, link)
}

#[cfg(unix)]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    if original.is_dir() {
        std::os::windows::fs::symlink_dir(original, link)
    } else {
        std::os::windows::fs::symlink_file(original, link)
            .or_else(|_| std::fs::hard_link(original, link))
    }
}

#[cfg(not(any(unix, windows)))]
fn platform_link_impl(original: &Path, link: &Path) -> io::Result<()> {
    std::fs::hard_link(original, link)
}

/// Check if a path is a symbolic link.
pub fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

// ─── L2: Permissions ───────────────────────────────────────────────────────

/// Platform-neutral access level for filesystem objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAccess {
    /// Owner-only read+write (0o600 on Unix).
    OwnerReadWrite,
    /// Owner read+write+execute (0o700 on Unix).
    OwnerFull,
    /// Owner read+write, group+other read (0o644 on Unix).
    PublicRead,
    /// Owner read+write+execute, group+other read+execute (0o755 on Unix).
    PublicExecute,
    /// No access for anyone except owner read (0o400 on Unix).
    Readonly,
    /// Custom Unix mode bits (no-op on non-Unix).
    #[cfg(unix)]
    Custom(u32),
}

impl PlatformAccess {
    /// Apply this access level to the file at `path`.
    pub fn apply(&self, path: &Path) -> io::Result<()> {
        apply_access(path, *self)
    }
}

#[cfg(unix)]
fn apply_access(path: &Path, access: PlatformAccess) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = match access {
        PlatformAccess::OwnerReadWrite => 0o600,
        PlatformAccess::OwnerFull => 0o700,
        PlatformAccess::PublicRead => 0o644,
        PlatformAccess::PublicExecute => 0o755,
        PlatformAccess::Readonly => 0o400,
        PlatformAccess::Custom(m) => m,
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn apply_access(path: &Path, access: PlatformAccess) -> io::Result<()> {
    let readonly = matches!(access, PlatformAccess::Readonly);
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_readonly(readonly);
    std::fs::set_permissions(path, perms)
}

/// Query the effective access level of a file.
pub fn query_access(path: &Path) -> io::Result<PlatformAccess> {
    query_access_impl(path)
}

#[cfg(unix)]
fn query_access_impl(path: &Path) -> io::Result<PlatformAccess> {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(path)?.permissions().mode() & 0o777;
    Ok(match mode {
        0o600 => PlatformAccess::OwnerReadWrite,
        0o700 => PlatformAccess::OwnerFull,
        0o644 => PlatformAccess::PublicRead,
        0o755 => PlatformAccess::PublicExecute,
        0o400 => PlatformAccess::Readonly,
        other => PlatformAccess::Custom(other),
    })
}

#[cfg(not(unix))]
fn query_access_impl(path: &Path) -> io::Result<PlatformAccess> {
    let perms = std::fs::metadata(path)?.permissions();
    if perms.readonly() {
        Ok(PlatformAccess::Readonly)
    } else {
        Ok(PlatformAccess::PublicRead)
    }
}

/// Ensure a directory exists with the specified access level.
pub fn ensure_dir_with_access(path: &Path, access: PlatformAccess) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    access.apply(path)
}

/// Ensure a file's parent directory exists with owner-only access.
pub fn ensure_secure_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir_with_access(parent, PlatformAccess::OwnerFull)?;
    }
    Ok(())
}
"#;

pub(in crate::commands::scaffold) const ENV_KEYS_RS: &str = r#"//! Environment variable names used across the ecoPrimals ecosystem.
//!
//! Centralizes literal env var keys so production code reads them via named
//! constants instead of scattered string literals.

/// Directory for BiomeOS Unix domain sockets (production deployments).
pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

/// XDG Base Directory runtime path (typically `/run/user/{uid}`).
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

/// Explicit override path for the biomeOS neural-api socket.
pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

/// Bind mode override for platforms with restricted socket access.
/// Values: "uds" (default), "tcp_only" (Android/SELinux), "both".
pub const PRIMAL_BIND_MODE: &str = "PRIMAL_BIND_MODE";
"#;

pub(in crate::commands::scaffold) const ERROR_RS: &str = r#"//! Common error types for this primal.
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

pub(in crate::commands::scaffold) const LIFECYCLE_RS: &str = r#"//! Primal lifecycle management.
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

pub(in crate::commands::scaffold) const HEALTH_RS: &str = r"//! Health check traits for observability.
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

pub(in crate::commands::scaffold) const PLATFORM_PATHS_RS: &str = r#"//! Platform-aware directory resolution for primal data storage.
//!
//! Resolves config, data, runtime, cache, and log directories per platform
//! without silicon deism. Supports Linux (XDG), macOS (~/Library), Windows
//! (%APPDATA%), and mobile sandbox pass-through.
//!
//! Environment overrides always win: `BIOMEOS_CONFIG_DIR`, `BIOMEOS_DATA_DIR`,
//! `BIOMEOS_RUNTIME_DIR`, `BIOMEOS_CACHE_DIR`, `BIOMEOS_LOG_DIR`.

use std::path::PathBuf;

/// Environment variable names for directory overrides.
pub mod env_overrides {
    /// Override for configuration directory.
    pub const CONFIG_DIR: &str = "BIOMEOS_CONFIG_DIR";
    /// Override for persistent data directory.
    pub const DATA_DIR: &str = "BIOMEOS_DATA_DIR";
    /// Override for runtime ephemeral directory.
    pub const RUNTIME_DIR: &str = "BIOMEOS_RUNTIME_DIR";
    /// Override for cache directory.
    pub const CACHE_DIR: &str = "BIOMEOS_CACHE_DIR";
    /// Override for log directory.
    pub const LOG_DIR: &str = "BIOMEOS_LOG_DIR";
}

const NAMESPACE: &str = "biomeos";

/// Resolved directory paths for a primal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimalDirs {
    /// Configuration files.
    pub config: PathBuf,
    /// Persistent data.
    pub data: PathBuf,
    /// Runtime ephemeral state (sockets, PID files).
    pub runtime: PathBuf,
    /// Cache (deletable without data loss).
    pub cache: PathBuf,
    /// Log files.
    pub logs: PathBuf,
}

impl PrimalDirs {
    /// Resolve all directories for the current platform.
    #[must_use]
    pub fn resolve(primal_name: &str) -> Self {
        Self {
            config: resolve_config(primal_name),
            data: resolve_data(primal_name),
            runtime: resolve_runtime(),
            cache: resolve_cache(primal_name),
            logs: resolve_logs(primal_name),
        }
    }

    /// Ensure all directories exist with appropriate permissions.
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails.
    pub fn ensure(&self) -> std::io::Result<()> {
        use super::platform_substrate::{PlatformAccess, ensure_dir_with_access};
        ensure_dir_with_access(&self.config, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.data, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.runtime, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.cache, PlatformAccess::PublicExecute)?;
        ensure_dir_with_access(&self.logs, PlatformAccess::PublicExecute)?;
        Ok(())
    }

    /// Get the socket path within the runtime directory.
    #[must_use]
    pub fn socket_path(&self, primal_name: &str) -> PathBuf {
        self.runtime.join(format!("{primal_name}.sock"))
    }

    /// Get the PID file path within the runtime directory.
    #[must_use]
    pub fn pid_path(&self, primal_name: &str) -> PathBuf {
        self.runtime.join(format!("{primal_name}.pid"))
    }
}

fn resolve_config(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::CONFIG_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_config_base().join(NAMESPACE).join(primal_name)
}

fn resolve_data(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::DATA_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_data_base().join(NAMESPACE).join(primal_name)
}

fn resolve_runtime() -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::RUNTIME_DIR) {
        return PathBuf::from(dir);
    }
    platform_runtime_base().join(NAMESPACE)
}

fn resolve_cache(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::CACHE_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_cache_base().join(NAMESPACE).join(primal_name)
}

fn resolve_logs(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::LOG_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_logs_base().join(NAMESPACE).join(primal_name)
}

#[cfg(target_os = "linux")]
fn platform_config_base() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from)
}
#[cfg(target_os = "macos")]
fn platform_config_base() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}
#[cfg(target_os = "windows")]
fn platform_config_base() -> PathBuf {
    std::env::var("APPDATA").map_or_else(|_| home_dir().join("AppData").join("Roaming"), PathBuf::from)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_config_base() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from)
}

#[cfg(target_os = "linux")]
fn platform_data_base() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map_or_else(|_| home_dir().join(".local").join("share"), PathBuf::from)
}
#[cfg(target_os = "macos")]
fn platform_data_base() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}
#[cfg(target_os = "windows")]
fn platform_data_base() -> PathBuf {
    std::env::var("APPDATA").map_or_else(|_| home_dir().join("AppData").join("Roaming"), PathBuf::from)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_data_base() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map_or_else(|_| home_dir().join(".local").join("share"), PathBuf::from)
}

#[cfg(target_os = "linux")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}
#[cfg(target_os = "macos")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}
#[cfg(target_os = "windows")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .map_or_else(|_| PathBuf::from(r"C:\Temp"), PathBuf::from)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

#[cfg(target_os = "linux")]
fn platform_cache_base() -> PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(|_| home_dir().join(".cache"), PathBuf::from)
}
#[cfg(target_os = "macos")]
fn platform_cache_base() -> PathBuf {
    home_dir().join("Library").join("Caches")
}
#[cfg(target_os = "windows")]
fn platform_cache_base() -> PathBuf {
    std::env::var("LOCALAPPDATA").map_or_else(|_| home_dir().join("AppData").join("Local"), PathBuf::from)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_cache_base() -> PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(|_| home_dir().join(".cache"), PathBuf::from)
}

#[cfg(target_os = "linux")]
fn platform_logs_base() -> PathBuf {
    std::env::var("XDG_STATE_HOME").map_or_else(|_| home_dir().join(".local").join("state"), PathBuf::from)
}
#[cfg(target_os = "macos")]
fn platform_logs_base() -> PathBuf {
    home_dir().join("Library").join("Logs")
}
#[cfg(target_os = "windows")]
fn platform_logs_base() -> PathBuf {
    std::env::var("LOCALAPPDATA").map_or_else(|_| home_dir().join("AppData").join("Local"), PathBuf::from)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_logs_base() -> PathBuf {
    std::env::var("XDG_STATE_HOME").map_or_else(|_| home_dir().join(".local").join("state"), PathBuf::from)
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}
"#;

pub(in crate::commands::scaffold) const PLATFORM_SIGNAL_RS: &str = r#"//! Platform-aware shutdown signal handling.
//!
//! Provides a single `shutdown_signal()` future that resolves when the process
//! should begin graceful shutdown, regardless of platform:
//! - **Unix** (Linux, macOS, iOS, Android, BSD): SIGTERM or SIGINT
//! - **Windows**: Ctrl+C

/// Wait for a platform-appropriate shutdown signal.
///
/// Use with `tokio::select!` to race against your main loop.
pub async fn shutdown_signal() {
    let signal = platform_signal_impl().await;
    tracing::info!(signal = %signal, "shutdown signal received");
}

/// Wait for shutdown and return the signal name.
pub async fn shutdown_signal_named() -> &'static str {
    let signal = platform_signal_impl().await;
    tracing::info!(signal = %signal, "shutdown signal received");
    signal
}

#[cfg(unix)]
async fn platform_signal_impl() -> &'static str {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint =
        signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => "SIGTERM",
        _ = sigint.recv() => "SIGINT",
    }
}

#[cfg(not(unix))]
async fn platform_signal_impl() -> &'static str {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to register Ctrl+C handler");
    "ctrl_c"
}

/// Register a shutdown hook.
///
/// The cleanup future runs after the shutdown signal is received.
pub fn on_shutdown<F, Fut>(cleanup: F) -> tokio::task::JoinHandle<()>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        shutdown_signal().await;
        cleanup().await;
    })
}
"#;
