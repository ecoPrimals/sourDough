//! JSON-RPC 2.0 protocol types and error codes.
//!
//! Wire format types for the primary ecoPrimals IPC mechanism.
//! All primals expose JSON-RPC 2.0 endpoints following the semantic
//! method naming standard (`domain.verb` pattern).

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 version constant.
pub const JSONRPC_VERSION: &str = "2.0";

/// Default IPC timeout for inter-primal calls (5 seconds).
///
/// Sufficient for local UDS and TCP on the same host. Override for
/// cross-gate mesh relay calls that traverse the network.
pub const DEFAULT_IPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Default timeout for mesh relay calls (15 seconds).
///
/// Mesh relay traverses: caller → local songBird → WAN → remote songBird → target.
/// The additional network hops warrant a longer timeout than local IPC.
pub const MESH_RELAY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

// --- Standard JSON-RPC 2.0 error codes ---

/// Parse error (-32700).
pub const PARSE_ERROR: i32 = -32700;
/// Invalid request (-32600).
pub const INVALID_REQUEST: i32 = -32600;
/// Method not found (-32601).
pub const METHOD_NOT_FOUND: i32 = -32601;
/// Invalid params (-32602).
pub const INVALID_PARAMS: i32 = -32602;
/// Internal error (-32603).
pub const INTERNAL_ERROR: i32 = -32603;

// --- ecoPrimals-specific error codes ---

/// Service unavailable.
pub const SERVICE_UNAVAILABLE: i32 = -32000;
/// Dependency failure.
pub const DEPENDENCY_FAILURE: i32 = -32001;
/// Circuit breaker open.
pub const CIRCUIT_BREAKER_OPEN: i32 = -32002;
/// Rate limited.
pub const RATE_LIMITED: i32 = -32003;
/// Not ready (primal starting up).
pub const NOT_READY: i32 = -32004;

// --- Request / Response / Error types ---

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0").
    pub jsonrpc: String,
    /// Method name using `domain.verb` semantic naming.
    pub method: String,
    /// Parameters (positional or named).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    /// Request ID (null for notifications).
    pub id: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new request.
    #[must_use]
    pub fn new(method: impl Into<String>, id: impl Into<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params: None,
            id: Some(id.into()),
        }
    }

    /// Create a request with parameters.
    #[must_use]
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Create a notification (no response expected).
    #[must_use]
    pub fn notification(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params: None,
            id: None,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version.
    pub jsonrpc: String,
    /// Result (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID correlation.
    pub id: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// Create a success response.
    #[must_use]
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Human-readable message.
    pub message: String,
    /// Additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new error.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Attach additional data.
    #[must_use]
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Standard: parse error.
    #[must_use]
    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::new(PARSE_ERROR, detail)
    }

    /// Standard: method not found.
    #[must_use]
    pub fn method_not_found(method: &str) -> Self {
        Self::new(METHOD_NOT_FOUND, format!("method not found: {method}"))
    }

    /// Standard: internal error.
    #[must_use]
    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR, detail)
    }

    /// ecoPrimals: circuit breaker open.
    #[must_use]
    pub fn circuit_breaker_open(service: &str) -> Self {
        Self::new(
            CIRCUIT_BREAKER_OPEN,
            format!("circuit breaker open for {service}"),
        )
    }

    /// Whether this error is retryable.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self.code,
            SERVICE_UNAVAILABLE | DEPENDENCY_FAILURE | RATE_LIMITED | NOT_READY
        )
    }
}

