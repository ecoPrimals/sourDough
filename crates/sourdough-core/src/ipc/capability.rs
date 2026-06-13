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
