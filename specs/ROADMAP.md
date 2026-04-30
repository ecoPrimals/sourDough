# sourDough Roadmap

**Version**: 0.2.0-dev
**Date**: April 30, 2026
**Vision**: The nascent budding primal for ecoPrimals

---

## Current State (v0.2.0-dev)

### Complete (v0.1.0 + v0.2.0)

- [x] Core traits: PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig
- [x] sourdough-core library: traits + types + JSON-RPC 2.0 IPC + tarpc RPC + PeekedStream transport
- [x] sourDough UniBin CLI: scaffold, validate, genomebin, doctor
- [x] Scaffold independence: generated primals are self-contained (budding primal pattern)
- [x] sourdough-genomebin Pure Rust library: platform detection, metadata, archive, validation
- [x] Validation tools: primal, unibin, ecobin compliance
- [x] Pure Rust: zero C dependencies (blake3 `pure` feature)
- [x] 247 tests, 95%+ coverage (llvm-cov)
- [x] `#![forbid(unsafe_code)]` on all crates
- [x] clippy pedantic + nursery clean (-D warnings)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] Rust edition 2024
- [x] scyBorg Provenance Trio license
- [x] Scaffold generates `{name}-server` crate with JSON-RPC server + capability wire handlers
- [x] Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml`
- [x] Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- [x] `PeekedStream` transport utility in sourdough-core (ecosystem convergence)
- [x] Socket path resolution + first-byte peek in scaffolded servers
- [x] `cargo deny check` passing (ecoBin v3.0 C-sys ban list)

### Stretch (SD-02, SD-03)

- [ ] Cross-compilation validation (musl targets)
- [ ] genomeBin signing (Pure Rust, sequoia-openpgp)

### Planned

- [ ] Ephemeral primal scaffolding (session-as-primal pattern)
- [ ] EphemeralOwner<T> in sourdough-core
- [ ] `sourdough harvest` command for plasmidBin integration

---

## Version Roadmap

### v0.1.0 -- Foundation + CLI + Scaffold Independence

**Delivered** (January - April 3, 2026):

- Core traits library with JSON-RPC 2.0 IPC and tarpc RPC
- UniBin CLI: scaffold, validate, genomebin, doctor
- Pure Rust genomebin library (replaces bash scripts)
- Self-contained primal scaffolding (budding primal pattern)
- 239 tests, 95%+ coverage, zero unsafe code
- 6 unused dependencies removed
- All lint suppressions documented with `#[expect(reason)]`

### v0.2.0 -- Scaffold Evolution + Ecosystem Convergence (Current)

**Delivered** (April 30, 2026):

- [x] Scaffold generates `{name}-server` crate with JSON-RPC server + capability wire
- [x] Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml`
- [x] Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- [x] `PeekedStream` transport in sourdough-core (ecosystem convergence)
- [x] Socket path resolution + first-byte peek in generated servers
- [x] Capability wire standard: health.liveness, health.readiness, health.check, capabilities.list
- [x] Template module refactored by domain (core/server/infra)
- [x] Deep debt cleanup: dependency alignment, hardcoding removal
- 247 tests, 95%+ coverage

**Stretch goals (still open)**:
- [ ] Cross-compile for x86_64-musl, aarch64-musl (SD-02)
- [ ] genomeBin signing via Pure Rust sequoia-openpgp (SD-03)
- [ ] Harvest to plasmidBin

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
| Test coverage | 95%+ | >90% maintained |
| Tests passing | 247 | All passing |
| Clippy | zero warnings (workspace lints) | zero warnings |
| Unsafe code | zero | zero |
| C dependencies | zero | zero |
| Max file size | 637 lines | <1000 lines |
| Build time | <15s incremental | <30s clean |

---

## Related Documents

- [ARCHITECTURE.md](ARCHITECTURE.md) -- how sourDough is built
- [SOURDOUGH_SPECIFICATION.md](SOURDOUGH_SPECIFICATION.md) -- what sourDough is
- [EPHEMERAL_PRIMAL_SCAFFOLDING.md](EPHEMERAL_PRIMAL_SCAFFOLDING.md) -- ephemeral primal spec
- [CONVENTIONS.md](../CONVENTIONS.md) -- coding conventions

---

**Version**: 0.2.0-dev
**Date**: April 30, 2026
**Status**: Scaffold evolution shipped, cross-compilation and signing next
