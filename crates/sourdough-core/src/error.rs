//! Common error types for primals.

use thiserror::Error;

/// Result type for primal operations.
pub type PrimalResult<T> = Result<T, PrimalError>;

/// Common errors that any primal might encounter.
#[derive(Debug, Error)]
pub enum PrimalError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Identity/signing error.
    #[error("identity error: {0}")]
    Identity(String),

    /// Discovery/registration error.
    #[error("discovery error: {0}")]
    Discovery(String),

    /// Lifecycle error (start/stop/reload).
    #[error("lifecycle error: {0}")]
    Lifecycle(String),

    /// Health check error.
    #[error("health error: {0}")]
    Health(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Timeout.
    #[error("operation timed out: {0}")]
    Timeout(String),

    /// Operation cancelled.
    #[error("operation cancelled: {0}")]
    Cancelled(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Dependency error (upstream service failed).
    #[error("dependency error: {service}: {message}")]
    Dependency {
        /// Name of the dependency that failed.
        service: String,
        /// Error message.
        message: String,
    },

    /// Custom domain-specific error.
    ///
    /// Use this for errors specific to your primal that don't fit
    /// the common categories.
    #[error("{domain} error: {message}")]
    Domain {
        /// Domain/primal name.
        domain: String,
        /// Error message.
        message: String,
    },
}

impl PrimalError {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an identity error.
    pub fn identity(msg: impl Into<String>) -> Self {
        Self::Identity(msg.into())
    }

    /// Create a discovery error.
    pub fn discovery(msg: impl Into<String>) -> Self {
        Self::Discovery(msg.into())
    }

    /// Create a lifecycle error.
    pub fn lifecycle(msg: impl Into<String>) -> Self {
        Self::Lifecycle(msg.into())
    }

    /// Create a dependency error.
    pub fn dependency(service: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Dependency {
            service: service.into(),
            message: msg.into(),
        }
    }

    /// Create a domain-specific error.
    pub fn domain(domain: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Domain {
            domain: domain.into(),
            message: msg.into(),
        }
    }

    /// Check if this is a retryable error.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::Timeout(_)
                | Self::Dependency { .. }
        )
    }
}

