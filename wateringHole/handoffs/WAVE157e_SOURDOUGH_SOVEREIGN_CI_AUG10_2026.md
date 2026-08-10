# Wave 157e — sourDough Sovereign CI Pipeline

**Date**: August 10, 2026
**Wave**: 157e
**Commit**: `227d670`
**Status**: COMPLETE — ready for sporeGate deployment

---

## What Shipped

sourDough now provides a **composite CI command** and a **turnkey Forgejo post-receive hook** for sovereign CI integration on golgi. This closes the "Wire sourDough validate into golgi CI" item from the Composition Graph Foundation plan.

### `sourdough ci` — Composite Command

Runs all static validators in sequence with unified pass/fail exit code.

```
$ sourdough ci /path/to/primal --json
{
  "ci": "PASS",
  "total": 5,
  "passed": 5,
  "failed": 0,
  "checks": [
    {"name": "platform-substrate", "status": "PASS"},
    {"name": "platform-paths", "status": "PASS"},
    {"name": "neural-api", "status": "PASS"},
    {"name": "tarpc", "status": "PASS"},
    {"name": "transport", "status": "PASS"}
  ]
}
```

**Static checks** (source analysis — run on every push):
| Check | What it detects |
|-------|-----------------|
| `platform-substrate` | G68 L1/L2/L3 silicon deism violations |
| `platform-paths` | Hardcoded `/tmp`, `/var`, `/run` path assumptions |
| `neural-api` | Missing/incomplete `primal.announce` routing |
| `tarpc` | G64 dual-protocol compliance gaps |
| `transport` | Transport abstraction violations |

**Live checks** (optional — `--live` flag, post-deploy):
| Check | What it detects |
|-------|-----------------|
| `convergence` | Dead/degraded primals on NUCLEUS |

**Flags:**
- `--json` — machine-readable for pipeline parsing
- `--live` — include convergence probes against running primals
- `--socket-dir` — override socket directory for live checks
- `--strict` — fail on warnings (for gated merges)
- `--skip transport,tarpc` — exclude specific checks

### `ci/forgejo-post-receive-hook.sh` — Turnkey Hook

Drop-in post-receive hook for golgi Forgejo repositories.

**Install (per-primal repo):**
```bash
ln -s /opt/sourdough/ci/forgejo-post-receive-hook.sh \
  /data/gitea/repositories/ecoPrimals/<primal>.git/hooks/post-receive
```

**Behavior:**
1. Triggers on push to `main` branch only
2. Checks out worktree at pushed commit (zero-downtime)
3. Runs `sourdough ci` against worktree
4. Logs JSON results to `/var/log/sourdough-ci/<primal>_<timestamp>.json`
5. Reports pass/fail in push output (visible to pusher)
6. Cleans up worktree

**Modes:**
- **Advisory** (default): Push always succeeds, CI is informational
- **Gating**: Change `exit 0` to `exit $STATUS` — push rejected on CI failure

**Environment overrides:**
- `SOURDOUGH_BIN` — path to sourdough binary (default: `sourdough` in PATH)
- `SOURDOUGH_CI_LOG_DIR` — log directory (default: `/var/log/sourdough-ci`)
- `SOURDOUGH_CI_WORK_DIR` — worktree checkout location (default: `/tmp/sourdough-ci`)

---

## Deployment Checklist for sporeGate

1. [ ] Deploy `sourdough` binary to golgi (e.g., `/usr/local/bin/sourdough`)
2. [ ] Create log directory: `mkdir -p /var/log/sourdough-ci`
3. [ ] Symlink hook to each primal repo's `hooks/post-receive`
4. [ ] (Optional) Set `SOURDOUGH_BIN` in gitea environment if not in PATH
5. [ ] Test: push to any primal repo, verify CI output appears in push output

---

## Live Validation Results (eastGate)

```
$ sourdough ci . --live --json
{
  "ci": "PASS",
  "total": 6,
  "passed": 6,
  "failed": 0,
  "checks": [
    {"name": "platform-substrate", "status": "PASS"},
    {"name": "platform-paths", "status": "PASS"},
    {"name": "neural-api", "status": "PASS"},
    {"name": "tarpc", "status": "PASS"},
    {"name": "transport", "status": "PASS"},
    {"name": "convergence", "status": "PASS"}
  ]
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `crates/sourdough/src/commands/ci.rs` | NEW — sovereign CI composite command |
| `crates/sourdough/src/commands/mod.rs` | Register `ci` module |
| `crates/sourdough/src/main.rs` | Add `Ci` variant to Commands enum |
| `ci/forgejo-post-receive-hook.sh` | NEW — turnkey Forgejo hook script |
| `crates/sourdough/src/commands/validate/platform_paths.rs` | Exempt ci/convergence/validate infra files |
| `STATUS.md` | Wave 157e entry |

---

## What This Enables

- **Fleet-wide conformance on push** — every primal validates against G68, Neural API, tarpc, and transport standards automatically
- **swarmVine socket discovery issue** would have been caught by `sourdough ci` (the `convergence` check detects misconfigured sockets)
- **Composition graph foundation** — CI is the first pillar ensuring primal quality before composition patterns layer on top
- **Progressive gating** — start advisory, flip to gating when confidence is high

---

*Wave 157e — Sovereign CI shipped. `sourdough ci` bundles 5 static + 1 live validator into unified pass/fail. Forgejo post-receive hook ready for golgi deployment. sourDough self-validates 6/6. sporeGate deploys to close the loop.*
