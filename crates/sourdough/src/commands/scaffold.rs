//! Scaffolding commands for creating new primals and crates.
//!
//! sourDough is the nascent primal — the budding primal. When it scaffolds
//! a new primal, the offspring is fully self-contained: all primal DNA
//! (traits, types, patterns) is inlined. No runtime dependency on sourDough.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(crate) enum ScaffoldCommand {
    /// Create a new primal
    #[command(name = "new-primal")]
    NewPrimal {
        /// Name of the primal (e.g., "rhizoCrypt")
        name: String,

        /// Description of the primal
        description: String,

        /// Output directory (defaults to parent of current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Add a new crate to an existing primal
    #[command(name = "new-crate")]
    NewCrate {
        /// Name of the primal
        primal: String,

        /// Name of the new crate (e.g., "rhizocrypt-storage")
        crate_name: String,

        /// Path to the primal directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn run(cmd: ScaffoldCommand) -> Result<()> {
    match cmd {
        ScaffoldCommand::NewPrimal {
            name,
            description,
            output,
        } => create_primal(&name, &description, output),
        ScaffoldCommand::NewCrate {
            primal,
            crate_name,
            path,
        } => create_crate(&primal, &crate_name, path),
    }
}

fn create_primal(name: &str, description: &str, output: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Creating new primal: {name}"));

    let output_dir = output.unwrap_or_else(|| PathBuf::from("..").join(name));
    std::fs::create_dir_all(&output_dir).context("Failed to create primal directory")?;

    let crates_dir = output_dir.join("crates");
    std::fs::create_dir_all(&crates_dir)?;

    write_workspace_cargo_toml(&output_dir, name)?;
    create_core_crate(&crates_dir, name)?;
    write_specs_directory(&output_dir, name, description)?;
    write_readme(&output_dir, name, description)?;
    write_conventions(&output_dir)?;

    crate::success(&format!(
        "Created primal '{name}' at {}",
        output_dir.display()
    ));
    crate::info("Next steps:");
    println!("  cd {}", output_dir.display());
    println!("  cargo build");
    println!("  cargo test");

    Ok(())
}

fn create_crate(primal: &str, crate_name: &str, path: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Adding crate '{crate_name}' to primal '{primal}'"));

    let primal_dir = path.unwrap_or_else(|| PathBuf::from("..").join(primal));

    if !primal_dir.exists() {
        anyhow::bail!("Primal directory not found: {}", primal_dir.display());
    }

    let crate_dir = primal_dir.join("crates").join(crate_name);
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let core_crate = format!("{}-core", primal.to_lowercase());
    let core_crate_ident = core_crate.replace('-', "_");

    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
description = "{crate_name} crate"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
{core_crate} = {{ path = "../{core_crate}" }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
thiserror = {{ workspace = true }}
"#,
        ),
    )?;

    std::fs::write(
        src_dir.join("lib.rs"),
        format!(
            r"//! # {crate_name}
//!
//! Part of the {primal} primal.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]

pub use {core_crate_ident}::PrimalError;
",
        ),
    )?;

    update_workspace_members(&primal_dir, crate_name)?;

    crate::success(&format!("Created crate '{crate_name}'"));
    crate::info("Workspace members updated in Cargo.toml.");

    Ok(())
}

// ---------------------------------------------------------------------------
// File generators
// ---------------------------------------------------------------------------

