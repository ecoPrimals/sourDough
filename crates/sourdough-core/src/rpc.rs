//! RPC layer for inter-primal communication.
//!
//! This module provides tarpc-based RPC interfaces for primals to communicate.
//! All primals expose common RPC endpoints for health, lifecycle, and discovery.

use crate::{error::PrimalError, health::HealthReport, identity::Did, lifecycle::PrimalState};
use serde::{Deserialize, Serialize};

/// Common RPC service that all primals must implement.
///
/// This provides the baseline interface for inter-primal communication.
#[tarpc::service]
pub trait PrimalRpc {
    /// Get the primal's current health status.
    async fn health() -> Result<HealthReport, String>;

    /// Get the primal's current lifecycle state.
    async fn state() -> Result<PrimalState, String>;

    /// Get the primal's decentralized identifier (DID).
    async fn did() -> Result<Did, String>;

    /// Ping the primal for liveness check.
    async fn ping() -> Result<String, String>;
}

/// RPC request wrapper for zero-copy optimization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Request ID for tracking.
    pub id: String,
    /// Method name.
    pub method: String,
    /// Parameters as JSON bytes.
    #[serde(with = "serde_bytes")]
    pub params: Vec<u8>,
}

/// RPC response wrapper for zero-copy optimization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Request ID for correlation.
    pub id: String,
    /// Result as JSON bytes (None if error).
    #[serde(with = "serde_bytes")]
    pub result: Option<Vec<u8>>,
    /// Error message if any.
    pub error: Option<String>,
}

impl RpcRequest {
    /// Create a new RPC request.
    #[must_use]
    pub fn new(id: impl Into<String>, method: impl Into<String>, params: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

impl RpcResponse {
    /// Create a successful response.
    #[must_use]
    pub fn success(id: impl Into<String>, result: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            result: Some(result),
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

    #[test]
    fn rpc_request_creation() {
        let req = RpcRequest::new("req-123", "health", vec![1, 2, 3]);

        assert_eq!(req.id, "req-123");
        assert_eq!(req.method, "health");
        assert_eq!(req.params, vec![1, 2, 3]);
    }

    #[test]
    fn rpc_response_success() {
        let resp = RpcResponse::success("req-123", vec![4, 5, 6]);

        assert_eq!(resp.id, "req-123");
        assert_eq!(resp.result, Some(vec![4, 5, 6]));
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
        let req = RpcRequest::new("test", "ping", vec![]);
        let json = serde_json::to_string(&req).unwrap();

        assert!(json.contains("test"));
        assert!(json.contains("ping"));
    }

    #[test]
    fn rpc_response_serialization() {
        let resp = RpcResponse::success("test", vec![1, 2]);
        let json = serde_json::to_string(&resp).unwrap();

        assert!(json.contains("test"));
    }

    #[test]
    fn rpc_request_with_empty_params() {
        let req = RpcRequest::new("empty", "method", vec![]);
        assert_eq!(req.params.len(), 0);
    }

    #[test]
    fn rpc_request_with_large_params() {
        let large_params = vec![42u8; 1000];
        let req = RpcRequest::new("large", "bulk_operation", large_params.clone());
        assert_eq!(req.params.len(), 1000);
        assert_eq!(req.params, large_params);
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
        let req = RpcRequest::new("roundtrip", "test_method", vec![1, 2, 3, 4, 5]);
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: RpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, req.id);
        assert_eq!(deserialized.method, req.method);
        assert_eq!(deserialized.params, req.params);
    }

    #[test]
    fn rpc_response_success_roundtrip() {
        let resp = RpcResponse::success("round", vec![10, 20, 30]);
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
        /// Create a new RPC client.
        ///
        /// # Errors
        ///
        /// Returns an error if the address cannot be resolved.
        pub async fn connect(
            addr: impl ToSocketAddrs,
        ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let addr = tokio::net::lookup_host(addr)
                .await?
                .next()
                .ok_or("Failed to resolve address")?;

            Ok(Self { addr })
        }

        /// Get the connected address.
        #[must_use]
        pub fn addr(&self) -> SocketAddr {
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
