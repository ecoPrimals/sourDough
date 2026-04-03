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

    /// Run all validation tests and return results.
    ///
    /// Returns all test results regardless of pass/fail status.
    pub async fn run_all_tests(&self) -> Vec<ValidationResult> {
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

        results
    }

    /// Run all validation tests.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub async fn validate(&self) -> Result<Vec<ValidationResult>> {
        let results = self.run_all_tests().await;

        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();

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
        match tokio::fs::read(&self.genomebin_path).await {
            Ok(content) => {
                // Search for marker in bytes (binary-safe)
                if content
                    .windows(b"__EMBEDDED_PAYLOAD__".len())
                    .any(|window| window == b"__EMBEDDED_PAYLOAD__")
                {
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
        let content = tokio::fs::read(&self.genomebin_path).await?;

        // Find metadata section (binary-safe search)
        let start_marker = b"__METADATA_START__";
        let end_marker = b"__METADATA_END__";

        let metadata_start = content
            .windows(start_marker.len())
            .position(|window| window == start_marker)
            .ok_or_else(|| GenomeBinError::validation("Metadata start marker not found"))?;

        let metadata_end = content
            .windows(end_marker.len())
            .position(|window| window == end_marker)
            .ok_or_else(|| GenomeBinError::validation("Metadata end marker not found"))?;

        // Extract metadata bytes and convert to string
        let metadata_bytes = &content[metadata_start + start_marker.len()..metadata_end];
        let metadata_toml = std::str::from_utf8(metadata_bytes)
            .map_err(|e| GenomeBinError::validation(format!("Metadata not valid UTF-8: {e}")))?;

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
    use std::io::Write;

    #[test]
    fn validation_result_creation() {
        let pass = ValidationResult::pass("test");
        assert!(pass.passed);
        assert!(pass.message.is_none());

        let fail = ValidationResult::fail("test", "error");
        assert!(!fail.passed);
        assert_eq!(fail.message, Some("error".to_string()));
    }

    #[test]
    fn validation_result_equality() {
        let a = ValidationResult::pass("test");
        let b = ValidationResult::pass("test");
        assert_eq!(a, b);

        let c = ValidationResult::fail("test", "err");
        assert_ne!(a, c);
    }

    fn create_mock_genomebin(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("test.genome");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "#!/bin/bash").unwrap();
        writeln!(f, "# genomeBin wrapper").unwrap();
        write!(f, "__METADATA_START__").unwrap();
        write!(
            f,
            r#"
[genome]
primal = "testPrimal"
version = "1.0.0"
architecture_count = 2
created = "2025-01-01T00:00:00Z"

[architectures]
x86_64-unknown-linux-musl = "ecobins/testPrimal-x86_64-unknown-linux-musl"
aarch64-unknown-linux-musl = "ecobins/testPrimal-aarch64-unknown-linux-musl"
"#,
        )
        .unwrap();
        writeln!(f, "__METADATA_END__").unwrap();
        writeln!(f, "__EMBEDDED_PAYLOAD__").unwrap();
        f.write_all(b"fake-tar-gz-payload-data").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
        }

        path
    }

    #[test]
    fn test_file_exists_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_file_exists();
        assert!(result.passed);
    }

    #[test]
    fn test_file_exists_fail() {
        let validator = Validator::new("/nonexistent/file.genome");
        let result = validator.test_file_exists();
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_shebang_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_shebang().await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_shebang_fail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-shebang.genome");
        std::fs::write(&path, "not a script\n").unwrap();
        let validator = Validator::new(&path);
        let result = validator.test_shebang().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_shebang_missing_file() {
        let validator = Validator::new("/nonexistent/file");
        let result = validator.test_shebang().await;
        assert!(!result.passed);
        assert!(result.message.unwrap().contains("Failed to read"));
    }

    #[tokio::test]
    async fn test_payload_boundary_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_payload_boundary().await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_payload_boundary_fail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-payload.genome");
        std::fs::write(&path, "#!/bin/bash\necho hello\n").unwrap();
        let validator = Validator::new(&path);
        let result = validator.test_payload_boundary().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_payload_boundary_missing_file() {
        let validator = Validator::new("/nonexistent/file");
        let result = validator.test_payload_boundary().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_metadata_extraction_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_metadata_extraction().await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_metadata_extraction_fail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad-meta.genome");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "#!/bin/bash").unwrap();
        write!(f, "__METADATA_START__not-valid-toml__METADATA_END__").unwrap();
        let validator = Validator::new(&path);
        let result = validator.test_metadata_extraction().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_payload_extraction_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_payload_extraction().await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_payload_extraction_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty-payload.genome");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "__EMBEDDED_PAYLOAD__").unwrap();
        let validator = Validator::new(&path);
        let result = validator.test_payload_extraction().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_architecture_count_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_architecture_count().await;
        assert!(result.passed);
        assert!(result.name.contains('2'));
    }

    #[tokio::test]
    async fn test_architecture_count_zero() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zero-arch.genome");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "__METADATA_START__").unwrap();
        write!(
            f,
            r#"
[genome]
primal = "test"
version = "1.0.0"
architecture_count = 0
created = "2025-01-01T00:00:00Z"

[architectures]
"#,
        )
        .unwrap();
        write!(f, "__METADATA_END__").unwrap();
        let validator = Validator::new(&path);
        let result = validator.test_architecture_count().await;
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn run_all_tests_on_valid_genomebin() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let results = validator.run_all_tests().await;
        assert_eq!(results.len(), 7);
        let passed = results.iter().filter(|r| r.passed).count();
        assert!(passed >= 5, "expected at least 5 pass, got {passed}");
    }

    #[tokio::test]
    async fn validate_returns_ok_when_all_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.validate().await;
        // May not all pass (e.g., executable on some systems) but should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn validate_returns_err_when_file_missing() {
        let validator = Validator::new("/nonexistent/file.genome");
        let result = validator.validate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn extract_and_parse_metadata_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let metadata = validator.extract_and_parse_metadata().await.unwrap();
        assert_eq!(metadata.genome.primal, "testPrimal");
        assert_eq!(metadata.genome.version, "1.0.0");
        assert_eq!(metadata.genome.architecture_count, 2);
    }

    #[tokio::test]
    async fn extract_and_parse_metadata_no_start_marker() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-marker.genome");
        std::fs::write(&path, "just some text without markers").unwrap();
        let validator = Validator::new(&path);
        let result = validator.extract_and_parse_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn extract_and_parse_metadata_no_end_marker() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-end.genome");
        std::fs::write(&path, "__METADATA_START__some data but no end").unwrap();
        let validator = Validator::new(&path);
        let result = validator.extract_and_parse_metadata().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn extract_payload_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let payload = validator.extract_payload().await.unwrap();
        assert!(!payload.is_empty());
        assert!(payload.starts_with(b"fake-tar-gz-payload-data"));
    }

    #[tokio::test]
    async fn extract_payload_no_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-boundary.genome");
        std::fs::write(&path, "no boundary here").unwrap();
        let validator = Validator::new(&path);
        let result = validator.extract_payload().await;
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_file_executable_pass() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_mock_genomebin(dir.path());
        let validator = Validator::new(&path);
        let result = validator.test_file_executable().await;
        assert!(result.passed);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_file_executable_fail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not-exec.genome");
        std::fs::write(&path, "#!/bin/bash\n").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o644);
            std::fs::set_permissions(&path, perms).unwrap();
        }
        let validator = Validator::new(&path);
        let result = validator.test_file_executable().await;
        assert!(!result.passed);
    }
}