fn write_workspace_cargo_toml(dir: &Path, name: &str) -> Result<()> {
    let core_crate_name = format!("{}-core", name.to_lowercase());
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            r#"[workspace]
resolver = "2"
members = [
    "crates/{core_crate_name}",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "AGPL-3.0-or-later"
repository = "https://github.com/ecoPrimals/{name}"
authors = ["ecoPrimals Project"]

[workspace.dependencies]
# Async runtime
tokio = {{ version = "1.40", features = ["macros", "rt-multi-thread", "signal", "net", "io-util", "time"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
toml = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"#,
        ),
    )?;
    Ok(())
}

fn primal_rust_type_name(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

fn create_core_crate(crates_dir: &Path, name: &str) -> Result<()> {
    let core_crate_name = format!("{}-core", name.to_lowercase());
    let core_dir = crates_dir.join(&core_crate_name);
    let src_dir = core_dir.join("src");

    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        core_dir.join("Cargo.toml"),
        generated_core_cargo_toml(&core_crate_name, name),
    )?;
    std::fs::write(src_dir.join("error.rs"), GENERATED_ERROR_RS)?;
    std::fs::write(src_dir.join("lifecycle.rs"), GENERATED_LIFECYCLE_RS)?;
    std::fs::write(src_dir.join("health.rs"), GENERATED_HEALTH_RS)?;
    std::fs::write(src_dir.join("lib.rs"), generated_lib_rs(name))?;

    Ok(())
}

fn write_specs_directory(dir: &Path, name: &str, description: &str) -> Result<()> {
    let specs_dir = dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    let date = chrono::Local::now().format("%B %d, %Y");
    std::fs::write(
        specs_dir.join(format!("{}_SPECIFICATION.md", name.to_uppercase())),
        format!(
            r"# {name} - Specification

**Version**: 0.1.0
**Date**: {date}
**Status**: Draft

---

## Purpose

{description}

## Architecture

(To be defined)

## Components

(To be defined)

---

**Date**: {date}
**Version**: 0.1.0
",
        ),
    )?;

    Ok(())
}

fn write_readme(dir: &Path, name: &str, description: &str) -> Result<()> {
    std::fs::write(
        dir.join("README.md"),
        format!(
            r"# {name}

**Status**: Draft
**Purpose**: {description}

---

## Quick Start

```bash
cargo build
cargo test
```

## Structure

```
{name}/
├── crates/
│   └── {core}-core/
└── specs/
```

---

*Scaffolded by sourDough — the nascent primal.*
",
            core = name.to_lowercase(),
        ),
    )?;

    Ok(())
}

fn write_conventions(dir: &Path) -> Result<()> {
    std::fs::write(
        dir.join("CONVENTIONS.md"),
        r"# Coding Conventions

This primal follows the ecoPrimals coding conventions.

## Quick Reference

- **Edition**: 2024
- **License**: AGPL-3.0-or-later (scyBorg triple license)
- **Linting**: `#![forbid(unsafe_code)]`, `#![warn(clippy::all, clippy::pedantic, clippy::nursery)]`
- **Docs**: `#![warn(missing_docs)]`
- **Max file size**: 1000 LOC
- **Test coverage**: 90%+
- **IPC**: JSON-RPC 2.0 (`domain.verb` naming)
- **Identity**: Self-sovereign DIDs
- **Discovery**: Runtime via universal adapter (zero hardcoding)

*Consistency is the foundation of collaboration.*
",
    )?;

    Ok(())
}

fn update_workspace_members(primal_dir: &Path, crate_name: &str) -> Result<()> {
    let cargo_path = primal_dir.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_path)
        .with_context(|| format!("failed to read {}", cargo_path.display()))?;
    let mut root: toml::Value =
        toml::from_str(&raw).context("workspace Cargo.toml is not valid TOML")?;
    let workspace = root
        .as_table_mut()
        .and_then(|t| t.get_mut("workspace"))
        .and_then(toml::Value::as_table_mut)
        .context("missing [workspace] table in Cargo.toml")?;
    let members = workspace
        .entry("members")
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    let arr = members
        .as_array_mut()
        .context("[workspace].members must be an array")?;
    let entry = format!("crates/{crate_name}");
    if arr.iter().any(|v| v.as_str() == Some(entry.as_str())) {
        crate::info(&format!(
            "Crate '{crate_name}' already listed in workspace members"
        ));
        return Ok(());
    }
    arr.push(toml::Value::String(entry));
    let updated = toml::to_string_pretty(&root).context("failed to serialize Cargo.toml")?;
    std::fs::write(&cargo_path, updated)
        .with_context(|| format!("failed to write {}", cargo_path.display()))?;
    crate::success(&format!(
        "Added 'crates/{crate_name}' to [workspace].members"
    ));
    Ok(())
}

// ---------------------------------------------------------------------------
// Inlined primal DNA templates — the offspring is self-contained after budding
// ---------------------------------------------------------------------------

fn generated_core_cargo_toml(core_crate_name: &str, name: &str) -> String {
    format!(
        r#"[package]
name = "{core_crate_name}"
description = "Core library for {name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
thiserror = {{ workspace = true }}
tracing = {{ workspace = true }}

[dev-dependencies]
tokio = {{ workspace = true, features = ["test-util"] }}
"#,
    )
}

fn generated_lib_rs(name: &str) -> String {
    let type_name = primal_rust_type_name(name);
    format!(
        r#"//! # {name} Core
//!
//! Core library for the {name} primal.
//!
//! Self-contained: all primal DNA (traits, types, patterns) is defined here.
//! This primal discovers other primals at runtime via JSON-RPC 2.0 IPC.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::nursery)]

pub mod error;
pub mod health;
pub mod lifecycle;

pub use error::{{PrimalError, PrimalResult}};
pub use health::{{HealthReport, HealthStatus, PrimalHealth}};
pub use lifecycle::{{PrimalLifecycle, PrimalState}};

/// The {name} primal.
pub struct {type_name}Primal {{
    state: PrimalState,
}}

impl {type_name}Primal {{
    /// Create a new primal instance.
    #[must_use]
    pub fn new() -> Self {{
        Self {{
            state: PrimalState::Created,
        }}
    }}
}}

impl Default for {type_name}Primal {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl PrimalLifecycle for {type_name}Primal {{
    fn state(&self) -> PrimalState {{
        self.state
    }}

    async fn start(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_start() {{
            return Err(PrimalError::lifecycle("cannot start from current state"));
        }}
        self.state = PrimalState::Running;
        Ok(())
    }}

    async fn stop(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_stop() {{
            return Err(PrimalError::lifecycle("cannot stop from current state"));
        }}
        self.state = PrimalState::Stopped;
        Ok(())
    }}
}}

impl PrimalHealth for {type_name}Primal {{
    fn health_status(&self) -> HealthStatus {{
        if self.state.is_running() {{
            HealthStatus::Healthy
        }} else {{
            HealthStatus::Unknown
        }}
    }}

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {{
        Ok(HealthReport::new("{name}", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_lifecycle() {{
        let mut primal = {type_name}Primal::new();
        assert_eq!(primal.state(), PrimalState::Created);

        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);

        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
    }}

    #[tokio::test]
    async fn test_health() {{
        let mut primal = {type_name}Primal::new();
        primal.start().await.unwrap();

        assert!(primal.health_status().is_healthy());

        let report = primal.health_check().await.unwrap();
        assert_eq!(report.name, "{name}");
    }}
}}
"#,
    )
}

const GENERATED_ERROR_RS: &str = r#"//! Common error types for this primal.
//!
//! Extend this enum with domain-specific variants as your primal evolves.

use thiserror::Error;

/// Result type alias for primal operations.
pub type PrimalResult<T> = Result<T, PrimalError>;

/// Common errors that any primal might encounter.
#[derive(Debug, Error)]
pub enum PrimalError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Lifecycle error (start/stop/reload).
    #[error("lifecycle error: {0}")]
    Lifecycle(String),

    /// Health check error.
    #[error("health error: {0}")]
    Health(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Timeout.
    #[error("operation timed out: {0}")]
    Timeout(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Dependency error (upstream service failed).
    #[error("dependency error: {service}: {message}")]
    Dependency {
        /// Name of the dependency that failed.
        service: String,
        /// Error message.
        message: String,
    },
}

impl PrimalError {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a lifecycle error.
    pub fn lifecycle(msg: impl Into<String>) -> Self {
        Self::Lifecycle(msg.into())
    }

    /// Create a dependency error.
    pub fn dependency(service: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Dependency {
            service: service.into(),
            message: msg.into(),
        }
    }

    /// Check if this is a retryable error.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::Dependency { .. }
        )
    }
}
"#;

const GENERATED_LIFECYCLE_RS: &str = r#"//! Primal lifecycle management.
//!
//! Every primal has a lifecycle: created, running, stopped.
//! This module provides the state machine and trait for managing it.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// State of a primal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalState {
    /// Not yet started.
    Created,
    /// Starting up.
    Starting,
    /// Running normally.
    Running,
    /// Stopping.
    Stopping,
    /// Stopped.
    Stopped,
    /// Failed.
    Failed,
}

impl PrimalState {
    /// Check if the primal is running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal can be started.
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(self, Self::Created | Self::Stopped | Self::Failed)
    }

    /// Check if the primal can be stopped.
    #[must_use]
    pub const fn can_stop(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for PrimalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Lifecycle trait for primals.
///
/// Implement this to define how your primal starts, stops, and reloads.
pub trait PrimalLifecycle: Send + Sync {
    /// Get the current state.
    fn state(&self) -> PrimalState;

    /// Start the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if startup fails.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Reload configuration (default: stop then start).
    ///
    /// # Errors
    ///
    /// Returns an error if reload fails.
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async {
            self.stop().await?;
            self.start().await
        }
    }

    /// Handle a shutdown signal (default: calls stop).
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async { self.stop().await }
    }
}
"#;

