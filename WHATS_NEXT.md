# What's Next for sourDough

**Version**: 0.4.0
**Date**: July 15, 2026
**Status**: Cross-arch parity complete. Deep debt zero. `#![forbid(unsafe_code)]` enforced. Zero P1.

---

## Shipped (v0.4.0 — June–July 2026, Transport + riboCipher + Cross-Arch)

### Wave 141a (July 15 — Cross-Architecture Parity + Deep Debt)
- [x] `#![forbid(unsafe_code)]` on all crate roots (compiler-enforced)
- [x] Windows cross-check passing (`cargo check --target x86_64-pc-windows-gnu`)
- [x] Platform-guarded file permission checks (Unix vs non-Unix)
- [x] Smart refactoring: validate/mod.rs decomposed into domain modules
- [x] Comprehensive test coverage for ipc/protocol.rs, stream.rs, error.rs, capability.rs
- [x] 487 tests, all production files < 608L

### Wave 112 (June 14 — riboCipher Deprecation Escalation)
- [x] WARN→ERROR for unsignalled connections (Wave 112 policy)
- [x] validate ribocipher accepts ERROR-level deprecation logs
- [x] Forgejo parity restored

### Wave 111 (June 13 — riboCipher + Deep Debt)
- [x] riboCipher reference implementation (detect_signal, RiboCipherAcceptLoop)
- [x] `sourdough validate ribocipher` — compliance audit subcommand
- [x] Scaffold templates emit riboCipher-compliant servers
- [x] Hardcoded names → env-driven constants (MESH_RELAY_HUB, TCP_FALLBACK_PORT)
- [x] Tokio features trimmed per-crate (minimal compile footprint)
- [x] Dead deps removed (camino, prod anyhow from genomebin)
- [x] server.rs decomposed (878L → 3 focused modules)
- [x] 437 tests, all production files < 700L

### Wave 107 (June 10 — Transport Ecosystem)
- [x] `sourdough-core::TransportEndpoint` — canonical wire format (uds/tcp/mesh_relay)
- [x] `sourdough-core::IpcClient` — transport-aware JSON-RPC 2.0 client
- [x] `sourdough-core::CircuitBreaker` — resilience pattern for IPC
- [x] `sourdough validate transport` / `transport-report` / `depot` — ecosystem auditing
- [x] `sourdough scaffold transport-kit` — self-contained transport module
- [x] Scaffold templates emit transport-injected primals
- [x] Release CI: aarch64-linux-android (Pixel 8 proven)

## Shipped (v0.3.1 — May 2026, Neural API)

- [x] Scaffold generates `announce.rs` (primal.announce)
- [x] Scaffolded primals auto-announce to biomeOS
- [x] `notify-plasmidbin.yml` workflow

## Shipped (v0.3.0 — May 2026, Deployment Internalization)

- [x] `sourdough sign` / `sourdough verify` — Ed25519 signatures
- [x] `sourdough validate ecobin <binary>` — static/stripped/size checks
- [x] `sourdough scaffold systemd` — hardened service units
- [x] `sourdough layout` — triple-first layout validation
- [x] `sourdough validate composition` — composition binary presence

---

## Next (v0.5.0 — Harvest + Package)

- [ ] `sourdough harvest --all` — cross-compile all primals per `sources.toml`
- [ ] `sourdough harvest --release` — checksum, stage, tag, push to GitHub Releases
- [ ] `sourdough package` — self-extracting genomeBin archives
- [ ] Embed manifest, checksums, signature in archive header
- [ ] `sourdough validate composition` gains Phase 3 live health probes

---

## Longer Term

### v0.6.0 — Deploy

- [ ] `sourdough deploy --target membrane` — full deploy+verify cycle
- [ ] Multi-target: membrane, gate, nest topologies
- [ ] Post-deploy smoke tests

### v1.0.0 — Stable

- [ ] All APIs stable (semantic versioning)
- [ ] Security audit complete
- [ ] genomeBin creation < 1 minute

---

## Quality Targets

| Metric | Current | Target |
|--------|---------|--------|
| Tests | 487 | All passing |
| Clippy | zero warnings | zero |
| Unsafe | zero (forbid on all roots) | zero |
| C deps | zero | zero |
| Max file | 608 lines | < 800 |
| unwrap/expect | zero (prod lib) | zero |
| Mocks in prod | zero | zero |
| Cross-arch | Windows + Linux | All tier-1 |

---

## How to Contribute

1. Check `specs/` for related specifications
2. Follow `CONVENTIONS.md` for coding standards
3. Run the full verification suite before submitting:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --check
   cargo deny check
   cargo doc --workspace --no-deps
   ```

---

**Date**: July 15, 2026
**Status**: Zero development debt. Cross-arch parity complete. `#![forbid(unsafe_code)]` enforced. Upstream primals adopting cross-platform transport (Wave 141a handoffs issued).
