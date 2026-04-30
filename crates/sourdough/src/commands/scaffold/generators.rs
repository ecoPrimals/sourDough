//! File generators for scaffolded primals and crates.
//!
//! Each generator writes one or more files into a scaffolded primal's tree.
//! They use templates from [`super::templates`] for inlined primal DNA.

use super::templates;
use anyhow::{Context, Result};
use std::path::Path;

pub(super) fn write_workspace_cargo_toml(dir: &Path, name: &str) -> Result<()> {
    let name_lower = name.to_lowercase();
    let core_crate_name = format!("{name_lower}-core");
    let server_crate_name = format!("{name_lower}-server");
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            r#"[workspace]
resolver = "2"
members = [
    "crates/{core_crate_name}",
    "crates/{server_crate_name}",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "AGPL-3.0-or-later"
repository = "https://github.com/ecoPrimals/{name}"
authors = ["ecoPrimals Project"]

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
rust_2018_idioms = {{ level = "warn", priority = -1 }}
unreachable_pub = "warn"

[workspace.lints.clippy]
all = {{ level = "warn", priority = -1 }}
pedantic = {{ level = "warn", priority = -1 }}
nursery = {{ level = "warn", priority = -1 }}

[profile.release]
lto = true
codegen-units = 1
strip = true

[workspace.dependencies]
# Async runtime
tokio = {{ version = "1.40", features = ["macros", "rt-multi-thread", "signal", "net", "io-util", "time"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
toml = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# CLI
clap = {{ version = "4.5", features = ["derive", "env"] }}

# Logging
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"#,
        ),
    )?;
    Ok(())
}

pub(super) fn create_server_crate(crates_dir: &Path, name: &str) -> Result<()> {
    let name_lower = name.to_lowercase();
    let server_crate_name = format!("{name_lower}-server");
    let core_crate_name = format!("{name_lower}-core");
    let server_dir = crates_dir.join(&server_crate_name);
    let src_dir = server_dir.join("src");

    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        server_dir.join("Cargo.toml"),
        templates::server_cargo_toml(&server_crate_name, &core_crate_name, name),
    )?;
    std::fs::write(src_dir.join("main.rs"), templates::server_main_rs(name))?;
    std::fs::write(src_dir.join("server.rs"), templates::server_rs(name))?;
    std::fs::write(src_dir.join("dispatch.rs"), templates::dispatch_rs(name))?;

    Ok(())
}

pub(super) fn write_deny_toml(dir: &Path) -> Result<()> {
    std::fs::write(dir.join("deny.toml"), templates::DENY_TOML)?;
    Ok(())
}

pub(super) fn write_github_workflows(dir: &Path, name: &str) -> Result<()> {
    let workflows_dir = dir.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir)?;

    std::fs::write(workflows_dir.join("ci.yml"), templates::ci_yml(name))?;
    std::fs::write(
        workflows_dir.join("notify-plasmidbin.yml"),
        templates::NOTIFY_PLASMIDBIN_YML,
    )?;

    Ok(())
}

pub(super) fn create_core_crate(crates_dir: &Path, name: &str) -> Result<()> {
    let core_crate_name = format!("{}-core", name.to_lowercase());
    let core_dir = crates_dir.join(&core_crate_name);
    let src_dir = core_dir.join("src");

    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        core_dir.join("Cargo.toml"),
        templates::core_cargo_toml(&core_crate_name, name),
    )?;
    std::fs::write(src_dir.join("error.rs"), templates::ERROR_RS)?;
    std::fs::write(src_dir.join("lifecycle.rs"), templates::LIFECYCLE_RS)?;
    std::fs::write(src_dir.join("health.rs"), templates::HEALTH_RS)?;
    std::fs::write(src_dir.join("lib.rs"), templates::lib_rs(name))?;

    Ok(())
}

