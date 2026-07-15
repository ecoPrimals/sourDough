//! Capability declaration types for `capabilities.list` wire standard.

use serde::{Deserialize, Serialize};

/// A capability that a primal can expose via `capabilities.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability domain (e.g., "storage", "crypto", "health").
    pub domain: String,
    /// Available methods within this domain.
    pub methods: Vec<String>,
    /// Capability version.
    pub version: String,
}

impl Capability {
    /// Create a new capability.
    #[must_use]
    pub fn new(domain: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            methods: Vec::new(),
            version: version.into(),
        }
    }

    /// Add a method to this capability.
    #[must_use]
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.methods.push(method.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_builder() {
        let cap = Capability::new("storage", "1.0")
            .with_method("storage.get")
            .with_method("storage.put");
        assert_eq!(cap.domain, "storage");
        assert_eq!(cap.version, "1.0");
        assert_eq!(cap.methods, vec!["storage.get", "storage.put"]);
    }

    #[test]
    fn capability_serde_roundtrip() {
        let cap = Capability::new("health", "0.1").with_method("health.check");
        let json = serde_json::to_string(&cap).unwrap();
        let back: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(back.domain, "health");
        assert_eq!(back.methods, vec!["health.check"]);
    }
}
