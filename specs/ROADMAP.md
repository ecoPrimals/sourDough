# sourDough Roadmap

**Version**: 0.3.0
**Date**: May 14, 2026
**Vision**: The nascent budding primal for ecoPrimals

---

## Current State (v0.3.0)

### Complete (v0.1.0 through v0.3.0)

- [x] Core traits: PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig
- [x] sourdough-core library: traits + types + JSON-RPC 2.0 IPC + zero-copy RPC + PeekedStream transport
- [x] sourDough UniBin CLI: scaffold, sign, verify, validate, layout, genomebin, doctor
- [x] Scaffold independence: generated primals are self-contained (budding primal pattern)
- [x] sourdough-genomebin Pure Rust library: platform detection, metadata, archive, validation, Ed25519 signing
- [x] Validation tools: primal, unibin, ecobin (project + binary), composition
- [x] Pure Rust: zero C dependencies (blake3 `pure` feature)
- [x] 281 tests, 95%+ coverage (llvm-cov)
- [x] `#![forbid(unsafe_code)]` on all crates
- [x] clippy pedantic + nursery clean (-D warnings)
- [x] Rust edition 2024
- [x] scyBorg Provenance Trio license
- [x] Scaffold generates `{name}-server` crate with JSON-RPC server + capability wire + MethodGate
- [x] Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml` + `release.yml`
- [x] Scaffold generates `deny.toml` (ecoBin v3.0, explicit `ring` ban)
- [x] Scaffold generates `method_gate.rs` (JH-0/JH-2 pre-dispatch gate)
- [x] Scaffold generates `btsp.negotiate` handler (NULL cipher fallback)
- [x] `PeekedStream` transport utility in sourdough-core (ecosystem convergence)
- [x] Socket path resolution + first-byte peek in scaffolded servers
- [x] `cargo deny check` passing (ecoBin v3.0 C-sys ban list + explicit `ring` ban)
- [x] Cross-compilation: `release.yml` musl matrix (x86_64, aarch64, armv7) — SD-02 resolved
- [x] genomeBin signing: Ed25519 detached signatures via `ed25519-dalek` — SD-03 resolved
- [x] `sourdough sign` / `sourdough verify` CLI commands
- [x] `sourdough validate ecobin <binary>` (static/stripped/size checks)
- [x] `sourdough scaffold systemd` (hardened `.service` units)
- [x] `sourdough layout` (triple-first layout validation)
- [x] `sourdough validate composition` (atomic + niche compositions from `ports.env`)

---

## Version Roadmap

### v0.1.0 -- Foundation + CLI + Scaffold Independence (COMPLETE)

**Delivered** (January - April 3, 2026):

- Core traits library with JSON-RPC 2.0 IPC and zero-copy RPC
- UniBin CLI: scaffold, validate, genomebin, doctor
- Pure Rust genomebin library (replaces bash scripts)
- Self-contained primal scaffolding (budding primal pattern)
- 239 tests, 95%+ coverage, zero unsafe code

### v0.2.0 -- Scaffold Evolution + Ecosystem Convergence (COMPLETE)

**Delivered** (April 30 - May 11, 2026):

- Scaffold generates server crate with JSON-RPC + capability wire + MethodGate
- Scaffold generates CI workflows (ci.yml, notify-plasmidbin.yml, release.yml)
- Scaffold generates deny.toml with ecosystem-standard bans
- PeekedStream transport, socket path resolution, first-byte peek
- Ed25519 signing module in sourdough-genomebin (SD-03)
- musl cross-compilation in release.yml (SD-02)
- 256 tests

### v0.3.0 -- Deployment Internalization (COMPLETE)

**Delivered** (May 14, 2026):

Per contract: `primalSpring/docs/SOURDOUGH_DEPLOYMENT_INTERNALIZATION.md`

- `sourdough sign` / `sourdough verify` — Ed25519 detached signatures
- `sourdough validate ecobin <binary>` — static linking, stripped, size budget
- `sourdough scaffold systemd` — hardened service units (membrane pattern)
- `sourdough layout` — triple-first binary layout validation
- `sourdough validate composition` — atomic + niche composition checks
- validate.rs refactored into module with composition submodule
- 281 tests

### v0.4.0 -- Harvest + Release (NEXT)

**Goals**: Cross-compile and release primals via CLI

- [ ] `sourdough harvest --all` — cross-compile all primals per `sources.toml`
- [ ] `sourdough harvest --release` — checksum, stage, tag, push to GitHub Releases
- [ ] Asset carry-forward from `auto-harvest.yml` into Rust
- [ ] `sourdough validate composition` gains Phase 3 live health probes

### v0.5.0 -- Package (genomeBin)

**Goals**: Self-extracting genomeBin archives

- [ ] `sourdough package` — creates self-extracting archives
- [ ] Embed manifest, checksums, signature in archive header
- [ ] Offline deployment to air-gapped gates

### v0.6.0 -- Deploy

**Goals**: Full deploy+verify cycle

- [ ] `sourdough deploy --target membrane` — fetch, provision, verify
- [ ] Multi-target support: membrane, gate, nest topologies
- [ ] Post-deploy smoke tests

### v1.0.0 -- Production Ready

**Goals**: Stable APIs, comprehensive adoption

- [ ] All APIs stable (semantic versioning, backward compatibility)
- [ ] Security audit complete
- [ ] genomeBin creation < 1 minute, installation < 30 seconds

---

## Quality Targets

| Metric | Current | Target (v1.0) |
|--------|---------|---------------|
| Test coverage | 95%+ | >90% maintained |
| Tests passing | 281 | All passing |
| Clippy | zero warnings (workspace lints) | zero warnings |
| Unsafe code | zero | zero |
| C dependencies | zero | zero |
| Max file size | 750 lines | <1000 lines |
| Build time | <15s incremental | <30s clean |

---

## Related Documents

- [ARCHITECTURE.md](ARCHITECTURE.md) -- how sourDough is built
- [SOURDOUGH_SPECIFICATION.md](SOURDOUGH_SPECIFICATION.md) -- what sourDough is
- [EPHEMERAL_PRIMAL_SCAFFOLDING.md](EPHEMERAL_PRIMAL_SCAFFOLDING.md) -- ephemeral primal spec
- [CONVENTIONS.md](../CONVENTIONS.md) -- coding conventions

---

**Version**: 0.3.0
**Date**: May 14, 2026
**Status**: Deployment internalization complete, harvest and release next
