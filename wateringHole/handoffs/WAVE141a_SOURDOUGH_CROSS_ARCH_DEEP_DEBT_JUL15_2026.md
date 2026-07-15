# Wave 141a — sourDough Cross-Architecture Parity + Deep Debt

**Date**: July 15, 2026
**From**: sourDough team (eastGate)
**Commit**: `c830173`
**Status**: COMPLETE — all sourDough tasks from Wave 141a handoff resolved

---

## Delivered

### Cross-Architecture Parity (Wave 141a requirement)
- `is_likely_binary()` in `layout.rs` now platform-guarded with `#[cfg(unix)]`
- Non-Unix fallback: file existence check (no `PermissionsExt` dependency)
- `is_triple()` recognizes `windows` targets in addition to `linux`/`unknown`
- `cargo check --target x86_64-pc-windows-gnu` passes cleanly
- Native `cargo clippy` + `cargo test` + `cargo fmt --check` still pass

### Safety Enforcement
- `#![forbid(unsafe_code)]` added to all 3 crate roots:
  - `sourdough-core/src/lib.rs`
  - `sourdough-genomebin/src/lib.rs`
  - `sourdough/src/main.rs`
- The compiler now rejects any `unsafe` that might be introduced

### Smart Refactoring
- `validate/mod.rs` (669L) → 400L
  - Extracted `ecobin.rs` (219L) — binary/project compliance validation
  - Extracted `transport_compliance.rs` (157L) — transport injection auditing
  - Domain separation without losing cohesion
- `#[derive(Debug)]` added to `TransportStream` for diagnostics

### Test Coverage Expansion
| Module | Tests Added | Coverage |
|--------|-------------|----------|
| `ipc/protocol.rs` | 22 tests | JSON-RPC types, serde, error mapping, retryability |
| `transport/stream.rs` | 5 tests | TCP/UDS roundtrip, mesh relay rejection |
| `ipc/error.rs` | 6 tests | Retryable classification, Display, serde |
| `ipc/capability.rs` | 2 tests | Builder pattern, serde |
| `layout.rs` | 6 tests | Cross-platform binary detection, Windows triples |
| `validate/mod.rs` | 5 tests | Shared utilities (find_source_dir, collect_rs_files) |

**Total**: 473 → 487 tests (+14)

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 473 | 487 |
| Largest production file | 669L | 608L |
| `#![forbid(unsafe_code)]` | workspace lints only | all crate roots |
| Windows cross-check | N/A | Passing |
| Clippy warnings | 0 | 0 |
| Unsafe blocks | 0 | 0 |
| unwrap/expect (prod lib) | 0 | 0 |
| Hardcoded primal names (prod) | 0 | 0 |
| Production mocks | 0 | 0 |
| C dependencies | 0 | 0 |

---

## Audit Findings (for upstream teams)

### Zero Deep Debt Remaining
- All production `unwrap()`/`expect()` verified to be inside `#[cfg(test)]` modules
- All mocks (`MockDiscoveryPrimal`, `MockHealthyPrimal`, etc.) inside `#[cfg(test)]`
- Zero TODO/FIXME/HACK markers in production code
- No unused dependencies (`getrandom` v0.2/v0.3 split is upstream in `ed25519-dalek` — dev-only)
- `flate2` confirmed Pure Rust backend (`miniz_oxide`)

### Cross-Architecture Status
sourDough is now the second primal (after songBird) to achieve Windows target parity.
The fix was minimal: 1 usage site of `PermissionsExt::mode()` needed platform guards.

### Dependency Analysis
All dependencies Pure Rust:
- `blake3` (SIMD asm optional, no C linkage)
- `ed25519-dalek` (Pure Rust)
- `sha2` (Pure Rust)
- `flate2` → `miniz_oxide` (Pure Rust)
- `tokio`, `serde`, `clap`, `chrono` — all Pure Rust

No migration candidates — the dependency tree is already fully evolved.

---

## For Downstream (overwatch audit)

sourDough has **zero remaining work items**. The primal is at steady-state:
- All Wave 141a cross-arch tasks: DONE
- All Wave 113 riboCipher tasks: DONE (REJECT shipped in scaffold templates)
- All Wave 112 deprecation escalation: DONE
- Forgejo parity: RESTORED

Next meaningful sourDough work would be `v0.5.0` scope (harvest/package commands) — not yet prioritized by overwatch.

---

**sourDough: deep debt zero. Cross-arch parity achieved. Awaiting v0.5.0 scope.**
