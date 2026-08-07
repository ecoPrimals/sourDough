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
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> std::io::Result<()> {
//! use sourdough_core::transport::{TransportEndpoint, connect_transport};
//!
//! let endpoint = TransportEndpoint::uds("/run/user/1000/biomeos/beardog.sock");
//! let stream = connect_transport(&endpoint).await?;
//! // Use stream for JSON-RPC without knowing the transport.
//! # Ok(())
//! # }
//! ```

mod endpoint;
mod listener;
mod peek;
pub mod ribocipher;
pub mod ribocipher_server;
mod socket;
mod stream;

pub use endpoint::TransportEndpoint;
pub use listener::{TransportListener, bind_transport};
pub use peek::{PeekedStream, Protocol, peek_protocol};
pub use ribocipher::{
    ProtocolType, SIGNAL_CLEAR, SIGNAL_MITO, SIGNAL_NUCLEAR, SignalResult, SignalTier,
    detect_signal, is_signal_byte, send_clear_signal,
};
pub use ribocipher_server::{
    ConnectionRoute, DetectionMeta, RiboCipherAcceptLoop, UnsignalledPolicy,
};
pub use socket::{resolve_socket_path, socket_path_in};
pub use stream::{TransportStream, connect_transport};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

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
        assert!(
            std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext == "sock")
        );
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

    #[tokio::test]
    async fn peek_ribocipher_clear_jsonrpc() {
        let data: &[u8] = &[0xEC, 0x01, b'{', b'"', b'j', b's'];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(
            protocol,
            Protocol::RiboCipher {
                protocol_type: ProtocolType::NdjsonRpc
            }
        );

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"{\"js");
    }

    #[tokio::test]
    async fn peek_ribocipher_clear_btsp() {
        let data: &[u8] = &[0xEC, 0x02, 0x00, 0x10];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(
            protocol,
            Protocol::RiboCipher {
                protocol_type: ProtocolType::BtspBinary
            }
        );

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, &[0x00, 0x10]);
    }

    #[tokio::test]
    async fn peek_ribocipher_mito_consumes_envelope() {
        let data: &[u8] = &[0xED, 0xAA, 0xBB, 0xCC, 0xDD, b'p', b'a', b'y'];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert!(matches!(protocol, Protocol::RiboCipher { .. }));

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"pay");
    }

    #[tokio::test]
    async fn peek_ribocipher_nuclear_consumes_envelope() {
        let data: &[u8] = &[0xEE, 1, 2, 3, 4, 5, 6, b'X'];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert!(matches!(protocol, Protocol::RiboCipher { .. }));

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"X");
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

    #[test]
    fn from_env_parses_tcp_json() {
        let json = r#"{"transport":"tcp","host":"10.0.0.5","port":7700}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("10.0.0.5", 7700));
    }

    #[test]
    fn from_env_parses_uds_json() {
        let json = r#"{"transport":"uds","path":"/run/user/1000/biomeos/myprimal.sock"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep.uds_path(), Some("/run/user/1000/biomeos/myprimal.sock"));
    }

    #[test]
    fn from_env_or_default_falls_back_without_env() {
        let ep = TransportEndpoint::from_primal_name("fallback", None);
        assert!(ep.uds_path().is_some());
        assert!(ep.uds_path().unwrap().contains("fallback"));
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        fn arb_uds_path() -> impl Strategy<Value = String> {
            "[a-z][a-z0-9_/]{1,50}\\.sock".prop_map(|s| format!("/tmp/{s}"))
        }

        fn arb_host() -> impl Strategy<Value = String> {
            prop_oneof![
                Just("127.0.0.1".to_owned()),
                Just("::1".to_owned()),
                Just("localhost".to_owned()),
                "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
            ]
        }

        fn arb_endpoint() -> impl Strategy<Value = TransportEndpoint> {
            prop_oneof![
                arb_uds_path().prop_map(TransportEndpoint::uds),
                (arb_host(), 0..=65535u16).prop_map(|(h, p)| TransportEndpoint::tcp(h, p)),
                ("[a-z]{3,10}-gate", "[a-z]{3,10}")
                    .prop_map(|(p, c)| TransportEndpoint::mesh_relay(p, c)),
            ]
        }

        proptest! {
            #[test]
            fn serde_roundtrip_preserves_endpoint(ep in arb_endpoint()) {
                let json = serde_json::to_string(&ep).unwrap();
                let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(ep, back);
            }

            #[test]
            fn display_uri_never_panics(ep in arb_endpoint()) {
                let _ = ep.display_uri();
                let _ = format!("{ep}");
            }

            #[test]
            fn transport_name_is_consistent(ep in arb_endpoint()) {
                let name = ep.transport_name();
                let json: serde_json::Value = serde_json::to_value(&ep).unwrap();
                prop_assert_eq!(json["transport"].as_str().unwrap(), name);
            }

            #[test]
            fn socket_path_always_ends_with_sock(
                name in "[a-z]{3,15}",
                family in proptest::option::of("[a-z0-9]{4,8}")
            ) {
                let path = socket_path_in("/tmp", &name, family.as_deref());
                prop_assert!(path.to_string_lossy().ends_with(".sock"));
                prop_assert!(path.to_string_lossy().contains(&name));
            }
        }
    }
}
