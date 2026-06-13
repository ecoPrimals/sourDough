# sourDough Wave 111 Handoff — riboCipher + Deep Debt

**Date**: 2026-06-13
**Version**: 0.4.0 (same semver, significant internals evolution)
**State**: riboCipher reference implementation complete. Deep debt resolved.

---

## What Shipped (Wave 111)

### riboCipher Transport Signal Standard

sourDough now implements the full RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD:

- **Reference implementation** (`sourdough-core/src/transport/ribocipher.rs`):
  - Signal constants: `SIGNAL_CLEAR` (0xEC), `SIGNAL_MITO` (0xED), `SIGNAL_NUCLEAR` (0xEE)
  - `ProtocolType` enum (Probe=0x00, NdjsonRpc=0x01, BtspBinary=0x02)
  - `detect_signal()` — async stream signal detection
  - `send_clear_signal()` — client-side signal prepend

- **Server reference** (`sourdough-core/src/transport/ribocipher_server.rs`):
  - `RiboCipherAcceptLoop` — canonical accept loop with routing
  - `UnsignalledPolicy` (Warn/Reject) for legacy connections
  - `ConnectionRoute` enum — typed routing decisions
  - `DetectionMeta` — signal tier, protocol, primal name

- **`sourdough validate ribocipher <path>`** — compliance audit subcommand
  - Reports: FULL, PARTIAL, or NONE compliance
  - Detects: signal detection, client sending, legacy handling, deprecation warnings
  - JSON output for CI: `--json` flag

- **Scaffold templates** emit riboCipher-compliant server code:
  - Generated `handle_connection` uses signal-first detection
  - Legacy `{` first-byte gets deprecation warning
  - `SIGNAL_CLEAR` constant present in all generated servers

- **Transport peek evolved**:
  - `Protocol::RiboCipher { protocol_type }` variant added
  - riboCipher signals consumed before stream handoff
  - Legacy JSON/binary still supported with `PeekedStream` replay

### Hardcoding → Capability-Based Discovery

- `"songbird"` mesh hub name → `MESH_RELAY_HUB` env var + `DEFAULT_MESH_RELAY_HUB` constant
- Port `50000` TCP fallback → `TCP_FALLBACK_PORT` env var + `DEFAULT_TCP_FALLBACK_PORT = 0`
- `"/tmp"` runtime fallback → `FALLBACK_RUNTIME_DIR` constant
- `"biomeos"` socket dir → `SOCKET_DIR_NAME` constant
- All in `env_keys.rs` — single source of truth for ecosystem conventions

### Dependency Evolution

- Removed dead `camino` dependency (zero usages in source)
- Moved `anyhow` from prod → dev-deps in genomebin (only in doc examples)
- Trimmed tokio from `features = ["full"]` to per-crate minimal sets:
  - core: `io-util`, `net`, `time`, `rt`
  - genomebin: `io-util`, `fs`, `rt`, `rt-multi-thread`
  - CLI: `macros`, `rt-multi-thread`, `fs`

### Smart Refactoring

- `server.rs` template (878L) decomposed into:
  - `server.rs` (372L) — transport + connection handling
  - `dispatch.rs` (185L) — JSON-RPC method routing
  - `method_gate.rs` (322L) — JH-0/JH-2 gate
- All production files now < 700 lines

### Test Coverage Expansion

- 437 tests total (up from 322 at Wave 107)
- New test modules: `peek.rs` (12 tests), `endpoint.rs` (16 tests), `ipc/client.rs` (9 tests)
- Zero test gaps in transport/IPC critical paths
- Full roundtrip test with mock UDS server

---

## Quality Summary

| Metric | Value |
|--------|-------|
| Tests | 437 passing, zero ignored |
| Clippy | zero warnings (pedantic + nursery) |
| Unsafe | zero (`forbid(unsafe_code)`) |
| unwrap/expect | zero in production library code |
| Mocks in production | zero (test-only) |
| C deps | zero |
| Max production file | 669 lines |
| riboCipher self-compliance | FULL |

---

## Ecosystem Impact

| Consumer | What Changed |
|----------|-------------|
| All new primals | Scaffold now emits riboCipher-compliant servers |
| primalSpring | `validate ribocipher` for ecosystem-wide compliance dashboard |
| songBird | Mesh relay hub name now env-configurable (no code change needed) |
| Existing primals | Can self-audit via `sourdough validate ribocipher .` |

---

## Downstream Action Items

| Primal | Action |
|--------|--------|
| All NUCLEUS primals | Add `[0xEC, 0x01]` signal prepend to client connections |
| songBird | Adopt riboCipher signal on accept loop (use `RiboCipherAcceptLoop` reference) |
| toadStool | Run `sourdough scaffold transport-kit toadStool` + re-scaffold with riboCipher |
| primalSpring | Add `validate ribocipher` to ecosystem audit pipeline |

---

## What's Next for sourDough

- v0.5.0 — Harvest + Package (pending upstream primal transport convergence)
- Phase 6: `sourdough harvest`, `sourdough package`, live composition probes
- Forgejo parity: `git.primals.eco` 500 error on push (infrastructure, not code)
