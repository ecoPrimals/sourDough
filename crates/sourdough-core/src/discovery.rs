//! Discovery traits for Songbird integration.
//!
//! Every primal needs to be discoverable. This module provides traits for
//! registering with Songbird's Universal Port Authority (UPA) and broadcasting
//! presence via BirdSong.

use crate::error::PrimalError;
use crate::identity::Did;
use serde::{Deserialize, Serialize};

/// Service registration for UPA.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Service name (e.g., "rhizocrypt", "loamspine").
    pub name: String,
    /// Service version (semver).
    pub version: String,
    /// Service endpoint URL.
    pub endpoint: String,
    /// Service capabilities.
    pub capabilities: Vec<UpaCapability>,
    /// Service metadata.
    pub metadata: std::collections::HashMap<String, String>,
    /// Health check endpoint (optional).
    pub health_endpoint: Option<String>,
}

impl ServiceRegistration {
    /// Create a new service registration.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            endpoint: endpoint.into(),
            capabilities: Vec::new(),
            metadata: std::collections::HashMap::new(),
            health_endpoint: None,
        }
    }

    /// Add a capability.
    #[must_use]
    pub fn with_capability(mut self, cap: UpaCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set health endpoint.
    #[must_use]
    pub fn with_health_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.health_endpoint = Some(endpoint.into());
        self
    }
}

/// UPA capability declaration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpaCapability {
    /// Capability name (e.g., "storage", "compute", "security").
    pub name: String,
    /// Capability version.
    pub version: String,
    /// Protocol (e.g., "grpc", "rest", "websocket").
    pub protocol: String,
    /// Additional capability metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

impl UpaCapability {
    /// Create a new capability.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>, protocol: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            protocol: protocol.into(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// BirdSong broadcast configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BirdSongConfig {
    /// Whether to broadcast presence.
    pub enabled: bool,
    /// Broadcast interval.
    pub interval_secs: u64,
    /// Lineage gating (only visible to family).
    pub lineage_gated: bool,
    /// Encryption enabled.
    pub encrypted: bool,
}

impl Default for BirdSongConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            lineage_gated: true,
            encrypted: true,
        }
    }
}

/// Discovery trait for primals.
///
/// Implement this trait to integrate with Songbird for service discovery.
pub trait PrimalDiscovery: Send + Sync {
    /// Get the service registration for UPA.
    fn registration(&self) -> ServiceRegistration;

    /// Register with UPA.
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails.
    fn register(
        &self,
    ) -> impl std::future::Future<Output = Result<RegistrationHandle, PrimalError>> + Send;

    /// Deregister from UPA.
    ///
    /// # Errors
    ///
    /// Returns an error if deregistration fails.
    fn deregister(
        &self,
    ) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Get BirdSong configuration (optional).
    ///
    /// Returns `None` if BirdSong is not used.
    fn birdsong_config(&self) -> Option<BirdSongConfig> {
        None
    }

    /// Discover a service by name.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    fn discover(
        &self,
        service_name: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send;

    /// Discover a service by capability.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    fn discover_by_capability(
        &self,
        capability: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send;
}

/// Handle to a service registration.
#[derive(Clone, Debug)]
pub struct RegistrationHandle {
    /// Registration ID.
    pub id: String,
    /// Service name.
    pub service_name: String,
    /// Registration timestamp.
    pub registered_at: crate::types::Timestamp,
}

/// Information about a discovered service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name.
    pub name: String,
    /// Service version.
    pub version: String,
    /// Service endpoint.
    pub endpoint: String,
    /// Service DID.
    pub did: Did,
    /// Service capabilities.
    pub capabilities: Vec<String>,
    /// Whether this service is in our lineage.
    pub is_family: bool,
}

