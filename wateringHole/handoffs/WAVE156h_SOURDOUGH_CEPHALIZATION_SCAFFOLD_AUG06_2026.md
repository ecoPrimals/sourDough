# Wave 156h — sourDough Cephalization Scaffold

**Date**: August 6, 2026
**From**: eastGate (sourDough team)
**Theme**: G64 Cephalization — scaffolded primals born dual-protocol

---

## Summary

sourDough scaffold templates now emit **dual-protocol primals** aligned with the G64 Cephalization convergence goal. New primals are born with:

- **JSON-RPC on `{name}.sock`** — bootstrap, discovery, diagnostics, browser
- **tarpc on `{name}.tarpc.sock`** — intra-gate composition, sub-ms binary framing

This means every newly scaffolded primal is immediately ready for tarpc composition without any manual wiring.

---

## Changes

### New Templates

| File | Location | Purpose |
|------|----------|---------|
| `tarpc_service.rs` | Core crate | `#[tarpc::service]` trait definition + response types |
| `tarpc_server.rs` | Server crate | UDS listener, `BaseChannel`, handler bridge |

### Modified Templates

| File | Change |
|------|--------|
| `templates/mod.rs` | New module + re-export |
| `templates/core.rs` | `lib.rs` includes `pub mod tarpc_service;` + tarpc workspace dep |
| `templates/server.rs` | Server Cargo.toml adds tarpc/tokio-serde/futures; `main.rs` starts tarpc listener |
| `generators.rs` | Workspace deps include tarpc 0.37 + features; writes `tarpc_service.rs` + `tarpc_server.rs` |

### Architecture

```
{name}-core/src/
├── tarpc_service.rs    ← NEW: #[tarpc::service] trait
├── lib.rs              ← exports tarpc_service module
└── ...

{name}-server/src/
├── tarpc_server.rs     ← NEW: listener + handler bridge
├── main.rs             ← starts tarpc (background) + JSON-RPC (anchor)
└── ...
```

### CLI Addition

`--disable-tarpc` flag on scaffolded primals for JSON-RPC-only mode (testing, constrained environments).

### Clippy Debt Cleared

- `unnecessary_raw_string_hashes` in ribocipher.rs (3 instances)
- `byte_char_slices` in transport/mod.rs
- `io_other_error` in ipc/protocol.rs test
- `case_sensitive_file_extension_comparisons` in endpoint.rs + client.rs tests

---

## Upstream Impact

### For Existing Primals (C2 — UDS Protocol Convergence)

The dual-socket pattern scaffolded here is the **canonical reference** for C2. Existing primals evolving to dual-protocol should follow this pattern:
1. Add a tarpc service trait to their core crate
2. Start a `.tarpc.sock` listener alongside their JSON-RPC socket
3. Bridge the tarpc trait to their existing primal implementation

### For sporeGate Re-harvest

sourDough itself doesn't need a tarpc socket (it's a CLI tool). But the templates it generates for primals do. After this change:
- `sourdough scaffold new-primal X` → X has tarpc ready
- X compiles immediately with `cargo build`
- X serves both protocols on startup

### Dependencies Introduced (in scaffolded output only)

| Dep | Version | Purpose |
|-----|---------|---------|
| `tarpc` | 0.37 | Binary RPC framework |
| `tokio-serde` | 0.9 | Bincode codec for tarpc transport |
| `futures` | 0.3 | `StreamExt` for tarpc accept loop |

These are in the workspace template only — sourDough's own `Cargo.toml` is unchanged.

---

## Verification

- 518 tests passing
- Clippy clean (zero warnings)
- `cargo fmt` clean
- `cargo check --target x86_64-pc-windows-gnu` ✓
- `cargo check --target aarch64-linux-android` ✓
- E2E scaffold tests: `scaffold_build_test_validate` + `scaffold_add_crate_build` ✓

---

## Completed (Post-Initial Scaffold)

| Item | Status | Notes |
|------|--------|-------|
| tarpc client template | **DONE** | Core template emits `connect()` function for primal-to-primal calls |
| `sourdough validate tarpc` | **DONE** | Audits primals for G64 dual-protocol compliance |
| Dual-socket health probe | **DONE** | `sourdough doctor` reports [dual]/[jsonrpc]/[tarpc-only] per socket |
| `TransportEndpoint::tarpc_endpoint()` | **DONE** | Derives `.tarpc.sock` from `.sock` endpoint |
| `IpcClient::tarpc_path()` | **DONE** | Resolves tarpc socket path for any connected primal |
| `ProtocolSupport` Display impl | **DONE** | Renders as "jsonrpc"/"tarpc"/"dual" for diagnostics |
| clap 4.5 → 4.6 | **DONE** | Latest CLI framework |

---

## Remaining Work

| Item | Priority | Notes |
|------|----------|-------|
| tarpc live health probe | P3 | Doctor could attempt tarpc handshake (requires tarpc dep in CLI) |
| `sourdough validate tarpc --fleet` | P3 | Batch validate all primals in a directory |

---

**sourDough role in G64**: scaffold the pattern. Other primals evolve to it independently (convergent evolution). The scaffold ensures new primals start at the convergence target. sourDough itself provides the tooling to validate compliance and the transport primitives to derive tarpc endpoints from JSON-RPC endpoints.
