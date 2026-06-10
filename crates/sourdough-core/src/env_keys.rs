//! Environment variable names used across the ecoPrimals ecosystem.
//!
//! Centralizes literal env var keys so production code reads them via named
//! constants instead of scattered string literals.

/// Directory for `BiomeOS` Unix domain sockets (production deployments).
pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

/// XDG Base Directory runtime path (typically `/run/user/{uid}`).
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

/// Explicit override path for the `biomeOS` neural-api socket.
pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

/// Transport endpoint injection (JSON-serialized `TransportEndpoint`).
///
/// Launchers set this to tell primals where to bind/listen. Primals never
/// self-bind in production — the transport is injected.
pub const TRANSPORT_ENDPOINT: &str = "TRANSPORT_ENDPOINT";

/// Bind mode override for platforms with restricted socket access.
///
/// Values: `"uds"` (default), `"tcp_only"` (skip UDS entirely — Android/SELinux),
/// `"both"` (bind both UDS and TCP).
///
/// When `tcp_only`, primals must not attempt `bind()` on Unix domain sockets.
/// This is required on Android/GrapheneOS where `SELinux` denies UDS bind in
/// non-standard paths.
pub const PRIMAL_BIND_MODE: &str = "PRIMAL_BIND_MODE";
