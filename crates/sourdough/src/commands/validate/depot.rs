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
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(meta) = entry.metadata() else { continue };

        if meta.is_dir() && is_triple(&name) {
            let Ok(triple_entries) = std::fs::read_dir(entry.path()) else {
                continue;
            };

            for binary in triple_entries.filter_map(std::result::Result::ok) {
                let bin_name = binary.file_name().to_string_lossy().to_string();
                if bin_name.starts_with('.') {
                    continue;
                }
                let Ok(bin_meta) = binary.metadata() else {
                    continue;
                };
                let bin_path = binary.path();
                if bin_meta.is_file() && bin_path.extension().is_none() {
                    entries.push(DepotEntry {
                        primal: bin_name,
                        triple: name.clone(),
                        modified: bin_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        size_bytes: bin_meta.len(),
                        stale: false,
                    });
                }
            }
        } else if meta.is_file() && entry.path().extension().is_none() && !name.starts_with('.') {
            entries.push(DepotEntry {
                primal: name,
                triple: "flat".to_owned(),
                modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                size_bytes: meta.len(),
                stale: false,
            });
        }
    }

    Ok(entries)
}

fn source_reference_time(source_dir: Option<&Path>) -> Option<SystemTime> {
    let dir = source_dir?;
    let mut latest = SystemTime::UNIX_EPOCH;
    let max_depth: usize = 3;

    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];

    while let Some((current, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.filter_map(std::result::Result::ok) {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() && depth < max_depth {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                stack.push((entry.path(), depth + 1));
            } else if meta.is_file() {
                if let Ok(modified) = meta.modified() {
                    if modified > latest {
                        latest = modified;
                    }
                }
            }
        }
    }

    if latest == SystemTime::UNIX_EPOCH {
        None
    } else {
        Some(latest)
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

    #[test]
    fn source_reference_time_finds_latest_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "content").unwrap();
        std::thread::sleep(Duration::from_millis(10));
        std::fs::write(dir.path().join("b.rs"), "newer").unwrap();
        let t = source_reference_time(Some(dir.path()));
        assert!(t.is_some());
    }

    #[test]
    fn source_reference_time_none_for_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(source_reference_time(Some(dir.path())).is_none());
    }

    #[test]
    fn run_nonexistent_depot_fails() {
        let result = run(Path::new("/nonexistent/depot"), None, 48, false);
        assert!(result.is_err());
    }

    #[test]
    fn run_empty_depot_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let result = run(dir.path(), None, 48, false);
        assert!(result.is_ok());
    }

    #[test]
    fn run_json_output_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("beardog"), "binary").unwrap();
        let result = run(dir.path(), None, 48, true);
        assert!(result.is_ok());
    }

    #[test]
    fn run_with_source_dir() {
        let depot = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let triple = depot.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("beardog"), "binary").unwrap();
        std::fs::write(source.path().join("main.rs"), "fn main() {}").unwrap();
        let result = run(depot.path(), Some(source.path()), 48, false);
        assert!(result.is_ok());
    }

    #[test]
    fn flat_layout_detected() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("beardog"), "binary").unwrap();
        let entries = collect_depot_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].triple, "flat");
    }

    #[test]
    fn hidden_files_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join(".hidden"), "binary").unwrap();
        std::fs::write(triple.join("visible"), "binary").unwrap();
        let entries = collect_depot_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].primal, "visible");
    }

    #[test]
    fn files_with_extensions_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let triple = dir.path().join("x86_64-unknown-linux-musl");
        std::fs::create_dir_all(&triple).unwrap();
        std::fs::write(triple.join("beardog.txt"), "not a binary").unwrap();
        std::fs::write(triple.join("beardog"), "binary").unwrap();
        let entries = collect_depot_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn print_json_produces_valid_output() {
        let entries = vec![DepotEntry {
            primal: "test".to_owned(),
            triple: "x86_64-unknown-linux-musl".to_owned(),
            modified: SystemTime::now(),
            size_bytes: 1024,
            stale: false,
        }];
        print_json(&entries);
    }
}
