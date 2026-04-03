//! Archive operations for genomeBins.
//!
//! Provides type-safe tar/gzip operations using Pure Rust libraries.
//!
//! ## Zero Unsafe Code
//!
//! All operations use:
//! - `tar` crate (Pure Rust)
//! - `flate2` with `miniz_oxide` backend (Pure Rust)
//!
//! No C dependencies, fully safe code.

use crate::error::{GenomeBinError, Result};
use bytes::Bytes;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Archive builder for creating tar.gz archives.
pub struct ArchiveBuilder {
    output: PathBuf,
    compression: Compression,
}

impl ArchiveBuilder {
    /// Create a new archive builder.
    #[must_use]
    pub fn new(output: impl Into<PathBuf>) -> Self {
        Self {
            output: output.into(),
            compression: Compression::best(),
        }
    }

    /// Set compression level (0-9).
    #[must_use]
    pub const fn compression(mut self, level: u32) -> Self {
        self.compression = Compression::new(level);
        self
    }

    /// Add files to archive and write to output.
    ///
    /// # Errors
    ///
    /// Returns an error if archive creation fails.
    pub async fn create(&self, files: &[(PathBuf, PathBuf)]) -> Result<Bytes> {
        // Create archive in memory first for validation
        let mut buffer = Vec::new();
        {
            let encoder = GzEncoder::new(&mut buffer, self.compression);
            let mut tar = tar::Builder::new(encoder);

            for (src, dest) in files {
                let mut file = std::fs::File::open(src)?;
                let metadata = file.metadata()?;

                let mut header = tar::Header::new_gnu();
                header.set_size(metadata.len());
                header.set_mode(0o755);
                header.set_cksum();

                tar.append_data(&mut header, dest, &mut file)
                    .map_err(GenomeBinError::ArchiveCreation)?;
            }

            tar.finish().map_err(GenomeBinError::ArchiveCreation)?;
        }

        // Write to file
        let mut file = tokio::fs::File::create(&self.output).await?;
        file.write_all(&buffer).await?;
        file.flush().await?;

        Ok(Bytes::from(buffer))
    }
}

/// Extract a tar.gz archive.
///
/// # Errors
///
/// Returns an error if extraction fails.
pub async fn extract(archive: &Path, output_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let decoder = GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    tokio::task::spawn_blocking({
        let output_dir = output_dir.to_path_buf();
        move || {
            tar.unpack(output_dir)
                .map_err(GenomeBinError::ArchiveExtraction)
        }
    })
    .await??;

    Ok(())
}

/// List files in a tar.gz archive.
///
/// # Errors
///
/// Returns an error if listing fails.
pub fn list_files(archive: &Path) -> Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive)?;
    let decoder = GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);

    let mut files = Vec::new();
    for entry in tar.entries().map_err(GenomeBinError::ArchiveExtraction)? {
        let entry = entry.map_err(GenomeBinError::ArchiveExtraction)?;
        if let Ok(path) = entry.path() {
            files.push(path.into_owned());
        }
    }

    Ok(files)
}

/// Calculate BLAKE3 checksum of a file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub async fn checksum_blake3(path: &Path) -> Result<String> {
    let content = tokio::fs::read(path).await?;
    let hash = blake3::hash(&content);
    Ok(hash.to_hex().to_string())
}

/// Calculate SHA256 checksum of a file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub async fn checksum_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let content = tokio::fs::read(path).await?;
    let hash = Sha256::digest(&content);
    Ok(format!("{hash:x}"))
}

/// Verify checksum of a file.
///
/// # Errors
///
/// Returns an error if checksums don't match.
pub async fn verify_checksum(path: &Path, expected: &str) -> Result<()> {
    let actual = checksum_sha256(path).await?;
    if actual != expected {
        return Err(GenomeBinError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn archive_creation() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        tokio::fs::write(&test_file, b"hello world").await.unwrap();

        let output = temp.path().join("archive.tar.gz");
        let builder = ArchiveBuilder::new(&output);

        let files = vec![(test_file.clone(), PathBuf::from("test.txt"))];
        builder.create(&files).await.unwrap();

        assert!(output.exists());
        assert!(output.metadata().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn archive_extraction() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        tokio::fs::write(&test_file, b"hello world").await.unwrap();

        let archive = temp.path().join("archive.tar.gz");
        let builder = ArchiveBuilder::new(&archive);

        let files = vec![(test_file.clone(), PathBuf::from("test.txt"))];
        builder.create(&files).await.unwrap();

        let extract_dir = temp.path().join("extracted");
        tokio::fs::create_dir(&extract_dir).await.unwrap();

        extract(&archive, &extract_dir).await.unwrap();

        let extracted_file = extract_dir.join("test.txt");
        assert!(extracted_file.exists());

        let content = tokio::fs::read(&extracted_file).await.unwrap();
        assert_eq!(content, b"hello world");
    }

    #[tokio::test]
    async fn checksum_calculation() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        tokio::fs::write(&test_file, b"hello world").await.unwrap();

        let blake3 = checksum_blake3(&test_file).await.unwrap();
        let sha256 = checksum_sha256(&test_file).await.unwrap();

        assert!(!blake3.is_empty());
        assert!(!sha256.is_empty());
        assert_ne!(blake3, sha256);
    }

    #[tokio::test]
    async fn checksum_verification() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        tokio::fs::write(&test_file, b"hello world").await.unwrap();

        let checksum = checksum_sha256(&test_file).await.unwrap();
        verify_checksum(&test_file, &checksum).await.unwrap();

        let result = verify_checksum(&test_file, "invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_archive_files_returns_entries() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        tokio::fs::write(&test_file, b"content").await.unwrap();

        let archive = temp.path().join("archive.tar.gz");
        let builder = ArchiveBuilder::new(&archive);
        let files = vec![(test_file, PathBuf::from("inner/test.txt"))];
        builder.create(&files).await.unwrap();

        let listed = list_files(&archive).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0], PathBuf::from("inner/test.txt"));
    }
}
