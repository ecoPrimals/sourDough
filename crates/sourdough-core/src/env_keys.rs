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

/// Default bind address for TCP listeners (all interfaces).
///
/// Used when a primal must fall back to TCP (e.g. `PRIMAL_BIND_MODE=tcp_only`)
/// and no explicit host is provided via `TRANSPORT_ENDPOINT`.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0";

/// Override for the mesh relay hub primal name.
///
/// Defaults to `"songbird"`. Set when the ecosystem uses a different
/// relay topology or the relay primal is renamed.
pub const MESH_RELAY_HUB: &str = "MESH_RELAY_HUB";

/// Override for the TCP fallback port on non-Unix platforms.
///
/// When UDS is unavailable and `TRANSPORT_ENDPOINT` is not set,
/// primals fall back to `127.0.0.1:{port}`. Set to `"0"` to let the
/// OS assign an ephemeral port.
pub const TCP_FALLBACK_PORT: &str = "TCP_FALLBACK_PORT";

/// Default mesh relay hub primal name (capability: routing/relay).
///
/// Primals discover each other at runtime via this relay when direct
/// transport is unavailable. The name is a convention, not a compile-time
/// coupling — override via `MESH_RELAY_HUB` env var.
pub const DEFAULT_MESH_RELAY_HUB: &str = "songbird";

/// Default socket directory name relative to the runtime directory.
///
/// Convention: `$XDG_RUNTIME_DIR/{SOCKET_DIR_NAME}/` or `/tmp/{SOCKET_DIR_NAME}/`.
pub const SOCKET_DIR_NAME: &str = "biomeos";

/// Fallback runtime directory when `XDG_RUNTIME_DIR` is not set.
pub const FALLBACK_RUNTIME_DIR: &str = "/tmp";

/// Default TCP fallback port for non-Unix platforms.
///
/// OS-assigned (0) is preferred for production. The non-zero value here
/// is only used when `TCP_FALLBACK_PORT` is not set and UDS is unavailable.
/// Primals should prefer injected transport over this fallback.
pub const DEFAULT_TCP_FALLBACK_PORT: u16 = 0;
