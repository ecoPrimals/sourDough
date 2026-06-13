//! riboCipher reference IPC server — canonical accept loop implementation.
//!
//! This module provides the reference implementation that other primal teams
//! study. It demonstrates the correct signal-first, legacy-fallback accept
//! loop pattern defined in `RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md`.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Connection arrives                                         │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │ Read first byte                                       │  │
//! │  │  ├─ 0xEC → Clear signal → read protocol_type → route │  │
//! │  │  ├─ 0xED → Mito signal → read HMAC tag → route       │  │
//! │  │  ├─ 0xEE → Nuclear signal → read payload → route     │  │
//! │  │  └─ other → WARN "DEPRECATED: unsignalled" → legacy  │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use sourdough_core::transport::ribocipher_server::{
//!     RiboCipherAcceptLoop, ConnectionRoute,
//! };
//!
//! # async fn example(listener: tokio::net::UnixListener) -> std::io::Result<()> {
//! let accept_loop = RiboCipherAcceptLoop::new("myPrimal");
//! loop {
//!     let (mut stream, _) = listener.accept().await?;
//!     let (route, _meta) = accept_loop.detect(&mut stream).await?;
//!     match route {
//!         ConnectionRoute::JsonRpc(()) => { /* handle JSON-RPC on stream */ }
//!         ConnectionRoute::BtspBinary(()) => { /* handle BTSP on stream */ }
//!         ConnectionRoute::Probe(()) => { /* health probe on stream */ }
//!         ConnectionRoute::Http(()) => { /* HTTP handler on stream */ }
//!         _ => {}
//!     }
//! }
//! # }
//! ```

use super::ribocipher::{ProtocolType, SignalResult, SignalTier, detect_signal};
use tokio::io::AsyncRead;

/// Accept loop router that implements riboCipher signal detection.
///
/// Instantiate one per listener, call `detect()` for each connection.
pub struct RiboCipherAcceptLoop {
    primal_name: &'static str,
    /// Wave 111-112: warn on legacy. Wave 113+: reject.
    unsignalled_policy: UnsignalledPolicy,
}

/// Policy for handling connections without a riboCipher signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsignalledPolicy {
    /// Log at WARN and fall through to legacy routing (Wave 111-112).
    Warn,
    /// Return an error, reject the connection (Wave 113+).
    Reject,
}

/// The routed result after signal detection.
#[derive(Debug)]
pub enum ConnectionRoute<S> {
    /// JSON-RPC 2.0 (protocol type 0x01 or legacy `{`).
    JsonRpc(S),
    /// BTSP binary handshake (protocol type 0x02).
    BtspBinary(S),
    /// BTSP JSON-line handshake (protocol type 0x03).
    BtspJsonLine(S),
    /// HTTP/1.1 (protocol type 0x04 or legacy HTTP verb).
    Http(S),
    /// Health probe (protocol type 0x00).
    Probe(S),
    /// Encrypted resume (protocol type 0x05).
    EncryptedResume(S),
    /// Dark Forest Beacon (protocol type 0x06).
    DarkForestBeacon(S),
    /// Mesh relay (protocol type 0x07).
    MeshRelay(S),
    /// Connection rejected (unsignalled + reject policy).
    Rejected,
}

/// Metadata about a detected connection for observability.
#[derive(Clone, Debug)]
pub struct DetectionMeta {
    /// Whether the connection used a riboCipher signal.
    pub signalled: bool,
    /// The signal tier used, if signalled.
    pub tier: Option<SignalTier>,
    /// The protocol type that was routed.
    pub protocol_type: Option<ProtocolType>,
    /// Whether this was a legacy fallback.
    pub legacy: bool,
}

impl RiboCipherAcceptLoop {
    /// Create a new accept loop router.
    #[must_use]
    pub const fn new(primal_name: &'static str) -> Self {
        Self {
            primal_name,
            unsignalled_policy: UnsignalledPolicy::Warn,
        }
    }

