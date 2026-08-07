# What's Next for sourDough

**Version**: 0.4.0
**Date**: August 7, 2026
**Status**: G68 Platform Substrate DONE. L1 links + L2 permissions reference impl + validator. 563 tests. Deep debt zero. Zero P0/P1/P2.

---

## Shipped (v0.4.0 — June–Aug 2026, Transport + riboCipher + Cross-Arch + Cephalization)

### Wave 157a (August 7 — G68 Platform Substrate Abstraction)
- [x] **G68 reference implementation**: `sourdough_core::platform_substrate` module
- [x] `platform_link()` — symlink on Unix, junction/hard-link on Windows, hard-link elsewhere
- [x] `PlatformAccess` enum (OwnerReadWrite/OwnerFull/PublicRead/PublicExecute/Readonly/Custom)
- [x] `PlatformAccess::apply()` / `query_access()` — platform-aware permission set/get
- [x] `ensure_dir_with_access()` / `ensure_secure_parent()` — secure directory helpers
- [x] `is_symlink()` — cross-platform symlink detection
- [x] `sourdough validate platform-substrate` subcommand (L1/L2/L3 detection, JSON output)
- [x] Scaffold templates emit `platform_substrate.rs` in generated primals
- [x] `specs/PLATFORM_SUBSTRATE_SPEC.md` written
- [x] 563 tests (18 new), all cross-targets green (Linux, Windows, Android), clippy clean

### Wave 156l (August 6 — C7 G65 Protocol Negotiation)
- [x] **C7 DONE**: `sourdough_core::protocol_negotiation` — canonical negotiation module
- [x] `IpcProtocol` enum (JsonRpc, Tarpc) with wire format, serde, Display, parse
- [x] `NegotiationRequest` / `NegotiationResponse` wire-format types
- [x] `negotiate_client()` / `negotiate_server()` async duplex functions
- [x] `select_protocol()` preference-ordered selection with JSON-RPC fallback
- [x] `NegotiationError` typed error enum
- [x] Scaffold emits G65-ready primals with `--negotiate` CLI flag
- [x] Generated servers support Phase 2 (dual-socket) AND Phase 3 (single-socket negotiation)
- [x] `validate tarpc` detects G65 compliance (NONE → DEP_ONLY → PARTIAL → FULL → G65)
- [x] 15 new tests (wire format, roundtrips, duplex negotiation, backward compat)
- [x] 541 tests, all cross-targets green (Linux, Windows, Android), clippy clean

### Wave 156j (August 6 — C6 Reference Implementation)
- [x] **C6 DONE**: `sourdough_core::tarpc_service::PrimalService` canonical tarpc trait
- [x] 8 baseline methods (health_liveness, health_readiness, health_check, capabilities_list, identity_did, system_ping, system_version, lifecycle_state)
- [x] Response types: `HealthResponse`, `TarpcCapability`, `IdentityResponse`
- [x] Client helpers: `connect_primal()`, `connect_primal_by_name()` (unix-gated)
- [x] `default_tarpc_socket_path()` convention helper
- [x] tarpc 0.37 + tokio-serde 0.9 in sourdough-core (production dep)
- [x] `validate tarpc` checks baseline `PrimalService` method presence
- [x] sourDough self-validates as FULL G64 compliance
- [x] 523 tests, all 3 cross-targets green, clippy clean

### Wave 156h (August 6 — G64 Cephalization Scaffold)
- [x] Scaffold emits dual-protocol primals: JSON-RPC on `.sock` + tarpc on `.tarpc.sock`
- [x] `tarpc_service.rs` core template (`#[tarpc::service]` trait + response types + client `connect()`)
- [x] `tarpc_server.rs` server template (UDS listener, `BaseChannel`, handler bridge)
- [x] Workspace deps: tarpc 0.37 (`serde-transport-bincode` + `unix`) + tokio-serde + futures
- [x] `--disable-tarpc` CLI flag for JSON-RPC-only mode
- [x] `validate tarpc` subcommand — audit primals for G64 dual-protocol compliance
- [x] `TransportEndpoint::tarpc_endpoint()` — derive tarpc socket from JSON-RPC endpoint
- [x] `IpcClient::tarpc_path()` — resolve tarpc socket for connected primals
- [x] Doctor reports dual-protocol socket status ([dual]/[jsonrpc]/[tarpc-only])
- [x] `ProtocolSupport` enum with `Display` for diagnostics
- [x] Clippy debt cleared: raw string hashes, byte_char_slices, io_other_error, case_sensitive_extension
- [x] clap 4.5 → 4.6
- [x] All 3 cross-targets green (native, Windows, Android)
- [x] 518 tests, clippy clean

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
| Tests | 541 | All passing |
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
**Status**: C7 G65 Protocol Negotiation DONE. sourDough defines both the canonical `PrimalService` tarpc trait (C6) and the protocol negotiation standard (C7). 541 tests, zero deep debt. sourDough is the standards holder, reference implementation, and tooling authority for G64+G65 cephalization.