impl From<PrimalError> for JsonRpcError {
    fn from(err: PrimalError) -> Self {
        match &err {
            PrimalError::Network(_) | PrimalError::Timeout(_) => {
                Self::new(SERVICE_UNAVAILABLE, err.to_string())
            }
            PrimalError::Dependency { .. } => Self::new(DEPENDENCY_FAILURE, err.to_string()),
            PrimalError::InvalidInput(_) => Self::new(INVALID_PARAMS, err.to_string()),
            PrimalError::NotFound(_) => Self::new(METHOD_NOT_FOUND, err.to_string()),
            PrimalError::Config(_)
            | PrimalError::Identity(_)
            | PrimalError::Discovery(_)
            | PrimalError::Lifecycle(_)
            | PrimalError::Health(_)
            | PrimalError::Io(_)
            | PrimalError::Serialization(_)
            | PrimalError::Storage(_)
            | PrimalError::Cancelled(_)
            | PrimalError::AlreadyExists(_)
            | PrimalError::PermissionDenied(_)
            | PrimalError::Internal(_)
            | PrimalError::Domain { .. } => Self::new(INTERNAL_ERROR, err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_new_sets_version_and_id() {
        let req = JsonRpcRequest::new("health.liveness", 1);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "health.liveness");
        assert_eq!(req.id, Some(serde_json::json!(1)));
        assert!(req.params.is_none());
    }

    #[test]
    fn request_with_params() {
        let req =
            JsonRpcRequest::new("data.store", 42).with_params(serde_json::json!({"key": "value"}));
        assert_eq!(req.params.unwrap()["key"], "value");
    }

    #[test]
    fn notification_has_no_id() {
        let notif = JsonRpcRequest::notification("lifecycle.shutdown");
        assert!(notif.id.is_none());
        assert_eq!(notif.method, "lifecycle.shutdown");
    }

    #[test]
    fn request_serde_roundtrip() {
        let req =
            JsonRpcRequest::new("test.method", 7).with_params(serde_json::json!({"arg": true}));
        let json = serde_json::to_string(&req).unwrap();
        let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.method, "test.method");
        assert_eq!(back.id, Some(serde_json::json!(7)));
        assert_eq!(back.params.unwrap()["arg"], true);
    }

    #[test]
    fn notification_serde_omits_id() {
        let notif = JsonRpcRequest::notification("event.fired");
        let json = serde_json::to_string(&notif).unwrap();
        assert!(!json.contains("\"id\"") || json.contains("\"id\":null"));
    }

    #[test]
    fn response_success() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"alive": true}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn response_error() {
        let err = JsonRpcError::method_not_found("bad.method");
        let resp = JsonRpcResponse::error(Some(serde_json::json!(2)), err);
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[test]
    fn response_serde_roundtrip_success() {
        let resp = JsonRpcResponse::success(serde_json::json!(5), serde_json::json!("ok"));
        let json = serde_json::to_string(&resp).unwrap();
        let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result.unwrap(), "ok");
        assert!(back.error.is_none());
    }

    #[test]
    fn response_serde_roundtrip_error() {
        let err = JsonRpcError::internal("something broke");
        let resp = JsonRpcResponse::error(Some(serde_json::json!(3)), err);
        let json = serde_json::to_string(&resp).unwrap();
        let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert!(back.result.is_none());
        let e = back.error.unwrap();
        assert_eq!(e.code, INTERNAL_ERROR);
        assert!(e.message.contains("something broke"));
    }

    #[test]
    fn error_new_and_with_data() {
        let err = JsonRpcError::new(-32000, "service down")
            .with_data(serde_json::json!({"retry_after_ms": 5000}));
        assert_eq!(err.code, -32000);
        assert_eq!(err.message, "service down");
        assert_eq!(err.data.unwrap()["retry_after_ms"], 5000);
    }

    #[test]
    fn error_parse_error() {
        let err = JsonRpcError::parse_error("unexpected token");
        assert_eq!(err.code, PARSE_ERROR);
        assert!(err.message.contains("unexpected token"));
    }

    #[test]
    fn error_method_not_found() {
        let err = JsonRpcError::method_not_found("foo.bar");
        assert_eq!(err.code, METHOD_NOT_FOUND);
        assert!(err.message.contains("foo.bar"));
    }

    #[test]
    fn error_circuit_breaker_open() {
        let err = JsonRpcError::circuit_breaker_open("beardog");
        assert_eq!(err.code, CIRCUIT_BREAKER_OPEN);
        assert!(err.message.contains("beardog"));
    }