    /// Set the policy for unsignalled connections.
    #[must_use]
    pub const fn with_policy(mut self, policy: UnsignalledPolicy) -> Self {
        self.unsignalled_policy = policy;
        self
    }

    /// Detect the riboCipher signal and route the connection.
    ///
    /// Returns the routed connection and metadata for observability.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if reading from the stream fails.
    pub async fn detect<S: AsyncRead + Unpin>(
        &self,
        stream: &mut S,
    ) -> std::io::Result<(ConnectionRoute<()>, DetectionMeta)> {
        let signal = detect_signal(stream).await?;

        match signal {
            SignalResult::Clear(protocol_type) => {
                let meta = DetectionMeta {
                    signalled: true,
                    tier: Some(SignalTier::Clear),
                    protocol_type: Some(protocol_type),
                    legacy: false,
                };
                let route = route_protocol_type(protocol_type);
                Ok((route, meta))
            }
            SignalResult::Mito { .. } => {
                let meta = DetectionMeta {
                    signalled: true,
                    tier: Some(SignalTier::Mito),
                    protocol_type: Some(ProtocolType::NdjsonRpc),
                    legacy: false,
                };
                Ok((ConnectionRoute::JsonRpc(()), meta))
            }
            SignalResult::Nuclear { .. } => {
                let meta = DetectionMeta {
                    signalled: true,
                    tier: Some(SignalTier::Nuclear),
                    protocol_type: Some(ProtocolType::NdjsonRpc),
                    legacy: false,
                };
                Ok((ConnectionRoute::JsonRpc(()), meta))
            }
            SignalResult::Legacy { first_byte } => Ok(self.handle_legacy(first_byte)),
        }
    }

    const fn handle_legacy(&self, first_byte: u8) -> (ConnectionRoute<()>, DetectionMeta) {
        let _ = self.primal_name;
        match self.unsignalled_policy {
            UnsignalledPolicy::Warn => {
                let (route, protocol_type) = legacy_route(first_byte);
                let meta = DetectionMeta {
                    signalled: false,
                    tier: None,
                    protocol_type,
                    legacy: true,
                };
                (route, meta)
            }
            UnsignalledPolicy::Reject => {
                let meta = DetectionMeta {
                    signalled: false,
                    tier: None,
                    protocol_type: None,
                    legacy: true,
                };
                (ConnectionRoute::Rejected, meta)
            }
        }
    }
}

const fn route_protocol_type(pt: ProtocolType) -> ConnectionRoute<()> {
    match pt {
        ProtocolType::Probe => ConnectionRoute::Probe(()),
        ProtocolType::NdjsonRpc => ConnectionRoute::JsonRpc(()),
        ProtocolType::BtspBinary => ConnectionRoute::BtspBinary(()),
        ProtocolType::BtspJsonLine => ConnectionRoute::BtspJsonLine(()),
        ProtocolType::Http => ConnectionRoute::Http(()),
        ProtocolType::EncryptedResume => ConnectionRoute::EncryptedResume(()),
        ProtocolType::DarkForestBeacon => ConnectionRoute::DarkForestBeacon(()),
        ProtocolType::MeshRelay => ConnectionRoute::MeshRelay(()),
    }
}

