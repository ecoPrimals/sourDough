//! Example: Create and validate a genomeBin
//!
//! Demonstrates the complete workflow: creation and validation.

use sourdough_genomebin::{GenomeBinBuilder, Validator};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("genomeBin Creation & Validation Example\n");

    let primal = "example-primal";
    let version = "1.0.0";
    let ecobins_dir = PathBuf::from("./ecobins");
    let output = PathBuf::from("example-primal-1.0.0.genome");

    println!("Configuration:");
    println!("  Primal:          {primal}");
    println!("  Version:         {version}");
    println!("  ecoBins Dir:     {}", ecobins_dir.display());
    println!("  Output:          {}", output.display());
    println!();

    if !ecobins_dir.exists() {
        eprintln!(
            "Error: ecoBins directory not found: {}",
            ecobins_dir.display()
        );
        eprintln!("   Please create ecoBins first or modify the path.");
        std::process::exit(1);
    }

    println!("Step 1: Building genomeBin...");
    let builder = GenomeBinBuilder::new(primal, version)
        .ecobins_dir(&ecobins_dir)
        .output(&output)
        .parallel(true);

    match builder.build().await {
        Ok(genome) => {
            println!("genomeBin configuration ready");
            println!("   Targets: {}", genome.targets().join(", "));
            println!();

            println!("Step 2: Creating genomeBin...");
            match genome.create().await {
                Ok(output_path) => {
                    println!("genomeBin created: {}", output_path.display());
                    println!();

                    println!("Step 3: Validating genomeBin...");
                    let validator = Validator::new(&output_path);
                    match validator.validate().await {
                        Ok(results) => {
                            for result in &results {
                                if result.passed {
                                    println!("  PASS {}", result.name);
                                } else {
                                    let msg = result.message.as_deref().unwrap_or("unknown");
                                    println!("  FAIL {}: {msg}", result.name);
                                }
                            }

                            let passed = results.iter().filter(|r| r.passed).count();
                            let total = results.len();
                            println!();
                            println!("Validation complete: {passed}/{total} tests passed");
                        }
                        Err(e) => {
                            eprintln!("Validation failed: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("genomeBin creation failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("genomeBin build failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
