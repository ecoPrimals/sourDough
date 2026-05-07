# Changelog

All notable changes to sourDough will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] — v0.2.0 Scaffold Evolution

### Added (v0.2.0 — May 7, 2026)
- Scaffold generates `.github/workflows/release.yml` (Tier 1 musl cross-compilation: x86_64, aarch64, armv7 — SD-02 resolved)
- `sourdough-genomebin::signing` module: Ed25519 detached signatures for genomeBin artifacts (SD-03 resolved)
- Signing API: `generate_keypair`, `sign_file`, `verify_file`, `write_signature`, `read_signature`, `write_verifying_key`, `read_verifying_key`
- Pure Rust `ed25519-dalek` + `rand_core` dependencies (zero C deps, ecoBin-compliant)
- Release workflow: BLAKE3 checksums via `b3sum`, GitHub Release publishing via `softprops/action-gh-release@v2`
- 8 new signing tests + 5 new e2e assertions for release.yml

### Added (v0.2.0 — April 30, 2026)
- Scaffold generates `{name}-server` crate with JSON-RPC 2.0 server skeleton
- Scaffold generates `.github/workflows/ci.yml` (lean single-job CI)
- Scaffold generates `.github/workflows/notify-plasmidbin.yml` (genomeBin auto-distribution)
- Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- Scaffold generates `btsp.negotiate` handler (NULL cipher fallback for BTSP Phase 3 compatibility)
- `PeekedStream` transport utility in sourdough-core (ecosystem convergence for first-byte peek)
- `peek_protocol` async function for JSON-RPC vs BTSP auto-detection
- `resolve_socket_path` and `socket_path_in` for ecosystem socket naming convention
- Generated server: `dispatch.rs` with 4 capability wire handlers (health.liveness, health.readiness, health.check, capabilities.list)
- Generated server: `server.rs` with UDS listener, first-byte peek, newline-delimited JSON-RPC
- Generated server: `main.rs` with clap CLI (`--family-id` arg, `FAMILY_ID` env var)
- Enhanced e2e tests: 14 new assertions verifying deny.toml, CI workflows, server crate, dispatch handlers, socket naming

### Removed (v0.2.0)
- `tarpc` dependency (was only used for a proc macro annotation on `PrimalRpc` trait; generated code never consumed)
- `tokio-serde`, `bincode`, `tokio-util` dev-dependencies (unused, residual from tarpc exploration)
- 40 transitive dependencies eliminated (211 → 171 total)
- 3 advisory ignores from `deny.toml` (all were tarpc-transitive: RUSTSEC-2025-0141, RUSTSEC-2026-0007, RUSTSEC-2024-0387)

### Changed (v0.2.0)
- `PrimalRpc` trait: transport-agnostic async trait (was tarpc proc-macro annotated)
- `PrimalRpcClient::connect`: returns `std::io::Result` (was `Box<dyn Error>`)
- `bytes` updated to 1.11.1 (RUSTSEC-2026-0007 BytesMut overflow patched)
- Scaffold `ci.yml` now includes `cargo deny check` step (supply chain enforcement)
- Scaffold `deny.toml` now allows `cc` as wrapper for `blake3`/`iana-time-zone-haiku` (ecosystem standard)
- CONVENTIONS.md: JSON-RPC 2.0 is now documented as primary IPC (was incorrectly showing tarpc)
- Scaffolded core crate now includes `[lints] workspace = true` (was missing)
- Workspace Cargo.toml template adds `clap` to workspace dependencies
- Scaffolded workspace members now include both `-core` and `-server` crates
- README template updated with server crate structure and capability wire table
- `templates.rs` (862L) refactored into module directory: `core.rs` (440L), `server.rs` (319L), `infra.rs` (110L)
- Hardcoded primal names removed from CLI help and doc examples (rhizoCrypt, loamSpine → generic)
- `chrono` and `tempfile` deps aligned to `workspace = true` (was using local version pins)
- Broken `DEVELOPMENT.md` link in sourdough-genomebin README fixed → `CONVENTIONS.md`
- All root docs updated to 0.2.0-dev: README, STATUS, WHATS_NEXT, ROADMAP, ARCHITECTURE, START_HERE
- Binary artifacts (tar.gz) removed from `archive/` directory
- 247 tests passing (up from 239), zero files over 650 lines

### Added (prior)
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
