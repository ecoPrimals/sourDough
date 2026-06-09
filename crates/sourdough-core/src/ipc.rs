//! JSON-RPC 2.0 IPC layer for inter-primal communication.
//!
//! This is the **primary** IPC mechanism for ecoPrimals. All primals expose
//! JSON-RPC 2.0 endpoints following the semantic method naming standard
//! (`domain.verb` pattern).
//!
//! The binary RPC in [`crate::rpc`] is the optional high-throughput
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

/// Default IPC timeout for inter-primal calls (5 seconds).
///
/// Sufficient for local UDS and TCP on the same host. Override for
/// cross-gate mesh relay calls that traverse the network.
pub const DEFAULT_IPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(primal) = &self.source_primal {
            write!(f, "[{}] {:?}: {}", primal, self.kind, self.message)
        } else {
            write!(f, "{:?}: {}", self.kind, self.message)
        }
    }
}

impl std::error::Error for IpcError {}

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
    /// Structured health status.
    pub status: crate::health::HealthStatus,
    /// Liveness flag.
    pub live: bool,
    /// Readiness flag.
    pub ready: bool,
    /// Dependency statuses.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,
}

// Re-export from dedicated module for backward compatibility.
pub use crate::methods;

// --- Transport-aware JSON-RPC Client ---

/// A transport-aware JSON-RPC 2.0 client that connects via [`TransportEndpoint`].
///
/// Uses `connect_transport()` under the hood — the caller never needs to know
/// whether the remote primal is on UDS, TCP, or relay.
///
/// [`TransportEndpoint`]: crate::transport::TransportEndpoint
pub struct IpcClient {
    endpoint: crate::transport::TransportEndpoint,
}

impl IpcClient {
    /// Create a new client targeting the given endpoint.
    #[must_use]
    pub const fn new(endpoint: crate::transport::TransportEndpoint) -> Self {
        Self { endpoint }
    }

    /// Resolve a primal by name using ecosystem socket conventions, then build a client.
    #[must_use]
    pub fn from_primal(primal_name: &str, family_id: Option<&str>) -> Self {
        Self {
            endpoint: crate::transport::TransportEndpoint::from_primal_name(primal_name, family_id),
        }
    }

    /// The endpoint this client targets.
    #[must_use]
    pub const fn endpoint(&self) -> &crate::transport::TransportEndpoint {
        &self.endpoint
    }

    /// Send a JSON-RPC request and return the response.
    ///
    /// Opens a connection, writes the request as a newline-delimited JSON line,
    /// reads the response line, and closes. This is the standard ecoPrimals
    /// one-shot RPC pattern.
    ///
    /// # Errors
    ///
    /// Returns `IpcError` on transport or protocol failure.
    pub async fn call(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, IpcError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let stream = crate::transport::connect_transport(&self.endpoint)
            .await
            .map_err(|e| {
                IpcError::new(
                    IpcErrorKind::Transport,
                    format!("connect to {}: {e}", self.endpoint),
                )
            })?;

        let mut buf_stream = BufReader::new(stream);

        let req_json = serde_json::to_string(request).map_err(|e| {
            IpcError::new(IpcErrorKind::Internal, format!("serialize request: {e}"))
        })?;

        let writer = buf_stream.get_mut();
        writer
            .write_all(req_json.as_bytes())
            .await
            .map_err(|e| IpcError::new(IpcErrorKind::Transport, format!("write: {e}")))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| IpcError::new(IpcErrorKind::Transport, format!("write newline: {e}")))?;

        let mut response_line = String::new();
        buf_stream
            .read_line(&mut response_line)
            .await
            .map_err(|e| IpcError::new(IpcErrorKind::Transport, format!("read response: {e}")))?;

