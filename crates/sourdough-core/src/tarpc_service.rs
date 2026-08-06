//! Canonical tarpc service definitions for the ecoPrimals ecosystem (G64 Cephalization).
//!
//! This module defines the **standard tarpc service trait** that all primals must implement
//! when serving a `.tarpc.sock` endpoint. It is the binary-protocol counterpart to the
//! JSON-RPC methods defined in [`crate::methods`].
//!
//! ## Dual-Protocol Architecture
//!
//! Every cephalization-era primal exposes two Unix Domain Sockets:
//! - `{name}.sock` — JSON-RPC 2.0 (discovery, diagnostics, browser/REST clients)
//! - `{name}.tarpc.sock` — tarpc binary (intra-gate composition, sub-ms latency)
//!
//! The tarpc service carries the same semantic operations as JSON-RPC but with:
//! - Binary framing (bincode) — no JSON parsing overhead
//! - Zero-copy where possible (`Vec<u8>` payloads)
//! - Strongly typed — compile-time method resolution
//!
//! ## Usage
//!
//! Primals implement this trait on their handler struct:
//!
//! ```ignore
//! use sourdough_core::tarpc_service::PrimalService;
//!
//! #[derive(Clone)]
//! struct MyHandler { /* ... */ }
//!
//! impl PrimalService for MyHandler {
//!     async fn health_liveness(self, _: tarpc::context::Context) -> bool { true }
//!     async fn health_readiness(self, _: tarpc::context::Context) -> bool { true }
//!     // ...
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Health check response for tarpc liveness/readiness probes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    /// Whether the primal is alive and responding.
    pub alive: bool,
    /// Whether the primal is ready to serve requests.
    pub ready: bool,
    /// Primal name for identification.
    pub primal: String,
    /// Version string.
    pub version: String,
}

/// Capability descriptor returned by `capabilities_list`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TarpcCapability {
    /// Capability domain (e.g. "content", "health", "security").
    pub domain: String,
    /// Available methods within this domain.
    pub methods: Vec<String>,
}

/// Identity response for `identity_did`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityResponse {
    /// The primal's DID (`did:method:id`).
    pub did: String,
    /// Protocol support level.
    pub protocol: String,
}

/// The canonical tarpc service that all cephalization-era primals implement.
///
/// This corresponds 1:1 with the standard JSON-RPC methods in [`crate::methods`]:
/// - `health.liveness` → `health_liveness`
/// - `health.readiness` → `health_readiness`
/// - `health.check` → `health_check`
/// - `capabilities.list` → `capabilities_list`
/// - `identity.did` → `identity_did`
/// - `system.ping` → `system_ping`
/// - `system.version` → `system_version`
/// - `lifecycle.state` → `lifecycle_state`
///
/// Domain-specific methods are defined by each primal's own `#[tarpc::service]` trait
/// that extends these baseline capabilities.
#[tarpc::service]
pub trait PrimalService {
    /// Liveness probe — is the process alive and able to respond?
    async fn health_liveness() -> bool;

    /// Readiness probe — is the primal ready to serve domain requests?
    async fn health_readiness() -> bool;

    /// Full health check with structured response.
    async fn health_check() -> HealthResponse;

    /// List all capabilities this primal offers.
    async fn capabilities_list() -> Vec<TarpcCapability>;

    /// Get the primal's decentralized identifier.
    async fn identity_did() -> IdentityResponse;

    /// Ping for latency measurement (returns "pong").
    async fn system_ping() -> String;

    /// Get the primal's version string.
    async fn system_version() -> String;

    /// Get the current lifecycle state (e.g. "running", "draining", "stopped").
    async fn lifecycle_state() -> String;
}

/// Default socket path convention for a primal's tarpc endpoint.
///
/// Convention: `$XDG_RUNTIME_DIR/biomeos/{name}.tarpc.sock`
#[must_use]
pub fn default_tarpc_socket_path(primal_name: &str) -> String {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    format!("{runtime_dir}/biomeos/{primal_name}.tarpc.sock")
}

/// Connect to a primal's tarpc service over UDS and return a client stub.
///
/// # Errors
///
/// Returns an IO error if the socket doesn't exist or connection fails.
#[cfg(unix)]
pub async fn connect_primal(socket_path: &str) -> Result<PrimalServiceClient, std::io::Error> {
    let transport =
        tarpc::serde_transport::unix::connect(socket_path, tokio_serde::formats::Bincode::default)
            .await?;

    Ok(PrimalServiceClient::new(tarpc::client::Config::default(), transport).spawn())
}

/// Connect to a primal by name using the default socket convention.
///
/// # Errors
///
/// Returns an IO error if the socket doesn't exist or connection fails.
#[cfg(unix)]
pub async fn connect_primal_by_name(
    primal_name: &str,
) -> Result<PrimalServiceClient, std::io::Error> {
    let path = default_tarpc_socket_path(primal_name);
    connect_primal(&path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_serde_roundtrip() {
        let resp = HealthResponse {
            alive: true,
            ready: true,
            primal: "test".to_owned(),
            version: "0.1.0".to_owned(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: HealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn tarpc_capability_serde_roundtrip() {
        let cap = TarpcCapability {
            domain: "health".to_owned(),
            methods: vec!["liveness".to_owned(), "readiness".to_owned()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let back: TarpcCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, back);
    }

    #[test]
    fn identity_response_serde_roundtrip() {
        let id = IdentityResponse {
            did: "did:key:z6MkTest".to_owned(),
            protocol: "dual".to_owned(),
        };
        let json = serde_json::to_string(&id).unwrap();
        let back: IdentityResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn default_tarpc_socket_path_format() {
        let path = default_tarpc_socket_path("beardog");
        assert!(path.ends_with("/biomeos/beardog.tarpc.sock"));
    }

    #[test]
    fn default_tarpc_socket_path_contains_runtime_dir() {
        let path = default_tarpc_socket_path("songbird");
        assert!(path.contains("biomeos"));
        assert!(path.ends_with(".tarpc.sock"));
    }
}
