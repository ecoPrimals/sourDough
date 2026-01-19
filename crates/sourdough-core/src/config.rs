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
    /// `BearDog` endpoint.
    pub beardog_endpoint: Option<String>,
    /// `Songbird` endpoint.
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
///
/// # Errors
///
/// Returns an error if the file cannot be read or if the TOML cannot be parsed.
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
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return false;
        };

        let Ok(modified) = metadata.modified() else {
            return false;
        };

        let changed = self.last_modified != Some(modified);
        if changed {
            self.last_modified = Some(modified);
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn common_config_default() {
        let config = CommonConfig::default();
        
        assert_eq!(config.name, "primal");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.data_dir, "./data");
        assert_eq!(config.listen_addr, "0.0.0.0");
        assert_eq!(config.listen_port, 8080);
        assert!(config.beardog_endpoint.is_none());
        assert!(config.songbird_endpoint.is_none());
        assert!(!config.instance_id.is_empty());
    }

    #[test]
    fn common_config_unique_instance_ids() {
        let config1 = CommonConfig::default();
        let config2 = CommonConfig::default();
        
        // Instance IDs should be unique (based on timestamp)
        assert_ne!(config1.instance_id, config2.instance_id);
    }

    #[test]
    fn common_config_serialization() {
        let config = CommonConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: CommonConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.name, parsed.name);
        assert_eq!(config.log_level, parsed.log_level);
    }

    #[test]
    fn load_toml_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(file, r#"
            name = "test-primal"
            log_level = "debug"
        "#).unwrap();
        
        let config: std::collections::HashMap<String, String> = 
            load_toml(&config_path).unwrap();
        
        assert_eq!(config.get("name"), Some(&"test-primal".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
    }

    #[test]
    fn load_toml_invalid_path() {
        let result: Result<CommonConfig, _> = load_toml("/nonexistent/config.toml");
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, PrimalError::Config(_)));
        }
    }

    #[test]
    fn load_toml_invalid_syntax() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("bad.toml");
        
        let mut file = std::fs::File::create(&config_path).unwrap();
        writeln!(file, "invalid toml syntax [[[").unwrap();
        
        let result: Result<CommonConfig, _> = load_toml(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn config_watcher_detects_change() {
        use std::thread;
        use std::time::Duration;
        
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("watch.toml");
        
        // Create initial file
        std::fs::write(&config_path, "test").unwrap();
        thread::sleep(Duration::from_millis(10));
        
        let mut watcher = ConfigWatcher::new(&config_path);
        
        // First check should detect change (no previous modified time)
        assert!(watcher.has_changed());
        
        // Second immediate check should not detect change
        assert!(!watcher.has_changed());
        
        // Modify file
        thread::sleep(Duration::from_millis(10));
        std::fs::write(&config_path, "modified").unwrap();
        thread::sleep(Duration::from_millis(10));
        
        // Should detect the modification
        assert!(watcher.has_changed());
        
        // No more changes
        assert!(!watcher.has_changed());
    }

    #[test]
    fn config_watcher_nonexistent_file() {
        let mut watcher = ConfigWatcher::new("/nonexistent/file.toml");
        
        // Should return false for nonexistent files
        assert!(!watcher.has_changed());
    }
}

