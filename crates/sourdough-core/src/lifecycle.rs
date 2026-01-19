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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions() {
        assert!(PrimalState::Created.can_start());
        assert!(!PrimalState::Created.can_stop());
        assert!(!PrimalState::Created.is_running());

        assert!(!PrimalState::Running.can_start());
        assert!(PrimalState::Running.can_stop());
        assert!(PrimalState::Running.is_running());

        assert!(PrimalState::Stopped.can_start());
        assert!(!PrimalState::Stopped.can_stop());

        assert!(PrimalState::Failed.can_start());
        assert!(!PrimalState::Failed.can_stop());
    }

    #[test]
    fn state_display() {
        assert_eq!(PrimalState::Created.to_string(), "created");
        assert_eq!(PrimalState::Running.to_string(), "running");
        assert_eq!(PrimalState::Failed.to_string(), "failed");
    }

    #[test]
    fn state_serialization() {
        let state = PrimalState::Running;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PrimalState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    // Mock implementation for testing trait
    struct MockPrimal {
        state: PrimalState,
        start_count: usize,
        stop_count: usize,
    }

    impl MockPrimal {
        fn new() -> Self {
            Self {
                state: PrimalState::Created,
                start_count: 0,
                stop_count: 0,
            }
        }
    }

    impl PrimalLifecycle for MockPrimal {
        fn state(&self) -> PrimalState {
            self.state
        }

        async fn start(&mut self) -> Result<(), PrimalError> {
            if !self.state.can_start() {
                return Err(PrimalError::lifecycle(format!(
                    "cannot start from state: {}",
                    self.state
                )));
            }
            self.state = PrimalState::Running;
            self.start_count += 1;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), PrimalError> {
            if !self.state.can_stop() {
                return Err(PrimalError::lifecycle(format!(
                    "cannot stop from state: {}",
                    self.state
                )));
            }
            self.state = PrimalState::Stopped;
            self.stop_count += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn lifecycle_start_stop() {
        let mut primal = MockPrimal::new();
        
        assert_eq!(primal.state(), PrimalState::Created);
        
        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);
        assert_eq!(primal.start_count, 1);
        
        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
        assert_eq!(primal.stop_count, 1);
    }

    #[tokio::test]
    async fn lifecycle_invalid_transitions() {
        let mut primal = MockPrimal::new();
        primal.state = PrimalState::Running;
        
        // Can't start when already running
        let result = primal.start().await;
        assert!(result.is_err());
        
        // Reset
        primal.state = PrimalState::Created;
        
        // Can't stop when not running
        let result = primal.stop().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn lifecycle_reload() {
        let mut primal = MockPrimal::new();
        primal.start().await.unwrap();
        
        assert_eq!(primal.start_count, 1);
        assert_eq!(primal.stop_count, 0);
        
        primal.reload().await.unwrap();
        
        assert_eq!(primal.start_count, 2);
        assert_eq!(primal.stop_count, 1);
        assert_eq!(primal.state(), PrimalState::Running);
    }

    #[tokio::test]
    async fn lifecycle_shutdown() {
        let mut primal = MockPrimal::new();
        primal.start().await.unwrap();
        
        primal.shutdown().await.unwrap();
        
        assert_eq!(primal.state(), PrimalState::Stopped);
        assert_eq!(primal.stop_count, 1);
    }
}
