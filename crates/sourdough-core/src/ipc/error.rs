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
