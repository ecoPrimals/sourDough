//! Platform-aware directory resolution for primal data storage.
//!
//! Every primal needs directories for config, data, runtime state, and cache.
//! This module resolves them per-platform without silicon deism:
//!
//! | Purpose | Linux | macOS | Windows | iOS/Android |
//! |---------|-------|-------|---------|-------------|
//! | Config | `$XDG_CONFIG_HOME/biomeos/{name}` | `~/Library/Application Support/biomeos/{name}` | `%APPDATA%/biomeos/{name}` | App sandbox |
//! | Data | `$XDG_DATA_HOME/biomeos/{name}` | `~/Library/Application Support/biomeos/{name}` | `%APPDATA%/biomeos/{name}` | App sandbox |
//! | Runtime | `$XDG_RUNTIME_DIR/biomeos` | `/tmp/biomeos` | `%TEMP%/biomeos` | App sandbox |
//! | Cache | `$XDG_CACHE_HOME/biomeos/{name}` | `~/Library/Caches/biomeos/{name}` | `%LOCALAPPDATA%/biomeos/{name}/cache` | App sandbox |
//! | Logs | `$XDG_STATE_HOME/biomeos/{name}` | `~/Library/Logs/biomeos/{name}` | `%LOCALAPPDATA%/biomeos/{name}/logs` | App sandbox |
//!
//! # Philosophy
//!
//! Primals never hardcode paths. They call `PrimalDirs::resolve("myprimal")` and
//! get platform-correct directories. The abstraction handles:
//! - XDG Base Directory Specification (Linux/FreeBSD)
//! - macOS `~/Library/` conventions
//! - Windows `%APPDATA%` / `%LOCALAPPDATA%` conventions
//! - Mobile sandbox pass-through (iOS/Android apps provide their own root)
//!
//! Environment overrides always win — a launcher or test harness can inject
//! `BIOMEOS_CONFIG_DIR`, `BIOMEOS_DATA_DIR`, etc. to relocate everything.

use std::path::PathBuf;

/// Environment variable names for directory overrides.
pub mod env_overrides {
    /// Override for configuration directory.
    pub const CONFIG_DIR: &str = "BIOMEOS_CONFIG_DIR";
    /// Override for persistent data directory.
    pub const DATA_DIR: &str = "BIOMEOS_DATA_DIR";
    /// Override for runtime ephemeral directory.
    pub const RUNTIME_DIR: &str = "BIOMEOS_RUNTIME_DIR";
    /// Override for cache directory.
    pub const CACHE_DIR: &str = "BIOMEOS_CACHE_DIR";
    /// Override for log directory.
    pub const LOG_DIR: &str = "BIOMEOS_LOG_DIR";
}

const NAMESPACE: &str = "biomeos";

/// Resolved directory paths for a primal.
///
/// All paths are guaranteed to be absolute. Directories are NOT created
/// automatically — call `ensure()` to create them with appropriate permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimalDirs {
    /// Configuration files (TOML, capability registry, keys).
    pub config: PathBuf,
    /// Persistent data (databases, state).
    pub data: PathBuf,
    /// Runtime ephemeral state (sockets, PID files) — cleared on reboot.
    pub runtime: PathBuf,
    /// Cache (rebuild-able, deletable without data loss).
    pub cache: PathBuf,
    /// Log files.
    pub logs: PathBuf,
}

impl PrimalDirs {
    /// Resolve all directories for a named primal on the current platform.
    ///
    /// Checks environment overrides first, then applies platform conventions.
    #[must_use]
    pub fn resolve(primal_name: &str) -> Self {
        Self {
            config: resolve_config(primal_name),
            data: resolve_data(primal_name),
            runtime: resolve_runtime(),
            cache: resolve_cache(primal_name),
            logs: resolve_logs(primal_name),
        }
    }

