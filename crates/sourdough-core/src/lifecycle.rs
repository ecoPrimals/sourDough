//! Primal lifecycle management.
//!
//! Every primal has a lifecycle: it starts, runs, and eventually stops.
//! This module provides traits for managing that lifecycle.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// State of a primal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalState {
    /// Not yet started.
    Created,
    /// Starting up.
    Starting,
    /// Running normally.
    Running,
    /// Stopping.
    Stopping,
    /// Stopped.
    Stopped,
    /// Failed.
    Failed,
}

impl PrimalState {
    /// Check if the primal is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal can be started.
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(self, Self::Created | Self::Stopped | Self::Failed)
    }

    /// Check if the primal can be stopped.
    #[must_use]
    pub const fn can_stop(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for PrimalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Lifecycle trait for primals.
///
/// Implement this trait to define how your primal starts, stops, and reloads.
///
/// # Example
///
/// ```rust,ignore
/// use sourdough_core::{PrimalLifecycle, PrimalState, PrimalError};
///
/// struct MyPrimal {
///     state: PrimalState,
///     // ...
/// }
///
/// #[async_trait::async_trait]
/// impl PrimalLifecycle for MyPrimal {
///     fn state(&self) -> PrimalState {
///         self.state
///     }
///
///     async fn start(&mut self) -> Result<(), PrimalError> {
///         self.state = PrimalState::Starting;
///         // Initialize resources...
///         self.state = PrimalState::Running;
///         Ok(())
///     }
///
///     async fn stop(&mut self) -> Result<(), PrimalError> {
///         self.state = PrimalState::Stopping;
///         // Clean up resources...
///         self.state = PrimalState::Stopped;
///         Ok(())
///     }
/// }
/// ```
pub trait PrimalLifecycle: Send + Sync {
    /// Get the current state.
    fn state(&self) -> PrimalState;

    /// Start the primal.
    ///
    /// This should:
    /// 1. Initialize resources
    /// 2. Start background tasks
    /// 3. Register with Songbird (if applicable)
    ///
    /// # Errors
    ///
    /// Returns an error if startup fails.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal.
    ///
    /// This should:
    /// 1. Deregister from Songbird
    /// 2. Stop background tasks
    /// 3. Clean up resources
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Reload configuration.
    ///
    /// Default implementation stops and restarts.
    ///
    /// # Errors
    ///
    /// Returns an error if reload fails.
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async {
            self.stop().await?;
            self.start().await
        }
    }

    /// Handle a shutdown signal.
    ///
    /// Called when the process receives SIGTERM or similar.
    /// Default implementation calls `stop()`.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async { self.stop().await }
    }
}

/// Context for lifecycle events.
#[derive(Clone, Debug)]
pub struct LifecycleContext {
    /// Reason for the lifecycle event.
    pub reason: LifecycleReason,
    /// Whether this is a graceful transition.
    pub graceful: bool,
}

/// Reason for a lifecycle transition.
#[derive(Clone, Debug)]
pub enum LifecycleReason {
    /// User/operator initiated.
    UserInitiated,
    /// Configuration reload.
    ConfigReload,
    /// Health check failure.
    HealthFailure,
    /// Dependency failure.
    DependencyFailure(String),
    /// Signal received (e.g., SIGTERM).
    Signal(i32),
    /// Internal error.
    Error(String),
}

