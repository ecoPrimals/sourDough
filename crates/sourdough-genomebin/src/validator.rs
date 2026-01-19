//! Validation for genomeBins.
//!
//! Comprehensive testing and validation (replaces `test-genomebin.sh`).

use crate::error::{GenomeBinError, Result};
use crate::metadata::Metadata;
use std::path::PathBuf;
use tracing::{debug, info};

/// Validation result for a single test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Test name
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Optional error message
    pub message: Option<String>,
}

impl ValidationResult {
    /// Create a passing result.
    #[must_use]
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: None,
        }
    }

    /// Create a failing result.
    #[must_use]
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: Some(message.into()),
        }
    }
}

/// genomeBin validator.
pub struct Validator {
    genomebin_path: PathBuf,
}

impl Validator {
    /// Create a new validator.
    #[must_use]
    pub fn new(genomebin_path: impl Into<PathBuf>) -> Self {
        Self {
            genomebin_path: genomebin_path.into(),
        }
    }

    /// Run all validation tests.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub async fn validate(&self) -> Result<Vec<ValidationResult>> {
        info!("Starting genomeBin validation");

        let mut results = Vec::new();

        results.push(self.test_file_exists());
        results.push(self.test_file_executable().await);
        results.push(self.test_shebang().await);
        results.push(self.test_payload_boundary().await);
        results.push(self.test_metadata_extraction().await);
        results.push(self.test_payload_extraction().await);
        results.push(self.test_architecture_count().await);

        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();

        info!("Validation complete: {passed}/{total} tests passed");

        if passed != total {
            return Err(GenomeBinError::validation(format!(
                "{passed}/{total} tests passed"
            )));
        }

        Ok(results)
    }

    fn test_file_exists(&self) -> ValidationResult {
        if self.genomebin_path.exists() {
            ValidationResult::pass("File exists")
        } else {
            ValidationResult::fail("File exists", "genomeBin file not found")
        }
    }

    async fn test_file_executable(&self) -> ValidationResult {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = tokio::fs::metadata(&self.genomebin_path).await {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 != 0 {
                    return ValidationResult::pass("File executable");
                }
            }
            ValidationResult::fail("File executable", "File is not executable")
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, skip this test
            ValidationResult::pass("File executable (skipped on non-Unix)")
        }
    }

    async fn test_shebang(&self) -> ValidationResult {
        match tokio::fs::read(&self.genomebin_path).await {
            Ok(content)
                if content.starts_with(b"#!/bin/bash")
                    || content.starts_with(b"#!/usr/bin/env bash") =>
            {
                ValidationResult::pass("Shebang present")
            }
            Ok(_) => ValidationResult::fail("Shebang present", "Invalid or missing shebang"),
            Err(e) => {
                ValidationResult::fail("Shebang present", format!("Failed to read file: {e}"))
            }
        }
    }

    async fn test_payload_boundary(&self) -> ValidationResult {
        match tokio::fs::read_to_string(&self.genomebin_path).await {
            Ok(content) => {
                if content.contains("__EMBEDDED_PAYLOAD__") {
                    ValidationResult::pass("Payload boundary found")
                } else {
                    ValidationResult::fail("Payload boundary found", "Marker not found")
                }
            }
            Err(e) => {
                ValidationResult::fail("Payload boundary found", format!("Failed to read: {e}"))
            }
        }
    }

    async fn test_metadata_extraction(&self) -> ValidationResult {
        match self.extract_and_parse_metadata().await {
            Ok(_) => ValidationResult::pass("Metadata extraction"),
            Err(e) => ValidationResult::fail("Metadata extraction", format!("{e}")),
        }
    }

    async fn test_payload_extraction(&self) -> ValidationResult {
        match self.extract_payload().await {
            Ok(payload) if !payload.is_empty() => {
                debug!("Extracted payload: {} bytes", payload.len());
                ValidationResult::pass("Payload extraction")
            }
            Ok(_) => ValidationResult::fail("Payload extraction", "Payload is empty"),
            Err(e) => ValidationResult::fail("Payload extraction", format!("{e}")),
        }
    }

    async fn test_architecture_count(&self) -> ValidationResult {
        match self.extract_and_parse_metadata().await {
            Ok(metadata) => {
                let count = metadata.genome.architecture_count;
                if count > 0 {
                    ValidationResult::pass(format!("Architecture count ({count})"))
                } else {
                    ValidationResult::fail("Architecture count", "No architectures found")
                }
            }
            Err(e) => ValidationResult::fail("Architecture count", format!("{e}")),
        }
    }

    async fn extract_and_parse_metadata(&self) -> Result<Metadata> {
        let content = tokio::fs::read_to_string(&self.genomebin_path).await?;

        // Find metadata section
        let metadata_start = content
            .find("__METADATA_START__")
            .ok_or_else(|| GenomeBinError::validation("Metadata start marker not found"))?;
        let metadata_end = content
            .find("__METADATA_END__")
            .ok_or_else(|| GenomeBinError::validation("Metadata end marker not found"))?;

        let metadata_toml = &content[metadata_start + "__METADATA_START__".len()..metadata_end];
        Metadata::from_toml(metadata_toml.trim())
    }

    async fn extract_payload(&self) -> Result<Vec<u8>> {
        let content = tokio::fs::read(&self.genomebin_path).await?;

        // Find payload boundary
        let boundary = b"__EMBEDDED_PAYLOAD__";
        let pos = content
            .windows(boundary.len())
            .position(|window| window == boundary)
            .ok_or(GenomeBinError::PayloadBoundaryNotFound)?;

        // Find the newline after the boundary
        let payload_start = content[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| pos + p + 1)
            .ok_or(GenomeBinError::PayloadBoundaryNotFound)?;

        Ok(content[payload_start..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_result_creation() {
        let pass = ValidationResult::pass("test");
        assert!(pass.passed);
        assert!(pass.message.is_none());

        let fail = ValidationResult::fail("test", "error");
        assert!(!fail.passed);
        assert_eq!(fail.message, Some("error".to_string()));
    }
}