    /// Ensure all directories exist with appropriate permissions.
    ///
    /// - Runtime dir: owner-only (0o700 on Unix)
    /// - Config dir: owner-only (0o700 on Unix)
    /// - Data/cache/logs: owner read+write (0o755 on Unix)
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if directory creation fails.
    pub fn ensure(&self) -> std::io::Result<()> {
        use crate::platform_substrate::{PlatformAccess, ensure_dir_with_access};

        ensure_dir_with_access(&self.config, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.data, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.runtime, PlatformAccess::OwnerFull)?;
        ensure_dir_with_access(&self.cache, PlatformAccess::PublicExecute)?;
        ensure_dir_with_access(&self.logs, PlatformAccess::PublicExecute)?;
        Ok(())
    }

    /// Get the socket path within the runtime directory.
    #[must_use]
    pub fn socket_path(&self, primal_name: &str) -> PathBuf {
        self.runtime.join(format!("{primal_name}.sock"))
    }

    /// Get the PID file path within the runtime directory.
    #[must_use]
    pub fn pid_path(&self, primal_name: &str) -> PathBuf {
        self.runtime.join(format!("{primal_name}.pid"))
    }
}

// ─── Config ────────────────────────────────────────────────────────────────

fn resolve_config(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::CONFIG_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_config_base().join(NAMESPACE).join(primal_name)
}

#[cfg(target_os = "linux")]
fn platform_config_base() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| home_dir().join(".config"),
        PathBuf::from,
    )
}

#[cfg(target_os = "macos")]
fn platform_config_base() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}

#[cfg(target_os = "windows")]
fn platform_config_base() -> PathBuf {
    std::env::var("APPDATA").map_or_else(
        |_| home_dir().join("AppData").join("Roaming"),
        PathBuf::from,
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_config_base() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| home_dir().join(".config"),
        PathBuf::from,
    )
}

// ─── Data ──────────────────────────────────────────────────────────────────

fn resolve_data(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::DATA_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_data_base().join(NAMESPACE).join(primal_name)
}

#[cfg(target_os = "linux")]
fn platform_data_base() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map_or_else(
        |_| home_dir().join(".local").join("share"),
        PathBuf::from,
    )
}

#[cfg(target_os = "macos")]
fn platform_data_base() -> PathBuf {
    home_dir().join("Library").join("Application Support")
}

#[cfg(target_os = "windows")]
fn platform_data_base() -> PathBuf {
    std::env::var("APPDATA").map_or_else(
        |_| home_dir().join("AppData").join("Roaming"),
        PathBuf::from,
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_data_base() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map_or_else(
        |_| home_dir().join(".local").join("share"),
        PathBuf::from,
    )
}

// ─── Runtime ───────────────────────────────────────────────────────────────

fn resolve_runtime() -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::RUNTIME_DIR) {
        return PathBuf::from(dir);
    }
    platform_runtime_base().join(NAMESPACE)
}

#[cfg(target_os = "linux")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| PathBuf::from("/tmp"),
        PathBuf::from,
    )
}

#[cfg(target_os = "macos")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

#[cfg(target_os = "windows")]
fn platform_runtime_base() -> PathBuf {
    std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .map_or_else(|_| PathBuf::from(r"C:\Temp"), PathBuf::from)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_runtime_base() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

// ─── Cache ─────────────────────────────────────────────────────────────────

fn resolve_cache(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::CACHE_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_cache_base().join(NAMESPACE).join(primal_name)
}

#[cfg(target_os = "linux")]
fn platform_cache_base() -> PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(
        |_| home_dir().join(".cache"),
        PathBuf::from,
    )
}

#[cfg(target_os = "macos")]
fn platform_cache_base() -> PathBuf {
    home_dir().join("Library").join("Caches")
}

#[cfg(target_os = "windows")]
fn platform_cache_base() -> PathBuf {
    std::env::var("LOCALAPPDATA").map_or_else(
        |_| home_dir().join("AppData").join("Local"),
        PathBuf::from,
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_cache_base() -> PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(
        |_| home_dir().join(".cache"),
        PathBuf::from,
    )
}

// ─── Logs ──────────────────────────────────────────────────────────────────

fn resolve_logs(primal_name: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_overrides::LOG_DIR) {
        return PathBuf::from(dir).join(primal_name);
    }
    platform_logs_base().join(NAMESPACE).join(primal_name)
}

