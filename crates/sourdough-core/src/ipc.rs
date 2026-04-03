//! JSON-RPC 2.0 IPC layer for inter-primal communication.
//!
//! This is the **primary** IPC mechanism for ecoPrimals. All primals expose
//! JSON-RPC 2.0 endpoints following the semantic method naming standard
//! (`domain.verb` pattern).
//!
//! The tarpc-based RPC in [`crate::rpc`] is the optional high-throughput
//! binary path for performance-critical communication.

#![expect(
    clippy::module_name_repetitions,
    reason = "IPC types like JsonRpcRequest are clearer with full prefix"
)]

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- JSON-RPC 2.0 Protocol Types ---

/// JSON-RPC 2.0 version constant.
pub const JSONRPC_VERSION: &str = "2.0";

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

// --- Standard JSON-RPC 2.0 Error Codes ---

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

// --- ecoPrimals IPC Error Codes (application-defined, -32000 to -32099) ---

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

// --- Typed IPC Error ---

/// Structured IPC error for inter-primal communication.
///
/// This provides richer error semantics than raw JSON-RPC error codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    /// Error category.
    pub kind: IpcErrorKind,
    /// Human-readable message.
    pub message: String,
    /// Source primal that generated the error.
    pub source_primal: Option<String>,
    /// Whether this error is retryable.
    pub retryable: bool,
}

/// Categories of IPC errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcErrorKind {
    /// Transport-level failure.
    Transport,
    /// Timeout waiting for response.
    Timeout,
    /// Upstream dependency unavailable.
    DependencyUnavailable,
    /// Circuit breaker tripped.
    CircuitBreakerOpen,
    /// Rate limit exceeded.
    RateLimited,
    /// Primal not ready.
    NotReady,
    /// Method not found.
    MethodNotFound,
    /// Invalid parameters.
    InvalidParams,
    /// Internal primal error.
    Internal,
}

impl IpcError {
    /// Create a new IPC error.
    #[must_use]
    pub fn new(kind: IpcErrorKind, message: impl Into<String>) -> Self {
        let retryable = matches!(
            kind,
            IpcErrorKind::Transport
                | IpcErrorKind::Timeout
                | IpcErrorKind::DependencyUnavailable
                | IpcErrorKind::RateLimited
                | IpcErrorKind::NotReady
        );
        Self {
            kind,
            message: message.into(),
            source_primal: None,
            retryable,
        }
    }

    /// Set the source primal.
    #[must_use]
    pub fn from_primal(mut self, primal: impl Into<String>) -> Self {
        self.source_primal = Some(primal.into());
        self
    }
}

// --- Capability Declaration ---

/// A capability that a primal can expose via `capabilities.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability domain (e.g., "storage", "crypto", "health").
    pub domain: String,
    /// Available methods within this domain.
    pub methods: Vec<String>,
    /// Capability version.
    pub version: String,
}

impl Capability {
    /// Create a new capability.
    #[must_use]
    pub fn new(domain: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            methods: Vec::new(),
            version: version.into(),
        }
    }

    /// Add a method to this capability.
    #[must_use]
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.methods.push(method.into());
        self
    }
}

/// Standard health probe response for `health.check`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthProbe {
    /// Primal name.
    pub primal: String,
    /// Primal version.
    pub version: String,
    /// Health status string: "healthy", "degraded", "unhealthy".
    pub status: String,
    /// Liveness flag.
    pub live: bool,
    /// Readiness flag.
    pub ready: bool,
    /// Dependency statuses.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,
}

// --- Standard Method Names ---

/// Standard method names following `domain.verb` semantic naming.
pub mod methods {
    /// Health domain methods.
    pub mod health {
        /// Full health check.
        pub const CHECK: &str = "health.check";
        /// Liveness probe (is the process alive?).
        pub const LIVENESS: &str = "health.liveness";
        /// Readiness probe (can it serve requests?).
        pub const READINESS: &str = "health.readiness";
    }

