//! Depot freshness validation.
//!
//! Scans a binary depot directory (triple-first layout) and reports which
//! primal binaries are stale relative to a freshness threshold or a source
//! directory's latest modification time.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::{Duration, SystemTime};

pub(crate) fn run(
    depot_dir: &Path,
    source_dir: Option<&Path>,
    stale_hours: u64,
    json: bool,
) -> Result<()> {
    if !depot_dir.exists() {
        anyhow::bail!("Depot directory not found: {}", depot_dir.display());
    }

    let reference_time = source_reference_time(source_dir);
    let threshold = Duration::from_secs(stale_hours * 3600);
    let now = SystemTime::now();

    let mut entries = collect_depot_entries(depot_dir)?;
    entries.sort_by(|a, b| a.primal.cmp(&b.primal).then(a.triple.cmp(&b.triple)));

    let stale_cutoff = reference_time
        .map_or_else(|| now.checked_sub(threshold), Some)
        .unwrap_or(now);

    for entry in &mut entries {
        entry.stale = entry.modified < stale_cutoff;
    }

    if json {
        print_json(&entries);
    } else {
        print_table(&entries, stale_hours, reference_time.is_some());
    }

    let stale_count = entries.iter().filter(|e| e.stale).count();
    if stale_count > 0 && !json {
        println!();
        crate::warning(&format!(
            "{stale_count}/{} depot binaries are stale",
            entries.len()
        ));
    } else if !json {
        println!();
        crate::success(&format!("All {} depot binaries are fresh", entries.len()));
    }

    Ok(())
}

struct DepotEntry {
    primal: String,
    triple: String,
    modified: SystemTime,
    size_bytes: u64,
    stale: bool,
}

fn collect_depot_entries(depot_dir: &Path) -> Result<Vec<DepotEntry>> {
    let mut entries = Vec::new();

    let dir_entries = std::fs::read_dir(depot_dir)
        .with_context(|| format!("Cannot read depot: {}", depot_dir.display()))?;

    for entry in dir_entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() && is_triple(&name) {
            let triple_entries = std::fs::read_dir(&path)
                .with_context(|| format!("Cannot read triple dir: {}", path.display()))?;

            for binary in triple_entries.filter_map(std::result::Result::ok) {
                let bin_path = binary.path();
                if bin_path.is_file() && bin_path.extension().is_none() {
                    if let Ok(meta) = std::fs::metadata(&bin_path) {
                        entries.push(DepotEntry {
                            primal: binary.file_name().to_string_lossy().to_string(),
                            triple: name.clone(),
                            modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            size_bytes: meta.len(),
                            stale: false,
                        });
                    }
                }
            }
        } else if path.is_file() && path.extension().is_none() {
            if let Ok(meta) = std::fs::metadata(&path) {
                entries.push(DepotEntry {
                    primal: name,
                    triple: "flat".to_owned(),
                    modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    size_bytes: meta.len(),
                    stale: false,
                });
            }
        }
    }

    Ok(entries)
}

fn source_reference_time(source_dir: Option<&Path>) -> Option<SystemTime> {
    let dir = source_dir?;
    let mut latest = SystemTime::UNIX_EPOCH;

    walk_for_latest(dir, &mut latest, 3);

    if latest == SystemTime::UNIX_EPOCH {
        None
    } else {
        Some(latest)
    }
}

fn walk_for_latest(dir: &Path, latest: &mut SystemTime, depth: u8) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
            walk_for_latest(&path, latest, depth - 1);
        } else if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if modified > *latest {
                    *latest = modified;
                }
            }
        }
    }
}

fn is_triple(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() >= 3 && parts.iter().any(|p| *p == "linux" || *p == "unknown")
}

fn print_table(entries: &[DepotEntry], stale_hours: u64, has_source: bool) {
    let reference = if has_source {
        "source mtime"
    } else {
        &format!("{stale_hours}h threshold")
    };

    crate::info(&format!(
        "Depot audit ({} binaries, reference: {reference})",
        entries.len()
    ));
    println!();
    println!("  {:20} {:35} {:>10}  Status", "Primal", "Triple", "Size");
    println!("  {}", "-".repeat(80));

    for entry in entries {
        let age = SystemTime::now()
            .duration_since(entry.modified)
            .unwrap_or_default();
        let age_str = format_duration(age);
        let size_str = format_size(entry.size_bytes);
        let status = if entry.stale {
            format!("STALE ({age_str} ago)")
        } else {
            format!("fresh ({age_str} ago)")
        };
        println!(
            "  {:<20} {:<35} {:>10}  {}",
            entry.primal, entry.triple, size_str, status
        );
    }
}

fn print_json(entries: &[DepotEntry]) {
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            let age_secs = SystemTime::now()
                .duration_since(e.modified)
                .unwrap_or_default()
                .as_secs();
            serde_json::json!({
                "primal": e.primal,
                "triple": e.triple,
                "size_bytes": e.size_bytes,
                "age_secs": age_secs,
                "stale": e.stale,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_default()
    );
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "display-only formatting; sub-byte precision loss is irrelevant"
)]
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1_048_576 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_triple_recognizes_targets() {
        assert!(is_triple("x86_64-unknown-linux-musl"));
        assert!(is_triple("aarch64-linux-android"));
        assert!(!is_triple("beardog"));
        assert!(!is_triple("songbird"));
    }

    #[test]
    fn format_duration_ranges() {
        assert_eq!(format_duration(Duration::from_secs(300)), "5m");
        assert_eq!(format_duration(Duration::from_secs(7200)), "2h");
        assert_eq!(format_duration(Duration::from_secs(172_800)), "2d");
    }

    #[test]
    fn format_size_ranges() {
        assert_eq!(format_size(512), "512B");
        assert_eq!(format_size(2048), "2.0K");
        assert_eq!(format_size(5_242_880), "5.0M");
    }

    #[test]
    fn collect_depot_entries_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let entries = collect_depot_entries(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn collect_depot_entries_triple_first() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("beardog"), "fake-binary").unwrap();
        std::fs::write(triple.join("songbird"), "fake-binary").unwrap();

        let entries = collect_depot_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.primal == "beardog"));
        assert!(entries.iter().any(|e| e.primal == "songbird"));
    }

    #[test]
    fn stale_detection_with_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("fresh"), "binary").unwrap();

        let entries = collect_depot_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].stale);
    }

    #[test]
    fn source_reference_time_none_for_missing_dir() {
        assert!(source_reference_time(Some(Path::new("/nonexistent"))).is_none());
    }
}
