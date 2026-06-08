//! Transport utilities for primal socket communication.
//!
//! Provides:
//! - [`TransportEndpoint`] — the canonical way to describe how to reach a service.
//!   Wire-compatible with `songbird_types::TransportEndpoint` (same serde tagged format).
//! - [`connect_transport`] — connect to a service via its resolved endpoint.
//! - [`PeekedStream`] — first-byte protocol auto-detection (JSON-RPC vs BTSP).
//! - [`resolve_socket_path`] — ecosystem socket path resolution.
//!
//! # Transport Injection Pattern
//!
//! Primals do not choose their transport — the launcher or Songbird decides.
//! Business logic receives a `TransportEndpoint` and calls `connect_transport()`.
//!
//! ```rust,ignore
//! use sourdough_core::transport::{TransportEndpoint, connect_transport};
//!
//! let endpoint = TransportEndpoint::uds("/run/user/1000/biomeos/beardog.sock");
//! let stream = connect_transport(&endpoint).await?;
//! // Use stream for JSON-RPC without knowing the transport.
//! ```

use crate::env_keys;
use serde::{Deserialize, Serialize};

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

// ---------------------------------------------------------------------------
// TransportEndpoint — canonical service resolution type
// ---------------------------------------------------------------------------

/// Structured transport endpoint — the canonical way to describe how to reach a service.
///
/// Wire-compatible with `songbird_types::TransportEndpoint`:
/// ```json
/// { "transport": "uds", "path": "/run/membrane/beardog.sock" }
/// { "transport": "tcp", "host": "192.168.1.144", "port": 7700 }
/// { "transport": "mesh_relay", "peer_id": "strand-gate", "capability": "security" }
/// ```
///
/// Primals never choose their transport — the launcher or Songbird decides.
/// Consumers match on the variant to select the appropriate connection strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {
    /// Unix Domain Socket — local primal on same host (fastest path).
    /// Path prefixed with `@` denotes a Linux abstract namespace socket.
    #[serde(rename = "uds")]
    Uds {
        /// Filesystem path to the socket.
        path: String,
    },

    /// TCP — direct network connection (cross-host or container).
    #[serde(rename = "tcp")]
    Tcp {
        /// Host address (IPv4, IPv6, or hostname).
        host: String,
        /// TCP port number.
        port: u16,
    },

    /// Mesh relay — primal reachable via Songbird's mesh network.
    #[serde(rename = "mesh_relay")]
    MeshRelay {
        /// Mesh peer identifier (e.g. `"strand-gate"`).
        peer_id: String,
        /// Capability being resolved on the remote peer.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Construct a UDS endpoint.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {
        Self::Uds { path: path.into() }
    }

    /// Construct a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Construct a mesh relay endpoint.
    #[must_use]
    pub fn mesh_relay(peer_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self::MeshRelay {
            peer_id: peer_id.into(),
            capability: capability.into(),
        }
    }

    /// Whether this endpoint is local (same-host, no network hop).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            Self::Uds { .. } => true,
            Self::Tcp { host, .. } => host == "127.0.0.1" || host == "::1" || host == "localhost",
            Self::MeshRelay { .. } => false,
        }
    }

    /// Whether this endpoint uses relay infrastructure.
    #[must_use]
    pub fn is_relayed(&self) -> bool {
        matches!(self, Self::MeshRelay { .. })
    }

    /// Transport name as it appears in the wire format.
    #[must_use]
    pub fn transport_name(&self) -> &'static str {
        match self {
            Self::Uds { .. } => "uds",
            Self::Tcp { .. } => "tcp",
            Self::MeshRelay { .. } => "mesh_relay",
        }
    }

    /// URI-style string for logging/diagnostics (not for parsing).
    #[must_use]
    pub fn display_uri(&self) -> String {
        match self {
            Self::Uds { path } => {
                if let Some(abstract_name) = path.strip_prefix('@') {
                    format!("unix-abstract://{abstract_name}")
                } else {
                    format!("unix://{path}")
                }
            }
            Self::Tcp { host, port } => {
                if host.contains(':') {
                    format!("tcp://[{host}]:{port}")
                } else {
                    format!("tcp://{host}:{port}")
                }
            }
            Self::MeshRelay {
                peer_id,
                capability,
            } => format!("mesh://{peer_id}/{capability}"),
        }
    }

    /// Returns the socket path if this is a UDS endpoint.
    #[must_use]
    pub fn uds_path(&self) -> Option<&str> {
        match self {
            Self::Uds { path } => Some(path),
            _ => None,
        }
    }

    /// Returns `(host, port)` if this is a TCP endpoint.
    #[must_use]
    pub fn tcp_addr(&self) -> Option<(&str, u16)> {
        match self {
            Self::Tcp { host, port } => Some((host, *port)),
            _ => None,
        }
    }

    /// Returns `(peer_id, capability)` if this is a mesh relay endpoint.
    #[must_use]
    pub fn mesh_peer(&self) -> Option<(&str, &str)> {
        match self {
            Self::MeshRelay {
                peer_id,
                capability,
            } => Some((peer_id, capability)),
            _ => None,
        }
    }

    /// Build a `TransportEndpoint` from the ecosystem socket path conventions.
    ///
    /// Uses [`resolve_socket_path`] to determine the UDS path, then wraps
    /// it as `TransportEndpoint::Uds`.
    #[must_use]
    pub fn from_primal_name(primal_name: &str, family_id: Option<&str>) -> Self {
        let path = resolve_socket_path(primal_name, family_id);
        Self::Uds {
            path: path.to_string_lossy().into_owned(),
        }
    }

    /// Build a `TransportEndpoint` from the platform default.
    ///
    /// On Unix: UDS at the standard biomeos socket path.
    /// On non-Unix or when UDS is unavailable: TCP localhost fallback.
    #[must_use]
    pub fn platform_default(primal_name: &str, family_id: Option<&str>) -> Self {
        if cfg!(unix) {
            Self::from_primal_name(primal_name, family_id)
        } else {
            Self::Tcp {
                host: "127.0.0.1".to_owned(),
                port: 50000,
            }
        }
    }
}

