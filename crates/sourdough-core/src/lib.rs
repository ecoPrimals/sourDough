//! # `SourDough` Core
//!
//! The essential traits and patterns that all ecoPrimals share.
//!
//! `SourDough` provides the minimal, agnostic foundation for building new primals.
//! It makes no assumptions about what your primal does—only that it needs to:
//!
//! - Have an identity (identity service via universal adapter)
//! - Be discoverable (discovery service via universal adapter)
//! - Have a lifecycle (start, stop, reload)
//! - Be observable (health checks)
//! - Be configurable
//!
//! ## Example
//!
//! ```
//! use sourdough_core::{PrimalLifecycle, PrimalState, PrimalError};
//!
//! struct MyPrimal { state: PrimalState }
//!
//! impl PrimalLifecycle for MyPrimal {
//!     fn state(&self) -> PrimalState { self.state }
//!
//!     async fn start(&mut self) -> Result<(), PrimalError> {
//!         self.state = PrimalState::Running;
//!         Ok(())
//!     }
//!
//!     async fn stop(&mut self) -> Result<(), PrimalError> {
//!         self.state = PrimalState::Stopped;
//!         Ok(())
//!     }
//! }
//! ```

pub mod config;
pub mod discovery;
pub mod error;
pub mod health;
pub mod identity;
pub mod ipc;
pub mod lifecycle;
pub mod rpc;
pub mod types;

// Re-exports for convenience
pub use config::{ConfigLoader, PrimalConfig};
pub use discovery::{PrimalDiscovery, ServiceRegistration, UpaCapability};
pub use error::{PrimalError, PrimalResult};
pub use health::{DependencyHealth, HealthStatus, PrimalHealth};
pub use identity::{Did, PrimalIdentity, Signature};
pub use ipc::{
    Capability, CircuitBreaker, CircuitState, HealthProbe, IpcError, IpcErrorKind, JsonRpcError,
    JsonRpcRequest, JsonRpcResponse,
};
pub use lifecycle::{PrimalLifecycle, PrimalState};
pub use types::{ContentHash, Timestamp};
