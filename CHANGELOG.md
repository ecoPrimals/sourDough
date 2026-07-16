# Changelog

All notable changes to sourDough will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Wave 142b — July 16, 2026)
- `Did::try_new()` — validated DID construction (rejects malformed input at boundary)
- `Did::method()` + `Did::method_specific_id()` — zero-alloc component extraction
- `Os::Android` + `LibC::Bionic` — proper platform detection for Android targets
- `Platform::is_android()` + `Platform::is_windows()` — API completeness
- Android cross-check passing (`cargo check --target aarch64-linux-android`)
- Proptest coverage for `Did` (serde roundtrip, validation, prefix rejection)
- Proptest coverage for `CommonConfig` (JSON/TOML roundtrip, instance_id invariant)

### Added (Wave 141a — July 15, 2026)
- `#![forbid(unsafe_code)]` on all 3 crate roots (compiler-enforced safety guarantee)
- Windows cross-architecture parity (`cargo check --target x86_64-pc-windows-gnu`)
- Platform-guarded `is_likely_binary()` with `#[cfg(unix)]` / `#[cfg(not(unix))]`
- `is_triple()` recognizes Windows targets
- `#[derive(Debug)]` on `TransportStream`
- Unit tests for `ipc/protocol.rs`, `transport/stream.rs`, `ipc/error.rs`, `ipc/capability.rs`
- Cross-platform tests for `layout.rs`
- Unit tests for `validate/` shared utilities

### Changed (Wave 141a–142b — July 15–16, 2026)
- Smart refactor: `validate/mod.rs` (669L → 400L) with `ecobin.rs` and `transport_compliance.rs`
- Test count: 473 → 502 (+29)
- Max production file: 669L → 608L

## v0.4.0 — Transport Ecosystem

### Added (v0.4.0 — June 2026)
- `sourdough-core::TransportEndpoint` — canonical wire format (serde tagged: uds/tcp/mesh_relay), wire-compatible with songbird_types
- `sourdough-core::connect_transport()` — transport-agnostic async stream connection
- `sourdough-core::IpcClient` — transport-aware JSON-RPC 2.0 client (call, call_with_timeout, health_liveness, register_capabilities, resolve_primal, announce)
- `sourdough-core::TransportStream` — unified async read/write across UDS/TCP (transport_name, set_nodelay)
- `sourdough-core::CircuitBreaker` — resilience pattern for inter-primal IPC
- `sourdough-core::methods` module — canonical `domain.verb` method constants (health, lifecycle, capabilities, identity, system, ipc, primal)
- `sourdough-core::env_keys` module — centralized env var name constants (TRANSPORT_ENDPOINT, BIOMEOS_SOCKET_DIR, XDG_RUNTIME_DIR, NEURAL_API_SOCKET)
- `TransportEndpoint::from_env_or_default()` — canonical injection entry point
- `TransportEndpoint::from_primal_name()` — ecosystem socket path conventions
- `TransportEndpoint::platform_default()` — cross-platform UDS/TCP default
- `sourdough validate transport <path>` — single primal transport compliance audit
- `sourdough validate transport-report --json` — ecosystem batch audit for CI/CD
- `sourdough validate depot --json` — binary freshness detection (--source, --stale-hours)
- `sourdough scaffold transport-kit <name>` — generates self-contained transport module (no sourdough-core dep)
- `sourdough migrate transport <path>` — migration tool for existing primals (dry-run + --apply)
- Scaffold templates emit transport-injected primals (TRANSPORT_ENDPOINT env var + CLI flag)
- Scaffold emits `ipc.register` call to songbird at startup via `announce::register_with_songbird`
- Release CI matrix includes `aarch64-linux-android` target
- IpcClient UDS roundtrip integration test (mock server validates full JSON-RPC exchange)
- `sourdough-core` re-exports `DEFAULT_IPC_TIMEOUT` (5 seconds)
- `IpcError` implements `std::fmt::Display` + `std::error::Error`
- `primal.announce` and `primal.shutdown` method constants
- `aarch64-linux-android` added to layout TIER1_TRIPLES

