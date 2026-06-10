# What's Next for sourDough

**Version**: 0.4.0
**Date**: June 10, 2026
**Status**: Transport ecosystem complete. Zero P1. Stadial-ready.

---

## Shipped (v0.4.0 — June 2026, Transport Ecosystem)

- [x] `sourdough-core::TransportEndpoint` — canonical wire format (uds/tcp/mesh_relay)
- [x] `sourdough-core::IpcClient` — transport-aware JSON-RPC 2.0 client
- [x] `sourdough-core::CircuitBreaker` — resilience pattern for IPC
- [x] `sourdough-core::methods` — canonical `domain.verb` method constants
- [x] `TransportEndpoint::from_env_or_default()` — canonical injection entry point
- [x] `sourdough validate transport` — single primal transport compliance audit
- [x] `sourdough validate transport-report --json` — ecosystem batch audit
- [x] `sourdough validate depot --json` — binary freshness detection
- [x] `sourdough scaffold transport-kit` — self-contained transport module for primals
- [x] `sourdough migrate transport` — migration tool for existing primals
- [x] Scaffold templates emit transport-injected primals
- [x] Release CI: aarch64-linux-android target (Pixel 8 proven)
- [x] `colored` → `owo-colors` (zero-alloc)
- [x] `HealthProbe.status` evolved to `HealthStatus` enum
- [x] Depot segfault fix (iterative traversal for musl compatibility)
- [x] 321 tests, 3 crates, all files < 800L

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
| Tests | 321 | All passing |
| Clippy | zero warnings | zero |
| Unsafe | zero (forbid) | zero |
| C deps | zero | zero |
| Max file | 785 lines | < 800 |

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

**Date**: June 10, 2026
**Status**: Zero development debt. Transport ecosystem shipped. Waiting on upstream primals (songBird ipc.resolve, biomeOS auto-register) before next milestone.
