//! G72 Dependency Pandemic validator.
//!
//! Scans workspace `Cargo.toml` files for dependency health issues:
//! - `tokio` with `features = ["full"]` (should be per-crate minimal)
//! - Known-excisable crates (pollster, reqwest when songBird available, etc.)
//! - Version misalignment across workspace members
//! - Bloated feature sets (more than N features on heavy crates)
//! - Duplicate functionality (multiple HTTP clients, multiple async runtimes)

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Crates known to be excisable in the ecoPrimals ecosystem.
const EXCISABLE_CRATES: &[(&str, &str)] = &[
    ("pollster", "use tokio::runtime or block_in_place instead"),
    ("reqwest", "use capability.call via biomeOS or songBird mesh relay"),
    ("hyper", "use capability.call via biomeOS or songBird mesh relay"),
    ("actix-web", "use JSON-RPC IPC — primals don't serve HTTP"),
    ("warp", "use JSON-RPC IPC — primals don't serve HTTP"),
    ("rocket", "use JSON-RPC IPC — primals don't serve HTTP"),
];

/// Feature sets that indicate over-inclusion.
const BLOATED_FEATURES: &[(&str, &str, &str)] = &[
    ("tokio", "full", "specify only needed features (rt, net, io-util, etc.)"),
    ("serde", "default", "use derive only — skip default if not needed"),
];

/// Crates that indicate duplicate functionality when both present.
const DUPLICATE_PAIRS: &[(&str, &str, &str)] = &[
    ("chrono", "time", "consolidate on `time` crate (lighter, no C deps)"),
    ("reqwest", "ureq", "pick one HTTP client or eliminate both via mesh"),
    ("log", "tracing", "consolidate on `tracing` (superset of log)"),
];

#[derive(Debug)]
struct Finding {
    file: PathBuf,
    severity: Severity,
    category: &'static str,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum Severity {
    Error,
    Warning,
    Info,
}

#[expect(clippy::unnecessary_wraps, reason = "CLI dispatch requires Result")]
pub(super) fn validate(path: &Path, json: bool) -> Result<()> {
    let cargo_files = collect_cargo_tomls(path);
    if cargo_files.is_empty() {
        if json {
            println!("{{\"compliance\": \"NO_WORKSPACE\", \"findings\": []}}");
        } else {
            crate::warning("No Cargo.toml files found");
        }
        return Ok(());
    }

    let mut findings = Vec::new();
    let mut workspace_deps: HashMap<String, Vec<(PathBuf, String)>> = HashMap::new();

    for cargo_path in &cargo_files {
        let Ok(content) = std::fs::read_to_string(cargo_path) else {
            continue;
        };

        check_bloated_features(cargo_path, &content, &mut findings);
        check_excisable_crates(cargo_path, &content, &mut findings);
        collect_dep_versions(cargo_path, &content, &mut workspace_deps);
    }

    check_duplicates(&cargo_files, path, &mut findings);
    check_version_misalignment(&workspace_deps, &mut findings);

    let error_count = findings.iter().filter(|f| matches!(f.severity, Severity::Error)).count();
    let warn_count = findings.iter().filter(|f| matches!(f.severity, Severity::Warning)).count();

    if json {
        print_json(&findings, path, error_count, warn_count);
    } else {
        print_human(&findings, path, error_count, warn_count);
    }

    Ok(())
}

fn collect_cargo_tomls(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_cargo_recursive(path, &mut files, 0);
    files
}

fn collect_cargo_recursive(dir: &Path, files: &mut Vec<PathBuf>, depth: u8) {
    if depth > 5 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str == ".git" || name_str == "node_modules" {
                continue;
            }
            collect_cargo_recursive(&path, files, depth + 1);
        } else if path.file_name().is_some_and(|n| n == "Cargo.toml") {
            files.push(path);
        }
    }
}

