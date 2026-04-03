//! `GenomeBin` creation and management commands.
//!
//! This module provides genomeBin operations using the Pure Rust `sourdough-genomebin`
//! library. The bash script fallback is maintained for compatibility during migration.

use anyhow::Result;
use clap::Subcommand;
use sourdough_genomebin::{GenomeBinBuilder, Validator};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(crate) enum GenomeBinCommand {
    /// Create a genomeBin from ecoBins
    Create {
        /// Primal name
        #[arg(long)]
        primal: String,

        /// Version
        #[arg(long)]
        version: String,

        /// Path to ecoBins directory
        #[arg(long)]
        ecobins: PathBuf,

        /// Output genomeBin path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Test a genomeBin
    Test {
        /// Path to genomeBin file
        genomebin: PathBuf,
    },

    /// Sign a genomeBin
    Sign {
        /// Path to genomeBin file
        genomebin: PathBuf,
    },
}

pub(crate) async fn run(cmd: GenomeBinCommand) -> Result<()> {
    match cmd {
        GenomeBinCommand::Create {
            primal,
            version,
            ecobins,
            output,
        } => create_genomebin(primal, version, ecobins, output).await,
        GenomeBinCommand::Test { genomebin } => test_genomebin(genomebin).await,
        GenomeBinCommand::Sign { genomebin } => sign_genomebin(&genomebin),
    }
}

async fn create_genomebin(
    primal: String,
    version: String,
    ecobins: PathBuf,
    output: PathBuf,
) -> Result<()> {
    crate::info(&format!(
        "Creating genomeBin for {primal} v{version} (Rust)"
    ));

    // Use Pure Rust implementation
    let builder = GenomeBinBuilder::new(&primal, &version)
        .ecobins_dir(&ecobins)
        .output(&output)
        .parallel(true);

    let genome = builder.build().await?;
    let output_path = genome.create().await?;

    let out = output_path.display();
    crate::success(&format!("genomeBin created: {out}"));
    let targets = genome.targets().join(", ");
    crate::info(&format!("Targets: {targets}"));

    Ok(())
}

async fn test_genomebin(genomebin: PathBuf) -> Result<()> {
    let gb = genomebin.display();
    crate::info(&format!("Testing genomeBin: {gb} (Rust)"));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {gb}");
    }

    let validator = Validator::new(&genomebin);
    let results = validator.run_all_tests().await;

    println!("\nValidation Results:");
    println!("═══════════════════");
    for result in &results {
        let n = &result.name;
        if result.passed {
            println!("  ✓ {n}");
        } else {
            let msg = result.message.as_deref().unwrap_or("unknown error");
            println!("  ✗ {n}: {msg}");
        }
    }
    println!();

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();

    if passed == total {
        crate::success(&format!("All tests passed ({passed}/{total})"));
        Ok(())
    } else {
        crate::error(&format!("Some tests failed ({passed}/{total})"));
        anyhow::bail!("Validation failed: {passed}/{total} tests passed");
    }
}

fn sign_genomebin(genomebin: &Path) -> Result<()> {
    let gb = genomebin.display();
    crate::info(&format!("Signing genomeBin: {gb}"));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {gb}");
    }

    anyhow::bail!(
        "genomeBin signing requires Pure Rust cryptography (sequoia-openpgp). \
         This will be implemented when BearDog identity services are available."
    )
}
