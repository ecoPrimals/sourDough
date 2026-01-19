//! Discovery traits for universal adapter integration.
//!
//! Every primal needs to be discoverable. This module provides traits for
//! registering with discovery services via the universal adapter and broadcasting
//! presence to the network.

use crate::error::PrimalError;
use crate::identity::Did;
use serde::{Deserialize, Serialize};

/// Service registration for discovery services.
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
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
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
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        protocol: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            protocol: protocol.into(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// `BirdSong` broadcast configuration.
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
/// Implement this trait to integrate with discovery services via the universal adapter.
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
    fn deregister(&self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Get `BirdSong` configuration (optional).
    ///
    /// Returns `None` if `BirdSong` is not used.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_registration_builder() {
        // In real usage, endpoint would be discovered dynamically
        // This test uses a placeholder for demonstration only
        let reg = ServiceRegistration::new("test-service", "1.0.0", "http://test-endpoint:0")
            .with_capability(UpaCapability::new("storage", "1.0", "grpc"))
            .with_metadata("region", "us-west")
            .with_health_endpoint("/health");

        assert_eq!(reg.name, "test-service");
        assert_eq!(reg.version, "1.0.0");
        assert_eq!(reg.endpoint, "http://test-endpoint:0");
        assert_eq!(reg.capabilities.len(), 1);
        assert_eq!(reg.metadata.get("region"), Some(&"us-west".to_string()));
        assert_eq!(reg.health_endpoint, Some("/health".to_string()));
    }

    #[test]
    fn upa_capability_creation() {
        let cap = UpaCapability::new("compute", "2.0", "rest");

        assert_eq!(cap.name, "compute");
        assert_eq!(cap.version, "2.0");
        assert_eq!(cap.protocol, "rest");
        assert!(cap.metadata.is_empty());
    }

    #[test]
    fn birdsong_config_default() {
        let config = BirdSongConfig::default();

        assert!(config.enabled);
        assert_eq!(config.interval_secs, 30);
        assert!(config.lineage_gated);
        assert!(config.encrypted);
    }

    #[test]
    fn birdsong_config_custom() {
        let config = BirdSongConfig {
            enabled: false,
            interval_secs: 60,
            lineage_gated: false,
            encrypted: false,
        };

        assert!(!config.enabled);
        assert_eq!(config.interval_secs, 60);
    }

    #[test]
    fn service_info_serialization() {
        let info = ServiceInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://test".to_string(),
            did: Did::new("did:key:test123"),
            capabilities: vec!["storage".to_string()],
            is_family: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ServiceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.name, parsed.name);
        assert_eq!(info.is_family, parsed.is_family);
    }

    #[test]
    fn registration_handle_creation() {
        let handle = RegistrationHandle {
            id: "reg-123".to_string(),
            service_name: "test-service".to_string(),
            registered_at: crate::types::Timestamp::now(),
        };

        assert_eq!(handle.id, "reg-123");
        assert_eq!(handle.service_name, "test-service");
    }

    // Mock implementation for testing
    struct MockDiscoveryPrimal {
        service_name: String,
    }

    impl MockDiscoveryPrimal {
        fn new(name: impl Into<String>) -> Self {
            Self {
                service_name: name.into(),
            }
        }
    }

    impl PrimalDiscovery for MockDiscoveryPrimal {
        fn registration(&self) -> ServiceRegistration {
            // In tests, use OS-assigned port (0) to avoid hardcoding
            ServiceRegistration::new(&self.service_name, "1.0.0", "http://test-endpoint:0")
                .with_capability(UpaCapability::new("test", "1.0", "grpc"))
        }

        async fn register(&self) -> Result<RegistrationHandle, PrimalError> {
            Ok(RegistrationHandle {
                id: format!("reg-{}", self.service_name),
                service_name: self.service_name.clone(),
                registered_at: crate::types::Timestamp::now(),
            })
        }

        async fn deregister(&self) -> Result<(), PrimalError> {
            Ok(())
        }

        fn birdsong_config(&self) -> Option<BirdSongConfig> {
            Some(BirdSongConfig::default())
        }

        async fn discover(&self, _service_name: &str) -> Result<Vec<ServiceInfo>, PrimalError> {
            Ok(vec![])
        }

        async fn discover_by_capability(
            &self,
            _capability: &str,
        ) -> Result<Vec<ServiceInfo>, PrimalError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn trait_registration() {
        let primal = MockDiscoveryPrimal::new("test-primal");

        let reg = primal.registration();
        assert_eq!(reg.name, "test-primal");
        assert_eq!(reg.capabilities.len(), 1);

        let handle = primal.register().await.unwrap();
        assert_eq!(handle.service_name, "test-primal");

        primal.deregister().await.unwrap();
    }

    #[tokio::test]
    async fn trait_birdsong_config() {
        let primal = MockDiscoveryPrimal::new("test");

        let config = primal.birdsong_config();
        assert!(config.is_some());

        let config = config.unwrap();
        assert!(config.enabled);
        assert!(config.encrypted);
    }

    #[tokio::test]
    async fn trait_discovery() {
        let primal = MockDiscoveryPrimal::new("test");

        let services = primal.discover("other-service").await.unwrap();
        assert!(services.is_empty());

        let services = primal.discover_by_capability("storage").await.unwrap();
        assert!(services.is_empty());
    }
}