pub(super) fn write_specs_directory(dir: &Path, name: &str, description: &str) -> Result<()> {
    let specs_dir = dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    let date = chrono::Local::now().format("%B %d, %Y");
    std::fs::write(
        specs_dir.join(format!("{}_SPECIFICATION.md", name.to_uppercase())),
        format!(
            r"# {name} - Specification

**Version**: 0.1.0
**Date**: {date}
**Status**: Draft

---

## Purpose

{description}

## Architecture

(To be defined)

## Components

(To be defined)

---

**Date**: {date}
**Version**: 0.1.0
",
        ),
    )?;

    Ok(())
}

pub(super) fn write_readme(dir: &Path, name: &str, description: &str) -> Result<()> {
    let name_lower = name.to_lowercase();
    std::fs::write(
        dir.join("README.md"),
        format!(
            r"# {name}

**Status**: Draft
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)
**Purpose**: {description}

---

## Quick Start

```bash
cargo build --release
cargo test --workspace
./target/release/{name_lower}  # starts JSON-RPC server on UDS
```

## Structure

```
{name}/
├── .github/workflows/     CI + plasmidBin notification
├── crates/
│   ├── {name_lower}-core/        Core traits (lifecycle, health)
│   └── {name_lower}-server/      JSON-RPC server + capability wire
├── deny.toml              Supply chain auditing
└── specs/
```

## Capability Wire

The server exposes these JSON-RPC 2.0 methods on `$XDG_RUNTIME_DIR/biomeos/{name_lower}.sock`:

| Method | Description |
|--------|-------------|
| `health.liveness` | Process liveness check |
| `health.readiness` | Readiness + capabilities |
| `health.check` | Full diagnostic report |
| `capabilities.list` | Primal name, version, methods |

---

*Scaffolded by sourDough — the nascent primal.*
",
        ),
    )?;

    Ok(())
}

pub(super) fn write_conventions(dir: &Path) -> Result<()> {
    std::fs::write(
        dir.join("CONVENTIONS.md"),
        r"# Coding Conventions

This primal follows the ecoPrimals coding conventions.

## Quick Reference

- **Edition**: 2024
- **License**: AGPL-3.0-or-later (scyBorg triple license)
- **Linting**: Workspace-level `clippy::pedantic` + `clippy::nursery`, `forbid(unsafe_code)`
- **Docs**: `warn(missing_docs)`
- **Max file size**: 1000 LOC
- **Test coverage**: 90%+
- **IPC**: JSON-RPC 2.0 (`domain.verb` naming)
- **Identity**: Self-sovereign DIDs
- **Discovery**: Runtime via universal adapter (zero hardcoding)

*Consistency is the foundation of collaboration.*
",
    )?;

    Ok(())
}

pub(super) fn update_workspace_members(primal_dir: &Path, crate_name: &str) -> Result<()> {
    let cargo_path = primal_dir.join("Cargo.toml");
    let raw = std::fs::read_to_string(&cargo_path)
        .with_context(|| format!("failed to read {}", cargo_path.display()))?;
    let mut root: toml::Value =
        toml::from_str(&raw).context("workspace Cargo.toml is not valid TOML")?;
    let workspace = root
        .as_table_mut()
        .and_then(|t| t.get_mut("workspace"))
        .and_then(toml::Value::as_table_mut)
        .context("missing [workspace] table in Cargo.toml")?;
    let members = workspace
        .entry("members")
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    let arr = members
        .as_array_mut()
        .context("[workspace].members must be an array")?;
    let entry = format!("crates/{crate_name}");
    if arr.iter().any(|v| v.as_str() == Some(entry.as_str())) {
        crate::info(&format!(
            "Crate '{crate_name}' already listed in workspace members"
        ));
        return Ok(());
    }
    arr.push(toml::Value::String(entry));
    let updated = toml::to_string_pretty(&root).context("failed to serialize Cargo.toml")?;
    std::fs::write(&cargo_path, updated)
        .with_context(|| format!("failed to write {}", cargo_path.display()))?;
    crate::success(&format!(
        "Added 'crates/{crate_name}' to [workspace].members"
    ));
    Ok(())
}
