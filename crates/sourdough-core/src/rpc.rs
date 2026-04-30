//! RPC layer for inter-primal communication.
//!
//! This module provides RPC interfaces for primals to communicate using the
//! standard zero-copy wire format. The [`PrimalRpc`] trait defines the baseline
//! service contract; implementations may use any transport (JSON-RPC 2.0 over
//! UDS, binary framing, etc.).

use crate::{error::PrimalError, health::HealthReport, identity::Did, lifecycle::PrimalState};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Serde adapters: `serde_bytes` does not implement its traits for `bytes::Bytes` (only `Vec<u8>`,
/// `serde_bytes::ByteBuf`, etc.). These helpers keep the same on-the-wire representation as
/// `#[serde(with = "serde_bytes")]` on `Vec<u8>` / `Option<Vec<u8>>` while storing `Bytes`.
mod rpc_bytes_serde {
    use bytes::Bytes;
    use serde::{Deserializer, Serialize, Serializer};

    pub(super) fn serialize_params<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_bytes::serialize(bytes.as_ref(), serializer)
    }

    pub(super) fn deserialize_params<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_bytes::deserialize::<Vec<u8>, D>(deserializer).map(Bytes::from)
    }

    struct SerSlice<'a>(&'a [u8]);

    impl Serialize for SerSlice<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serde_bytes::serialize(self.0, serializer)
        }
    }

    #[expect(
        clippy::ref_option,
        reason = "serde serialize_with requires &Option<Bytes> signature"
    )]
    pub(super) fn serialize_result<S>(v: &Option<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            None => serializer.serialize_none(),
            Some(b) => serializer.serialize_some(&SerSlice(b.as_ref())),
        }
    }

    pub(super) fn deserialize_result<'de, D>(deserializer: D) -> Result<Option<Bytes>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = serde_bytes::deserialize::<Option<Vec<u8>>, D>(deserializer)?;
        Ok(opt.map(Bytes::from))
    }
}

/// Common RPC service that all primals must implement.
///
/// This provides the baseline interface for inter-primal communication.
/// Implementations are transport-agnostic: the same contract works over
/// JSON-RPC 2.0 (primary) or binary framing (high-throughput).
pub trait PrimalRpc {
    /// Get the primal's current health status.
    fn health(&self) -> impl std::future::Future<Output = Result<HealthReport, String>> + Send;

    /// Get the primal's current lifecycle state.
    fn state(&self) -> impl std::future::Future<Output = Result<PrimalState, String>> + Send;

    /// Get the primal's decentralized identifier (DID).
    fn did(&self) -> impl std::future::Future<Output = Result<Did, String>> + Send;

    /// Ping the primal for liveness check.
    fn ping(&self) -> impl std::future::Future<Output = Result<String, String>> + Send;
}

/// RPC request wrapper for zero-copy optimization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Request ID for tracking.
    pub id: String,
    /// Method name.
    pub method: String,
    /// Parameters as JSON bytes (`serde_bytes`-compatible wire format; see `rpc_bytes_serde`).
    #[serde(
        serialize_with = "rpc_bytes_serde::serialize_params",
        deserialize_with = "rpc_bytes_serde::deserialize_params"
    )]
    pub params: Bytes,
}

/// RPC response wrapper for zero-copy optimization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Request ID for correlation.
    pub id: String,
    /// Result as JSON bytes (None if error; `serde_bytes`-compatible wire format).
    #[serde(
        serialize_with = "rpc_bytes_serde::serialize_result",
        deserialize_with = "rpc_bytes_serde::deserialize_result"
    )]
    pub result: Option<Bytes>,
    /// Error message if any.
    pub error: Option<String>,
}

impl RpcRequest {
    /// Create a new RPC request.
    #[must_use]
    pub fn new(id: impl Into<String>, method: impl Into<String>, params: impl Into<Bytes>) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params: params.into(),
        }
    }
}

impl RpcResponse {
    /// Create a successful response.
    #[must_use]
    pub fn success(id: impl Into<String>, result: impl Into<Bytes>) -> Self {
        Self {
            id: id.into(),
            result: Some(result.into()),
            error: None,
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            result: None,
            error: Some(error.into()),
        }
    }
}

