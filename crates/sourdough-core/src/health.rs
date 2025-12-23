//! Health check traits for observability.
//!
//! Every primal needs to be observable. This module provides traits for
//! health checks that can be used by orchestrators, load balancers, and
//! monitoring systems.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// Overall health status of a primal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy and ready to serve requests.
    Healthy,
    /// Unhealthy but may recover.
    Degraded {
        /// Reason for degraded status.
        reason: String,
    },
    /// Unhealthy and not serving requests.
    Unhealthy {
        /// Reason for unhealthy status.
        reason: String,
    },
    /// Health unknown (e.g., startup in progress).
    Unknown,
}

impl HealthStatus {
    /// Check if the status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status allows serving requests.
    #[must_use]
    pub const fn is_serving(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded { reason } => write!(f, "degraded: {reason}"),
            Self::Unhealthy { reason } => write!(f, "unhealthy: {reason}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health of a dependency.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Name of the dependency.
    pub name: String,
    /// Type of dependency (e.g., "database", "service", "file").
    pub dependency_type: String,
    /// Health status.
    pub status: HealthStatus,
    /// Latency to the dependency (optional).
    pub latency_ms: Option<u64>,
    /// Last check time.
    pub last_check: crate::types::Timestamp,
}

impl DependencyHealth {
    /// Create a healthy dependency.
    #[must_use]
    pub fn healthy(name: impl Into<String>, dep_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependency_type: dep_type.into(),
            status: HealthStatus::Healthy,
            latency_ms: None,
            last_check: crate::types::Timestamp::now(),
        }
    }

    /// Create an unhealthy dependency.
    #[must_use]
    pub fn unhealthy(name: impl Into<String>, dep_type: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependency_type: dep_type.into(),
            status: HealthStatus::Unhealthy { reason: reason.into() },
            latency_ms: None,
            last_check: crate::types::Timestamp::now(),
        }
    }

    /// Set latency.
    #[must_use]
    pub const fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Full health report for a primal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    /// Primal name.
    pub name: String,
    /// Primal version.
    pub version: String,
    /// Overall status.
    pub status: HealthStatus,
    /// Liveness (is the process alive?).
    pub liveness: bool,
    /// Readiness (can it serve requests?).
    pub readiness: bool,
    /// Dependency health.
    pub dependencies: Vec<DependencyHealth>,
    /// Report timestamp.
    pub timestamp: crate::types::Timestamp,
    /// Additional details.
    pub details: std::collections::HashMap<String, String>,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Unknown,
            liveness: true,
            readiness: false,
            dependencies: Vec::new(),
            timestamp: crate::types::Timestamp::now(),
            details: std::collections::HashMap::new(),
        }
    }

    /// Set status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.readiness = status.is_serving();
        self.status = status;
        self
    }

    /// Add a dependency.
    #[must_use]
    pub fn with_dependency(mut self, dep: DependencyHealth) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add a detail.
    #[must_use]
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
}

/// Health check trait for primals.
///
/// Implement this trait to provide health information about your primal.
pub trait PrimalHealth: Send + Sync {
    /// Get the current health status.
    ///
    /// This should be a quick check suitable for high-frequency polling.
    fn health_status(&self) -> HealthStatus;

    /// Perform a full health check.
    ///
    /// This can be more expensive and include dependency checks.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check itself fails (not if unhealthy).
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;

    /// Check liveness (is the process alive?).
    ///
    /// Default returns `true` if state is Running.
    fn is_live(&self) -> bool {
        true
    }

    /// Check readiness (can it serve requests?).
    ///
    /// Default returns `true` if healthy.
    fn is_ready(&self) -> bool {
        self.health_status().is_serving()
    }

    /// Get dependency health (optional).
    ///
    /// Override to report on external dependencies.
    fn dependency_health(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<DependencyHealth>, PrimalError>> + Send {
        async { Ok(Vec::new()) }
    }
}