fn check_bloated_features(cargo_path: &Path, content: &str, findings: &mut Vec<Finding>) {
    for &(crate_name, feature, suggestion) in BLOATED_FEATURES {
        let patterns = [
            format!("features = [\"{feature}\""),
            format!("features = [\"{feature}\"]"),
            format!("\"{feature}\""),
        ];

        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('#')
                && contains_dep_context(trimmed, crate_name)
                && patterns.iter().any(|p| trimmed.contains(p.as_str()))
            {
                findings.push(Finding {
                    file: cargo_path.to_path_buf(),
                    severity: Severity::Error,
                    category: "bloated-features",
                    message: format!("`{crate_name}` uses `{feature}` — {suggestion}"),
                });
                break;
            }
        }

        // Also check multi-line dep declarations
        if content.contains(&format!("[dependencies.{crate_name}]"))
            || content.contains(&format!("[dev-dependencies.{crate_name}]"))
        {
            let section_marker = format!("[dependencies.{crate_name}]");
            let dev_marker = format!("[dev-dependencies.{crate_name}]");
            let in_section = content.contains(&section_marker) || content.contains(&dev_marker);
            if in_section && content.contains(&format!("\"{feature}\"")) {
                let already_found = findings.iter().any(|f| {
                    f.file == cargo_path
                        && f.message.contains(crate_name)
                        && f.message.contains(feature)
                });
                if !already_found {
                    findings.push(Finding {
                        file: cargo_path.to_path_buf(),
                        severity: Severity::Error,
                        category: "bloated-features",
                        message: format!("`{crate_name}` uses `{feature}` — {suggestion}"),
                    });
                }
            }
        }
    }
}

fn contains_dep_context(line: &str, crate_name: &str) -> bool {
    line.contains(crate_name)
}

fn check_excisable_crates(cargo_path: &Path, content: &str, findings: &mut Vec<Finding>) {
    for &(crate_name, reason) in EXCISABLE_CRATES {
        let dep_patterns = [
            format!("{crate_name} = "),
            format!("{crate_name} = \""),
            format!("{crate_name} = {{"),
            format!("[dependencies.{crate_name}]"),
            format!("[dev-dependencies.{crate_name}]"),
        ];

        let has_dep = content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && dep_patterns.iter().any(|p| trimmed.starts_with(p.as_str()))
        }) || content.contains(&format!("[dependencies.{crate_name}]"))
            || content.contains(&format!("[dev-dependencies.{crate_name}]"));

        if has_dep {
            // Skip if it's a workspace dependency declaration (in root Cargo.toml)
            if content.contains("[workspace.dependencies]")
                && !content.contains("[dependencies]")
            {
                findings.push(Finding {
                    file: cargo_path.to_path_buf(),
                    severity: Severity::Warning,
                    category: "excisable-dep",
                    message: format!(
                        "`{crate_name}` in workspace deps — {reason}"
                    ),
                });
            } else {
                findings.push(Finding {
                    file: cargo_path.to_path_buf(),
                    severity: Severity::Warning,
                    category: "excisable-dep",
                    message: format!("`{crate_name}` detected — {reason}"),
                });
            }
        }
    }
}

fn collect_dep_versions(
    cargo_path: &Path,
    content: &str,
    workspace_deps: &mut HashMap<String, Vec<(PathBuf, String)>>,
) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Match: `crate_name = "version"` or `crate_name = { version = "x.y" ... }`
        if let Some((name, version)) = extract_dep_version(trimmed) {
            workspace_deps
                .entry(name)
                .or_default()
                .push((cargo_path.to_path_buf(), version));
        }
    }
}

fn extract_dep_version(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim().to_owned();
    if name.starts_with('[') || name.contains('.') || name.is_empty() {
        return None;
    }

    let value = parts[1].trim();

    // Simple: `name = "1.2.3"`
    if value.starts_with('"') {
        let version = value.trim_matches('"').to_owned();
        if version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Some((name, version));
        }
    }

    // Inline table: `name = { version = "1.2.3", ... }`
    if value.starts_with('{') {
        if let Some(ver_start) = value.find("version") {
            let after = &value[ver_start..];
            if let Some(quote_start) = after.find('"') {
                let rest = &after[quote_start + 1..];
                if let Some(quote_end) = rest.find('"') {
                    let version = rest[..quote_end].to_owned();
                    if version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                        return Some((name, version));
                    }
                }
            }
        }
    }

    None
}

