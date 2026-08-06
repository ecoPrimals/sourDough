# What's Next for sourDough

**Version**: 0.4.0
**Date**: August 6, 2026
**Status**: G64 Cephalization scaffold shipped. Dual-protocol primals (JSON-RPC + tarpc). 502 tests. Deep debt zero. Zero P0/P1/P2.

---

## Shipped (v0.4.0 — June–Aug 2026, Transport + riboCipher + Cross-Arch + Cephalization)

### Wave 156h (August 6 — G64 Cephalization Scaffold)
- [x] Scaffold emits dual-protocol primals: JSON-RPC on `.sock` + tarpc on `.tarpc.sock`
- [x] `tarpc_service.rs` core template (`#[tarpc::service]` trait + response types)
- [x] `tarpc_server.rs` server template (UDS listener, `BaseChannel`, handler bridge)
- [x] Workspace deps: tarpc 0.37 (`serde-transport-bincode` + `unix`) + tokio-serde + futures
- [x] `--disable-tarpc` CLI flag for JSON-RPC-only mode
- [x] Clippy debt cleared: raw string hashes, byte_char_slices, io_other_error, case_sensitive_extension
- [x] All 3 cross-targets green (native, Windows, Android)
- [x] 502 tests, clippy clean

### Wave 142b (July 16 — Type-System Evolution + Android Parity)
- [x] `Did::try_new()` validated DID construction (rejects malformed input)
- [x] `Did::method()` + `Did::method_specific_id()` zero-alloc extractors
- [x] `Os::Android` + `LibC::Bionic` platform detection
- [x] `Platform::is_android()` + `Platform::is_windows()` API helpers
- [x] Android cross-check passing (`cargo check --target aarch64-linux-android`)
- [x] Proptest for Did + CommonConfig (property-based serde roundtrips)
- [x] 502 tests, all production files < 608L

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
| Tests | 502 | All passing |
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

**Date**: August 6, 2026
**Status**: G64 Cephalization scaffold shipped. Dual-protocol primals born ready. All 4 depot architectures green. Zero development debt. Phase 2 abstraction: sourDough scaffolds the abstraction for other primals.