    /// Lifecycle domain methods.
    pub mod lifecycle {
        /// Get current state.
        pub const STATE: &str = "lifecycle.state";
        /// Trigger reload.
        pub const RELOAD: &str = "lifecycle.reload";
    }

    /// Capability domain methods.
    pub mod capabilities {
        /// List all capabilities.
        pub const LIST: &str = "capabilities.list";
    }

    /// Identity domain methods.
    pub mod identity {
        /// Get primal DID.
        pub const DID: &str = "identity.did";
    }

    /// System domain methods.
    pub mod system {
        /// Ping for liveness.
        pub const PING: &str = "system.ping";
        /// Get primal version.
        pub const VERSION: &str = "system.version";
    }
}

// --- Circuit Breaker ---

/// Simple circuit breaker for IPC resilience.
#[derive(Debug)]
pub struct CircuitBreaker {
    service: String,
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure: Option<std::time::Instant>,
    reset_timeout: std::time::Duration,
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation.
    Closed,
    /// Too many failures, rejecting calls.
    Open,
    /// Testing if service recovered.
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    #[must_use]
    pub fn new(
        service: impl Into<String>,
        failure_threshold: u32,
        reset_timeout: std::time::Duration,
    ) -> Self {
        Self {
            service: service.into(),
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            reset_timeout,
        }
    }

    /// Check if a call is allowed.
    #[must_use]
    pub fn allow_call(&mut self) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Record a successful call.
    pub const fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failed call.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Get current state.
    #[must_use]
    pub const fn state(&self) -> CircuitState {
        self.state
    }

    /// Get the service name.
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }
}

#[cfg(test)]
mod tests {
    use super::methods::{capabilities, health, identity, lifecycle, system};
    use super::*;

