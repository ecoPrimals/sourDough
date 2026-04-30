# What's Next for sourDough

**Version**: 0.2.0-dev
**Date**: April 30, 2026

---

## Shipped (v0.2.0 — April 30, 2026)

- [x] Scaffold generates `{name}-server` crate (JSON-RPC 2.0 + capability wire standard)
- [x] Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml`
- [x] Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- [x] `PeekedStream` transport in sourdough-core (ecosystem convergence)
- [x] Socket path resolution (`$XDG_RUNTIME_DIR/biomeos/{name}-{family_id}.sock`)
- [x] First-byte peek (JSON-RPC vs BTSP auto-detection)
- [x] Capability wire handlers (health.liveness, health.readiness, health.check, capabilities.list)
- [x] CONVENTIONS.md corrected (JSON-RPC 2.0 primary, binary RPC secondary)
- [x] Templates refactored into module directory (core/server/infra)

---

## Immediate (v0.2.x)

### Cross-Compilation Validation (SD-02)

- Validate musl builds on x86_64 and aarch64
- Binary analysis: verify no C symbols, fully static linking
- Wire into plasmidBin CI for genomeBin distribution

### genomeBin Signing (SD-03)

- Pure Rust signing via sequoia-openpgp
- BLAKE3 checksums for all artifacts
- Harvest to plasmidBin distribution surface

---

## Near Term (v0.3.0)

### Ephemeral Primal Scaffolding

See `specs/EPHEMERAL_PRIMAL_SCAFFOLDING.md` for the full specification.

- `EphemeralOwner<T>` utility in sourdough-core
- Spawn/teardown protocols with lifecycle registration
- Drop guard for panic-safe deregistration
- Scoped capability namespacing (`session.{id}.*`)
- Reference implementations: session-as-primal, NPC-as-primal

---

## Medium Term (v0.4.0)

### Integration Libraries

- `sourdough harvest` command interfacing with plasmidBin `sources.toml`
- `GenomeBinLauncher`: install, health_check, update, uninstall any primal
- `GenomeBinRegistry`: discover and manage available primals
- biomeOS ephemeral lifecycle support

---

## Quality Targets

| Metric | Current | Target |
|--------|---------|--------|
| Test coverage | 95%+ | >90% maintained |
| Tests passing | 247 | All passing |
| Clippy | zero warnings (workspace lints) | zero warnings |
| Unsafe code | zero (forbid) | zero |
| C dependencies | zero | zero |
| Max file size | 637 lines | <1000 lines |

---

## How to Contribute

1. Pick an item from this list
2. Check `specs/` for related specifications
3. Follow `CONVENTIONS.md` for coding standards
4. Run the full verification suite before submitting:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all -- --check
   cargo deny check
   cargo doc --workspace --no-deps
   ```

---

**Date**: April 30, 2026
**Status**: v0.2.0 scaffold evolution shipped, cross-compilation and signing next