impl fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_uri())
    }
}

// ---------------------------------------------------------------------------
// connect_transport — connect to a resolved endpoint
// ---------------------------------------------------------------------------

/// A transport-agnostic connected stream.
///
/// Business logic writes to this without knowing whether the underlying
/// connection is UDS, TCP, or relayed.
pub enum TransportStream {
    /// Connected Unix domain socket.
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    /// Connected TCP stream.
    Tcp(tokio::net::TcpStream),
}

impl AsyncRead for TransportStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a service via its resolved [`TransportEndpoint`].
///
/// Returns a [`TransportStream`] that implements `AsyncRead + AsyncWrite`,
/// ready for JSON-RPC or binary framing.
///
/// # Errors
///
/// Returns `io::Error` on connection failure. `MeshRelay` endpoints are
/// not directly connectable — they require routing through Songbird.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let addr = format!("{host}:{port}");
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "mesh relay ({peer_id}/{capability}) requires Songbird routing — \
                 use capability.call via Songbird instead of direct connect"
            ),
        )),
    }
}

// ---------------------------------------------------------------------------
// PeekedStream — protocol auto-detection
// ---------------------------------------------------------------------------

/// Stream wrapper that replays a single peeked byte before delegating.
///
/// After reading the first byte off a connection for protocol detection
/// (`{` → JSON-RPC, else BTSP binary), this wrapper makes the byte
/// available for the chosen handler to re-read.
///
/// # Example
///
/// ```no_run
/// use sourdough_core::transport::{peek_protocol, Protocol};
///
/// # async fn example(stream: tokio::net::TcpStream) -> std::io::Result<()> {
/// let (protocol, peeked) = peek_protocol(stream).await?;
/// match protocol {
///     Protocol::JsonRpc => { /* handle JSON-RPC with peeked stream */ }
///     Protocol::Binary  => { /* handle BTSP with peeked stream */ }
/// }
/// # Ok(())
/// # }
/// ```
pub struct PeekedStream<S> {
    peeked: Option<u8>,
    inner: S,
}

impl<S> PeekedStream<S> {
    /// Wrap a stream with a single pre-read byte.
    pub const fn new(inner: S, first_byte: u8) -> Self {
        Self {
            peeked: Some(first_byte),
            inner,
        }
    }

    /// Access the inner stream.
    pub const fn inner(&self) -> &S {
        &self.inner
    }

    /// Consume the wrapper, returning the inner stream.
    ///
    /// Any un-read peeked byte is lost.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PeekedStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if let Some(byte) = this.peeked.take() {
            buf.put_slice(&[byte]);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PeekedStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

/// Detected wire protocol from first-byte peek.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    /// JSON-RPC 2.0 (first byte is `{`, 0x7B).
    JsonRpc,
    /// Binary framing (BTSP or other).
    Binary,
}

