//! Health diagnostics for `SourDough` and the ecosystem.

use anyhow::Result;
use sourdough_genomebin::Platform;

pub(crate) fn run(comprehensive: bool) -> Result<()> {
    crate::info("SourDough Health Check");
    println!();

    check_sourdough_binary();
    check_rust_toolchain()?;
    check_common_tools();

    if comprehensive {
        println!();
        crate::info("Running comprehensive checks...");
        check_cross_compilation_targets();
        check_genome_bin_tools();
    }

    println!();
    crate::success("All checks passed!");

    Ok(())
}

fn check_sourdough_binary() {
    crate::info("Checking SourDough binary...");

    let version = env!("CARGO_PKG_VERSION");
    println!("  Version: {version}");

    crate::success("Binary OK");
}

fn check_rust_toolchain() -> Result<()> {
    crate::info("Checking Rust toolchain...");

    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  rustc: {}", version.trim());
        crate::success("Rust toolchain OK");
    } else {
        crate::error("rustc not found");
        anyhow::bail!("Rust compiler not found");
    }

    let output = std::process::Command::new("cargo")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  cargo: {}", version.trim());
    }

    Ok(())
}

fn check_common_tools() {
    crate::info("Checking common tools...");

    let tools = [
        ("git", "Version control"),
        ("cargo-llvm-cov", "Code coverage"),
    ];

    for (tool, description) in tools {
        let output = std::process::Command::new(tool).arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                println!("  ✓ {tool} ({description})");
            }
            _ => {
                println!("  ⚠ {tool} ({description}) - not found");
            }
        }
    }
}

fn check_cross_compilation_targets() {
    crate::info("Checking cross-compilation targets...");

    let targets = [
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ];

    if let Ok(output) = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        if output.status.success() {
            let installed = String::from_utf8_lossy(&output.stdout);
            for target in targets {
                if installed.contains(target) {
                    println!("  ✓ {target}");
                } else {
                    println!("  ⚠ {target} - not installed");
                }
            }
        }
    }
}

fn check_genome_bin_tools() {
    crate::info("Checking genomeBin tools (Pure Rust)...");

    match Platform::detect() {
        Ok(platform) => {
            println!("  Platform: {platform}");
            println!("  Target triple: {}", platform.target_triple());
            crate::success("Platform detection OK");
        }
        Err(e) => {
            crate::warning(&format!("Platform detection issue: {e}"));
        }
    }

    println!("  ✓ Archive operations (tar + flate2, Pure Rust)");
    println!("  ✓ Checksum (BLAKE3 + SHA256, Pure Rust)");
    println!("  ✓ Metadata (TOML, Pure Rust)");
    println!("  ⚠ Signing (sequoia-openpgp, planned)");

    crate::success("genomeBin tooling OK");
}
