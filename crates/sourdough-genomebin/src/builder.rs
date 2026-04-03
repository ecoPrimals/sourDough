//! `GenomeBin` builder for creating self-extracting deployment packages.
//!
//! This replaces `create-genomebin.sh` with a type-safe, concurrent Rust implementation.

use crate::archive::ArchiveBuilder;
use crate::error::{GenomeBinError, Result};
use crate::metadata::Metadata;
use bytes::Bytes;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

/// genomeBin builder.
///
/// Provides a type-safe, concurrent API for creating genomeBins.
///
/// # Example
///
/// ```rust,no_run
/// use sourdough_genomebin::GenomeBinBuilder;
///
/// # async fn example() -> anyhow::Result<()> {
/// let builder = GenomeBinBuilder::new("myprimal", "1.0.0")
///     .ecobins_dir("./ecobins")
///     .output("myprimal-1.0.0.genome")
///     .parallel(true);
///
/// let genome = builder.build().await?;
/// genome.create().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GenomeBinBuilder {
    primal: String,
    version: String,
    ecobins_dir: PathBuf,
    output: PathBuf,
    parallel: bool,
    wrapper_script: Option<PathBuf>,
}

impl GenomeBinBuilder {
    /// Create a new genomeBin builder.
    #[must_use]
    pub fn new(primal: impl Into<String>, version: impl Into<String>) -> Self {
        let primal = primal.into();
        let version = version.into();
        Self {
            output: PathBuf::from(format!("{primal}-{version}.genome")),
            primal,
            version,
            ecobins_dir: PathBuf::from("ecobins"),
            parallel: true,
            wrapper_script: None,
        }
    }

    /// Set the ecoBins directory.
    #[must_use]
    pub fn ecobins_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.ecobins_dir = dir.into();
        self
    }

    /// Set the output path.
    #[must_use]
    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = path.into();
        self
    }

    /// Enable or disable parallel processing.
    #[must_use]
    pub const fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set a custom wrapper script (optional).
    ///
    /// If not set, uses the default embedded wrapper.
    #[must_use]
    pub fn wrapper_script(mut self, path: impl Into<PathBuf>) -> Self {
        self.wrapper_script = Some(path.into());
        self
    }

    /// Build the genomeBin.
    ///
    /// # Errors
    ///
    /// Returns an error if building fails.
    pub async fn build(self) -> Result<GenomeBin> {
        info!("Building genomeBin for {} v{}", self.primal, self.version);

        // Validate inputs
        if !self.ecobins_dir.exists() {
            return Err(GenomeBinError::EcoBinsDirNotFound(self.ecobins_dir));
        }

        // Find ecoBins
        let ecobins = self.find_ecobins().await?;
        if ecobins.is_empty() {
            return Err(GenomeBinError::NoEcoBinsFound {
                primal: self.primal.clone(),
                dir: self.ecobins_dir.clone(),
            });
        }

        info!("Found {} ecoBins", ecobins.len());

        Ok(GenomeBin {
            primal: self.primal,
            version: self.version,
            ecobins,
            output: self.output,
            parallel: self.parallel,
            wrapper_script: self.wrapper_script,
        })
    }

    async fn find_ecobins(&self) -> Result<HashMap<String, PathBuf>> {
        let mut ecobins = HashMap::new();
        let mut entries = tokio::fs::read_dir(&self.ecobins_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Parse filename: primal-target or primal-target.ecobin
                if filename.starts_with(&self.primal) {
                    // Extract target triple from filename
                    let target = self.extract_target_from_filename(filename);
                    debug!("Found ecoBin: {} -> {}", target, path.display());
                    ecobins.insert(target, path);
                }
            }
        }

        Ok(ecobins)
    }

    fn extract_target_from_filename(&self, filename: &str) -> String {
        // Remove .ecobin extension if present
        let name = filename.strip_suffix(".ecobin").unwrap_or(filename);

        // Remove primal name prefix
        let suffix = name
            .strip_prefix(&self.primal)
            .and_then(|s| s.strip_prefix('-'))
            .unwrap_or(name);

        suffix.to_string()
    }
}

/// A genomeBin ready for creation.
pub struct GenomeBin {
    primal: String,
    version: String,
    ecobins: HashMap<String, PathBuf>,
    output: PathBuf,
    parallel: bool,
    wrapper_script: Option<PathBuf>,
}

impl GenomeBin {
    /// Create the genomeBin.
    ///
    /// This is the main operation that builds the self-extracting archive.
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails.
    pub async fn create(&self) -> Result<PathBuf> {
        info!("Creating genomeBin: {}", self.output.display());

        // 1. Create metadata
        let metadata = self.create_metadata()?;

        // 2. Create payload archive (tar.gz of all ecoBins)
        let payload = self.create_payload().await?;

        // 3. Create self-extracting wrapper
        self.create_wrapper(&metadata, &payload).await?;

        info!("genomeBin created successfully");
        Ok(self.output.clone())
    }

    fn create_metadata(&self) -> Result<Metadata> {
        Metadata::new(&self.primal, &self.version, self.ecobins.clone())
    }

    async fn create_payload(&self) -> Result<Bytes> {
        info!("Creating payload archive");

        let temp_dir = tempfile::tempdir()?;
        let archive_path = temp_dir.path().join("payload.tar.gz");

        let files: Vec<(PathBuf, PathBuf)> = self
            .ecobins
            .iter()
            .map(|(target, path)| {
                let dest = PathBuf::from(format!("ecobins/{}-{}", self.primal, target));
                (path.clone(), dest)
            })
            .collect();

        if self.parallel && files.len() > 1 {
            info!("Pre-reading {} ecoBins concurrently", files.len());
            let mut set = tokio::task::JoinSet::new();
            for (src, _) in &files {
                let src = src.clone();
                set.spawn(async move { tokio::fs::read(&src).await });
            }
            while let Some(result) = set.join_next().await {
                result??;
            }
        }

        let builder = ArchiveBuilder::new(&archive_path).compression(9);
        builder.create(&files).await
    }