### Changed (v0.4.0)
- `colored` replaced with `owo-colors` (zero-alloc, zero transitive deps)
- `HealthProbe.status` evolved from `String` to `HealthStatus` enum (type-safe)
- `Timestamp::Display` now outputs proper ISO 8601 (`2024-06-09T01:46:40.000Z`) via chrono
- `PrimalRpcClient` documented as binary-protocol path (IpcClient is canonical)
- `validate depot` uses iterative traversal (fixes musl stack overflow segfault)
- `migrate transport` guidance updated: no sourdough-core dep, use scaffold transport-kit
- `config.rs` uses `env_keys::TRANSPORT_ENDPOINT` constant (was raw string)
- `ipc.rs` refactored: methods extracted to own module (801→745 lines), CircuitBreaker extracted
- Version bumped to 0.4.0

### Fixed (v0.4.0)
- `validate depot` segfault on musl (recursive walk → iterative stack-based traversal)
- Transport-report no longer flags sourDough itself for sourdough-core dependency
- Template-related false positives excluded from transport self-bind detection

## v0.3.1 — Neural API

### Added (v0.3.1 — May 23-25, 2026)
- Scaffold generates `announce.rs` in server crate: Neural API `primal.announce` startup logic (Wave 42/43 standard)
- Scaffolded primals auto-announce to biomeOS on startup for adaptive routing (fire-and-forget, graceful degradation)
- `primal.announce` classified as Public in MethodGate (JH-0 compliant)
- Tiered biomeOS socket discovery: `$NEURAL_API_SOCKET` → `$XDG_RUNTIME_DIR/biomeos/` → `/tmp/biomeos/`
- `dispatch.rs` METHODS constant includes `primal.announce` (advertised in capabilities.list)
- Announce template includes TODO markers for team-specific capabilities, signal_tiers, cost_hints, latency_estimates
- `.github/workflows/notify-plasmidbin.yml` added (Wave 49 — triggers plasmidBin auto-build on push)

### Fixed (v0.3.1)
- Inbound `primal.announce` handler no longer conflates capabilities with methods (Wave 44 audit)
  - `capabilities` field now returns domain names via `crate::announce::capabilities()`
  - `methods` field returns individual RPC method names from `METHODS` constant

### Changed (v0.3.1)
- Docs updated to plasmidBin-first patterns (Wave 49 post-primordial mandate)
- Scaffolded README template references plasmidBin as production channel
- Binary paths in docs use triple-first layout (`primals/<triple>/<name>`)

## v0.3.0 — Deployment Internalization

### Added (v0.3.0 — May 14, 2026)
- `sourdough sign <binary>` top-level CLI command: Ed25519 detached signatures (`.sig` sidecar)
- `sourdough sign --generate-key`: generates Ed25519 keypair (`signing.key` + `signing.pub`)
- `sourdough verify <binary>`: verifies Ed25519 signatures against public key
- `sourdough validate ecobin <binary>`: validates compiled binaries (static linking, stripped, size budget, ldd)
- `sourdough genomebin sign` now delegates to real Ed25519 signing (was `sequoia-openpgp` error stub)
- `sourdough scaffold systemd <primal>`: generates hardened systemd `.service` units (ecosystem membrane pattern)
- `sourdough layout <dir>`: validates triple-first binary layout (`primals/{triple}/{name}`)
- `sourdough validate composition <name>`: validates composition binary presence (tower, node, nest, nucleus, meta, full)
- Predefined compositions match `ports.env` atomic model (tower = beardog+songbird+skunkbat, etc.)
- 8 new CLI integration tests (sign roundtrip, tamper detection, systemd generation, layout, composition)
- Deployment internalization contract fully aligned (per `primalSpring/docs/SOURDOUGH_DEPLOYMENT_INTERNALIZATION.md`)

### Changed (v0.3.0)
- `sourdough validate ecobin` auto-detects path type: file (binary validation) vs directory (project validation)
- CLI help updated with `Sign`, `Verify`, `Layout` subcommands

## v0.2.0 — Scaffold Evolution

### Added (v0.2.0 — May 11, 2026)
- Scaffold generates `method_gate.rs` in server crate (JH-0/JH-2 pre-dispatch capability gate)
- MethodGate wired before dispatch: `gate.check(method)` with JSON-RPC error on denial
- Generated types: `MethodVisibility`, `GateMode`, `CallerContext`, `ResourceEnvelope`, `GateDenial`
- `classify_method()`: health.*, identity.get, capabilities.list, auth.*, lifecycle.status, btsp.negotiate → Public
- Ships in `GateMode::Permissive` (ecosystem default, zero behavioral change until JH-2)
- 11 unit tests in generated `method_gate.rs` (permissive, enforcing, allowlist, classify, serde)
- `deny.toml`: explicit `ring` ban across sourDough and scaffold template (ecosystem parity)

