//! Scaffolding commands for creating new primals and crates.
//!
//! sourDough is the nascent primal — the budding primal. When it scaffolds
//! a new primal, the offspring is fully self-contained: all primal DNA
//! (traits, types, patterns) is inlined. No runtime dependency on sourDough.

mod generators;
mod templates;

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

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

    generators::write_workspace_cargo_toml(&output_dir, name)?;
    generators::create_core_crate(&crates_dir, name)?;
    generators::create_server_crate(&crates_dir, name)?;
    generators::write_deny_toml(&output_dir)?;
    generators::write_github_workflows(&output_dir, name)?;
    generators::write_specs_directory(&output_dir, name, description)?;
    generators::write_readme(&output_dir, name, description)?;
    generators::write_conventions(&output_dir)?;

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

[lints]
workspace = true

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

pub use {core_crate_ident}::PrimalError;
",
        ),
    )?;

    generators::update_workspace_members(&primal_dir, crate_name)?;

    crate::success(&format!("Created crate '{crate_name}'"));
    crate::info("Workspace members updated in Cargo.toml.");

    Ok(())
}

/// Convert a primal name to a Rust type name (uppercase first letter).
fn primal_rust_type_name(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}
