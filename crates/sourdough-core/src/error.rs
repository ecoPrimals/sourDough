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
            Self::Network(_) | Self::Timeout(_) | Self::Dependency { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_config() {
        let err = PrimalError::config("invalid setting");
        assert!(matches!(err, PrimalError::Config(_)));
        assert_eq!(err.to_string(), "configuration error: invalid setting");
    }

    #[test]
    fn error_identity() {
        let err = PrimalError::identity("invalid DID");
        assert!(matches!(err, PrimalError::Identity(_)));
        assert_eq!(err.to_string(), "identity error: invalid DID");
    }

    #[test]
    fn error_discovery() {
        let err = PrimalError::discovery("service not found");
        assert!(matches!(err, PrimalError::Discovery(_)));
        assert_eq!(err.to_string(), "discovery error: service not found");
    }

    #[test]
    fn error_lifecycle() {
        let err = PrimalError::lifecycle("cannot start");
        assert!(matches!(err, PrimalError::Lifecycle(_)));
        assert_eq!(err.to_string(), "lifecycle error: cannot start");
    }

    #[test]
    fn error_dependency() {
        let err = PrimalError::dependency("database", "connection failed");
        assert!(matches!(err, PrimalError::Dependency { .. }));
        assert_eq!(
            err.to_string(),
            "dependency error: database: connection failed"
        );
    }

    #[test]
    fn error_domain() {
        let err = PrimalError::domain("custom", "domain error");
        assert!(matches!(err, PrimalError::Domain { .. }));
        assert_eq!(err.to_string(), "custom error: domain error");
    }

    #[test]
    fn error_retryable() {
        assert!(PrimalError::Network("timeout".to_string()).is_retryable());
        assert!(PrimalError::Timeout("slow".to_string()).is_retryable());
        assert!(PrimalError::dependency("db", "down").is_retryable());

        assert!(!PrimalError::Config("bad".to_string()).is_retryable());
        assert!(!PrimalError::InvalidInput("wrong".to_string()).is_retryable());
        assert!(!PrimalError::PermissionDenied("forbidden".to_string()).is_retryable());
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: PrimalError = io_err.into();
        assert!(matches!(err, PrimalError::Io(_)));
    }

    #[test]
    fn error_variants_coverage() {
        // Ensure all variants can be created
        let _ = PrimalError::Config("test".to_string());
        let _ = PrimalError::Identity("test".to_string());
        let _ = PrimalError::Discovery("test".to_string());
        let _ = PrimalError::Lifecycle("test".to_string());
        let _ = PrimalError::Health("test".to_string());
        let _ = PrimalError::Serialization("test".to_string());
        let _ = PrimalError::Network("test".to_string());
        let _ = PrimalError::Storage("test".to_string());
        let _ = PrimalError::Timeout("test".to_string());
        let _ = PrimalError::Cancelled("test".to_string());
        let _ = PrimalError::NotFound("test".to_string());
        let _ = PrimalError::AlreadyExists("test".to_string());
        let _ = PrimalError::PermissionDenied("test".to_string());
        let _ = PrimalError::InvalidInput("test".to_string());
        let _ = PrimalError::Internal("test".to_string());
    }

    #[test]
    fn result_type_alias() {
        fn test_function(val: i32) -> PrimalResult<i32> {
            if val > 0 {
                Ok(val)
            } else {
                Err(PrimalError::config("value must be positive"))
            }
        }

        assert_eq!(test_function(42).unwrap(), 42);
        assert!(test_function(-1).is_err());
    }
}