#[cfg(target_os = "linux")]
fn platform_logs_base() -> PathBuf {
    std::env::var("XDG_STATE_HOME").map_or_else(
        |_| home_dir().join(".local").join("state"),
        PathBuf::from,
    )
}

#[cfg(target_os = "macos")]
fn platform_logs_base() -> PathBuf {
    home_dir().join("Library").join("Logs")
}

#[cfg(target_os = "windows")]
fn platform_logs_base() -> PathBuf {
    std::env::var("LOCALAPPDATA").map_or_else(
        |_| home_dir().join("AppData").join("Local"),
        PathBuf::from,
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_logs_base() -> PathBuf {
    std::env::var("XDG_STATE_HOME").map_or_else(
        |_| home_dir().join(".local").join("state"),
        PathBuf::from,
    )
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_or_else(|_| PathBuf::from("/tmp"), PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_produces_non_empty_paths() {
        let dirs = PrimalDirs::resolve("testprimal");
        assert!(!dirs.config.as_os_str().is_empty());
        assert!(!dirs.data.as_os_str().is_empty());
        assert!(!dirs.runtime.as_os_str().is_empty());
        assert!(!dirs.cache.as_os_str().is_empty());
        assert!(!dirs.logs.as_os_str().is_empty());
    }

    #[test]
    fn resolve_includes_primal_name() {
        let dirs = PrimalDirs::resolve("beardog");
        assert!(dirs.config.to_string_lossy().contains("beardog"));
        assert!(dirs.data.to_string_lossy().contains("beardog"));
        assert!(dirs.cache.to_string_lossy().contains("beardog"));
        assert!(dirs.logs.to_string_lossy().contains("beardog"));
    }

    #[test]
    fn resolve_includes_namespace() {
        let dirs = PrimalDirs::resolve("squirrel");
        assert!(dirs.config.to_string_lossy().contains(NAMESPACE));
        assert!(dirs.runtime.to_string_lossy().contains(NAMESPACE));
    }

    #[test]
    fn socket_path_in_runtime() {
        let dirs = PrimalDirs::resolve("beardog");
        let sock = dirs.socket_path("beardog");
        assert!(sock.to_string_lossy().ends_with("beardog.sock"));
        assert!(sock.starts_with(&dirs.runtime));
    }

    #[test]
    fn pid_path_in_runtime() {
        let dirs = PrimalDirs::resolve("beardog");
        let pid = dirs.pid_path("beardog");
        assert!(pid.to_string_lossy().ends_with("beardog.pid"));
        assert!(pid.starts_with(&dirs.runtime));
    }

    #[test]
    fn env_override_constants_are_defined() {
        assert_eq!(env_overrides::CONFIG_DIR, "BIOMEOS_CONFIG_DIR");
        assert_eq!(env_overrides::DATA_DIR, "BIOMEOS_DATA_DIR");
        assert_eq!(env_overrides::RUNTIME_DIR, "BIOMEOS_RUNTIME_DIR");
        assert_eq!(env_overrides::CACHE_DIR, "BIOMEOS_CACHE_DIR");
        assert_eq!(env_overrides::LOG_DIR, "BIOMEOS_LOG_DIR");
    }

    #[test]
    fn ensure_creates_directories_with_explicit_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let dirs = PrimalDirs {
            config: base.join("config").join("myprimal"),
            data: base.join("data").join("myprimal"),
            runtime: base.join("run"),
            cache: base.join("cache").join("myprimal"),
            logs: base.join("log").join("myprimal"),
        };

        dirs.ensure().unwrap();

        assert!(dirs.config.exists());
        assert!(dirs.data.exists());
        assert!(dirs.runtime.exists());
        assert!(dirs.cache.exists());
        assert!(dirs.logs.exists());
    }

    #[test]
    fn different_primals_get_different_dirs() {
        let a = PrimalDirs::resolve("alpha");
        let b = PrimalDirs::resolve("bravo");
        assert_ne!(a.config, b.config);
        assert_ne!(a.data, b.data);
        assert_eq!(a.runtime, b.runtime);
    }

    #[test]
    fn home_dir_returns_something() {
        let h = home_dir();
        assert!(!h.as_os_str().is_empty());
    }
}
