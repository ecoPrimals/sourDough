# sourDough Status

**Version**: 0.4.0
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits library + JSON-RPC 2.0 IPC + TransportEndpoint + IpcClient + CircuitBreaker + PeekedStream + zero-copy RPC
- `sourdough`: CLI binary (scaffold, validate, sign, verify, layout, migrate, doctor)
- `sourdough-genomebin`: Pure Rust genomeBin operations

## Compliance

- [x] `forbid(unsafe_code)` via workspace lints on all crates
- [x] `clippy::pedantic` + `clippy::nursery` zero warnings (workspace-level `[workspace.lints]`)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] `cargo fmt` clean
- [x] `cargo doc` zero warnings, all doctests compile (0 ignored)
- [x] Zero C application dependencies (Pure Rust entire dependency tree)
- [x] `cargo deny check` passing (ecoBin C-sys ban, explicit `ring` ban, supply chain audit)
- [x] Zero hardcoded primal names in production code (Discovery grade A)
- [x] JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- [x] Binary RPC secondary high-throughput path with `bytes::Bytes` zero-copy
- [x] Edition 2024
- [x] scyBorg triple license (AGPL-3.0-or-later, ORC, CC-BY-SA-4.0)
- [x] 321 tests, zero ignored
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [x] Transport injection: primals accept `TRANSPORT_ENDPOINT` env var
- [x] All production files under 800 lines
- [x] Release CI: x86_64-musl, aarch64-musl, armv7-musleabihf, aarch64-linux-android
- [x] Cross-arch proven (Pixel 8 / GrapheneOS deployment validated)

## Crate Health

| Crate | Tests | Max Lines |
|-------|-------|-----------|
| sourdough-core | 161 | 781 (ipc.rs) |
| sourdough (CLI) | 60 (26 unit + 34 integration) | 785 (templates/server.rs) |
| sourdough-genomebin | 87 | 553 (validator.rs) |
| doctests | 11 | — |

## v0.4.0 (June 2026 — Transport Ecosystem)

- `sourdough-core::TransportEndpoint` — wire-compatible with songbird_types (serde tagged: uds/tcp/mesh_relay)
- `sourdough-core::connect_transport()` — transport-agnostic stream connection
- `sourdough-core::IpcClient` — transport-aware JSON-RPC 2.0 client with timeout + liveness + resolve + announce
- `sourdough-core::TransportStream` — unified async read/write across UDS/TCP
- `sourdough-core::CircuitBreaker` — resilience pattern for IPC
- `sourdough-core::methods` module — canonical `domain.verb` method constants
- `sourdough-core::env_keys` — centralized env var name constants
- `TransportEndpoint::from_env_or_default()` — canonical entry point for transport injection
- `sourdough validate transport` — single primal transport compliance audit
- `sourdough validate transport-report` — ecosystem batch audit (--json for CI)
- `sourdough validate depot` — binary freshness detection (--json, --source, --stale-hours)
- `sourdough scaffold transport-kit` — self-contained transport module for other primals
- `sourdough migrate transport` — migration tool for existing primals
- Scaffold templates emit transport-injected primals (TRANSPORT_ENDPOINT env + CLI)
- Scaffold emits `ipc.register` call to songbird at startup
- Release CI includes `aarch64-linux-android` target
- `colored` dep → `owo-colors` (zero-alloc, zero transitive deps)
- `HealthProbe.status` evolved from String to `HealthStatus` enum
- `Timestamp::Display` proper ISO 8601 via chrono
- 321 tests (up from 281)

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
