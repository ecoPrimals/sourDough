//! Metadata handling for genomeBins.
//!
//! Type-safe representation of genomeBin metadata.toml files.

use crate::error::{GenomeBinError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// genomeBin metadata.
///
/// This replaces string-based bash metadata handling with type-safe structs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metadata {
    /// Genome information
    pub genome: GenomeInfo,
    /// Architecture mappings (target -> ecoBin path)
    pub architectures: HashMap<String, PathBuf>,
}

/// Genome information section.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenomeInfo {
    /// Primal name
    pub primal: String,
    /// Version string
    pub version: String,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// Number of architectures included
    pub architecture_count: usize,
}

impl Metadata {
    /// Create new metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal name or version is invalid.
    pub fn new(
        primal: impl Into<String>,
        version: impl Into<String>,
        architectures: HashMap<String, PathBuf>,
    ) -> Result<Self> {
        let primal = primal.into();
        let version = version.into();

        // Validate primal name (alphanumeric and hyphens only)
        if !primal
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(GenomeBinError::InvalidPrimalName(primal));
        }

        // Validate version (basic semver check)
        if version.is_empty() || !version.chars().any(|c| c.is_ascii_digit()) {
            return Err(GenomeBinError::InvalidVersion(version));
        }

        let architecture_count = architectures.len();
        let created = chrono::Utc::now().to_rfc3339();

        Ok(Self {
            genome: GenomeInfo {
                primal,
                version,
                created,
                architecture_count,
            },
            architectures,
        })
    }

    /// Load metadata from TOML string.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML is invalid.
    pub fn from_toml(toml: &str) -> Result<Self> {
        toml::from_str(toml).map_err(GenomeBinError::MetadataParse)
    }

    /// Load metadata from file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_toml(&content)
    }

    /// Serialize metadata to TOML string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(GenomeBinError::MetadataSerialize)
    }

    /// Write metadata to file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub async fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let toml = self.to_toml()?;
        tokio::fs::write(path, toml).await?;
        Ok(())
    }

    /// Get the primal name.
    #[must_use]
    pub fn primal(&self) -> &str {
        &self.genome.primal
    }

    /// Get the version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.genome.version
    }

    /// Get available architectures.
    #[must_use]
    pub fn architectures(&self) -> Vec<&str> {
        self.architectures.keys().map(String::as_str).collect()
    }

    /// Find ecoBin for target.
    ///
    /// Returns the path to the ecoBin for the given target, if available.
    #[must_use]
    pub fn find_ecobin(&self, target: &str) -> Option<&Path> {
        self.architectures.get(target).map(PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_creation() {
        let mut arches = HashMap::new();
        arches.insert(
            "x86_64-musl".to_string(),
            PathBuf::from("ecobins/primal-x86_64-musl"),
        );
        arches.insert(
            "aarch64-musl".to_string(),
            PathBuf::from("ecobins/primal-aarch64-musl"),
        );

        let metadata = Metadata::new("testprimal", "1.0.0", arches).unwrap();

        assert_eq!(metadata.primal(), "testprimal");
        assert_eq!(metadata.version(), "1.0.0");
        assert_eq!(metadata.genome.architecture_count, 2);
    }

    #[test]
    fn metadata_invalid_primal_name() {
        let arches = HashMap::new();
        let err = Metadata::new("test primal!", "1.0.0", arches).unwrap_err();
        assert!(matches!(err, GenomeBinError::InvalidPrimalName(_)));
    }

    #[test]
    fn metadata_invalid_version() {
        let arches = HashMap::new();
        let err = Metadata::new("testprimal", "", arches).unwrap_err();
        assert!(matches!(err, GenomeBinError::InvalidVersion(_)));
    }

    #[test]
    fn metadata_toml_roundtrip() {
        let mut arches = HashMap::new();
        arches.insert(
            "x86_64-musl".to_string(),
            PathBuf::from("ecobins/primal-x86_64-musl"),
        );

        let metadata = Metadata::new("testprimal", "1.0.0", arches).unwrap();
        let toml = metadata.to_toml().unwrap();
        let parsed = Metadata::from_toml(&toml).unwrap();

        assert_eq!(metadata.primal(), parsed.primal());
        assert_eq!(metadata.version(), parsed.version());
    }

    #[test]
    fn find_ecobin() {
        let mut arches = HashMap::new();
        arches.insert(
            "x86_64-musl".to_string(),
            PathBuf::from("ecobins/primal-x86_64-musl"),
        );

        let metadata = Metadata::new("testprimal", "1.0.0", arches).unwrap();

        assert!(metadata.find_ecobin("x86_64-musl").is_some());
        assert!(metadata.find_ecobin("aarch64-musl").is_none());
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn valid_primal_names_accepted(name in "[a-zA-Z][a-zA-Z0-9_-]{0,30}") {
                let arches = HashMap::new();
                let result = Metadata::new(&name, "1.0.0", arches);
                prop_assert!(result.is_ok());
            }

            #[test]
            fn invalid_primal_names_rejected(name in ".*[!@#$%^&*() ].*") {
                let arches = HashMap::new();
                let result = Metadata::new(&name, "1.0.0", arches);
                prop_assert!(result.is_err());
            }

            #[test]
            fn metadata_toml_roundtrip_preserves_data(
                primal in "[a-z]{3,10}",
                version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
            ) {
                let mut arches = HashMap::new();
                arches.insert("x86_64-musl".to_string(), PathBuf::from("ecobins/test-x86_64-musl"));
                let metadata = Metadata::new(&primal, &version, arches).unwrap();
                let toml_str = metadata.to_toml().unwrap();
                let parsed = Metadata::from_toml(&toml_str).unwrap();
                prop_assert_eq!(metadata.primal(), parsed.primal());
                prop_assert_eq!(metadata.version(), parsed.version());
            }
        }
    }
}
