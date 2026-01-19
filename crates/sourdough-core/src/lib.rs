//! # `SourDough` Core
//!
//! The essential traits and patterns that all ecoPrimals share.
//!
//! `SourDough` provides the minimal, agnostic foundation for building new primals.
//! It makes no assumptions about what your primal does—only that it needs to:
//!
//! - Have an identity (`BearDog` integration)
//! - Be discoverable (`Songbird` integration)
//! - Have a lifecycle (start, stop, reload)
//! - Be observable (health checks)
//! - Be configurable
//!
//! ## Example
//!
//! ```rust,ignore
//! use sourdough_core::{
//!     PrimalLifecycle, PrimalHealth, PrimalConfig,
//!     HealthStatus, PrimalError,
//! };
//!
//! pub struct MyPrimal {
//!     config: MyConfig,
//!     running: bool,
//! }
//!
//! #[async_trait::async_trait]
//! impl PrimalLifecycle for MyPrimal {
//!     async fn start(&mut self) -> Result<(), PrimalError> {
//!         self.running = true;
//!         Ok(())
//!     }
//!
//!     async fn stop(&mut self) -> Result<(), PrimalError> {
//!         self.running = false;
//!         Ok(())
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod discovery;
pub mod error;
pub mod health;
pub mod identity;
pub mod lifecycle;
pub mod rpc;
pub mod types;

// Re-exports for convenience
pub use config::{ConfigLoader, PrimalConfig};
pub use discovery::{PrimalDiscovery, ServiceRegistration, UpaCapability};
pub use error::{PrimalError, PrimalResult};
pub use health::{DependencyHealth, HealthStatus, PrimalHealth};
pub use identity::{Did, PrimalIdentity, Signature};
pub use lifecycle::{PrimalLifecycle, PrimalState};
pub use types::{ContentHash, Timestamp};
