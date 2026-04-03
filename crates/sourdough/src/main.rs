//! # `SourDough` `UniBin` CLI
//!
//! The reference `UniBin` implementation for ecoPrimals.
//!
//! This binary provides tooling for:
//! - Scaffolding new primals
//! - Creating genomeBins
//! - Validating primal compliance
//! - Health diagnostics

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

pub(crate) mod commands;

#[derive(Parser)]
#[command(name = "sourdough")]
#[command(about = "🍞 SourDough - The reference UniBin for ecoPrimals")]
#[command(version)]
#[command(
    long_about = "SourDough provides tooling for creating, validating, and deploying ecoPrimals.\n\nUse 'sourdough <command> --help' for more information on a specific command."
)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold new primals and crates
    Scaffold {
        #[command(subcommand)]
        scaffold_cmd: commands::scaffold::ScaffoldCommand,
    },

    /// Create and manage genomeBins
    #[command(name = "genomebin")]
    GenomeBin {
        #[command(subcommand)]
        genomebin_cmd: commands::genomebin::GenomeBinCommand,
    },

    /// Validate primal compliance
    Validate {
        #[command(subcommand)]
        validate_cmd: commands::validate::ValidateCommand,
    },

    /// Run health diagnostics
    Doctor {
        /// Run comprehensive checks
        #[arg(long)]
        comprehensive: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    // Execute command
    match cli.command {
        Commands::Scaffold { scaffold_cmd } => {
            commands::scaffold::run(scaffold_cmd)?;
        }
        Commands::GenomeBin { genomebin_cmd } => {
            commands::genomebin::run(genomebin_cmd).await?;
        }
        Commands::Validate { validate_cmd } => {
            commands::validate::run(validate_cmd)?;
        }
        Commands::Doctor { comprehensive } => {
            commands::doctor::run(comprehensive)?;
        }
    }

    Ok(())
}

/// Print success message
pub(crate) fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print error message
pub(crate) fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// Print warning message
pub(crate) fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print info message
pub(crate) fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}