fn check_duplicates(cargo_files: &[PathBuf], base_path: &Path, findings: &mut Vec<Finding>) {
    // Aggregate all deps across the workspace
    let mut all_deps: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for cargo_path in cargo_files {
        let Ok(content) = std::fs::read_to_string(cargo_path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with('[') {
                continue;
            }
            if let Some((name, _)) = extract_dep_version(trimmed) {
                all_deps.entry(name).or_default().push(cargo_path.clone());
            }
        }
    }

    for &(crate_a, crate_b, suggestion) in DUPLICATE_PAIRS {
        if all_deps.contains_key(crate_a) && all_deps.contains_key(crate_b) {
            findings.push(Finding {
                file: base_path.join("Cargo.toml"),
                severity: Severity::Info,
                category: "duplicate-functionality",
                message: format!(
                    "both `{crate_a}` and `{crate_b}` present in workspace — {suggestion}"
                ),
            });
        }
    }
}

fn check_version_misalignment(
    workspace_deps: &HashMap<String, Vec<(PathBuf, String)>>,
    findings: &mut Vec<Finding>,
) {
    for (dep_name, versions) in workspace_deps {
        if versions.len() < 2 {
            continue;
        }

        let unique_versions: Vec<&String> = {
            let mut v: Vec<&String> = versions.iter().map(|(_, ver)| ver).collect();
            v.sort();
            v.dedup();
            v
        };

        if unique_versions.len() > 1 {
            let version_list = unique_versions
                .iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(Finding {
                file: versions[0].0.clone(),
                severity: Severity::Warning,
                category: "version-misalignment",
                message: format!(
                    "`{dep_name}` has {n} different versions: {version_list}",
                    n = unique_versions.len()
                ),
            });
        }
    }
}

const fn compliance_level(errors: usize, warnings: usize) -> &'static str {
    match (errors, warnings) {
        (0, 0) => "G72",
        (0, _) => "G72-prod",
        _ => "partial",
    }
}

fn print_json(findings: &[Finding], _base_path: &Path, errors: usize, warnings: usize) {
    let level = compliance_level(errors, warnings);
    println!("{{");
    println!("  \"compliance\": \"{level}\",");
    println!("  \"errors\": {errors},");
    println!("  \"warnings\": {warnings},");
    println!("  \"findings\": [");
    for (i, f) in findings.iter().enumerate() {
        let comma = if i + 1 < findings.len() { "," } else { "" };
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let file = f.file.display();
        let msg = f.message.replace('"', "\\\"");
        println!(
            "    {{\"severity\": \"{sev}\", \"category\": \"{cat}\", \"file\": \"{file}\", \"message\": \"{msg}\"}}{}",
            comma,
            cat = f.category,
        );
    }
    println!("  ]");
    println!("}}");
}

fn print_human(findings: &[Finding], base_path: &Path, errors: usize, warnings: usize) {
    let level = compliance_level(errors, warnings);
    crate::info(&format!(
        "G72 Dependency Pandemic audit: {}",
        base_path.display()
    ));
    println!();

    if findings.is_empty() {
        crate::success("No dependency issues found — G72 compliant");
        return;
    }

    // Group by category
    let mut by_category: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_category.entry(f.category).or_default().push(f);
    }

    for (category, items) in &by_category {
        println!("  {category}:");
        for f in items {
            let prefix = match f.severity {
                Severity::Error => "✗",
                Severity::Warning => "⚠",
                Severity::Info => "ℹ",
            };
            let rel_path = f.file.strip_prefix(base_path).unwrap_or(&f.file);
            println!("    {prefix} {}: {}", rel_path.display(), f.message);
        }
        println!();
    }

    println!("  Compliance: {level} ({errors} errors, {warnings} warnings)");
}
