//! Scaffolding commands for creating new primals and crates.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum ScaffoldCommand {
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

pub async fn run(cmd: ScaffoldCommand) -> Result<()> {
    match cmd {
        ScaffoldCommand::NewPrimal {
            name,
            description,
            output,
        } => create_primal(name, description, output).await,
        ScaffoldCommand::NewCrate {
            primal,
            crate_name,
            path,
        } => create_crate(primal, crate_name, path).await,
    }
}

async fn create_primal(name: String, description: String, output: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Creating new primal: {}", name));

    // Determine output directory
    let output_dir = output.unwrap_or_else(|| PathBuf::from("..").join(&name));

    // Create directory structure
    std::fs::create_dir_all(&output_dir).context("Failed to create primal directory")?;

    // Create workspace Cargo.toml
    create_workspace_cargo_toml(&output_dir, &name, &description)?;

    // Create crates directory
    let crates_dir = output_dir.join("crates");
    std::fs::create_dir_all(&crates_dir)?;

    // Create core crate
    create_core_crate(&crates_dir, &name)?;

    // Create specs directory
    create_specs_directory(&output_dir, &name, &description)?;

    // Create README
    create_readme(&output_dir, &name, &description)?;

    // Create CONVENTIONS.md symlink or copy
    create_conventions_file(&output_dir)?;

    crate::success(&format!(
        "Created primal '{}' at {}",
        name,
        output_dir.display()
    ));
    crate::info("Next steps:");
    println!("  cd {}", output_dir.display());
    println!("  cargo build");
    println!("  cargo test");

    Ok(())
}

async fn create_crate(primal: String, crate_name: String, path: Option<PathBuf>) -> Result<()> {
    crate::info(&format!(
        "Adding crate '{}' to primal '{}'",
        crate_name, primal
    ));

    let primal_dir = path.unwrap_or_else(|| PathBuf::from("..").join(&primal));

    if !primal_dir.exists() {
        anyhow::bail!("Primal directory not found: {}", primal_dir.display());
    }

    let crate_dir = primal_dir.join("crates").join(&crate_name);
    std::fs::create_dir_all(&crate_dir)?;

    // Create crate Cargo.toml
    create_crate_cargo_toml(&crate_dir, &crate_name)?;

    // Create src directory with lib.rs
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let lib_rs = src_dir.join("lib.rs");
    std::fs::write(
        &lib_rs,
        format!(
            "//! # {}\n//!\n//! Part of the {} primal\n\n#![warn(missing_docs)]\n#![warn(clippy::all)]\n#![warn(clippy::pedantic)]\n",
            crate_name,
            primal
        ),
    )?;

    // Update workspace Cargo.toml
    update_workspace_members(&primal_dir, &crate_name)?;

    crate::success(&format!("Created crate '{}'", crate_name));
    crate::info("Don't forget to add it to workspace members!");

    Ok(())
}

