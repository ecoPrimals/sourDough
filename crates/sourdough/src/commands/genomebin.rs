//! GenomeBin creation and management commands.

use anyhow::Result;
use clap::Subcommand;
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
    crate::info(&format!("Creating genomeBin for {} v{}", primal, version));

    if !ecobins.exists() {
        anyhow::bail!("ecoBins directory not found: {}", ecobins.display());
    }

    // Find the create-genomebin.sh script
    let script_path = find_genomebin_script("create-genomebin.sh")?;

    // Execute the script
    let status = std::process::Command::new(&script_path)
        .arg("--primal")
        .arg(&primal)
        .arg("--version")
        .arg(&version)
        .arg("--ecobins")
        .arg(&ecobins)
        .arg("--output")
        .arg(&output)
        .status()?;

    if !status.success() {
        anyhow::bail!("genomeBin creation failed");
    }

    Ok(())
}

async fn test_genomebin(genomebin: PathBuf) -> Result<()> {
    crate::info(&format!("Testing genomeBin: {}", genomebin.display()));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {}", genomebin.display());
    }

    // Find the test-genomebin.sh script
    let script_path = find_genomebin_script("test-genomebin.sh")?;

    // Execute the script
    let status = std::process::Command::new(&script_path)
        .arg(&genomebin)
        .status()?;

    if !status.success() {
        anyhow::bail!("genomeBin testing failed");
    }

    Ok(())
}

async fn sign_genomebin(genomebin: PathBuf) -> Result<()> {
    crate::info(&format!("Signing genomeBin: {}", genomebin.display()));

    if !genomebin.exists() {
        anyhow::bail!("genomeBin not found: {}", genomebin.display());
    }

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
