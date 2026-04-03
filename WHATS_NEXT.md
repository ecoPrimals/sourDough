# What's Next for sourDough

**Version**: 0.1.0
**Date**: April 3, 2026

---

## Immediate (v0.2.0)

### Cross-Compilation Validation

- Validate musl builds on x86_64 and aarch64
- Binary analysis: verify no C symbols, fully static linking
- CI pipeline for cross-target builds

### genomeBin Signing

- Pure Rust signing via sequoia-openpgp
- SHA256 checksums for all artifacts
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

- `GenomeBinLauncher`: install, health_check, update, uninstall any primal
- `GenomeBinRegistry`: discover and manage available primals
- biomeOS ephemeral lifecycle support
- Standard JSON-RPC protocol for primal management

---

## Quality Targets

| Metric | Current | Target |
|--------|---------|--------|
| Test coverage | 96%+ | >90% maintained |
| Tests passing | 239+ | All passing |
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
   cargo doc --workspace --no-deps
   cargo llvm-cov --workspace
   ```

---

**Date**: April 3, 2026
**Status**: Foundation complete, cross-compilation and signing next
