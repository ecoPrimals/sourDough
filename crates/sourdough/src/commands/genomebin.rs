//! `GenomeBin` creation and management commands.
//!
//! This module provides genomeBin operations using the Pure Rust `sourdough-genomebin`
//! library. The bash script fallback is maintained for compatibility during migration.

use anyhow::Result;
use clap::Subcommand;
use sourdough_genomebin::{GenomeBinBuilder, Validator};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum GenomeBinCommand {
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

pub async fn run(cmd: GenomeBinCommand) -> Result<()> {
    match cmd {
        GenomeBinCommand::Create {
            primal,
            version,
            ecobins,
            output,
        } => create_genomebin(primal, version, ecobins, output).await,
        GenomeBinCommand::Test { genomebin } => test_genomebin(genomebin).await,
        GenomeBinCommand::Sign { genomebin } => sign_genomebin(genomebin).await,
    }
}

async fn create_genomebin(
    primal: String,
    version: String,
    ecobins: PathBuf,
    output: PathBuf,
) -> Result<()> {
    crate::info(&format!("Creating genomeBin for {} v{} (Rust)", primal, version));

    // Use Pure Rust implementation
    let builder = GenomeBinBuilder::new(&primal, &version)
        .ecobins_dir(&ecobins)
        .output(&output)
        .parallel(true);

    let genome = builder.build().await?;
    let output_path = genome.create().await?;

    crate::success(&format!("genomeBin created: {}", output_path.display()));
    crate::info(&format!("Targets: {}", genome.targets().join(", ")));

    Ok(())
}

async fn test_genomebin(genomebin: PathBuf) -> Result<()> {
    crate::info(&format!("Testing genomeBin: {} (Rust)", genomebin.display()));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {}", genomebin.display());
    }

    // Use Pure Rust validator
    let validator = Validator::new(&genomebin);
    let results = validator.validate().await?;

    // Display results
    for result in &results {
        if result.passed {
            crate::success(&format!("✓ {}", result.name));
        } else {
            crate::error(&format!(
                "✗ {}: {}",
                result.name,
                result.message.as_deref().unwrap_or("unknown error")
            ));
        }
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();

    if passed == total {
        crate::success(&format!("All tests passed ({passed}/{total})"));
        Ok(())
    } else {
        anyhow::bail!("Some tests failed ({passed}/{total})");
    }
}

async fn sign_genomebin(genomebin: PathBuf) -> Result<()> {
    crate::info(&format!("Signing genomeBin: {}", genomebin.display()));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {}", genomebin.display());
    }

    // GPG signing is currently disabled (requires C dependencies)
    // Future: Implement Pure Rust signing via sequoia-openpgp
    crate::warning("GPG signing not yet implemented in Rust");
    crate::info("Falling back to bash script for signing...");

    // Find the sign-genomebin.sh script
    let script_path = find_genomebin_script("sign-genomebin.sh")?;

    // Execute the script
    let status = std::process::Command::new(&script_path)
        .arg(&genomebin)
        .status()?;

    if !status.success() {
        anyhow::bail!("genomeBin signing failed");
    }

    Ok(())
}

/// Find a genomeBin script in the standard locations
fn find_genomebin_script(script_name: &str) -> Result<PathBuf> {
    // Try several locations:
    // 1. Relative to current executable (for development)
    // 2. Installed location (/usr/local/share/sourdough/genomebin/)
    // 3. System location (/usr/share/sourdough/genomebin/)
    // 4. Workspace location (for development)

    let candidates = vec![
        // Development: relative to workspace root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../genomebin/scripts")
            .join(script_name),
        // Installed: /usr/local/share
        PathBuf::from("/usr/local/share/sourdough/genomebin/scripts").join(script_name),
        // Installed: /usr/share
        PathBuf::from("/usr/share/sourdough/genomebin/scripts").join(script_name),
        // Local user install
        std::env::var("HOME")
            .ok()
            .map(|home| {
                PathBuf::from(home)
                    .join(".local/share/sourdough/genomebin/scripts")
                    .join(script_name)
            })
            .unwrap_or_default(),
    ];

    for candidate in candidates {
        if candidate.exists() && candidate.is_file() {
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "genomeBin script '{}' not found. Please ensure sourDough is properly installed.",
        script_name
    )
}
