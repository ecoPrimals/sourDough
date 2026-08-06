# Wave 156l — C7: G65 Protocol Negotiation Reference Implementation

**Date**: August 6, 2026
**From**: eastGate
**Primal**: sourDough
**Work Item**: C7

---

## Summary

Extracted squirrel's protocol negotiation logic (432 lines) into `sourdough-core`
as the canonical G65 reference implementation. This module enables Phase 3 of
cephalization: a single socket that negotiates between tarpc and JSON-RPC at
connection time, replacing the Phase 2 dual-socket pattern.

---

## What Was Done

### New Module: `sourdough_core::protocol_negotiation`

| Component | Purpose |
|-----------|---------|
| `IpcProtocol` | Enum (`JsonRpc`, `Tarpc`) with wire format, serde, Display, parse |
| `NegotiationRequest` | Client → Server: `"PROTOCOLS: tarpc,jsonrpc\n"` |
| `NegotiationResponse` | Server → Client: `"PROTOCOL: tarpc\n"` |
| `negotiate_client()` | Async: send preferences, receive selection |
| `negotiate_server()` | Async: read request, select best, respond (timeout-aware) |
| `select_protocol()` | Preference-ordered selection with JSON-RPC fallback |
| `NegotiationError` | Typed error (InvalidRequest, NoValidProtocols, Timeout, Io, UnknownProtocol) |

### Scaffold Template Evolution

- Server `main.rs` gains `--negotiate` / `NEGOTIATE_PROTOCOL` flag
- When enabled, single socket runs `negotiate_server()` before routing
- Phase 2 (dual-socket) remains the default for backward compatibility
- Generated core crate includes `protocol_negotiation.rs` module
- `handle_negotiated_connection()` routes tarpc or JSON-RPC based on negotiation result

### Validation Tooling

- `validate tarpc` gains G65 compliance level detection
- Levels: NONE → DEP_ONLY → PARTIAL → FULL → **G65**
- JSON output includes `has_protocol_negotiation` field
- Detects patterns: `negotiate_server`, `negotiate_client`, `IpcProtocol`, `PROTOCOLS:`

---

## Wire Protocol

```text
Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
Server → Client: "PROTOCOL: tarpc\n"
[Connection proceeds with selected protocol]
```

**Backward compatibility**: If the server doesn't receive `PROTOCOLS:` within
the timeout (default 100ms), it assumes JSON-RPC. Legacy clients work unchanged.

---

## Test Coverage

15 new tests in `sourdough_core::protocol_negotiation::tests`:
- Wire format roundtrips (request + response)
- Parse edge cases (invalid prefix, unknown protocols)
- `select_protocol()` preference ordering
- Full duplex negotiation (tarpc preferred, jsonrpc-only, backward compat)
- Serde roundtrip for `IpcProtocol`

---

## Adoption Path

1. **sourDough** (this wave) — reference implementation in `sourdough-core`
2. **All 15 primals** — adopt `protocol_negotiation` module from their own core crate
3. **cellMembrane** — discovery aware of G65 (single socket = negotiation-capable)
4. **songBird** — routing transparent to protocol (negotiation happens at connection)

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 541 (up from 526) |
| New tests | 15 |
| Cross-targets | Linux + Windows + Android all green |
| Clippy | zero warnings |
| Deep debt | zero |

---

## Next Steps (not this wave)

- [ ] Full tarpc framing on negotiated stream (currently stub handoff)
- [ ] squirrel migrates to use `sourdough_core::protocol_negotiation` instead of local impl
- [ ] cellMembrane exposes negotiation-aware discovery
- [ ] songBird: protocol-transparent routing (connection negotiates, router doesn't care)
- [ ] All primals add `--negotiate` flag to their binaries