const GENERATED_HEALTH_RS: &str = r"//! Health check traits for observability.
//!
//! Every primal needs to be observable. This module provides health check
//! traits usable by orchestrators, load balancers, and monitoring systems.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// Overall health status of a primal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy and ready to serve requests.
    Healthy,
    /// Unhealthy but may recover.
    Degraded {
        /// Reason for degraded status.
        reason: String,
    },
    /// Unhealthy and not serving requests.
    Unhealthy {
        /// Reason for unhealthy status.
        reason: String,
    },
    /// Health unknown (e.g., startup in progress).
    Unknown,
}

impl HealthStatus {
    /// Check if the status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status allows serving requests.
    #[must_use]
    pub const fn is_serving(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }
}

/// Health report for a primal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    /// Primal name.
    pub name: String,
    /// Primal version.
    pub version: String,
    /// Overall status.
    pub status: HealthStatus,
    /// Liveness (is the process alive?).
    pub liveness: bool,
    /// Readiness (can it serve requests?).
    pub readiness: bool,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Unknown,
            liveness: true,
            readiness: false,
        }
    }

    /// Set status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.readiness = status.is_serving();
        self.status = status;
        self
    }
}

/// Health check trait for primals.
///
/// Implement this to provide health information about your primal.
pub trait PrimalHealth: Send + Sync {
    /// Get the current health status (quick check).
    fn health_status(&self) -> HealthStatus;

    /// Perform a full health check (may be expensive).
    ///
    /// # Errors
    ///
    /// Returns an error if the health check itself fails.
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;

    /// Check liveness (is the process alive?).
    fn is_live(&self) -> bool {
        true
    }

    /// Check readiness (can it serve requests?).
    fn is_ready(&self) -> bool {
        self.health_status().is_serving()
    }
}
";
