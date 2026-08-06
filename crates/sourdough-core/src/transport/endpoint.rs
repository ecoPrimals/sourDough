//! `TransportEndpoint` — the canonical way to describe how to reach a service.
//!
//! Wire-compatible with `songbird_types::TransportEndpoint` (same serde tagged format).

use crate::env_keys;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub const fn is_relayed(&self) -> bool {
        matches!(self, Self::MeshRelay { .. })
    }

    /// Transport name as it appears in the wire format.
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
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
            Self::Uds { path } => path.strip_prefix('@').map_or_else(
                || format!("unix://{path}"),
                |abstract_name| format!("unix-abstract://{abstract_name}"),
            ),
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
    /// Uses [`super::resolve_socket_path`] to determine the UDS path, then wraps
    /// it as `TransportEndpoint::Uds`.
    #[must_use]
    pub fn from_primal_name(primal_name: &str, family_id: Option<&str>) -> Self {
        let path = super::resolve_socket_path(primal_name, family_id);
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
            let port = std::env::var(env_keys::TCP_FALLBACK_PORT)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(env_keys::DEFAULT_TCP_FALLBACK_PORT);
            Self::Tcp {
                host: "127.0.0.1".to_owned(),
                port,
            }
        }
    }

    /// Parse from the `TRANSPORT_ENDPOINT` env var, falling back to UDS default.
    ///
    /// This is the canonical entry point for primals accepting injected transport:
    /// ```text
    /// TRANSPORT_ENDPOINT='{"transport":"uds","path":"/run/user/1000/biomeos/myprimal.sock"}'
    /// ```
    #[must_use]
    pub fn from_env_or_default(primal_name: &str, family_id: Option<&str>) -> Self {
        std::env::var(env_keys::TRANSPORT_ENDPOINT)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Self::from_primal_name(primal_name, family_id))
    }
}

impl fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_uri())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_default_unix_is_uds() {
        if cfg!(unix) {
            let ep = TransportEndpoint::platform_default("testprimal", None);
            assert!(matches!(ep, TransportEndpoint::Uds { .. }));
            assert!(ep.uds_path().unwrap().contains("testprimal"));
        }
    }

    #[test]
    fn valid_json_parses_to_tcp_endpoint() {
        let json = r#"{"transport":"tcp","host":"10.0.0.5","port":7700}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("10.0.0.5", 7700));
    }

    #[test]
    fn invalid_json_does_not_parse() {
        let result: Result<TransportEndpoint, _> = serde_json::from_str("not-json");
        assert!(result.is_err());
    }

    #[test]
    fn from_env_or_default_falls_back_when_no_env() {
        let ep = TransportEndpoint::from_primal_name("myprimal", None);
        assert!(ep.uds_path().is_some());
        assert!(ep.uds_path().unwrap().contains("myprimal"));
    }

    #[test]
    fn from_primal_name_produces_uds() {
        let ep = TransportEndpoint::from_primal_name("beardog", None);
        assert!(matches!(ep, TransportEndpoint::Uds { .. }));
        let path = ep.uds_path().unwrap();
        assert!(path.contains("beardog"));
        assert_eq!(
            std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str()),
            Some("sock")
        );
    }

    #[test]
    fn from_primal_name_with_family() {
        let ep = TransportEndpoint::from_primal_name("beardog", Some("abc123"));
        let path = ep.uds_path().unwrap();
        assert!(path.contains("beardog"));
        assert!(path.contains("abc123"));
    }

    #[test]
    fn uds_constructor() {
        let ep = TransportEndpoint::uds("/tmp/test.sock");
        assert_eq!(ep.uds_path(), Some("/tmp/test.sock"));
        assert_eq!(ep.tcp_addr(), None);
        assert_eq!(ep.mesh_peer(), None);
        assert!(ep.is_local());
        assert!(!ep.is_relayed());
    }

    #[test]
    fn tcp_constructor() {
        let ep = TransportEndpoint::tcp("192.168.1.5", 7700);
        assert_eq!(ep.tcp_addr(), Some(("192.168.1.5", 7700)));
        assert_eq!(ep.uds_path(), None);
        assert!(!ep.is_local());
    }

    #[test]
    fn tcp_localhost_is_local() {
        assert!(TransportEndpoint::tcp("127.0.0.1", 80).is_local());
        assert!(TransportEndpoint::tcp("::1", 443).is_local());
        assert!(TransportEndpoint::tcp("localhost", 9000).is_local());
    }

    #[test]
    fn mesh_relay_constructor() {
        let ep = TransportEndpoint::mesh_relay("east-gate", "crypto");
        assert_eq!(ep.mesh_peer(), Some(("east-gate", "crypto")));
        assert!(ep.is_relayed());
        assert!(!ep.is_local());
    }

    #[test]
    fn display_uri_uds() {
        assert_eq!(
            TransportEndpoint::uds("/run/test.sock").display_uri(),
            "unix:///run/test.sock"
        );
    }

    #[test]
    fn display_uri_abstract_uds() {
        assert_eq!(
            TransportEndpoint::uds("@abstract").display_uri(),
            "unix-abstract://abstract"
        );
    }

    #[test]
    fn display_uri_tcp_ipv4() {
        assert_eq!(
            TransportEndpoint::tcp("10.0.0.1", 8080).display_uri(),
            "tcp://10.0.0.1:8080"
        );
    }

    #[test]
    fn display_uri_tcp_ipv6() {
        assert_eq!(
            TransportEndpoint::tcp("::1", 443).display_uri(),
            "tcp://[::1]:443"
        );
    }

    #[test]
    fn display_uri_mesh() {
        assert_eq!(
            TransportEndpoint::mesh_relay("peer", "cap").display_uri(),
            "mesh://peer/cap"
        );
    }

    #[test]
    fn display_matches_display_uri() {
        let ep = TransportEndpoint::tcp("host", 1234);
        assert_eq!(format!("{ep}"), ep.display_uri());
    }

    #[test]
    fn transport_name_matches_serde_tag() {
        let uds = TransportEndpoint::uds("/x");
        let tcp = TransportEndpoint::tcp("h", 1);
        let relay = TransportEndpoint::mesh_relay("p", "c");

        assert_eq!(uds.transport_name(), "uds");
        assert_eq!(tcp.transport_name(), "tcp");
        assert_eq!(relay.transport_name(), "mesh_relay");
    }

    #[test]
    fn serde_roundtrip_all_variants() {
        let variants = vec![
            TransportEndpoint::uds("/tmp/test.sock"),
            TransportEndpoint::tcp("192.168.1.5", 7700),
            TransportEndpoint::mesh_relay("east-gate", "storage"),
        ];
        for ep in variants {
            let json = serde_json::to_string(&ep).unwrap();
            let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
            assert_eq!(ep, back);
        }
    }
}