const fn legacy_route(first_byte: u8) -> (ConnectionRoute<()>, Option<ProtocolType>) {
    match first_byte {
        b'{' | b'[' => (ConnectionRoute::JsonRpc(()), Some(ProtocolType::NdjsonRpc)),
        b'G' | b'P' | b'H' | b'D' | b'O' | b'T' | b'C' => {
            (ConnectionRoute::Http(()), Some(ProtocolType::Http))
        }
        _ => (
            ConnectionRoute::BtspBinary(()),
            Some(ProtocolType::BtspBinary),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn clear_jsonrpc_signal_routes_correctly() {
        let mut data = Cursor::new(vec![0xEC, 0x01]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::JsonRpc(())));
        assert!(meta.signalled);
        assert_eq!(meta.tier, Some(SignalTier::Clear));
        assert_eq!(meta.protocol_type, Some(ProtocolType::NdjsonRpc));
        assert!(!meta.legacy);
    }

    #[tokio::test]
    async fn clear_btsp_binary_routes_correctly() {
        let mut data = Cursor::new(vec![0xEC, 0x02]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::BtspBinary(())));
        assert!(meta.signalled);
        assert_eq!(meta.protocol_type, Some(ProtocolType::BtspBinary));
    }

    #[tokio::test]
    async fn clear_probe_routes_correctly() {
        let mut data = Cursor::new(vec![0xEC, 0x00]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::Probe(())));
        assert_eq!(meta.protocol_type, Some(ProtocolType::Probe));
    }

    #[tokio::test]
    async fn clear_http_routes_correctly() {
        let mut data = Cursor::new(vec![0xEC, 0x04]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, _meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::Http(())));
    }

    #[tokio::test]
    async fn mito_signal_routes_to_jsonrpc_default() {
        let mut data = Cursor::new(vec![0xED, 0xAA, 0xBB, 0xCC, 0xDD]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::JsonRpc(())));
        assert!(meta.signalled);
        assert_eq!(meta.tier, Some(SignalTier::Mito));
    }

    #[tokio::test]
    async fn nuclear_signal_routes_to_jsonrpc_default() {
        let mut data = Cursor::new(vec![0xEE, 1, 2, 3, 4, 5, 6]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::JsonRpc(())));
        assert!(meta.signalled);
        assert_eq!(meta.tier, Some(SignalTier::Nuclear));
    }

    #[tokio::test]
    async fn legacy_json_warns_and_routes() {
        let mut data = Cursor::new(vec![b'{']);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::JsonRpc(())));
        assert!(!meta.signalled);
        assert!(meta.legacy);
        assert_eq!(meta.protocol_type, Some(ProtocolType::NdjsonRpc));
    }

    #[tokio::test]
    async fn legacy_http_verb_routes() {
        for first in [b'G', b'P', b'H', b'D', b'O', b'T', b'C'] {
            let mut data = Cursor::new(vec![first]);
            let accept_loop = RiboCipherAcceptLoop::new("test");
            let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

            assert!(matches!(route, ConnectionRoute::Http(())));
            assert!(meta.legacy);
        }
    }

    #[tokio::test]
    async fn legacy_binary_fallback() {
        let mut data = Cursor::new(vec![0x01]);
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::BtspBinary(())));
        assert!(meta.legacy);
    }

    #[tokio::test]
    async fn reject_policy_blocks_legacy() {
        let mut data = Cursor::new(vec![b'{']);
        let accept_loop = RiboCipherAcceptLoop::new("test").with_policy(UnsignalledPolicy::Reject);
        let (route, meta) = accept_loop.detect(&mut data).await.unwrap();

        assert!(matches!(route, ConnectionRoute::Rejected));
        assert!(!meta.signalled);
        assert!(meta.legacy);
    }

    #[tokio::test]
    async fn empty_stream_returns_error() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let accept_loop = RiboCipherAcceptLoop::new("test");
        let result = accept_loop.detect(&mut data).await;
        assert!(result.is_err());
    }

    #[test]
    fn all_protocol_types_route() {
        for byte in 0x00..=0x07u8 {
            let pt = ProtocolType::from_byte(byte).unwrap();
            let route = route_protocol_type(pt);
            assert!(!matches!(route, ConnectionRoute::Rejected));
        }
    }

    #[test]
    fn detection_meta_debug() {
        let meta = DetectionMeta {
            signalled: true,
            tier: Some(SignalTier::Clear),
            protocol_type: Some(ProtocolType::NdjsonRpc),
            legacy: false,
        };
        let debug = format!("{meta:?}");
        assert!(debug.contains("signalled: true"));
    }
}
