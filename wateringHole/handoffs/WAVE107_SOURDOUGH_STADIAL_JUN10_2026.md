# sourDough Wave 107 Handoff — Stadial Ready

**Date**: 2026-06-10
**Version**: 0.4.0
**State**: Zero development debt. Zero P1. All tooling operational.

---

## What Shipped (v0.4.0)

### Transport Ecosystem (canonical standard for all primals)

- `TransportEndpoint` wire format — serde tagged enum (uds/tcp/mesh_relay)
- `connect_transport()` — transport-agnostic async stream connection
- `IpcClient` — JSON-RPC 2.0 client with timeout, liveness, resolve, announce
- `TransportStream` — unified async read/write (transport_name, set_nodelay)
- `CircuitBreaker` — resilience pattern for inter-primal IPC
- `methods` module — canonical domain.verb constants (health, lifecycle, ipc, primal, system)
- `TransportEndpoint::from_env_or_default()` — canonical injection entry point
- `env_keys` module — TRANSPORT_ENDPOINT, BIOMEOS_SOCKET_DIR, XDG_RUNTIME_DIR

### CLI Tooling

- `sourdough validate transport` — single primal transport compliance audit
- `sourdough validate transport-report --json` — ecosystem batch audit for CI
- `sourdough validate depot --json` — binary freshness detection for cellMembrane
- `sourdough scaffold transport-kit` — self-contained transport module (no sourdough-core dep)
- `sourdough migrate transport` — migration tool for existing primals
- `sourdough validate composition` — composition binary presence (tower/nucleus/full/niche)

### Quality

- 322 tests, zero ignored
- All production files < 800 lines
- Zero unsafe, zero C deps, pure Rust entire tree
- clippy pedantic + nursery clean
- deny.toml passing (ring, openssl banned)
- Release CI: x86_64-musl, aarch64-musl, armv7-musleabihf, aarch64-linux-android

---

## Ecosystem Impact

| Consumer | How They Use sourDough |
|----------|----------------------|
| 13 NUCLEUS primals | Scaffolded patterns, TransportEndpoint wire format |
| cellMembrane | `validate depot --json` for automated peptidoglycan rebuild |
| primalSpring | `validate transport-report` for ecosystem compliance dashboard |
| plasmidBin | genomeBin CI/CD pipeline |
| All gates | `validate composition` for deployment verification |

---

## Upstream Dependencies (what sourDough needs from others)

| Primal | What | Status |
|--------|------|--------|
| songBird | `ipc.resolve` returning TransportEndpoint | **RESOLVED** (ff86204c — topology-aware mesh routing) |
| biomeOS | Auto-register primals with songBird after launch | **RESOLVED** (v4.19, 421433dc) |
| — | **All blockers cleared** — sourDough has zero upstream dependencies | — |

### Response to SONGBIRD-IPC-RESOLVE-M1 Resolution

With songBird's `ipc.resolve` now returning `MeshRelay` endpoints, sourDough evolved:
- `IpcClient::call()` now transparently routes MeshRelay through local songBird via `capability.call`
- `IpcClient::resolve_and_connect()` — canonical one-call discovery flow
- `methods::capability::CALL` constant — the mesh relay forwarding method
- `MESH_RELAY_TIMEOUT` (15s) — accounts for WAN hop latency

### Response to Bind Abstraction (grapheneGate/Pixel 8)

Android SELinux denies UDS `bind()` in non-standard paths. sourDough now provides:
- `BindMode` enum in `sourdough-core::bind_mode` (Uds / TcpOnly / Both)
- `PRIMAL_BIND_MODE` env var in `env_keys`
- Scaffold templates check `PRIMAL_BIND_MODE=tcp_only` before UDS bind
- `validate transport-report` detects bind-mode awareness as a platform guard
- **Net effect**: newly scaffolded primals work on grapheneGate from day one

---

## Downstream Gaps Found (for upstream primal teams)

Identified via `sourdough validate transport-report` ecosystem audit:

| Primal | Issue | Recommendation |
|--------|-------|----------------|
| toadStool | No TransportEndpoint adoption (last remaining) | Run `sourdough scaffold transport-kit toadStool` |
| toadStool, coralReef, barraCuda, sweetGrass, squirrel | Writes to /tmp even when --socket passed | Use `SOCKET_DIR` from env, not `/tmp` |
| skunkBat | Hardcoded TCP port 9750 | Migrate to UDS-only, TCP as fallback |

---

## What's Next for sourDough

v0.5.0 — Harvest + Package (blocked on upstream primals completing transport):
- `sourdough harvest --all` — cross-compile per sources.toml
- `sourdough package` — self-extracting genomeBin archives
- `sourdough validate composition` Phase 3 live health probes

---

## Depot Status

sourDough binary in depot is STALE (pre-v0.4.0). Needs rebuild via peptidoglycan
to pick up: validate depot fix (segfault), transport-report --json, depot --json.

**Action**: cellMembrane peptidoglycan rebuild → push to VPS depot.
