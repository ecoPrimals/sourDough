# Wave 142b — sourDough Type-System Evolution + Android Parity

**Date**: July 16, 2026
**From**: sourDough team (eastGate)
**Commits**: `6115e4a` (Android platform), `24419d7` (type evolution)
**Status**: COMPLETE — all depot architectures harvestable, type system evolved

---

## Delivered

### Android Platform Parity (Wave 141b→142b)
- `Os::Android` variant with `#[cfg(target_os = "android")]` detection
- `LibC::Bionic` variant (Android's C library)
- `target_triple()` → `"aarch64-linux-android"` (matches Rust target spec)
- `simple_target()` → `"aarch64-android"`
- Fixed unfulfilled `#[expect(unreachable_code)]` on Android builds
- **Result**: `cargo check --target aarch64-linux-android` passes cleanly
- sourDough removed from "3 pending Android" in depot harvest

### Type-System Evolution
- `Did::try_new()` — validated DID construction for external input boundaries
- `Did::method()` — zero-alloc method component extraction
- `Did::method_specific_id()` — zero-alloc identifier extraction
- `Platform::is_android()` + `Platform::is_windows()` — API completeness

### Property-Based Testing
- Proptest for `Did`: serde roundtrip, `try_new` acceptance, prefix rejection
- Proptest for `CommonConfig`: JSON roundtrip, TOML roundtrip, instance_id invariant
- Total proptest modules: 6 (transport, types, rpc, metadata, identity, config)

---

## Metrics

| Metric | Wave 141a | Wave 142b |
|--------|-----------|-----------|
| Tests | 487 | **502** (+15) |
| Proptest modules | 4 | **6** |
| Cross-targets | 2 (Linux, Windows) | **3** (+ Android) |
| Depot architectures harvestable | 3/4 | **4/4** |
| Deep debt | Zero | Zero |

---

## Depot Harvest Readiness

sourDough is now harvestable by sporeGate for all 4 architectures:

| Target | Check | Status |
|--------|-------|--------|
| `x86_64-unknown-linux-musl` | `cargo check` | Green |
| `aarch64-unknown-linux-musl` | `cargo check` | Green |
| `x86_64-pc-windows-gnu` | `cargo check` | Green |
| `aarch64-linux-android` | `cargo check` | **Green** (fixed this wave) |

---

## Phase 2 Assessment (Silicon Atheism — Abstraction Over Gating)

sourDough is a **CLI tool** — not a runtime daemon with transport connections.
Phase 2 (trait-based transport backends replacing `#[cfg]` gating) is **N/A** for sourDough.

The cross-platform work that was needed was purely in:
1. File permission detection (`layout.rs`) — now `#[cfg(unix)]` gated
2. Platform detection (`platform.rs`) — now recognizes all 4 depot OSes

No `UnixStream`, `TcpListener`, or other transport code exists in sourDough
production code — it generates these patterns for other primals via scaffold templates.

---

## For Upstream Teams

### sporeGate
sourDough is ready for re-harvest. All 4 architectures compile cleanly.

### Ecosystem API Note
`Did::try_new()` is available for primals that receive DIDs from external
sources (network, user input, configuration). Existing `Did::new()` remains
for internal/trusted construction. This is a non-breaking addition.

---

**sourDough: 502 tests. 4/4 architectures. Deep debt zero. Awaiting v0.5.0 scope.**