    #[test]
    fn jsonrpc_request_roundtrip() {
        let req = JsonRpcRequest::new("health.check", serde_json::json!(1))
            .with_params(serde_json::json!({"deep": true}));
        let json = serde_json::to_string(&req).expect("serialize");
        let back: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.jsonrpc, JSONRPC_VERSION);
        assert_eq!(back.method, "health.check");
        assert_eq!(back.id, Some(serde_json::json!(1)));
        assert_eq!(back.params, Some(serde_json::json!({"deep": true})));
    }

    #[test]
    fn jsonrpc_notification_serializes_null_id() {
        let n = JsonRpcRequest::notification("system.ping");
        let v = serde_json::to_value(&n).expect("to_value");
        assert!(v.get("id").is_none() || v["id"].is_null());
    }

    #[test]
    fn jsonrpc_response_success_roundtrip() {
        let res =
            JsonRpcResponse::success(serde_json::json!("req-1"), serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&res).expect("serialize");
        let back: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.error, None);
        assert_eq!(back.result, Some(serde_json::json!({"ok": true})));
        assert_eq!(back.id, Some(serde_json::json!("req-1")));
    }

    #[test]
    fn jsonrpc_response_error_roundtrip() {
        let err = JsonRpcError::method_not_found("foo.bar");
        let res = JsonRpcResponse::error(Some(serde_json::json!(42)), err);
        let json = serde_json::to_string(&res).expect("serialize");
        let back: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.result.is_none());
        assert_eq!(back.id, Some(serde_json::json!(42)));
        let e = back.error.expect("error field");
        assert_eq!(e.code, METHOD_NOT_FOUND);
        assert!(e.message.contains("foo.bar"));
    }

    #[test]
    fn jsonrpc_error_standard_codes() {
        assert_eq!(JsonRpcError::parse_error("bad").code, PARSE_ERROR);
        assert_eq!(JsonRpcError::internal("x").code, INTERNAL_ERROR);
        assert_eq!(
            JsonRpcError::circuit_breaker_open("upstream").code,
            CIRCUIT_BREAKER_OPEN
        );
    }

    #[test]
    fn jsonrpc_error_retryable_classification() {
        assert!(JsonRpcError::new(SERVICE_UNAVAILABLE, "x").is_retryable());
        assert!(JsonRpcError::new(DEPENDENCY_FAILURE, "x").is_retryable());
        assert!(JsonRpcError::new(RATE_LIMITED, "x").is_retryable());
        assert!(JsonRpcError::new(NOT_READY, "x").is_retryable());
        assert!(!JsonRpcError::new(CIRCUIT_BREAKER_OPEN, "x").is_retryable());
        assert!(!JsonRpcError::new(METHOD_NOT_FOUND, "x").is_retryable());
    }

    #[test]
    fn primal_error_maps_to_jsonrpc() {
        let e: JsonRpcError = PrimalError::Network("down".into()).into();
        assert_eq!(e.code, SERVICE_UNAVAILABLE);
        let e: JsonRpcError = PrimalError::dependency("db", "no").into();
        assert_eq!(e.code, DEPENDENCY_FAILURE);
        let e: JsonRpcError = PrimalError::InvalidInput("bad".into()).into();
        assert_eq!(e.code, INVALID_PARAMS);
        let e: JsonRpcError = PrimalError::NotFound("x".into()).into();
        assert_eq!(e.code, METHOD_NOT_FOUND);
        let e: JsonRpcError = PrimalError::config("c").into();
        assert_eq!(e.code, INTERNAL_ERROR);
    }

    #[test]
    fn circuit_breaker_closed_allows_and_opens() {
        let mut cb = CircuitBreaker::new("svc", 2, std::time::Duration::from_secs(60));
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
    }

    #[test]
    fn circuit_breaker_opens_then_half_open_after_reset() {
        let reset = std::time::Duration::from_millis(20);
        let mut cb = CircuitBreaker::new("svc", 1, reset);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
        std::thread::sleep(reset + std::time::Duration::from_millis(10));
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn capability_builder() {
        let cap = Capability::new("health", "1.0.0")
            .with_method("check")
            .with_method("liveness");
        assert_eq!(cap.domain, "health");
        assert_eq!(cap.version, "1.0.0");
        assert_eq!(cap.methods, vec!["check", "liveness"]);
    }

    #[test]
    fn ipc_error_retryable_by_kind() {
        assert!(IpcError::new(IpcErrorKind::Transport, "t").retryable);
        assert!(IpcError::new(IpcErrorKind::NotReady, "n").retryable);
        assert!(!IpcError::new(IpcErrorKind::CircuitBreakerOpen, "c").retryable);
        assert!(!IpcError::new(IpcErrorKind::Internal, "i").retryable);
    }

    #[test]
    fn ipc_error_from_primal_sets_source() {
        let e = IpcError::new(IpcErrorKind::Internal, "msg").from_primal("p1");
        assert_eq!(e.source_primal.as_deref(), Some("p1"));
    }

    #[test]
    fn method_name_constants() {
        assert_eq!(health::CHECK, "health.check");
        assert_eq!(health::LIVENESS, "health.liveness");
        assert_eq!(health::READINESS, "health.readiness");
        assert_eq!(lifecycle::STATE, "lifecycle.state");
        assert_eq!(lifecycle::RELOAD, "lifecycle.reload");
        assert_eq!(capabilities::LIST, "capabilities.list");
        assert_eq!(identity::DID, "identity.did");
        assert_eq!(system::PING, "system.ping");
        assert_eq!(system::VERSION, "system.version");
    }

    #[test]
    fn health_probe_roundtrip() {
        let mut deps = HashMap::new();
        deps.insert("db".to_string(), "up".to_string());
        let probe = HealthProbe {
            primal: "test".into(),
            version: "0.1.0".into(),
            status: "healthy".into(),
            live: true,
            ready: true,
            dependencies: deps,
        };
        let json = serde_json::to_string(&probe).expect("serialize");
        let back: HealthProbe = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.primal, "test");
        assert_eq!(back.dependencies.get("db").map(String::as_str), Some("up"));
    }
}