/// Peek the first byte of a stream to determine the wire protocol.
///
/// Returns the detected [`Protocol`] and a [`PeekedStream`] that replays
/// the consumed byte transparently.
///
/// # Errors
///
/// Returns `io::Error` if the read fails or the connection is closed
/// before any data arrives.
pub async fn peek_protocol<S: AsyncRead + Unpin>(
    mut stream: S,
) -> std::io::Result<(Protocol, PeekedStream<S>)> {
    use tokio::io::AsyncReadExt;

    let mut byte = [0u8; 1];
    let n = stream.read(&mut byte).await?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "connection closed before first byte",
        ));
    }

    let protocol = if byte[0] == b'{' {
        Protocol::JsonRpc
    } else {
        Protocol::Binary
    };

    Ok((protocol, PeekedStream::new(stream, byte[0])))
}

/// Resolve the socket path for a primal using ecosystem conventions.
///
/// Path: `$BIOMEOS_SOCKET_DIR/{name}-{family_id}.sock` (production with family),
/// or `$XDG_RUNTIME_DIR/biomeos/{name}.sock` (development, no family ID).
///
/// Falls back to `/tmp/biomeos/` if neither env var is set.
#[must_use]
pub fn resolve_socket_path(primal_name: &str, family_id: Option<&str>) -> std::path::PathBuf {
    let socket_dir = std::env::var(env_keys::BIOMEOS_SOCKET_DIR).unwrap_or_else(|_| {
        let runtime_dir =
            std::env::var(env_keys::XDG_RUNTIME_DIR).unwrap_or_else(|_| "/tmp".to_owned());
        format!("{runtime_dir}/biomeos")
    });

    socket_path_in(&socket_dir, primal_name, family_id)
}

