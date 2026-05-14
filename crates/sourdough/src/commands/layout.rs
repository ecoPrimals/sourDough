//! Binary layout validation and enforcement.
//!
//! Validates or reorganizes primal binary trees to follow the
//! ecosystem triple-first convention: `primals/<triple>/<name>`.

use anyhow::{Context, Result};
use std::path::Path;

/// Known Tier 1 musl triples for genomeBin distribution.
const TIER1_TRIPLES: &[&str] = &[
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "armv7-unknown-linux-musleabihf",
];

/// Validate that a directory uses triple-first layout.
///
/// Triple-first: `primals/<triple>/<name>` instead of flat `primals/<name>`.
pub(crate) fn validate(dir: &Path) -> Result<()> {
    crate::info(&format!(
        "Validating triple-first layout: {}",
        dir.display()
    ));

    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let mut errors: Vec<String> = Vec::new();
    let mut triple_dirs: Vec<String> = Vec::new();
    let mut flat_binaries: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read directory: {}", dir.display()))?;

    for entry in entries.filter_map(std::result::Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if path.is_dir() && is_triple(&name) {
            triple_dirs.push(name.clone());
            check_triple_dir(&path, &name, &mut errors);
        } else if path.is_file() && is_likely_binary(&path) {
            flat_binaries.push(name);
        }
    }

    println!();

    if !triple_dirs.is_empty() {
        crate::success(&format!(
            "Found {} triple directories: {}",
            triple_dirs.len(),
            triple_dirs.join(", ")
        ));
    }

    if !flat_binaries.is_empty() {
        for bin in &flat_binaries {
            errors.push(format!(
                "Flat binary '{bin}' — should be under a triple directory"
            ));
        }
    }

    if triple_dirs.is_empty() && flat_binaries.is_empty() {
        crate::warning("No binaries or triple directories found");
    }

    for triple in TIER1_TRIPLES {
        if !triple_dirs.iter().any(|t| t == triple) {
            crate::warning(&format!("Missing Tier 1 triple: {triple}"));
        }
    }

    println!();
    if errors.is_empty() {
        crate::success("Layout validation passed");
        Ok(())
    } else {
        for e in &errors {
            crate::error(e);
        }
        let n = errors.len();
        anyhow::bail!("Layout validation failed with {n} error(s)");
    }
}

fn is_triple(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() >= 3 && parts.iter().any(|p| *p == "linux" || *p == "unknown")
}

fn is_likely_binary(path: &Path) -> bool {
    if path.extension().is_some() {
        return false;
    }
    std::fs::metadata(path)
        .map(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.permissions().mode() & 0o111 != 0
        })
        .unwrap_or(false)
}

fn check_triple_dir(path: &Path, triple: &str, errors: &mut Vec<String>) {
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let count = entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().is_file())
                .count();
            if count == 0 {
                errors.push(format!("Triple directory '{triple}' is empty"));
            }
        }
        Err(e) => errors.push(format!("Cannot read triple directory '{triple}': {e}")),
    }
}
