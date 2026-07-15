//! IPC error types for structured inter-primal communication failures.

use serde::{Deserialize, Serialize};

/// Structured IPC error for inter-primal communication.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_kinds() {
        let retryable = [
            IpcErrorKind::Transport,
            IpcErrorKind::Timeout,
            IpcErrorKind::DependencyUnavailable,
            IpcErrorKind::RateLimited,
            IpcErrorKind::NotReady,
        ];
        for kind in retryable {
            let err = IpcError::new(kind, "test");
            assert!(err.retryable, "{kind:?} should be retryable");
        }
    }

    #[test]
    fn non_retryable_kinds() {
        let non_retryable = [
            IpcErrorKind::CircuitBreakerOpen,
            IpcErrorKind::MethodNotFound,
            IpcErrorKind::InvalidParams,
            IpcErrorKind::Internal,
        ];
        for kind in non_retryable {
            let err = IpcError::new(kind, "test");
            assert!(!err.retryable, "{kind:?} should not be retryable");
        }
    }

    #[test]
    fn from_primal_sets_source() {
        let err = IpcError::new(IpcErrorKind::Timeout, "slow").from_primal("songbird");
        assert_eq!(err.source_primal.as_deref(), Some("songbird"));
    }

    #[test]
    fn display_with_primal() {
        let err = IpcError::new(IpcErrorKind::Transport, "conn refused").from_primal("beardog");
        let s = err.to_string();
        assert!(s.contains("beardog"));
        assert!(s.contains("Transport"));
        assert!(s.contains("conn refused"));
    }

    #[test]
    fn display_without_primal() {
        let err = IpcError::new(IpcErrorKind::Internal, "oops");
        let s = err.to_string();
        assert!(!s.contains('['));
        assert!(s.contains("Internal"));
        assert!(s.contains("oops"));
    }

    #[test]
    fn serde_roundtrip() {
        let err =
            IpcError::new(IpcErrorKind::MethodNotFound, "bad.method").from_primal("test-primal");
        let json = serde_json::to_string(&err).unwrap();
        let back: IpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, IpcErrorKind::MethodNotFound);
        assert_eq!(back.message, "bad.method");
        assert_eq!(back.source_primal.as_deref(), Some("test-primal"));
        assert!(!back.retryable);
    }
}
