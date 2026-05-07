# sourDough Status

**Version**: 0.2.0-dev (unreleased)
**Edition**: Rust 2024
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)

## Current State

- `sourdough-core`: Core traits library (PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig) + JSON-RPC 2.0 IPC + zero-copy RPC + PeekedStream transport
- `sourdough`: CLI binary (scaffold, validate, genomebin, doctor)
- `sourdough-genomebin`: Pure Rust genomeBin operations

## Compliance

- [x] `forbid(unsafe_code)` via workspace lints on all crates
- [x] `clippy::pedantic` + `clippy::nursery` zero warnings (workspace-level `[workspace.lints]`)
- [x] All `#[allow()]` replaced with `#[expect(reason)]`
- [x] `cargo fmt` clean
- [x] `cargo doc` zero warnings, all doctests compile (0 ignored)
- [x] Zero C application dependencies (Pure Rust, blake3 `pure` feature)
- [x] `cargo deny check` passing (ecoBin v3.0 C-sys ban list, supply chain audit)
- [x] Zero hardcoded primal names in crate code (Discovery grade A)
- [x] JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- [x] Binary RPC secondary high-throughput path with `bytes::Bytes` zero-copy
- [x] Edition 2024
- [x] scyBorg triple license (AGPL-3.0-or-later, ORC, CC-BY-SA-4.0)
- [x] 95%+ test coverage via `cargo llvm-cov` (247 tests, target: 90%)
- [x] Scaffold independence: scaffolded primals are self-contained (no sourdough-core dependency)
- [x] Release profile: LTO, codegen-units=1, strip
- [x] scaffold.rs refactored into module (mod + generators + templates)
- [x] doctor genomeBin tools: real implementation with platform detection
- [x] Parallel genomeBin processing implemented
- [x] E2E tests: scaffold -> build -> test -> validate lifecycle
- [x] WHATS_NEXT.md and START_HERE.md documentation
- [x] `deny.toml` supply chain auditing (SD-01 resolved)
- [x] `tar` crate updated to 0.4.45 (RUSTSEC-2026-0067, RUSTSEC-2026-0068 resolved)
- [x] **v0.2.0**: Scaffold generates `{name}-server` crate with JSON-RPC server + capability wire handlers
- [x] **v0.2.0**: Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml`
- [x] **v0.2.0**: Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- [x] **v0.2.0**: `PeekedStream` transport utility in sourdough-core (ecosystem convergence)
- [x] **v0.2.0**: Socket path resolution (`$XDG_RUNTIME_DIR/biomeos/{name}-{family_id}.sock`)
- [x] **v0.2.0**: First-byte peek in generated server (JSON-RPC vs BTSP auto-detection)
- [x] **v0.2.0**: Capability wire standard (health.liveness, health.readiness, health.check, capabilities.list)
- [x] **v0.2.0**: CONVENTIONS.md drift fixed (JSON-RPC 2.0 primary, binary RPC secondary)
- [x] **v0.2.0**: Scaffold core crate now inherits `[lints] workspace = true`
- [x] **v0.2.0**: Scaffold `ci.yml` now includes `cargo deny check` step
- [x] **v0.2.0**: Scaffold `deny.toml` allows `cc` wrapper for blake3 (ecosystem standard)
- [x] **v0.2.0**: `tarpc` removed (unused, 40 deps eliminated); `PrimalRpc` is transport-agnostic
- [x] **v0.2.0**: `PrimalRpcClient::connect` returns `std::io::Result` (was `Box<dyn Error>`)
- [x] **v0.2.0**: `bytes` patched 1.11.1 (RUSTSEC-2026-0007); deny.toml advisory ignores cleared
- [x] **v0.2.0**: Total dependencies: 171 (down from 211)
- [x] **v0.2.0**: Scaffold generates `btsp.negotiate` handler (NULL cipher fallback, BTSP Phase 3 ready)
- [x] **v0.2.0**: Scaffold generates `release.yml` (musl cross-compilation matrix: x86_64, aarch64, armv7) — SD-02 resolved
- [x] **v0.2.0**: genomeBin Ed25519 signing module (`ed25519-dalek`, pure Rust, zero C deps) — SD-03 resolved

## Crate Health

| Crate | Tests | Coverage | Max Lines |
|-------|-------|----------|-----------|
| sourdough-core | 135 | ~95% | all < 650 |
| sourdough (CLI) | 25+ (integration + e2e) | ~90% | all < 540 |
| sourdough-genomebin | 87 | ~96% | all < 560 |

## Recent Changes (May 7, 2026 — SD-02/SD-03 resolution)

- Scaffold generates `.github/workflows/release.yml` (Tier 1 musl cross-compilation matrix: x86_64, aarch64, armv7)
- `sourdough-genomebin` signing module: Ed25519 detached signatures (BLAKE3 hash → sign → `.sig` sidecar)
- Pure Rust `ed25519-dalek` + `rand_core` — zero C dependencies, ecoBin-compliant
- Signing API: `generate_keypair`, `sign_file`, `verify_file`, `write_signature`, `read_signature`, `write_verifying_key`, `read_verifying_key`
- 8 new signing tests (keypair gen, sign/verify roundtrip, tamper detection, key persistence)
- E2e tests: 5 new assertions for release.yml (musl targets, BLAKE3 checksums, GitHub Releases)
- 255 tests passing (up from 247)
- SD-02 (musl cross-compilation) and SD-03 (genomeBin signing) both resolved

## Prior Changes (May 2, 2026 — v0.2.0 scaffold evolution)

- Scaffold now generates `{name}-core` + `{name}-server` crates (JSON-RPC server with capability wire standard)
- Scaffold generates `.github/workflows/ci.yml` + `notify-plasmidbin.yml` (CI + genomeBin distribution)
- Scaffold generates `deny.toml` (ecoBin v3.0 supply chain auditing)
- `PeekedStream` + `peek_protocol` added to sourdough-core (first-byte protocol auto-detection)
- `resolve_socket_path` + `socket_path_in` added to sourdough-core (ecosystem socket naming)
- CONVENTIONS.md corrected: JSON-RPC 2.0 as primary IPC, binary RPC as secondary
- tarpc dependency removed (40 transitive deps eliminated); PrimalRpc is now transport-agnostic
- `bytes` patched to 1.11.1 (RUSTSEC-2026-0007 resolved)
- deny.toml advisory ignores cleared (all were tarpc-transitive)
- Generated core crate now inherits `[lints] workspace = true`
- 247 tests passing (up from 239), enhanced e2e assertions for v0.2.0 artifacts
- Generated server includes: dispatch with 5 capability handlers (+ `btsp.negotiate`), first-byte peek, socket naming, tracing
- Scaffold `ci.yml` now enforces `cargo deny check`
- Scaffold `deny.toml` allows `cc` wrapper for blake3 (ecosystem standard)

## Prior Changes (April 3, 2026)

- Workspace-level lint configuration (`[workspace.lints]`) replaces per-crate `#![warn]`
- Release profile optimizations (LTO, codegen-units=1, strip)
- scaffold.rs refactored: mod.rs (command dispatch) + generators.rs (file writing) + templates.rs (primal DNA)
- All 3 ignored doctests fixed to compile (native async trait syntax, no async_trait dependency)
- Parallel genomeBin processing implemented (concurrent ecoBin pre-reading)
- doctor genomeBin tools: real implementation (platform detection, library validation)
- 5 new CLI integration tests (genomebin test/sign paths, doctor comprehensive output)
- 2 new e2e tests (scaffold -> build -> test -> validate lifecycle)
- WHATS_NEXT.md and START_HERE.md created per CONVENTIONS.md requirements
- server --port N/A documented (sourDough is meta-primal, not a daemon)
- sourdough-genomebin Cargo.toml migrated to workspace metadata
- Generated scaffold code updated to use workspace lints