    async fn create_wrapper(&self, metadata: &Metadata, payload: &Bytes) -> Result<()> {
        info!("Creating self-extracting wrapper");

        let wrapper = self.load_wrapper_script().await?;
        let metadata_toml = metadata.to_toml()?;

        let mut output = tokio::fs::File::create(&self.output).await?;

        // Write wrapper script
        output.write_all(wrapper.as_bytes()).await?;

        // Write metadata section
        output.write_all(b"\n__METADATA_START__\n").await?;
        output.write_all(metadata_toml.as_bytes()).await?;
        output.write_all(b"\n__METADATA_END__\n").await?;

        // Write payload marker
        output.write_all(b"__EMBEDDED_PAYLOAD__\n").await?;

        // Write payload (binary data)
        output.write_all(payload).await?;

        output.flush().await?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&self.output).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&self.output, perms).await?;
        }

        Ok(())
    }

    async fn load_wrapper_script(&self) -> Result<String> {
        if let Some(custom) = &self.wrapper_script {
            return Ok(tokio::fs::read_to_string(custom).await?);
        }

        // Use default embedded wrapper
        Ok(DEFAULT_WRAPPER_SCRIPT.to_string())
    }

    /// Get the primal name.
    #[must_use]
    pub fn primal(&self) -> &str {
        &self.primal
    }

    /// Get the version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get available targets.
    #[must_use]
    pub fn targets(&self) -> Vec<&str> {
        self.ecobins.keys().map(String::as_str).collect()
    }

    /// Get the output path.
    #[must_use]
    pub fn output(&self) -> &Path {
        &self.output
    }
}

/// Default wrapper script for self-extracting genomeBins.
///
/// This bash wrapper is a transitional artifact. The roadmap calls for
/// replacing it with a Pure Rust self-extractor binary to achieve full
/// ecoBin compliance. See specs/ROADMAP.md v0.5.0.
const DEFAULT_WRAPPER_SCRIPT: &str = r#"#!/bin/bash
set -euo pipefail

# genomeBin Self-Extractor
# Generated by sourDough genomeBin tooling

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "🧬 genomeBin Installer"
echo "======================"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    armv7*|armv8*) ARCH="arm" ;;
    *) echo "ERROR: Unsupported architecture: $ARCH"; exit 1 ;;
esac

echo "Platform: $OS/$ARCH"

# Extract payload
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "Extracting..."

# Find payload boundary and extract
PAYLOAD_LINE=$(grep -a -n "^__EMBEDDED_PAYLOAD__$" "$0" | cut -d: -f1)
PAYLOAD_LINE=$((PAYLOAD_LINE + 1))

tail -n +$PAYLOAD_LINE "$0" | tar xzf - -C "$TEMP_DIR"

# Find and install appropriate ecoBin
# Try exact match, then musl fallback, then best available
for TARGET_PATTERN in \
    "$ARCH-*-$OS-*" \
    "$ARCH-*-$OS-musl" \
    "$ARCH-musl" \
    "$ARCH-*" \
    "*"; do
    
    ECOBIN=$(find "$TEMP_DIR/ecobins" -type f -name "*$TARGET_PATTERN*" | head -n1)
    if [ -n "$ECOBIN" ]; then
        break
    fi
done

if [ -z "$ECOBIN" ]; then
    echo "ERROR: No compatible ecoBin found"
    exit 1
fi

echo "Selected: $(basename "$ECOBIN")"

# Install
mkdir -p "$INSTALL_DIR"
PRIMAL_NAME=$(basename "$ECOBIN" | cut -d'-' -f1)
cp "$ECOBIN" "$INSTALL_DIR/$PRIMAL_NAME"
chmod +x "$INSTALL_DIR/$PRIMAL_NAME"

echo "✅ Installed to: $INSTALL_DIR/$PRIMAL_NAME"
echo ""
echo "Add to PATH if needed:"
echo "  export PATH=\"$INSTALL_DIR:\$PATH\""

exit 0
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creation() {
        let builder = GenomeBinBuilder::new("testprimal", "1.0.0");
        assert_eq!(builder.primal, "testprimal");
        assert_eq!(builder.version, "1.0.0");
        assert!(builder.parallel);
    }

    #[test]
    fn builder_configuration() {
        let builder = GenomeBinBuilder::new("test", "1.0.0")
            .ecobins_dir("/custom/path")
            .output("/output/test.genome")
            .parallel(false);

        assert_eq!(builder.ecobins_dir, PathBuf::from("/custom/path"));
        assert_eq!(builder.output, PathBuf::from("/output/test.genome"));
        assert!(!builder.parallel);
    }

    #[test]
    fn extract_target_from_filename() {
        let builder = GenomeBinBuilder::new("sourdough", "1.0.0");

        assert_eq!(
            builder.extract_target_from_filename("sourdough-x86_64-musl"),
            "x86_64-musl"
        );
        assert_eq!(
            builder.extract_target_from_filename("sourdough-x86_64-musl.ecobin"),
            "x86_64-musl"
        );
        assert_eq!(
            builder.extract_target_from_filename("sourdough-aarch64-darwin"),
            "aarch64-darwin"
        );
    }
}