/// Helper to convert `PrimalError` to RPC string error.
impl From<PrimalError> for String {
    fn from(err: PrimalError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn rpc_request_creation() {
        let req = RpcRequest::new("req-123", "health", Bytes::from(vec![1, 2, 3]));

        assert_eq!(req.id, "req-123");
        assert_eq!(req.method, "health");
        assert_eq!(req.params, Bytes::from(vec![1, 2, 3]));
    }

    #[test]
    fn rpc_response_success() {
        let resp = RpcResponse::success("req-123", Bytes::from(vec![4, 5, 6]));

        assert_eq!(resp.id, "req-123");
        assert_eq!(resp.result, Some(Bytes::from(vec![4, 5, 6])));
        assert!(resp.error.is_none());
    }

    #[test]
    fn rpc_response_error() {
        let resp = RpcResponse::error("req-456", "Something went wrong");

        assert_eq!(resp.id, "req-456");
        assert!(resp.result.is_none());
        assert_eq!(resp.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn rpc_request_serialization() {
        let req = RpcRequest::new("test", "ping", Bytes::new());
        let json = serde_json::to_string(&req).unwrap();

        assert!(json.contains("test"));
        assert!(json.contains("ping"));
    }

    #[test]
    fn rpc_response_serialization() {
        let resp = RpcResponse::success("test", Bytes::from(vec![1, 2]));
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains("test"));
    }

    #[test]
    fn rpc_request_with_empty_params() {
        let req = RpcRequest::new("empty", "method", Bytes::new());
        assert_eq!(req.params.len(), 0);
    }

    #[test]
    fn rpc_request_with_large_params() {
        let large_params = vec![42u8; 1000];
        let req = RpcRequest::new("large", "bulk_operation", Bytes::from(large_params.clone()));
        assert_eq!(req.params.len(), 1000);
        assert_eq!(req.params, Bytes::from(large_params));
    }

    #[test]
    fn rpc_response_error_with_long_message() {
        let long_error = "E".repeat(500);
        let resp = RpcResponse::error("err", &long_error);
        assert_eq!(resp.error, Some(long_error));
        assert!(resp.result.is_none());
    }

    #[test]
    fn primal_error_to_string_conversion() {
        let err = PrimalError::lifecycle("Test lifecycle error");
        let err_string: String = err.into();
        assert!(err_string.contains("lifecycle"));
    }

    #[test]
    fn rpc_request_roundtrip_serialization() {
        let req = RpcRequest::new("roundtrip", "test_method", Bytes::from(vec![1, 2, 3, 4, 5]));
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, req.id);
        assert_eq!(deserialized.method, req.method);
        assert_eq!(deserialized.params, req.params);
    }

    #[test]
    fn rpc_response_success_roundtrip() {
        let resp = RpcResponse::success("round", Bytes::from(vec![10, 20, 30]));
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: RpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, resp.id);
        assert_eq!(deserialized.result, resp.result);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn rpc_response_error_roundtrip() {
        let resp = RpcResponse::error("err-round", "Error message");
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: RpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, resp.id);
        assert!(deserialized.result.is_none());
        assert_eq!(deserialized.error, resp.error);
    }
}

/// RPC client helper for connecting to primals.
pub mod client {
    use std::net::SocketAddr;
    use tokio::net::ToSocketAddrs;

    /// RPC client for primal communication.
    pub struct PrimalRpcClient {
        addr: SocketAddr,
    }

    impl PrimalRpcClient {
        /// Create a new RPC client by resolving an address.
        ///
        /// # Errors
        ///
        /// Returns [`std::io::Error`] if the address cannot be resolved.
        pub async fn connect(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
            let addr = tokio::net::lookup_host(addr).await?.next().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "no addresses resolved",
                )
            })?;

            Ok(Self { addr })
        }

        /// Get the connected address.
        #[must_use]
        pub const fn addr(&self) -> SocketAddr {
            self.addr
        }
    }
}

/// RPC server helper for hosting primal services.
pub mod server {
    use std::net::SocketAddr;

    /// RPC server configuration.
    #[derive(Clone, Debug)]
    pub struct ServerConfig {
        /// Bind address (0.0.0.0 for all interfaces).
        pub bind_addr: String,
        /// Port (0 for OS-assigned ephemeral port).
        pub port: u16,
    }

    impl Default for ServerConfig {
        fn default() -> Self {
            Self {
                bind_addr: "0.0.0.0".to_string(),
                port: 0, // OS assigns available port
            }
        }
    }

    impl ServerConfig {
        /// Create a new server configuration.
        #[must_use]
        pub fn new(bind_addr: impl Into<String>, port: u16) -> Self {
            Self {
                bind_addr: bind_addr.into(),
                port,
            }
        }

        /// Get the socket address.
        ///
        /// # Errors
        ///
        /// Returns an error if the address cannot be parsed.
        pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
            format!("{}:{}", self.bind_addr, self.port).parse()
        }
    }
}

#[cfg(test)]
mod client_server_tests {
    use super::{client, server};

    #[tokio::test]
    async fn server_config_default() {
        let config = server::ServerConfig::default();

        assert_eq!(config.bind_addr, "0.0.0.0");
        assert_eq!(config.port, 0); // OS-assigned
    }

    #[tokio::test]
    async fn server_config_custom() {
        let config = server::ServerConfig::new("127.0.0.1", 8000);

        assert_eq!(config.bind_addr, "127.0.0.1");
        assert_eq!(config.port, 8000);
    }

    #[tokio::test]
    async fn server_config_socket_addr() {
        let config = server::ServerConfig::new("127.0.0.1", 0); // Port 0 = OS-assigned
        let addr = config.socket_addr().unwrap();

        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_eq!(addr.port(), 0);
    }

    #[tokio::test]
    async fn server_config_socket_addr_with_port() {
        let config = server::ServerConfig::new("0.0.0.0", 9000);
        let addr = config.socket_addr().unwrap();

        assert_eq!(addr.ip().to_string(), "0.0.0.0");
        assert_eq!(addr.port(), 9000);
    }

    #[tokio::test]
    async fn server_config_socket_addr_invalid() {
        let config = server::ServerConfig::new("invalid-address", 8000);
        let result = config.socket_addr();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn server_config_clone() {
        let config1 = server::ServerConfig::new("192.0.2.1", 3000);
        let config2 = config1.clone();

        assert_eq!(config1.bind_addr, config2.bind_addr);
        assert_eq!(config1.port, config2.port);
    }

    #[tokio::test]
    async fn client_connect_to_localhost() {
        // Test that client can resolve localhost
        let result = client::PrimalRpcClient::connect("127.0.0.1:8080").await;

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.addr().ip().to_string(), "127.0.0.1");
        assert_eq!(client.addr().port(), 8080);
    }

    #[tokio::test]
    async fn client_connect_with_port_zero() {
        let result = client::PrimalRpcClient::connect("localhost:0").await;

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.addr().port(), 0);
    }

    #[tokio::test]
    async fn client_connect_invalid_address() {
        let result = client::PrimalRpcClient::connect("").await;

        assert!(result.is_err());
    }
}
