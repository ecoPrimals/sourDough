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
