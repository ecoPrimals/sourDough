# sourDough Roadmap

**Version**: 0.1.0
**Date**: April 3, 2026
**Vision**: The nascent budding primal for ecoPrimals

---

## Current State (v0.1.0)

### Complete

- [x] Core traits: PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig
- [x] sourdough-core library: traits + types + JSON-RPC 2.0 IPC + tarpc RPC
- [x] sourDough UniBin CLI: scaffold, validate, genomebin, doctor
- [x] Scaffold independence: generated primals are self-contained (budding primal pattern)
- [x] sourdough-genomebin Pure Rust library: platform detection, metadata, archive, validation
- [x] Validation tools: primal, unibin, ecobin compliance
- [x] Pure Rust: zero C dependencies
- [x] 229 tests, 94.40% coverage (llvm-cov)
- [x] `#![forbid(unsafe_code)]` on all crates
- [x] clippy pedantic + nursery clean (-D warnings)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] Rust edition 2024
- [x] scyBorg Provenance Trio license

### In Progress

- [ ] Cross-compilation validation (musl targets)
- [ ] genomeBin signing (Pure Rust, sequoia-openpgp)

### Planned

- [ ] Ephemeral primal scaffolding (session-as-primal pattern)
- [ ] EphemeralOwner<T> in sourdough-core
- [ ] biomeOS/neuralAPI connectors

---

## Version Roadmap

### v0.1.0 -- Foundation + CLI + Scaffold Independence (Current)

**Delivered** (January - April 2026):

- Core traits library with JSON-RPC 2.0 IPC and tarpc RPC
- UniBin CLI: scaffold, validate, genomebin, doctor
- Pure Rust genomebin library (replaces bash scripts)
- Self-contained primal scaffolding (budding primal pattern)
- 229 tests, 94.40% coverage, zero unsafe code
- 6 unused dependencies removed
- All lint suppressions documented with `#[expect(reason)]`

### v0.2.0 -- Cross-Compilation and Signing (Next)

**Goals**: Production-ready distribution pipeline

- [ ] Cross-compile for x86_64-musl, aarch64-musl, x86_64-macos, aarch64-macos
- [ ] Binary analysis: no C symbols, static linking verified
- [ ] genomeBin signing via Pure Rust sequoia-openpgp
- [ ] SHA256 checksums
- [ ] Harvest to plasmidBin

**Success criteria**:
- Builds on all targets without C dependencies
- Binaries are statically linked
- Signed genomeBins verify correctly

### v0.3.0 -- Ephemeral Primal Scaffolding

**Goals**: Support short-lived primals (sessions, NPCs, matches, mods)

See `specs/EPHEMERAL_PRIMAL_SCAFFOLDING.md` for full specification.

- [ ] `EphemeralOwner<T>` utility in sourdough-core
- [ ] Spawn protocol: `start()` + `lifecycle.register` with `ephemeral: true`
- [ ] Teardown protocol: `lifecycle.deregister` + `stop()` + cleanup
- [ ] Drop guard for safety-net deregistration on panic
- [ ] Scoped capability namespacing (`session.{id}.*`, `npc.{id}.*`)
- [ ] Reference implementations: session-as-primal, NPC-as-primal

**Success criteria**:
- Ephemeral primals register/deregister with biomeOS at runtime
- Provenance outlives runtime (rhizoCrypt DAG persists after teardown)
- Drop guard ensures cleanup on panic

### v0.4.0 -- Integration Libraries

**Goals**: Programmatic primal management for biomeOS and neuralAPI

- [ ] GenomeBinLauncher: install, health_check, update, uninstall any primal
- [ ] GenomeBinRegistry: discover and manage available primals
- [ ] biomeOS ephemeral lifecycle support
- [ ] Standard JSON-RPC protocol for primal management
- [ ] Integration tests with biomeOS team

**Success criteria**:
- biomeOS can programmatically launch any primal
- neuralAPI can manage primal dependencies
- Standard protocol works across all primals

### v1.0.0 -- Production Ready

**Goals**: Stable APIs, comprehensive adoption

- [ ] All APIs stable (semantic versioning, backward compatibility)
- [ ] All primals use sourDough scaffolding
- [ ] All ecoBins use sourDough genomeBin tooling
- [ ] biomeOS and neuralAPI use sourDough libraries
- [ ] Security audit complete
- [ ] genomeBin creation < 1 minute, installation < 30 seconds

---

## Quality Targets

| Metric | Current | Target (v1.0) |
|--------|---------|---------------|
| Test coverage | 94.40% | >90% maintained |
| Tests passing | 229/229 | All passing |
| Clippy | zero warnings | zero warnings |
| Unsafe code | zero | zero |
| C dependencies | zero | zero |
| Max file size | 789 lines | <1000 lines |
| Build time | <15s incremental | <30s clean |

---

## Related Documents

- [ARCHITECTURE.md](ARCHITECTURE.md) -- how sourDough is built
- [SOURDOUGH_SPECIFICATION.md](SOURDOUGH_SPECIFICATION.md) -- what sourDough is
- [EPHEMERAL_PRIMAL_SCAFFOLDING.md](EPHEMERAL_PRIMAL_SCAFFOLDING.md) -- ephemeral primal spec
- [CONVENTIONS.md](../CONVENTIONS.md) -- coding conventions

---

**Version**: 0.1.0
**Date**: April 3, 2026
**Status**: Foundation complete, cross-compilation and signing next
