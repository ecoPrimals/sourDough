//! Error types for genomeBin operations.
//!
//! All errors use `thiserror` for idiomatic, structured error handling.

use std::path::PathBuf;

/// Result type for genomeBin operations.
pub type Result<T> = std::result::Result<T, GenomeBinError>;

/// Errors that can occur during genomeBin operations.
#[derive(Debug, thiserror::Error)]
pub enum GenomeBinError {
    /// Invalid primal name (must be alphanumeric with hyphens)
    #[error("Invalid primal name '{0}': must contain only alphanumeric characters and hyphens")]
    InvalidPrimalName(String),

    /// Invalid version string
    #[error("Invalid version '{0}': must be valid semver (e.g., 1.0.0)")]
    InvalidVersion(String),

    /// ecoBin directory not found
    #[error("ecoBins directory not found: {0}")]
    EcoBinsDirNotFound(PathBuf),

    /// No ecoBins found for primal
    #[error("No ecoBins found for primal '{primal}' in {dir}")]
    NoEcoBinsFound {
        /// Primal name
        primal: String,
        /// Directory searched
        dir: PathBuf,
    },

    /// ecoBin not found for specific target
    #[error("No ecoBin found for target '{target}'. Available: {available:?}")]
    EcoBinNotFoundForTarget {
        /// Target triple that was not found
        target: String,
        /// Available targets
        available: Vec<String>,
    },

    /// Archive creation failed
    #[error("Failed to create archive: {0}")]
    ArchiveCreation(#[source] std::io::Error),

    /// Archive extraction failed
    #[error("Failed to extract archive: {0}")]
    ArchiveExtraction(#[source] std::io::Error),

    /// Metadata parsing failed
    #[error("Failed to parse metadata: {0}")]
    MetadataParse(#[source] toml::de::Error),

    /// Metadata serialization failed
    #[error("Failed to serialize metadata: {0}")]
    MetadataSerialize(#[source] toml::ser::Error),

    /// Payload boundary not found in genomeBin
    #[error("Payload boundary marker not found in genomeBin file")]
    PayloadBoundaryNotFound,

    /// Checksum mismatch
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected checksum
        expected: String,
        /// Actual checksum
        actual: String,
    },

    /// Platform detection failed
    #[error("Failed to detect platform: {0}")]
    PlatformDetection(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Task join error
    #[error("Task execution failed: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    /// Validation failed
    #[error("Validation failed: {0}")]
    Validation(String),
}

impl GenomeBinError {
    /// Create a validation error.
    #[must_use]
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a platform detection error.
    #[must_use]
    pub fn platform_detection(msg: impl Into<String>) -> Self {
        Self::PlatformDetection(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_display() {
        let e = GenomeBinError::validation("check failed");
        assert_eq!(e.to_string(), "Validation failed: check failed");
    }

    #[test]
    fn platform_detection_error_display() {
        let e = GenomeBinError::platform_detection("unknown arch");
        assert_eq!(e.to_string(), "Failed to detect platform: unknown arch");
    }

    #[test]
    fn invalid_primal_name_display() {
        let e = GenomeBinError::InvalidPrimalName("bad name!".into());
        assert!(e.to_string().contains("bad name!"));
    }

    #[test]
    fn invalid_version_display() {
        let e = GenomeBinError::InvalidVersion("not.semver".into());
        assert!(e.to_string().contains("not.semver"));
    }

    #[test]
    fn ecobins_dir_not_found_display() {
        let e = GenomeBinError::EcoBinsDirNotFound(PathBuf::from("/missing"));
        assert!(e.to_string().contains("/missing"));
    }

    #[test]
    fn no_ecobins_found_display() {
        let e = GenomeBinError::NoEcoBinsFound {
            primal: "test".into(),
            dir: PathBuf::from("/dir"),
        };
        assert!(e.to_string().contains("test"));
    }

    #[test]
    fn payload_boundary_not_found_display() {
        let e = GenomeBinError::PayloadBoundaryNotFound;
        assert!(e.to_string().contains("Payload boundary"));
    }

    #[test]
    fn checksum_mismatch_display() {
        let e = GenomeBinError::ChecksumMismatch {
            expected: "aaa".into(),
            actual: "bbb".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("aaa"));
        assert!(msg.contains("bbb"));
    }

    #[test]
    fn io_error_from() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let e: GenomeBinError = io.into();
        assert!(e.to_string().contains("gone"));
    }
}
