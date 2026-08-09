# Wave 157b — sourDough Vertebrate Evolution Tooling

**Date**: Aug 9, 2026 | **Commits**: `1cbac92`..`aa1a2f8` (4 commits)
**From**: eastGate overwatch | **Status**: DEPLOYED to golgi

---

## Summary

sourDough ships the self-audit tooling for Vertebrate Evolution. Every primal team
can now validate their platform abstractions, Neural API compliance, live convergence,
and RPC surface integrity from a single binary.

---

## Commits

| Hash | Description |
|------|-------------|
| `1cbac92` | Platform paths + signal abstraction (PrimalDirs, shutdown_signal) |
| `ead66ea` | Neural API routing validator + braid/convergence method constants |
| `edfa26e` | Live convergence validator (replaces convergence_check.py) |
| `aa1a2f8` | RPC surface audit — P0-A stub + P0-B divergence detection |

---

## New Validators (CI-ready)

### `sourdough validate rpc-surface --socket <path>`
- **P0-A detection**: Canary probe catches health-fallback stubs (bearDog pattern)
- **P0-B detection**: Declared methods returning -32601 (nestGate pattern)
- **Fresh connection per probe**: Handles one-shot primals
- **Compliance**: VERIFIED / STUB / DIVERGED / BROKEN
- **CI usage**: `sourdough validate rpc-surface --socket $SOCKET --methods $METHODS --json`

### `sourdough validate convergence [--socket-dir <dir>]`
- Probes all running primals in socket directory
- Reports: CONVERGED / PARTIAL / DRIFT / NO_PRIMALS
- **CI usage**: `sourdough validate convergence --json`

### `sourdough validate neural-api <path>`
- Static source scan for Neural API routing compliance
- Checks: dispatch methods, announce wire format, songBird registration, capabilities
- **CI usage**: `sourdough validate neural-api /path/to/primal --json`

### `sourdough validate platform-paths <path>`
- Detects hardcoded `/tmp`, `/var`, `/run` and raw XDG env reads
- **CI usage**: `sourdough validate platform-paths /path/to/primal --json`

---

## New Core Abstractions (scaffold-ready)

### `sourdough_core::platform_paths::PrimalDirs`
- Resolves config/data/runtime/cache/logs per-platform
- XDG (Linux), ~/Library (macOS), %APPDATA% (Windows), sandbox (mobile)
- `BIOMEOS_*_DIR` env overrides for test harness / launcher injection
- `ensure()` creates dirs with G68 permissions

### `sourdough_core::platform_signal`
- `shutdown_signal()` — SIGTERM/SIGINT (Unix), Ctrl+C (Windows)
- `on_shutdown(cleanup)` — fire-and-forget hook
- Scaffold server templates use `tokio::select!` for graceful exit

### `sourdough_core::methods::{convergence, braid}`
- Canonical method constants for primalSpring registry gap
- `convergence.check`, `convergence.batch_check`
- `braid.list`, `braid.query`, `braid.get_by_hash`, `braid.create`, etc.

---

## Live Validation Results (eastGate)

```
beardog-default.sock  → STUB     (P0-A confirmed: health fallback)
sweetgrass.sock       → VERIFIED (riboCipher enforcement, proper -32002)
convergence check     → PARTIAL  (1/8 alive, 7 degraded)
```

---

## CI Integration

Add to primal CI pipelines:

```yaml
# Static checks (per-primal, on push)
- run: sourdough validate neural-api . --json
- run: sourdough validate platform-paths . --json
- run: sourdough validate platform-substrate . --json

# Live checks (post-deploy, on NUCLEUS)
- run: sourdough validate convergence --json
- run: sourdough validate rpc-surface --socket /run/user/1000/biomeos/$PRIMAL.sock --json
```

---

## Stats

- **619 tests** green
- **16/16 cross-arch** verified
- **Clippy clean** (pedantic + nursery)
- **Zero unsafe**, zero C-deps, zero unwrap in production
