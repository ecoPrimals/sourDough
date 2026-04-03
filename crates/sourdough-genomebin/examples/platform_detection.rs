//! Example: Platform detection
//!
//! Demonstrates runtime platform discovery with zero hardcoding.

use sourdough_genomebin::Platform;

fn main() -> anyhow::Result<()> {
    println!("genomeBin Platform Detection Example\n");

    let platform = Platform::detect()?;

    println!("Detected Platform:");
    println!("  OS:              {}", platform.os());
    println!("  Architecture:    {}", platform.arch());
    println!("  LibC:            {}", platform.libc());
    println!("  Display:         {platform}");
    println!();

    println!("Target Triples:");
    println!("  Full:            {}", platform.target_triple());
    println!("  Simple:          {}", platform.simple_target());
    println!();

    println!("Fallback Targets (Universal Compatibility):");
    for (i, target) in platform.fallback_targets().iter().enumerate() {
        let rank = i + 1;
        println!("  {rank}: {target}");
    }
    println!();

    println!("Platform Checks:");
    println!("  Is Linux:        {}", platform.is_linux());
    println!("  Is macOS:        {}", platform.is_macos());
    println!("  Is musl:         {}", platform.is_musl());

    Ok(())
}