/// Build a socket path from explicit components (no env var reads).
#[must_use]
pub fn socket_path_in(
    socket_dir: &str,
    primal_name: &str,
    family_id: Option<&str>,
) -> std::path::PathBuf {
    let filename = family_id
        .filter(|id| !id.is_empty() && *id != "default")
        .map_or_else(
            || format!("{primal_name}.sock"),
            |fid| format!("{primal_name}-{fid}.sock"),
        );

    std::path::PathBuf::from(socket_dir).join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    // --- TransportEndpoint tests ---

    #[test]
    fn uds_serializes_tagged() {
        let ep = TransportEndpoint::uds("/run/membrane/beardog.sock");
        let json = serde_json::to_value(&ep).expect("serialize");
        assert_eq!(json["transport"], "uds");
        assert_eq!(json["path"], "/run/membrane/beardog.sock");
        assert!(json.get("host").is_none());
    }

    #[test]
    fn tcp_serializes_tagged() {
        let ep = TransportEndpoint::tcp("192.168.1.144", 7700);
        let json = serde_json::to_value(&ep).expect("serialize");
        assert_eq!(json["transport"], "tcp");
        assert_eq!(json["host"], "192.168.1.144");
        assert_eq!(json["port"], 7700);
    }

    #[test]
    fn mesh_relay_serializes_tagged() {
        let ep = TransportEndpoint::mesh_relay("strand-gate", "security");
        let json = serde_json::to_value(&ep).expect("serialize");
        assert_eq!(json["transport"], "mesh_relay");
        assert_eq!(json["peer_id"], "strand-gate");
        assert_eq!(json["capability"], "security");
    }

    #[test]
    fn round_trips_all_variants() {
        for ep in [
            TransportEndpoint::uds("/tmp/test.sock"),
            TransportEndpoint::tcp("10.0.0.1", 8080),
            TransportEndpoint::mesh_relay("east-gate", "crypto"),
        ] {
            let json_str = serde_json::to_string(&ep).expect("serialize");
            let de: TransportEndpoint = serde_json::from_str(&json_str).expect("deserialize");
            assert_eq!(ep, de);
        }
    }

    #[test]
    fn deserializes_from_songbird_wire_format() {
        let uds: TransportEndpoint =
            serde_json::from_str(r#"{"transport":"uds","path":"/run/membrane/beardog.sock"}"#)
                .expect("uds wire");
        assert_eq!(uds, TransportEndpoint::uds("/run/membrane/beardog.sock"));

        let tcp: TransportEndpoint =
            serde_json::from_str(r#"{"transport":"tcp","host":"192.168.1.144","port":7700}"#)
                .expect("tcp wire");
        assert_eq!(tcp, TransportEndpoint::tcp("192.168.1.144", 7700));

        let relay: TransportEndpoint = serde_json::from_str(
            r#"{"transport":"mesh_relay","peer_id":"strand-gate","capability":"security"}"#,
        )
        .expect("relay wire");
        assert_eq!(
            relay,
            TransportEndpoint::mesh_relay("strand-gate", "security")
        );
    }

    #[test]
    fn is_local_classification() {
        assert!(TransportEndpoint::uds("/tmp/test.sock").is_local());
        assert!(TransportEndpoint::tcp("127.0.0.1", 80).is_local());
        assert!(TransportEndpoint::tcp("::1", 80).is_local());
        assert!(TransportEndpoint::tcp("localhost", 80).is_local());
        assert!(!TransportEndpoint::tcp("192.168.1.5", 7700).is_local());
        assert!(!TransportEndpoint::mesh_relay("peer", "cap").is_local());
    }

    #[test]
    fn is_relayed_classification() {
        assert!(!TransportEndpoint::uds("/x").is_relayed());
        assert!(!TransportEndpoint::tcp("h", 1).is_relayed());
        assert!(TransportEndpoint::mesh_relay("p", "c").is_relayed());
    }

    #[test]
    fn display_uri_formats() {
        assert_eq!(
            TransportEndpoint::uds("/run/test.sock").display_uri(),
            "unix:///run/test.sock"
        );
        assert_eq!(
            TransportEndpoint::uds("@abstract-name").display_uri(),
            "unix-abstract://abstract-name"
        );
        assert_eq!(
            TransportEndpoint::tcp("10.0.0.1", 7700).display_uri(),
            "tcp://10.0.0.1:7700"
        );
        assert_eq!(
            TransportEndpoint::tcp("::1", 8080).display_uri(),
            "tcp://[::1]:8080"
        );
        assert_eq!(
            TransportEndpoint::mesh_relay("east-gate", "crypto").display_uri(),
            "mesh://east-gate/crypto"
        );
    }

    #[test]
    fn accessor_methods() {
        let uds = TransportEndpoint::uds("/tmp/sock");
        assert_eq!(uds.uds_path(), Some("/tmp/sock"));
        assert_eq!(uds.tcp_addr(), None);
        assert_eq!(uds.mesh_peer(), None);

        let tcp = TransportEndpoint::tcp("host", 99);
        assert_eq!(tcp.uds_path(), None);
        assert_eq!(tcp.tcp_addr(), Some(("host", 99)));
        assert_eq!(tcp.mesh_peer(), None);

        let relay = TransportEndpoint::mesh_relay("p", "c");
        assert_eq!(relay.uds_path(), None);
        assert_eq!(relay.tcp_addr(), None);
        assert_eq!(relay.mesh_peer(), Some(("p", "c")));
    }

    #[test]
    fn transport_name_matches_wire() {
        assert_eq!(TransportEndpoint::uds("/x").transport_name(), "uds");
        assert_eq!(TransportEndpoint::tcp("h", 1).transport_name(), "tcp");
        assert_eq!(
            TransportEndpoint::mesh_relay("p", "c").transport_name(),
            "mesh_relay"
        );
    }

    #[test]
    fn display_trait_matches_display_uri() {
        let ep = TransportEndpoint::tcp("host.example", 443);
        assert_eq!(format!("{ep}"), ep.display_uri());
    }

    #[test]
    fn from_primal_name_uses_socket_conventions() {
        let ep = TransportEndpoint::from_primal_name("beardog", None);
        assert!(matches!(ep, TransportEndpoint::Uds { .. }));
        let path = ep.uds_path().expect("uds path");
        assert!(path.contains("beardog"));
        assert!(path.ends_with(".sock"));
    }

    #[test]
    fn hash_impl_deduplicates() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TransportEndpoint::uds("/a"));
        set.insert(TransportEndpoint::uds("/a"));
        set.insert(TransportEndpoint::tcp("h", 1));
        assert_eq!(set.len(), 2);
    }

    // --- PeekedStream tests ---

    #[tokio::test]
    async fn peeked_stream_replays_json_rpc_byte() {
        let data: &[u8] = b"{\"jsonrpc\":\"2.0\"}";
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(protocol, Protocol::JsonRpc);

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn peeked_stream_replays_binary_byte() {
        let data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(protocol, Protocol::Binary);

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn peek_empty_stream_returns_eof() {
        let data: &[u8] = b"";
        let result = peek_protocol(data).await;
        assert!(result.is_err());
    }

    #[test]
    fn socket_path_with_family_id() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", Some("abc123"));
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal-abc123.sock")
        );
    }

    #[test]
    fn socket_path_without_family_id() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", None);
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal.sock")
        );
    }

    #[test]
    fn socket_path_default_family_id_ignored() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", Some("default"));
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal.sock")
        );
    }

    #[test]
    fn socket_path_empty_family_id_ignored() {
        let path = socket_path_in("/tmp/biomeos", "myprimal", Some(""));
        assert_eq!(path, std::path::PathBuf::from("/tmp/biomeos/myprimal.sock"));
    }
}