fn create_workspace_cargo_toml(dir: &Path, name: &str, _description: &str) -> Result<()> {
    let core_crate_name = format!("{}-core", name.to_lowercase());
    let cargo_toml = dir.join("Cargo.toml");

    // Try to find sourdough-core path relative to current binary
    let sourdough_core_path = find_sourdough_core_path(dir)?;
    let sourdough_core_dep = if let Some(path) = sourdough_core_path {
        format!(r#"sourdough-core = {{ path = "{}" }}"#, path)
    } else {
        // Fallback to reasonable default path (user will need to adjust)
        // Using a sibling directory assumption as the most common case
        r#"sourdough-core = { path = "../../sourDough/crates/sourdough-core" }
# NOTE: Adjust the path above to point to your sourDough installation
# OR use: sourdough-core = "0.1.0"  (once published)"#
            .to_string()
    };

    let content = format!(
        r#"[workspace]
resolver = "2"
members = [
    "crates/{crate_name}",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "AGPL-3.0"
repository = "https://github.com/ecoPrimals/{primal_name}"
authors = ["ecoPrimals Project"]

[workspace.dependencies]
# Core
{sourdough_core_dep}

# Async runtime
tokio = {{ version = "1.40", features = ["full"] }}

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
        crate_name = core_crate_name,
        primal_name = name,
        sourdough_core_dep = sourdough_core_dep
    );

    std::fs::write(cargo_toml, content)?;
    Ok(())
}

/// Try to find the path to sourdough-core from the target directory
fn find_sourdough_core_path(target_dir: &Path) -> Result<Option<String>> {
    // Try to find sourDough relative to the target
    // Common patterns:
    // 1. ../sourDough (sibling directory)
    // 2. ../../sourDough (parent's sibling)
    // 3. Via SOURDOUGH_PATH environment variable

    if let Ok(env_path) = std::env::var("SOURDOUGH_PATH") {
        let core_path = PathBuf::from(&env_path).join("crates/sourdough-core");
        if core_path.exists() {
            return Ok(Some(core_path.to_string_lossy().to_string()));
        }
    }

    // Try common relative paths
    for candidate in &["../sourDough", "../../sourDough", "../../../sourDough"] {
        let abs_candidate = target_dir.join(candidate).join("crates/sourdough-core");
        if abs_candidate.exists() {
            // Make it relative to target_dir
            if let Some(rel_path) = pathdiff::diff_paths(&abs_candidate, target_dir) {
                return Ok(Some(rel_path.to_string_lossy().to_string()));
            }
        }
    }

    Ok(None)
}

fn create_core_crate(crates_dir: &Path, name: &str) -> Result<()> {
    let core_crate_name = format!("{}-core", name.to_lowercase());
    let crate_dir = crates_dir.join(&core_crate_name);
    let src_dir = crate_dir.join("src");

    std::fs::create_dir_all(&src_dir)?;

    // Create Cargo.toml
    let cargo_toml = crate_dir.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
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
sourdough-core = {{ workspace = true }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
thiserror = {{ workspace = true }}
tracing = {{ workspace = true }}
"#,
            core_crate_name = core_crate_name,
            name = name
        ),
    )?;

    // Create lib.rs with trait implementations
    let lib_rs = src_dir.join("lib.rs");
    std::fs::write(
        &lib_rs,
        format!(
            r#"//! # {0} Core
//!
//! Core library for the {0} primal.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use sourdough_core::{{
    PrimalLifecycle, PrimalHealth, PrimalState, PrimalError,
    health::{{HealthStatus, HealthReport}},
}};

/// Main primal structure
pub struct {1}Primal {{
    state: PrimalState,
}}

impl {1}Primal {{
    /// Create a new primal instance
    #[must_use]
    pub fn new() -> Self {{
        Self {{
            state: PrimalState::Created,
        }}
    }}
}}

impl Default for {1}Primal {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl PrimalLifecycle for {1}Primal {{
    fn state(&self) -> PrimalState {{
        self.state
    }}

    async fn start(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_start() {{
            return Err(PrimalError::lifecycle("Cannot start from current state"));
        }}
        self.state = PrimalState::Running;
        Ok(())
    }}

    async fn stop(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_stop() {{
            return Err(PrimalError::lifecycle("Cannot stop from current state"));
        }}
        self.state = PrimalState::Stopped;
        Ok(())
    }}
}}

impl PrimalHealth for {1}Primal {{
    fn health_status(&self) -> HealthStatus {{
        if self.state.is_running() {{
            HealthStatus::Healthy
        }} else {{
            HealthStatus::Unknown
        }}
    }}

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {{
        Ok(HealthReport::new("{0}", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_lifecycle() {{
        let mut primal = {1}Primal::new();
        
        assert_eq!(primal.state(), PrimalState::Created);
        
        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);
        
        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
    }}

    #[tokio::test]
    async fn test_health() {{
        let mut primal = {1}Primal::new();
        primal.start().await.unwrap();
        
        assert!(primal.health_status().is_healthy());
        
        let report = primal.health_check().await.unwrap();
        assert_eq!(report.name, "{0}");
    }}
}}
"#,
            name,
            // Capitalize first letter for struct name
            name.chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>()
                + &name[1..]
        ),
    )?;

    Ok(())
}

fn create_specs_directory(dir: &Path, name: &str, description: &str) -> Result<()> {
    let specs_dir = dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    // Create SPECIFICATION.md
    let spec_file = specs_dir.join(format!("{}_SPECIFICATION.md", name.to_uppercase()));
    std::fs::write(
        &spec_file,
        format!(
            r#"# {} - Specification

**Version**: 0.1.0  
**Date**: {}  
**Status**: Draft

---

## Purpose

{}

## Architecture

(To be defined)

## Components

(To be defined)

---

**Date**: {}  
**Version**: 0.1.0
"#,
            name,
            chrono::Local::now().format("%B %d, %Y"),
            description,
            chrono::Local::now().format("%B %d, %Y")
        ),
    )?;

    Ok(())
}

fn create_readme(dir: &Path, name: &str, description: &str) -> Result<()> {
    let readme = dir.join("README.md");
    std::fs::write(
        &readme,
        format!(
            r#"# {}

**Status**: Draft  
**Purpose**: {}

---

## Quick Start

```bash
cargo build
cargo test
```

## Structure

```
{}/
├── crates/
│   └── {}-core/
└── specs/
```

---

**Created with SourDough** 🍞
"#,
            name,
            description,
            name,
            name.to_lowercase()
        ),
    )?;

    Ok(())
}

fn create_conventions_file(dir: &Path) -> Result<()> {
    let conventions = dir.join("CONVENTIONS.md");

    // For now, create a simple reference
    std::fs::write(
        &conventions,
        r#"# Coding Conventions

This primal follows the ecoPrimals coding conventions.

See: `../sourDough/CONVENTIONS.md` for complete guidelines.

## Quick Reference

- **Edition**: 2021
- **Linting**: `#![warn(clippy::all, clippy::pedantic)]`
- **Docs**: `#![warn(missing_docs)]`
- **Max file size**: 1000 LOC
- **Test coverage**: 90%+

---

*Consistency is the foundation of collaboration.*
"#,
    )?;

    Ok(())
}

fn create_crate_cargo_toml(dir: &Path, name: &str) -> Result<()> {
    let cargo_toml = dir.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        format!(
            r#"[package]
name = "{name}"
description = "{name} crate"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
sourdough-core = {{ workspace = true }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
thiserror = {{ workspace = true }}
"#,
            name = name
        ),
    )?;

    Ok(())
}

fn update_workspace_members(_primal_dir: &PathBuf, crate_name: &str) -> Result<()> {
    // Simple append to members array (in production, would parse TOML properly)
    crate::info(&format!(
        "Add 'crates/{}' to workspace members in Cargo.toml",
        crate_name
    ));

    Ok(())
}
