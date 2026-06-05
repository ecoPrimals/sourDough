//! Composition validation — verifies that all primals in a named
//! composition are present as binaries in the deployment directory.
//!
//! Composition definitions are loaded from TOML (data-driven, not hardcoded).
//! Resolution order:
//! 1. External manifest via `--manifest <path>` (if provided)
//! 2. `compositions.toml` alongside the primals directory
//! 3. Embedded default manifest (shipped with sourDough)

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Embedded default composition definitions.
const DEFAULT_MANIFEST: &str = include_str!("../../../../../compositions.toml");

/// Parsed composition manifest.
#[derive(serde::Deserialize)]
struct Manifest {
    compositions: HashMap<String, Composition>,
}

/// A single composition entry.
#[derive(serde::Deserialize)]
struct Composition {
    primals: Vec<String>,
}

/// Validate a named composition against a deployment directory.
pub(crate) fn validate(
    composition: &str,
    primals_dir: &Path,
    triple_first: bool,
    manifest_path: Option<&Path>,
) -> Result<()> {
    let manifest = load_manifest(primals_dir, manifest_path)?;
    let primals = resolve(composition, &manifest);
    let n = primals.len();
    crate::info(&format!(
        "Validating composition '{composition}' ({n} primals)"
    ));
    println!();

    let mut missing: Vec<String> = Vec::new();
    let mut found: Vec<String> = Vec::new();

    for primal in &primals {
        if binary_exists(primals_dir, primal, triple_first) {
            found.push(primal.clone());
            crate::success(primal);
        } else {
            missing.push(primal.clone());
            crate::error(&format!("{primal} — binary not found"));
        }
    }

    println!();
    crate::info(&format!("Result: {}/{n} primals present", found.len()));

    if missing.is_empty() {
        crate::success("Composition is complete");
        Ok(())
    } else {
        let m = missing.len();
        let list = missing.join(", ");
        anyhow::bail!("Composition incomplete: {m} missing ({list})");
    }
}

/// Load composition manifest from tiered sources.
fn load_manifest(primals_dir: &Path, explicit_path: Option<&Path>) -> Result<Manifest> {
    // Tier 1: explicit path
    if let Some(path) = explicit_path {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read manifest: {}", path.display()))?;
        return toml::from_str(&content)
            .with_context(|| format!("invalid manifest TOML: {}", path.display()));
    }

    // Tier 2: compositions.toml alongside primals directory
    let sibling = primals_dir.join("compositions.toml");
    if sibling.exists() {
        let content = std::fs::read_to_string(&sibling)
            .with_context(|| format!("failed to read {}", sibling.display()))?;
        return toml::from_str(&content)
            .with_context(|| format!("invalid TOML: {}", sibling.display()));
    }

    // Tier 3: parent directory
    if let Some(parent) = primals_dir.parent() {
        let parent_manifest = parent.join("compositions.toml");
        if parent_manifest.exists() {
            let content = std::fs::read_to_string(&parent_manifest)
                .with_context(|| format!("failed to read {}", parent_manifest.display()))?;
            return toml::from_str(&content)
                .with_context(|| format!("invalid TOML: {}", parent_manifest.display()));
        }
    }

    // Tier 4: embedded defaults
    toml::from_str(DEFAULT_MANIFEST).context("embedded compositions.toml is invalid")
}

