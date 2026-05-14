//! Composition validation — verifies that all primals in a named
//! composition are present as binaries in the deployment directory.

use anyhow::Result;
use std::path::Path;

/// Predefined compositions matching `ports.env` atomic model.
const COMPOSITIONS: &[(&str, &[&str])] = &[
    // Atomic compositions
    ("tower", &["beardog", "songbird", "skunkbat"]),
    (
        "node",
        &[
            "beardog",
            "songbird",
            "skunkbat",
            "toadstool",
            "barracuda",
            "coralreef",
        ],
    ),
    (
        "nest",
        &[
            "beardog",
            "songbird",
            "skunkbat",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
        ],
    ),
    (
        "nucleus",
        &[
            "beardog",
            "songbird",
            "skunkbat",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
        ],
    ),
    ("meta", &["biomeos", "squirrel", "petaltongue"]),
    (
        "full",
        &[
            "beardog",
            "songbird",
            "skunkbat",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "biomeos",
            "squirrel",
            "petaltongue",
        ],
    ),
    // Per-spring niche compositions
    (
        "niche-hotspring",
        &[
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
        ],
    ),
    (
        "niche-neuralspring",
        &[
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "biomeos",
            "squirrel",
        ],
    ),
    (
        "niche-wetspring",
        &[
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "biomeos",
            "squirrel",
            "petaltongue",
        ],
    ),
    (
        "niche-groundspring",
        &[
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
        ],
    ),
    (
        "niche-healthspring",
        &[
            "beardog",
            "songbird",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "biomeos",
            "squirrel",
        ],
    ),
];

pub(crate) fn validate(composition: &str, primals_dir: &Path, triple_first: bool) -> Result<()> {
    let primals = resolve(composition);
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

fn resolve(name: &str) -> Vec<String> {
    for (comp_name, members) in COMPOSITIONS {
        if name.eq_ignore_ascii_case(comp_name) {
            return members.iter().map(|s| (*s).to_owned()).collect();
        }
    }

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

    #[test]
    fn resolve_tower() {
        let primals = resolve("tower");
        assert_eq!(primals, vec!["beardog", "songbird", "skunkbat"]);
    }

    #[test]
    fn resolve_case_insensitive() {
        let primals = resolve("TOWER");
        assert_eq!(primals, vec!["beardog", "songbird", "skunkbat"]);
    }

    #[test]
    fn resolve_nucleus_has_10_primals() {
        let primals = resolve("nucleus");
        assert_eq!(primals.len(), 10);
        assert!(primals.contains(&"beardog".to_owned()));
        assert!(primals.contains(&"sweetgrass".to_owned()));
    }

    #[test]
    fn resolve_full_has_13_primals() {
        let primals = resolve("full");
        assert_eq!(primals.len(), 13);
    }

    #[test]
    fn resolve_custom_csv() {
        let primals = resolve("beardog,songbird,toadstool");
        assert_eq!(primals, vec!["beardog", "songbird", "toadstool"]);
    }

    #[test]
    fn resolve_custom_csv_with_spaces() {
        let primals = resolve("beardog, songbird , toadstool");
        assert_eq!(primals, vec!["beardog", "songbird", "toadstool"]);
    }

    #[test]
    fn resolve_empty_returns_empty() {
        let primals = resolve("");
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
        let tower = resolve("tower");
        let nucleus = resolve("nucleus");
        for p in &tower {
            assert!(
                nucleus.contains(p),
                "tower member '{p}' missing from nucleus"
            );
        }
    }

    #[test]
    fn nucleus_is_subset_of_full() {
        let nucleus = resolve("nucleus");
        let full = resolve("full");
        for p in &nucleus {
            assert!(full.contains(p), "nucleus member '{p}' missing from full");
        }
    }

    #[test]
    fn niche_compositions_resolve() {
        let hotspring = resolve("niche-hotspring");
        assert_eq!(hotspring.len(), 9);
        assert!(hotspring.contains(&"toadstool".to_owned()));

        let neural = resolve("niche-neuralspring");
        assert_eq!(neural.len(), 7);
        assert!(neural.contains(&"squirrel".to_owned()));
    }

    #[test]
    fn niche_compositions_are_subsets_of_full() {
        let full = resolve("full");
        for (name, _) in super::COMPOSITIONS {
            if name.starts_with("niche-") {
                let niche = resolve(name);
                for p in &niche {
                    assert!(
                        full.contains(p),
                        "niche '{name}' member '{p}' missing from full"
                    );
                }
            }
        }
    }
}