        serde_json::from_str(response_line.trim()).map_err(|e| {
            IpcError::new(IpcErrorKind::Internal, format!("deserialize response: {e}"))
        })
    }

    /// Send a JSON-RPC request with a timeout.
    ///
    /// Equivalent to [`call`](Self::call) but returns `IpcError` with
    /// `IpcErrorKind::Timeout` if the operation exceeds the given duration.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport failure, protocol error, or timeout.
    pub async fn call_with_timeout(
        &self,
        request: &JsonRpcRequest,
        timeout: std::time::Duration,
    ) -> Result<JsonRpcResponse, IpcError> {
        tokio::time::timeout(timeout, self.call(request))
            .await
            .map_err(|_| {
                IpcError::new(
                    IpcErrorKind::Timeout,
                    format!(
                        "request to {} timed out after {}ms",
                        self.endpoint,
                        timeout.as_millis()
                    ),
                )
            })?
    }

    /// Probe liveness of the target primal.
    ///
    /// Returns `Ok(true)` if the primal responds with a result, `Ok(false)` if
    /// it responds with an error, or an `Err` if the transport fails entirely.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if the transport connection fails.
    pub async fn health_liveness(&self) -> Result<bool, IpcError> {
        let req = JsonRpcRequest::new(methods::health::LIVENESS, 1);
        let resp = self.call(&req).await?;
        Ok(resp.error.is_none())
    }

    /// Register a capability set with songbird via `ipc.register`.
    ///
    /// This is the ecosystem standard for primals to announce their
    /// capabilities at startup so other primals can discover them.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or protocol failure.
    pub async fn register_capabilities(
        &self,
        primal_name: &str,
        capabilities: &[Capability],
        endpoint: &crate::transport::TransportEndpoint,
    ) -> Result<JsonRpcResponse, IpcError> {
        let params = serde_json::json!({
            "primal": primal_name,
            "capabilities": capabilities,
            "endpoint": endpoint,
        });
        let req = JsonRpcRequest::new(methods::ipc::REGISTER, 1).with_params(params);
        self.call(&req).await
    }

    /// Resolve another primal's endpoint via songbird `ipc.resolve`.
    ///
    /// Returns the structured `TransportEndpoint` for the given primal,
    /// enabling fully dynamic discovery without hardcoded paths.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] if transport fails or songbird returns an error.
    pub async fn resolve_primal(
        &self,
        primal_name: &str,
    ) -> Result<crate::transport::TransportEndpoint, IpcError> {
        let params = serde_json::json!({ "primal": primal_name });
        let req = JsonRpcRequest::new(methods::ipc::RESOLVE, 1).with_params(params);
        let resp = self.call(&req).await?;

        let result = resp.result.ok_or_else(|| {
            let msg = resp
                .error
                .map_or_else(|| "no result".to_owned(), |e| e.message);
            IpcError::new(IpcErrorKind::DependencyUnavailable, msg)
        })?;

        serde_json::from_value(result).map_err(|e| {
            IpcError::new(IpcErrorKind::Internal, format!("deserialize endpoint: {e}"))
        })
    }

    /// Announce this primal to the ecosystem via `primal.announce`.
    ///
    /// Combines `ipc.register` (capability registration) with a startup
    /// announcement that other primals can subscribe to. This is the
    /// canonical "I'm alive and serving" signal.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] on transport or protocol failure.
    pub async fn announce(
        &self,
        primal_name: &str,
        version: &str,
        capabilities: &[Capability],
        endpoint: &crate::transport::TransportEndpoint,
    ) -> Result<JsonRpcResponse, IpcError> {
        let params = serde_json::json!({
            "primal": primal_name,
            "version": version,
            "capabilities": capabilities,
            "endpoint": endpoint,
        });
        let req = JsonRpcRequest::new(methods::primal::ANNOUNCE, 1).with_params(params);
        self.call(&req).await
    }
}

impl std::fmt::Debug for IpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcClient")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::methods::{capabilities, health, identity, ipc, lifecycle, primal, system};
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
        assert!(IpcError::new(IpcErrorKind::Timeout, "t").retryable);
        assert!(IpcError::new(IpcErrorKind::NotReady, "n").retryable);
        assert!(!IpcError::new(IpcErrorKind::CircuitBreakerOpen, "c").retryable);
        assert!(!IpcError::new(IpcErrorKind::Internal, "i").retryable);
    }

    #[tokio::test]
    async fn call_with_timeout_returns_timeout_error() {
        let client = IpcClient::new(crate::transport::TransportEndpoint::uds(
            "/tmp/nonexistent-primal-for-timeout-test.sock",
        ));
        let req = JsonRpcRequest::new("health.liveness", 1);
        let result = client
            .call_with_timeout(&req, std::time::Duration::from_millis(50))
            .await;
        let err = result.unwrap_err();
        assert!(err.kind == IpcErrorKind::Transport || err.kind == IpcErrorKind::Timeout);
    }

    #[test]
    fn default_timeout_is_reasonable() {
        assert_eq!(DEFAULT_IPC_TIMEOUT, std::time::Duration::from_secs(5));
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
        assert_eq!(ipc::RESOLVE, "ipc.resolve");
        assert_eq!(ipc::REGISTER, "ipc.register");
        assert_eq!(primal::ANNOUNCE, "primal.announce");
        assert_eq!(primal::SHUTDOWN, "primal.shutdown");
    }

    #[test]
    fn ipc_client_from_primal_constructs_uds() {
        let client = IpcClient::from_primal("songbird", None);
        let ep = client.endpoint();
        assert!(matches!(
            ep,
            crate::transport::TransportEndpoint::Uds { .. }
        ));
        assert!(ep.uds_path().unwrap().contains("songbird"));
    }

    #[test]
    fn health_probe_roundtrip() {
        let mut deps = HashMap::new();
        deps.insert("db".to_string(), "up".to_string());
        let probe = HealthProbe {
            primal: "test".into(),
            version: "0.1.0".into(),
            status: crate::health::HealthStatus::Healthy,
            live: true,
            ready: true,
            dependencies: deps,
        };
        let json = serde_json::to_string(&probe).expect("serialize");
        let back: HealthProbe = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.primal, "test");
        assert_eq!(back.status, crate::health::HealthStatus::Healthy);
        assert_eq!(back.dependencies.get("db").map(String::as_str), Some("up"));
    }

    #[tokio::test]
    async fn ipc_client_call_roundtrip_over_uds() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut buf = BufReader::new(stream);
            let mut line = String::new();
            buf.read_line(&mut line).await.unwrap();

            let req: JsonRpcRequest = serde_json::from_str(line.trim()).unwrap();
            let resp = JsonRpcResponse::success(req.id.unwrap(), serde_json::json!({"pong": true}));
            let resp_json = serde_json::to_string(&resp).unwrap();
            buf.get_mut()
                .write_all(format!("{resp_json}\n").as_bytes())
                .await
                .unwrap();
        });

        let ep = crate::transport::TransportEndpoint::uds(sock_path.to_str().unwrap());
        let client = IpcClient::new(ep);
        let req = JsonRpcRequest::new("system.ping", 42);
        let resp = client.call(&req).await.unwrap();

        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["pong"], true);
        server.await.unwrap();
    }
}
