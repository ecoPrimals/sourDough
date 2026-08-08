# sourDough Status

**Version**: 0.4.0
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits + JSON-RPC 2.0 IPC + TransportEndpoint + IpcClient + riboCipher + CircuitBreaker + zero-copy RPC + G65 Protocol Negotiation + G68 Platform Substrate
- `sourdough`: CLI binary (scaffold, validate [transport, ribocipher, depot, composition, platform-substrate], sign, migrate, doctor)
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
- [x] Canonical `PrimalService` tarpc trait (C6 reference implementation)
- [x] G65 Protocol Negotiation module (C7 reference implementation)
- [x] G66 Transport Abstraction: `TransportListener` + `bind_transport()` + silicon deism validator
- [x] G68 Platform Substrate: `platform_link()` + `PlatformAccess` + L1/L2/L3 validator
- [x] 579 tests, zero ignored
- [x] Zero unwrap/expect in library production code
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [x] Transport injection: primals accept `TRANSPORT_ENDPOINT` env var
- [x] All production files under 608 lines
- [x] `#![forbid(unsafe_code)]` on all 3 crate roots (compiler-enforced)
- [x] Per-crate tokio feature selection (minimal compile footprint)
- [x] Release CI: x86_64-musl, aarch64-musl, armv7-musleabihf, aarch64-linux-android, riscv64-musl, aarch64-apple-darwin
- [x] Cross-arch proven: Pixel 8 (Android), Mac Mini M4 (Darwin), Milk-V Jupiter 2 (RISC-V), Raspberry Pi (aarch64)
- [x] 8 cross-targets verified: Windows, Darwin, GNU, Android, RISC-V (gnu+musl), ARMv7, aarch64-musl

## Crate Health

| Crate | Tests | Max Lines |
|-------|-------|-----------|
| sourdough-core | 308 | 601 (discovery.rs) |
| sourdough (CLI) | 36 (2 e2e + 34 integration) | 608 (ribocipher.rs) |
| sourdough-genomebin | 75 + 92 | 599 (platform.rs) |
| doctests | 12 | — |

## v0.4.0 (June–Aug 2026 — Transport Ecosystem + riboCipher + Cross-Arch + Cephalization)

### Wave 157a (August 7, 2026 — G68 Platform Substrate + Cross-Arch Expansion)
- **G68 reference implementation**: `sourdough_core::platform_substrate` module
- **L1 Links**: `platform_link()` — symlink on Unix, junction/hard-link on Windows, hard-link elsewhere
- **L2 Permissions**: `PlatformAccess` enum (OwnerReadWrite/OwnerFull/PublicRead/PublicExecute/Readonly/Custom)
- `PlatformAccess::apply()` / `query_access()` — platform-aware permission set/get
- `ensure_dir_with_access()` / `ensure_secure_parent()` — secure directory helpers
- `is_symlink()` — cross-platform symlink detection
- `sourdough validate platform-substrate` subcommand — L1/L2/L3 silicon deism detector
- **Scanner refinement**: prod/test split, contextual `set_mode`, 3 compliance levels (G68/G68-prod/partial)
- Scaffold templates emit `platform_substrate.rs` in generated primals' core crates
- **Cross-arch expansion**: Darwin (Mac Mini M4), RISC-V (Milk-V Jupiter 2), Raspberry Pi
- CI: 6 cross-check targets in CI, 6 release architectures
- Platform: `is_riscv()`, `is_arm()`, `is_unix()` helpers added to genomebin
- 579 tests, all 8 cross-targets green, clippy clean

### Wave 156s (August 6, 2026 — G66 Transport Abstraction)
- **G66 server-side**: `TransportListener` enum (Unix/Tcp) + `bind_transport()`
- `TransportListener::accept()` returns `TransportStream` — business logic never touches raw listeners
- `TransportListener::is_local()` / `local_endpoint()` for trust decisions
- Scaffold templates emit full G66 transport module (`transport.rs`) with all components
- Scaffold announce template fixed for silicon deism (cfg-guarded Unix-only code)
- `validate transport` enhanced: silicon deism detection (Unix APIs outside transport layer)
- 545 tests, all cross-targets green (Linux, Windows, Android), clippy clean

### Wave 156l (August 6, 2026 — C7 G65 Protocol Negotiation)
- **C7 DONE**: `sourdough_core::protocol_negotiation` — canonical single-socket negotiation module
- `IpcProtocol` enum (JsonRpc, Tarpc) with wire format, serde, Display, parse
- `NegotiationRequest` / `NegotiationResponse` — wire-format types (`PROTOCOLS: tarpc,jsonrpc\n`)
- `negotiate_client()` / `negotiate_server()` — async duplex negotiation functions
- `select_protocol()` — preference-ordered protocol selection with JSON-RPC fallback
- `NegotiationError` — typed error enum (InvalidRequest, NoValidProtocols, Timeout, Io)
- Scaffold templates emit G65-ready primals with `--negotiate` CLI flag
- Generated servers support both Phase 2 (dual-socket) and Phase 3 (single-socket negotiation)
- `validate tarpc` detects G65 compliance level (NONE → DEP_ONLY → PARTIAL → FULL → G65)
- 15 new tests covering wire format, roundtrips, duplex negotiation, backward compat
- 541 tests total, all cross-targets green (Linux, Windows, Android), clippy clean

### Wave 156j (August 6, 2026 — C6 Reference Implementation)
- **C6 DONE**: `sourdough_core::tarpc_service::PrimalService` — canonical tarpc trait for all primals
- 8 baseline methods: health_liveness, health_readiness, health_check, capabilities_list,
  identity_did, system_ping, system_version, lifecycle_state
- Response types: `HealthResponse`, `TarpcCapability`, `IdentityResponse`
- Client helpers: `connect_primal()`, `connect_primal_by_name()` (unix-gated)
- `default_tarpc_socket_path()` convention helper
- tarpc 0.37 + tokio-serde 0.9 in sourdough-core (not just scaffold templates)
- `validate tarpc` checks for baseline `PrimalService` method presence
- sourDough self-validates as FULL G64 compliance
- 523 tests, all 3 cross-targets green, clippy clean

### Wave 156h (August 6, 2026 — G64 Cephalization Scaffold)
- Scaffold templates emit dual-protocol primals: JSON-RPC on `.sock` + tarpc on `.tarpc.sock`
- New `tarpc_service.rs` template in core crate (service trait + response types + client `connect()`)
- New `tarpc_server.rs` template in server crate (UDS listener + handler bridge)
- Workspace deps template includes tarpc 0.37 + tokio-serde + futures
- Server `main.rs` starts tarpc as background task, JSON-RPC as lifecycle anchor
- `--disable-tarpc` CLI flag for JSON-RPC-only mode
- Clippy deep debt: raw string hashes, byte_char_slices, io_other_error, case_sensitive_extension
- `validate tarpc` subcommand — audit primals for G64 dual-protocol compliance
- Discovery types evolved: `ServiceInfo` + `ServiceRegistration` carry tarpc endpoints
- `ProtocolSupport` enum with `Display` (JsonRpcOnly / TarpcOnly / DualProtocol) for capability routing
- `TransportEndpoint::tarpc_endpoint()` — derive tarpc socket from JSON-RPC endpoint
- `IpcClient::tarpc_path()` — resolve tarpc socket path for any connected primal
- Doctor reports dual-protocol socket status ([dual]/[jsonrpc]/[tarpc-only])
- Core `Cargo.toml` template includes `tokio-serde` for client codec
- Property-based tests for discovery type roundtrips
- clap 4.5 → 4.6
- 518 tests, all 3 cross-targets green, clippy clean

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
