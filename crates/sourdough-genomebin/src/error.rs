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
