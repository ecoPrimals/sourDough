# sourDough Status

**Version**: 0.1.0 (unreleased)
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits library (PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig) + JSON-RPC 2.0 IPC + tarpc RPC
- `sourdough`: CLI binary (scaffold, validate, genomebin, doctor)
- `sourdough-genomebin`: Pure Rust genomeBin operations

## Compliance

- [x] `forbid(unsafe_code)` via workspace lints on all crates
- [x] `clippy::pedantic` + `clippy::nursery` zero warnings (workspace-level `[workspace.lints]`)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] `cargo fmt` clean
- [x] `cargo doc` zero warnings, all doctests compile (0 ignored)
- [x] Zero C application dependencies (Pure Rust, blake3 `pure` feature)
- [x] `cargo deny check` passing (ecoBin v3.0 C-sys ban list, supply chain audit)
- [x] Zero hardcoded primal names in crate code (Discovery grade A)
- [x] JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- [x] tarpc secondary high-throughput path with `bytes::Bytes` zero-copy
- [x] Edition 2024
- [x] scyBorg triple license (AGPL-3.0-or-later, ORC, CC-BY-SA-4.0)
- [x] 95%+ test coverage via `cargo llvm-cov` (237+ tests, target: 90%)
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [x] Release profile: LTO, codegen-units=1, strip
- [x] scaffold.rs refactored into module (mod + generators + templates)
- [x] doctor genomeBin tools: real implementation with platform detection
- [x] Parallel genomeBin processing implemented
- [x] E2E tests: scaffold -> build -> test -> validate lifecycle
- [x] WHATS_NEXT.md and START_HERE.md documentation
- [x] `deny.toml` supply chain auditing (SD-01 resolved)
- [x] `tar` crate updated to 0.4.45 (RUSTSEC-2026-0067, RUSTSEC-2026-0068 resolved)
- [ ] Cross-compilation validation (musl) — SD-02, stretch
- [ ] genomeBin signing (Pure Rust, sequoia-openpgp) — SD-03, stretch

## Crate Health

| Crate | Tests | Coverage | Max Lines |
|-------|-------|----------|-----------|
| sourdough-core | 128 | ~95% | all < 650 |
| sourdough (CLI) | 25+ (integration + e2e) | ~90% | all < 450 |
| sourdough-genomebin | 79 | ~96% | all < 560 |

## Recent Changes (April 3, 2026)

- Workspace-level lint configuration (`[workspace.lints]`) replaces per-crate `#![warn]`
- Release profile optimizations (LTO, codegen-units=1, strip)
- scaffold.rs refactored: mod.rs (command dispatch) + generators.rs (file writing) + templates.rs (primal DNA)
- All 3 ignored doctests fixed to compile (native async trait syntax, no async_trait dependency)
- Parallel genomeBin processing implemented (concurrent ecoBin pre-reading)
- doctor genomeBin tools: real implementation (platform detection, library validation)
- 5 new CLI integration tests (genomebin test/sign paths, doctor comprehensive output)
- 2 new e2e tests (scaffold -> build -> test -> validate lifecycle)
- WHATS_NEXT.md and START_HERE.md created per CONVENTIONS.md requirements
- server --port N/A documented (sourDough is meta-primal, not a daemon)
- sourdough-genomebin Cargo.toml migrated to workspace metadata
- Generated scaffold code updated to use workspace lints
