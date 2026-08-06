# sourDough Status

**Version**: 0.4.0
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits + JSON-RPC 2.0 IPC + TransportEndpoint + IpcClient + riboCipher + CircuitBreaker + zero-copy RPC
- `sourdough`: CLI binary (scaffold, validate [transport, ribocipher, depot, composition], sign, migrate, doctor)
- `sourdough-genomebin`: Pure Rust genomeBin operations
- **Scaffold produces dual-protocol primals** (G64 Cephalization): JSON-RPC on `.sock` + tarpc on `.tarpc.sock`

## Compliance

- [x] `forbid(unsafe_code)` via workspace lints on all crates
- [x] `clippy::pedantic` + `clippy::nursery` zero warnings (workspace-level `[workspace.lints]`)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] `cargo fmt` clean
- [x] `cargo doc` zero warnings, all doctests compile (0 ignored)
- [x] Zero C application dependencies (Pure Rust entire dependency tree)
- [x] `cargo deny check` passing (ecoBin C-sys ban, explicit `ring` ban, supply chain audit)
- [x] Zero hardcoded primal names in production code — env-driven discovery constants
- [x] JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- [x] riboCipher transport signal standard (Wave 111) — reference implementation
- [x] MethodGate (JH-0) pre-dispatch capability gate on scaffolded primals
- [x] Binary RPC secondary high-throughput path with `bytes::Bytes` zero-copy
- [x] Edition 2024
- [x] scyBorg triple license (AGPL-3.0-or-later, ORC, CC-BY-SA-4.0)
- [x] Dual-protocol scaffold (G64 Cephalization): JSON-RPC + tarpc service trait
- [x] 510 tests, zero ignored
- [x] Zero unwrap/expect in library production code
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [x] Transport injection: primals accept `TRANSPORT_ENDPOINT` env var
- [x] All production files under 608 lines
- [x] `#![forbid(unsafe_code)]` on all 3 crate roots (compiler-enforced)
- [x] Per-crate tokio feature selection (minimal compile footprint)
- [x] Release CI: x86_64-musl, aarch64-musl, armv7-musleabihf, aarch64-linux-android
- [x] Cross-arch proven (Pixel 8 / GrapheneOS deployment validated)
- [x] Windows cross-check passing (`cargo check --target x86_64-pc-windows-gnu`)

## Crate Health

| Crate | Tests | Max Lines |
|-------|-------|-----------|
| sourdough-core | 295 | 502 (rpc.rs) |
| sourdough (CLI) | 36 (2 e2e + 34 integration) | 608 (ribocipher.rs) |
| sourdough-genomebin | 75 + 92 | 599 (platform.rs) |
| doctests | 12 | — |

## v0.4.0 (June–Aug 2026 — Transport Ecosystem + riboCipher + Cross-Arch + Cephalization)

### Wave 156h (August 6, 2026 — G64 Cephalization Scaffold)
- Scaffold templates emit dual-protocol primals: JSON-RPC on `.sock` + tarpc on `.tarpc.sock`
- New `tarpc_service.rs` template in core crate (service trait + response types)
- New `tarpc_server.rs` template in server crate (UDS listener + handler bridge)
- Workspace deps template includes tarpc 0.37 + tokio-serde + futures
- Server `main.rs` starts tarpc as background task, JSON-RPC as lifecycle anchor
- `--disable-tarpc` CLI flag for JSON-RPC-only mode
- Clippy deep debt: raw string hashes, byte_char_slices, io_other_error, case_sensitive_extension
- `validate tarpc` subcommand — audit primals for G64 dual-protocol compliance
- Discovery types evolved: `ServiceInfo` + `ServiceRegistration` carry tarpc endpoints
- `ProtocolSupport` enum (JsonRpcOnly / TarpcOnly / DualProtocol) for capability-based routing
- Property-based tests for discovery type roundtrips
- 510 tests, all 3 cross-targets green, clippy clean

### Wave 142b (July 16, 2026 — Type-System Evolution + Android Parity)
- `Did::try_new()` — validated DID construction (rejects malformed input)
- `Did::method()` + `Did::method_specific_id()` — zero-alloc extractors
- `Os::Android` + `LibC::Bionic` — proper platform detection for Android targets
- `Platform::is_android()` + `Platform::is_windows()` — API completeness
- Proptest coverage for Did (serde, validation) and CommonConfig (JSON/TOML roundtrip)
- `cargo check --target aarch64-linux-android` passes cleanly
- 502 tests (up from 487 at Wave 141a)

### Wave 141a (July 15, 2026 — Cross-Architecture Parity + Deep Debt)
- `#![forbid(unsafe_code)]` on all 3 crate roots (compiler-enforced safety)
- Windows cross-check passing (`cargo check --target x86_64-pc-windows-gnu`)
- `is_likely_binary` + `is_triple` platform-guarded for non-Unix targets
- Smart refactoring: validate/mod.rs (669L → 400L) → ecobin.rs + transport_compliance.rs
- Comprehensive test coverage for ipc/protocol.rs, transport/stream.rs, ipc/error.rs, ipc/capability.rs
- 487 tests (up from 437 at Wave 111)
- All production files under 608 lines

### Wave 112 (June 14, 2026 — riboCipher Deprecation Escalation)
- riboCipher WARN→ERROR escalation (Wave 112 policy)
- Updated validate ribocipher to accept ERROR-level deprecation logs
- Forgejo parity restored

### Wave 111 (June 13, 2026 — riboCipher + Deep Debt)
- riboCipher reference implementation (detect_signal, send_clear_signal, RiboCipherAcceptLoop)
- `sourdough validate ribocipher` — compliance audit subcommand
- Scaffold templates emit riboCipher-compliant servers
- Hardcoded names → env-driven constants (MESH_RELAY_HUB, TCP_FALLBACK_PORT, SOCKET_DIR_NAME)
- Tokio features trimmed per-crate (no more `features = ["full"]`)
- Dead deps removed (camino, prod anyhow from genomebin)
- server.rs template decomposed (878L → 372/185/322)
- Comprehensive tests for peek.rs, endpoint.rs, ipc/client.rs
- 437 tests (up from 322 at Wave 107)

### Wave 107 (June 10, 2026 — Transport Ecosystem)
- `sourdough-core::TransportEndpoint` — wire-compatible with songbird_types
- `sourdough-core::IpcClient` — JSON-RPC 2.0 client with timeout + liveness + resolve + announce
- `sourdough-core::CircuitBreaker` — resilience pattern for IPC
- `sourdough-core::env_keys` — centralized env var + discovery constants
- `sourdough validate transport` / `transport-report` / `depot` — ecosystem auditing
- `sourdough scaffold transport-kit` — self-contained transport module
- `sourdough migrate transport` — migration tool
- Scaffold emits transport-injected, announcing primals
- Release CI includes `aarch64-linux-android`

## v0.3.1 (May 2026 — Neural API)

- Scaffold generates `announce.rs` (primal.announce Wave 42 standard)
- Scaffolded primals auto-announce to biomeOS on startup
- `notify-plasmidbin.yml` workflow

## v0.3.0 (May 2026 — Deployment Internalization)

- `sourdough sign` / `sourdough verify` — Ed25519 detached signatures
- `sourdough validate ecobin <binary>` — static/stripped/size validation
- `sourdough scaffold systemd` — hardened .service units
- `sourdough layout` — triple-first binary layout validation
- `sourdough validate composition` — composition binary presence checking