### Added (v0.2.0 — May 7, 2026)
- Scaffold generates `.github/workflows/release.yml` (Tier 1 musl cross-compilation: x86_64, aarch64, armv7 — SD-02 resolved)
- `sourdough-genomebin::signing` module: Ed25519 detached signatures for genomeBin artifacts (SD-03 resolved)
- Signing API: `generate_keypair`, `sign_file`, `verify_file`, `write_signature`, `read_signature`, `write_verifying_key`, `read_verifying_key`
- Pure Rust `ed25519-dalek` + `rand_core` dependencies (zero C deps, ecoBin-compliant)
- Release workflow: BLAKE3 checksums via `b3sum`, GitHub Release publishing via `softprops/action-gh-release@v2`
- 8 new signing tests + 5 new e2e assertions for release.yml

### Added (v0.2.0 — April 30, 2026)
- Scaffold generates `{name}-server` crate with JSON-RPC 2.0 server skeleton
- Scaffold generates `.github/workflows/ci.yml` (lean single-job CI)
- Scaffold generates `.github/workflows/notify-plasmidbin.yml` (genomeBin auto-distribution)
- Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- Scaffold generates `btsp.negotiate` handler (NULL cipher fallback for BTSP Phase 3 compatibility)
- `PeekedStream` transport utility in sourdough-core (ecosystem convergence for first-byte peek)
- `peek_protocol` async function for JSON-RPC vs BTSP auto-detection
- `resolve_socket_path` and `socket_path_in` for ecosystem socket naming convention
- Generated server: `dispatch.rs` with 4 capability wire handlers (health.liveness, health.readiness, health.check, capabilities.list)
- Generated server: `server.rs` with UDS listener, first-byte peek, newline-delimited JSON-RPC
- Generated server: `main.rs` with clap CLI (`--family-id` arg, `FAMILY_ID` env var)
- Enhanced e2e tests: 14 new assertions verifying deny.toml, CI workflows, server crate, dispatch handlers, socket naming

### Removed (v0.2.0)
- `tarpc` dependency (was only used for a proc macro annotation on `PrimalRpc` trait; generated code never consumed)
- `tokio-serde`, `bincode`, `tokio-util` dev-dependencies (unused, residual from tarpc exploration)
- 40 transitive dependencies eliminated (211 → 171 total)
- 3 advisory ignores from `deny.toml` (all were tarpc-transitive: RUSTSEC-2025-0141, RUSTSEC-2026-0007, RUSTSEC-2024-0387)

### Changed (v0.2.0)
- `PrimalRpc` trait: transport-agnostic async trait (was tarpc proc-macro annotated)
- `PrimalRpcClient::connect`: returns `std::io::Result` (was `Box<dyn Error>`)
- `bytes` updated to 1.11.1 (RUSTSEC-2026-0007 BytesMut overflow patched)
- Scaffold `ci.yml` now includes `cargo deny check` step (supply chain enforcement)
- Scaffold `deny.toml` now allows `cc` as wrapper for `blake3`/`iana-time-zone-haiku` (ecosystem standard)
- CONVENTIONS.md: JSON-RPC 2.0 is now documented as primary IPC (was incorrectly showing tarpc)
- Scaffolded core crate now includes `[lints] workspace = true` (was missing)
- Workspace Cargo.toml template adds `clap` to workspace dependencies
- Scaffolded workspace members now include both `-core` and `-server` crates
- README template updated with server crate structure and capability wire table
- `templates.rs` (862L) refactored into module directory: `core.rs` (440L), `server.rs` (319L), `infra.rs` (110L)
- Hardcoded primal names removed from CLI help and doc examples (rhizoCrypt, loamSpine → generic)
- `chrono` and `tempfile` deps aligned to `workspace = true` (was using local version pins)
- Broken `DEVELOPMENT.md` link in sourdough-genomebin README fixed → `CONVENTIONS.md`
- All root docs updated to 0.2.0-dev: README, STATUS, WHATS_NEXT, ROADMAP, ARCHITECTURE, START_HERE
- Binary artifacts (tar.gz) removed from `archive/` directory
- 247 tests passing (up from 239), zero files over 650 lines

