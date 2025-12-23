//! Configuration traits and utilities.
//!
//! Every primal needs configuration. This module provides patterns for
//! loading configuration from files, environment variables, and runtime.

use crate::error::PrimalError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::Path;

/// Common configuration that all primals share.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Primal name.
    pub name: String,
    /// Primal instance ID (unique per deployment).
    pub instance_id: String,
    /// Log level.
    pub log_level: String,
    /// Data directory.
    pub data_dir: String,
    /// Listen address.
    pub listen_addr: String,
    /// Listen port.
    pub listen_port: u16,
    /// BearDog endpoint.
    pub beardog_endpoint: Option<String>,
    /// Songbird endpoint.
    pub songbird_endpoint: Option<String>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            name: "primal".to_string(),
            instance_id: uuid_simple(),
            log_level: "info".to_string(),
            data_dir: "./data".to_string(),
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 8080,
            beardog_endpoint: None,
            songbird_endpoint: None,
        }
    }
}

/// Configuration loader trait.
///
/// Implement this to define how your primal loads configuration.
pub trait ConfigLoader: Sized {
    /// Load configuration from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    fn from_file(path: impl AsRef<Path>) -> Result<Self, PrimalError>;

    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required variables are missing.
    fn from_env() -> Result<Self, PrimalError>;

    /// Load configuration with defaults.
    ///
    /// Tries file first, then env, then defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if no configuration source works.
    fn load(config_path: Option<&Path>) -> Result<Self, PrimalError>;
}

/// Default configuration loader implementation for any deserializable type.
pub fn load_toml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, PrimalError> {
    let contents = std::fs::read_to_string(path.as_ref())
        .map_err(|e| PrimalError::config(format!("failed to read config: {e}")))?;
    toml::from_str(&contents)
        .map_err(|e| PrimalError::config(format!("failed to parse config: {e}")))
}

/// Configuration trait for primals.
///
/// Implement this to provide configuration access and management.
pub trait PrimalConfig: Send + Sync {
    /// Configuration type for this primal.
    type Config: Clone + Send + Sync;

    /// Get the current configuration.
    fn config(&self) -> &Self::Config;

    /// Get the common configuration.
    fn common_config(&self) -> &CommonConfig;

    /// Validate configuration.
    ///
    /// Called after loading to ensure configuration is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid.
    fn validate_config(config: &Self::Config) -> Result<(), PrimalError>;

    /// Apply new configuration.
    ///
    /// Called during reload to apply new configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be applied.
    fn apply_config(
        &mut self,
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;
}

/// Generate a simple UUID-like string.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

/// Configuration file watcher (optional utility).
#[derive(Debug)]
pub struct ConfigWatcher {
    path: std::path::PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl ConfigWatcher {
    /// Create a new config watcher.
    #[must_use]
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            last_modified: None,
        }
    }

    /// Check if the config file has changed.
    pub fn has_changed(&mut self) -> bool {
        let metadata = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(_) => return false,
        };

        let changed = self.last_modified.map_or(true, |last| modified != last);
        if changed {
            self.last_modified = Some(modified);
        }
        changed
    }
}

