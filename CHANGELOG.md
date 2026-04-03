# Changelog

All notable changes to sourDough will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `deny.toml` supply chain auditing with ecoBin v3.0 C-sys ban list (16 crates)
- Workspace-level lint configuration (`[workspace.lints]`): pedantic, nursery, forbid(unsafe_code)
- Release profile optimizations: LTO, codegen-units=1, strip
- E2E tests: full scaffold -> build -> test -> validate lifecycle (2 tests)
- 5 new CLI integration tests for genomebin test/sign paths and doctor comprehensive
- Doctor genomeBin tools: real implementation with platform detection
- Parallel genomeBin processing via `tokio::task::JoinSet`
- `WHATS_NEXT.md` and `START_HERE.md` per CONVENTIONS.md requirements
- server --port N/A documented in specification (sourDough is meta-primal)

### Changed
- blake3 dependency uses `pure` feature (no C/asm build dependency)
- `tar` crate updated to 0.4.45 (fixes RUSTSEC-2026-0067, RUSTSEC-2026-0068)
- Removed cosmetic "BearDog" primal name from genomebin sign error message (Discovery A)
- Scaffold command refactored: `scaffold.rs` (789 lines) -> `scaffold/{mod,generators,templates}.rs` (max 438)
- All 3 ignored doctests rewritten to compile (native async trait syntax, edition 2024)
- `sourdough-genomebin` Cargo.toml migrated to workspace metadata
- Generated scaffold code emits workspace lints and release profile
- ARCHITECTURE.md updated with accurate file map and line counts (29 files, ~8100 lines)
- CONVENTIONS.md updated to reflect workspace-level linting
- README.md quality table updated (239 tests, 96%+ coverage, max file 637 lines)

### Removed
- Per-crate `#![forbid(unsafe_code)]` and `#![warn(clippy::...)]` (replaced by workspace lints)
- Dead code `#[expect(dead_code)]` on `parallel` field in builder.rs (now implemented)
- Monolithic `scaffold.rs` (replaced by module directory)

---

### Prior Session (April 3, 2026 — scaffold independence)

### Added
- Scaffold independence: scaffolded primals are fully self-contained with inlined core traits
- 54 new tests for sourdough-genomebin (validator, error, platform coverage)
- `EPHEMERAL_PRIMAL_SCAFFOLDING.md` spec for session-as-primal pattern
- JSON-RPC 2.0 primary IPC implementation with semantic `domain.verb` method naming
- `#![forbid(unsafe_code)]` on all crate roots
- `clippy::pedantic` + `clippy::nursery` enforced workspace-wide
- `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml` configuration files
- LICENSE file with scyBorg triple license

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