### Added (prior)
- `deny.toml` supply chain auditing with ecoBin v3.0 C-sys ban list (16 crates)
- Workspace-level lint configuration (`[workspace.lints]`): pedantic, nursery, forbid(unsafe_code)
- Release profile optimizations: LTO, codegen-units=1, strip
- E2E tests: full scaffold -> build -> test -> validate lifecycle (2 tests)
- 5 new CLI integration tests for genomebin test/sign paths and doctor comprehensive
- Doctor genomeBin tools: real implementation with platform detection
- Parallel genomeBin processing via `tokio::task::JoinSet`
- `WHATS_NEXT.md` and `START_HERE.md` per CONVENTIONS.md requirements
- server --port N/A documented in specification (sourDough is meta-primal)

### Changed
- blake3 dependency uses `pure` feature (no C/asm build dependency)
- `tar` crate updated to 0.4.45 (fixes RUSTSEC-2026-0067, RUSTSEC-2026-0068)
- Removed cosmetic "BearDog" primal name from genomebin sign error message (Discovery A)
- Scaffold command refactored: `scaffold.rs` (789 lines) -> `scaffold/{mod,generators,templates}.rs` (max 438)
- All 3 ignored doctests rewritten to compile (native async trait syntax, edition 2024)
- `sourdough-genomebin` Cargo.toml migrated to workspace metadata
- Generated scaffold code emits workspace lints and release profile
- ARCHITECTURE.md updated with accurate file map and line counts (29 files, ~8100 lines)
- CONVENTIONS.md updated to reflect workspace-level linting
- README.md quality table updated (239 tests, 96%+ coverage, max file 637 lines)

### Removed
- Per-crate `#![forbid(unsafe_code)]` and `#![warn(clippy::...)]` (replaced by workspace lints)
- Dead code `#[expect(dead_code)]` on `parallel` field in builder.rs (now implemented)
- Monolithic `scaffold.rs` (replaced by module directory)

---

### Prior Session (April 3, 2026 — scaffold independence)

### Added
- Scaffold independence: scaffolded primals are fully self-contained with inlined core traits
- 54 new tests for sourdough-genomebin (validator, error, platform coverage)
- `EPHEMERAL_PRIMAL_SCAFFOLDING.md` spec for session-as-primal pattern
- JSON-RPC 2.0 primary IPC implementation with semantic `domain.verb` method naming
- `#![forbid(unsafe_code)]` on all crate roots
- `clippy::pedantic` + `clippy::nursery` enforced workspace-wide
- `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml` configuration files
- LICENSE file with scyBorg triple license

### Changed
- Upgraded Rust edition from 2021 to 2024
- `sourdough-genomebin` now uses workspace dependencies
- Replaced all `#[allow()]` with `#[expect(reason)]` per ecosystem standard
- Scaffold `new-primal` generates self-contained primals without sourdough-core dependency
- Scaffold `new-crate` uses path dependency to primal's own core crate
- Generated workspace Cargo.toml uses granular tokio features instead of `"full"`
- genomebin sign command now returns explicit error guiding toward pure Rust sequoia-openpgp

### Fixed
- 4 clippy `missing_const_for_fn` errors in sourdough-genomebin
- Orphaned `tests/e2e/rpc_communication.rs` now compiles and runs
- Deprecated `assert_cmd::Command::cargo_bin` usage
- Clippy `needless_raw_string_hashes` in scaffold templates
- Clippy `write_with_newline` in genomebin validator tests

### Removed
- 6 unused dependencies: `ed25519-dalek`, `config` (crate), `futures`, `walkdir`, `ignore`, `pathdiff`
- Unused `sysinfo` dependency from sourdough-genomebin
- `genomebin/` bash scripts directory (archived; replaced by Pure Rust sourdough-genomebin)
- `DEVELOPMENT.md` and `ECOBIN_CERTIFICATION.md` (archived as fossil record)
- `find_sourdough_core_path` function from scaffold (no longer needed)
- `find_genomebin_script` function from genomebin command

## [0.1.0] - 2026-01-19

### Added
- Initial `sourdough-core` library with traits: `PrimalLifecycle`, `PrimalHealth`, `PrimalIdentity`, `PrimalDiscovery`, `PrimalConfig`
- `sourdough` CLI binary with scaffold, validate, genomebin, doctor commands
- `sourdough-genomebin` library for Pure Rust genomeBin operations
- Comprehensive specs: SOURDOUGH_SPECIFICATION.md, ARCHITECTURE.md, ROADMAP.md
- 151 passing tests across all crates