    #[test]
    fn error_retryable_codes() {
        assert!(JsonRpcError::new(SERVICE_UNAVAILABLE, "").is_retryable());
        assert!(JsonRpcError::new(DEPENDENCY_FAILURE, "").is_retryable());
        assert!(JsonRpcError::new(RATE_LIMITED, "").is_retryable());
        assert!(JsonRpcError::new(NOT_READY, "").is_retryable());
    }

    #[test]
    fn error_non_retryable_codes() {
        assert!(!JsonRpcError::new(PARSE_ERROR, "").is_retryable());
        assert!(!JsonRpcError::new(INVALID_REQUEST, "").is_retryable());
        assert!(!JsonRpcError::new(METHOD_NOT_FOUND, "").is_retryable());
        assert!(!JsonRpcError::new(INVALID_PARAMS, "").is_retryable());
        assert!(!JsonRpcError::new(INTERNAL_ERROR, "").is_retryable());
        assert!(!JsonRpcError::new(CIRCUIT_BREAKER_OPEN, "").is_retryable());
    }

    #[test]
    fn error_serde_roundtrip() {
        let err = JsonRpcError::new(-32001, "dependency failed")
            .with_data(serde_json::json!({"service": "songbird"}));
        let json = serde_json::to_string(&err).unwrap();
        let back: JsonRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, back);
    }

    #[test]
    fn from_primal_error_network() {
        let pe = PrimalError::Network("connection refused".into());
        let rpc_err: JsonRpcError = pe.into();
        assert_eq!(rpc_err.code, SERVICE_UNAVAILABLE);
    }

    #[test]
    fn from_primal_error_timeout() {
        let pe = PrimalError::Timeout("5s elapsed".into());
        let rpc_err: JsonRpcError = pe.into();
        assert_eq!(rpc_err.code, SERVICE_UNAVAILABLE);
    }

    #[test]
    fn from_primal_error_dependency() {
        let pe = PrimalError::dependency("beardog", "unreachable");
        let rpc_err: JsonRpcError = pe.into();
        assert_eq!(rpc_err.code, DEPENDENCY_FAILURE);
    }

    #[test]
    fn from_primal_error_invalid_input() {
        let pe = PrimalError::InvalidInput("bad param".into());
        let rpc_err: JsonRpcError = pe.into();
        assert_eq!(rpc_err.code, INVALID_PARAMS);
    }

    #[test]
    fn from_primal_error_not_found() {
        let pe = PrimalError::NotFound("no such method".into());
        let rpc_err: JsonRpcError = pe.into();
        assert_eq!(rpc_err.code, METHOD_NOT_FOUND);
    }

    #[test]
    fn from_primal_error_internal_variants() {
        let variants: Vec<PrimalError> = vec![
            PrimalError::Config("bad config".into()),
            PrimalError::Internal("oops".into()),
            std::io::Error::other("read failed").into(),
        ];
        for pe in variants {
            let rpc_err: JsonRpcError = pe.into();
            assert_eq!(rpc_err.code, INTERNAL_ERROR);
        }
    }

    #[test]
    fn constants_match_spec() {
        assert_eq!(PARSE_ERROR, -32700);
        assert_eq!(INVALID_REQUEST, -32600);
        assert_eq!(METHOD_NOT_FOUND, -32601);
        assert_eq!(INVALID_PARAMS, -32602);
        assert_eq!(INTERNAL_ERROR, -32603);
        assert_eq!(SERVICE_UNAVAILABLE, -32000);
        assert_eq!(DEPENDENCY_FAILURE, -32001);
        assert_eq!(CIRCUIT_BREAKER_OPEN, -32002);
        assert_eq!(RATE_LIMITED, -32003);
        assert_eq!(NOT_READY, -32004);
    }

    #[test]
    fn timeout_constants() {
        assert_eq!(DEFAULT_IPC_TIMEOUT, std::time::Duration::from_secs(5));
        assert_eq!(MESH_RELAY_TIMEOUT, std::time::Duration::from_secs(15));
    }
}