/// Resolve a composition name to its list of primals.
fn resolve(name: &str, manifest: &Manifest) -> Vec<String> {
    let name_lower = name.to_lowercase();

    // Try named composition from manifest
    if let Some(comp) = manifest.compositions.get(&name_lower) {
        return comp.primals.clone();
    }

    // Fall back to CSV for ad-hoc composition
    name.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn binary_exists(primals_dir: &Path, primal: &str, triple_first: bool) -> bool {
    if primals_dir.join(primal).exists() {
        return true;
    }

    if triple_first {
        if let Ok(entries) = std::fs::read_dir(primals_dir) {
            for entry in entries.filter_map(std::result::Result::ok) {
                if entry.path().is_dir() && entry.path().join(primal).exists() {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manifest() -> Manifest {
        toml::from_str(DEFAULT_MANIFEST).unwrap()
    }

    #[test]
    fn resolve_tower() {
        let m = default_manifest();
        let primals = resolve("tower", &m);
        assert_eq!(primals, vec!["beardog", "songbird", "skunkbat"]);
    }

    #[test]
    fn resolve_case_insensitive() {
        let m = default_manifest();
        let primals = resolve("TOWER", &m);
        assert_eq!(primals, vec!["beardog", "songbird", "skunkbat"]);
    }

    #[test]
    fn resolve_nucleus_has_10_primals() {
        let m = default_manifest();
        let primals = resolve("nucleus", &m);
        assert_eq!(primals.len(), 10);
        assert!(primals.contains(&"beardog".to_owned()));
        assert!(primals.contains(&"sweetgrass".to_owned()));
    }

    #[test]
    fn resolve_full_has_13_primals() {
        let m = default_manifest();
        let primals = resolve("full", &m);
        assert_eq!(primals.len(), 13);
    }

    #[test]
    fn resolve_custom_csv() {
        let m = default_manifest();
        let primals = resolve("beardog,songbird,toadstool", &m);
        assert_eq!(primals, vec!["beardog", "songbird", "toadstool"]);
    }

    #[test]
    fn resolve_custom_csv_with_spaces() {
        let m = default_manifest();
        let primals = resolve("beardog, songbird , toadstool", &m);
        assert_eq!(primals, vec!["beardog", "songbird", "toadstool"]);
    }

    #[test]
    fn resolve_empty_returns_empty() {
        let m = default_manifest();
        let primals = resolve("", &m);
        assert!(primals.is_empty());
    }

    #[test]
    fn binary_exists_flat() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("beardog"), "binary").unwrap();
        assert!(binary_exists(dir.path(), "beardog", false));
        assert!(!binary_exists(dir.path(), "songbird", false));
    }

    #[test]
    fn binary_exists_triple_first() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("beardog"), "binary").unwrap();

        assert!(!binary_exists(dir.path(), "beardog", false));
        assert!(binary_exists(dir.path(), "beardog", true));
    }

    #[test]
    fn tower_is_subset_of_nucleus() {
        let m = default_manifest();
        let tower = resolve("tower", &m);
        let nucleus = resolve("nucleus", &m);
        for p in &tower {
            assert!(
                nucleus.contains(p),
                "tower member '{p}' missing from nucleus"
            );
        }
    }

    #[test]
    fn nucleus_is_subset_of_full() {
        let m = default_manifest();
        let nucleus = resolve("nucleus", &m);
        let full = resolve("full", &m);
        for p in &nucleus {
            assert!(full.contains(p), "nucleus member '{p}' missing from full");
        }
    }

    #[test]
    fn niche_compositions_resolve() {
        let m = default_manifest();
        let hotspring = resolve("niche-hotspring", &m);
        assert_eq!(hotspring.len(), 9);
        assert!(hotspring.contains(&"toadstool".to_owned()));

        let neural = resolve("niche-neuralspring", &m);
        assert_eq!(neural.len(), 7);
        assert!(neural.contains(&"squirrel".to_owned()));
    }

    #[test]
    fn niche_compositions_are_subsets_of_full() {
        let m = default_manifest();
        let full = resolve("full", &m);
        for name in m.compositions.keys() {
            if name.starts_with("niche-") {
                let niche = resolve(name, &m);
                for p in &niche {
                    assert!(
                        full.contains(p),
                        "niche '{name}' member '{p}' missing from full"
                    );
                }
            }
        }
    }

    #[test]
    fn embedded_manifest_parses() {
        let m: Manifest = toml::from_str(DEFAULT_MANIFEST).unwrap();
        assert!(m.compositions.contains_key("tower"));
        assert!(m.compositions.contains_key("full"));
    }

    #[test]
    fn external_manifest_overrides() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("compositions.toml"),
            r#"
[compositions.custom]
primals = ["alpha", "beta"]
"#,
        )
        .unwrap();

        let m = load_manifest(dir.path(), None).unwrap();
        let primals = resolve("custom", &m);
        assert_eq!(primals, vec!["alpha", "beta"]);
    }
}
