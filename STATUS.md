# sourDough Status

**Version**: 0.1.0 (unreleased)
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits library (PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig) + JSON-RPC 2.0 IPC + tarpc RPC
- `sourdough`: CLI binary (scaffold, validate, genomebin, doctor)
- `sourdough-genomebin`: Pure Rust genomeBin operations

## Compliance

- [x] `#![forbid(unsafe_code)]` on all crates
- [x] `clippy::pedantic` + `clippy::nursery` zero warnings (`-D warnings`)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] `cargo fmt` clean
- [x] `cargo doc` zero warnings
- [x] Zero C dependencies (Pure Rust)
- [x] JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- [x] tarpc secondary high-throughput path
- [x] Edition 2024
- [x] scyBorg triple license (AGPL-3.0-or-later, ORC, CC-BY-SA-4.0)
- [x] 94.40% test coverage via `cargo llvm-cov` (229 tests, target: 90%)
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [ ] Cross-compilation validation (musl)
- [ ] genomeBin signing (Pure Rust, sequoia-openpgp)

## Crate Health

| Crate | Tests | Coverage | Lines |
|-------|-------|----------|-------|
| sourdough-core | 121 | ~95% | all < 800 |
| sourdough (CLI) | 54 | ~90% | all < 800 |
| sourdough-genomebin | 54 | ~96% | all < 600 |

## Recent Changes (April 3, 2026)

- Scaffold independence: scaffolded primals receive inlined core traits (budding primal pattern)
- 54 new tests for sourdough-genomebin (validator, error, platform)
- 6 unused dependencies removed (ed25519-dalek, config, futures, walkdir, ignore, pathdiff)
- genomebin sign command: removed bash script fallback, pure Rust path forward
- genomebin/ bash scripts archived (replaced by sourdough-genomebin crate)
- DEVELOPMENT.md, ECOBIN_CERTIFICATION.md archived as fossil record
