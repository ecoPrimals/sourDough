# Changelog

All notable changes to sourDough will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Scaffold independence: scaffolded primals are fully self-contained with inlined core traits
- 54 new tests for sourdough-genomebin (validator, error, platform coverage)
- `EPHEMERAL_PRIMAL_SCAFFOLDING.md` spec for session-as-primal pattern
- JSON-RPC 2.0 primary IPC implementation with semantic `domain.verb` method naming
- `#![forbid(unsafe_code)]` on all crate roots
- `clippy::pedantic` + `clippy::nursery` enforced workspace-wide
- `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml` configuration files
- LICENSE file with scyBorg triple license
- This CHANGELOG

### Changed
- Upgraded Rust edition from 2021 to 2024
- `sourdough-genomebin` now uses workspace dependencies
- Replaced all `#[allow()]` with `#[expect(reason)]` per ecosystem standard
- Scaffold `new-primal` generates self-contained primals without sourdough-core dependency
- Scaffold `new-crate` uses path dependency to primal's own core crate
- Generated workspace Cargo.toml uses granular tokio features instead of `"full"`
- genomebin sign command now returns explicit error guiding toward pure Rust sequoia-openpgp

### Fixed
- 4 clippy `missing_const_for_fn` errors in sourdough-genomebin
- Orphaned `tests/e2e/rpc_communication.rs` now compiles and runs
- Deprecated `assert_cmd::Command::cargo_bin` usage
- Clippy `needless_raw_string_hashes` in scaffold templates
- Clippy `write_with_newline` in genomebin validator tests

### Removed
- 6 unused dependencies: `ed25519-dalek`, `config` (crate), `futures`, `walkdir`, `ignore`, `pathdiff`
- Unused `sysinfo` dependency from sourdough-genomebin
- `genomebin/` bash scripts directory (archived; replaced by Pure Rust sourdough-genomebin)
- `DEVELOPMENT.md` and `ECOBIN_CERTIFICATION.md` (archived as fossil record)
- `find_sourdough_core_path` function from scaffold (no longer needed)
- `find_genomebin_script` function from genomebin command

## [0.1.0] - 2026-01-19

### Added
- Initial `sourdough-core` library with traits: `PrimalLifecycle`, `PrimalHealth`, `PrimalIdentity`, `PrimalDiscovery`, `PrimalConfig`
- `sourdough` CLI binary with scaffold, validate, genomebin, doctor commands
- `sourdough-genomebin` library for Pure Rust genomeBin operations
- Comprehensive specs: SOURDOUGH_SPECIFICATION.md, ARCHITECTURE.md, ROADMAP.md
- 151 passing tests across all crates
