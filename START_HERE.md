# Start Here

New to sourDough? This guide gets you oriented.

---

## What is sourDough?

sourDough is the **nascent primal** for ecoPrimals. It has three jobs:

1. **Scaffold new primals** that are self-contained and independent
2. **Define the transport standard** (TransportEndpoint wire format, IPC patterns)
3. **Provide ecosystem tooling** for validation, migration, and depot management

---

## Prerequisites

- Rust 2024 edition (rustc 1.87+)
- `cargo-deny` for supply chain auditing

---

## Build and Test

```bash
cargo test --workspace
cargo build --release  # for local development; production binary from plasmidBin
```

---

## Repository Structure

```
sourDough/
├── Cargo.toml                     Workspace manifest (lints, deps, release profile)
├── compositions.toml              Named compositions (tower, nucleus, full, niche-*)
├── crates/
│   ├── sourdough/                 CLI binary
│   │   ├── src/commands/
│   │   │   ├── scaffold/          Primal scaffolding (mod + generators + templates)
│   │   │   ├── validate/          Compliance: transport, depot, composition, ecobin
│   │   │   ├── migrate.rs         Transport migration tool
│   │   │   ├── sign.rs            Ed25519 binary signing
│   │   │   ├── layout.rs          Triple-first layout validation
│   │   │   ├── genomebin.rs       genomeBin CLI commands
│   │   │   └── doctor.rs          Health diagnostics
│   │   └── tests/                 Integration + e2e tests
│   ├── sourdough-core/            Core traits + transport + IPC
│   │   └── src/
│   │       ├── transport/         TransportEndpoint, PeekedStream, riboCipher, socket
│   │       ├── ipc/              JSON-RPC 2.0: IpcClient, JsonRpcRequest/Response, IpcError
│   │       ├── methods.rs         Canonical domain.verb method constants
│   │       ├── circuit_breaker.rs CircuitBreaker resilience pattern
│   │       ├── env_keys.rs        Centralized env var + discovery constants
│   │       ├── config.rs          CommonConfig, ConfigLoader, ConfigWatcher
│   │       ├── health.rs          HealthStatus, PrimalHealth, HealthReport
│   │       ├── lifecycle.rs       PrimalLifecycle + PrimalState
│   │       ├── identity.rs        PrimalIdentity + DID types
│   │       ├── discovery.rs       PrimalDiscovery
│   │       ├── rpc.rs             Binary RPC (high-throughput path)
│   │       ├── error.rs           PrimalError types
│   │       └── types.rs           ContentHash, Timestamp
│   └── sourdough-genomebin/       Pure Rust genomeBin operations
│       └── src/
│           ├── platform.rs        Runtime OS/arch detection
│           ├── builder.rs         genomeBin creation pipeline
│           ├── validator.rs       genomeBin validation
│           ├── signing.rs         Ed25519 detached signatures
│           ├── metadata.rs        TOML metadata handling
│           ├── archive.rs         tar/gzip operations
│           └── error.rs           Error types
├── specs/                         Specifications and architecture docs
├── sporeprint/                    Validation summary for ecosystem dashboard
├── wateringHole/handoffs/         Active handoff documents
└── archive/                       Fossil record of past sessions
```

---

## Key Concepts

### Budding Primal Pattern

When sourDough scaffolds a new primal, the offspring is **self-contained**:
all core traits are inlined into the generated code. No compile-time or
runtime dependency on sourDough.

### Transport Standard

The canonical transport wire format is `TransportEndpoint` — a serde-tagged enum:
```json
{"transport": "uds", "path": "/run/user/1000/biomeos/myprimal.sock"}
{"transport": "tcp", "host": "127.0.0.1", "port": 7800}
{"transport": "mesh_relay", "peer_id": "strandgate", "capability": "security"}
```

Primals implement this locally (the wire format is the contract, not the library).
Use `sourdough scaffold transport-kit <name>` to generate a self-contained module.

### Primal Sovereignty

Primals know only themselves. They discover other primals at runtime via
capability-based addressing. No hardcoded service names, ports, or endpoints.
The launcher injects transport via `TRANSPORT_ENDPOINT` env var.

---

## Common Tasks

### Scaffold a new primal

```bash
sourdough scaffold new-primal myPrimal "Description" --output ../myPrimal
cd ../myPrimal && cargo build && cargo test
```

### Generate transport module for existing primal

```bash
sourdough scaffold transport-kit myPrimal
# Produces a self-contained transport.rs to drop into your crate
```

### Validate compliance

```bash
sourdough validate primal ../myPrimal
sourdough validate transport ../myPrimal
sourdough validate ribocipher ../myPrimal       # Wave 111 signal standard
sourdough validate transport-report --primals-dir ../ --json
sourdough validate depot primals/ --stale-hours 24
sourdough validate composition nucleus --primals-dir primals/ --triple-first
```

### Migrate existing primal to transport injection

```bash
sourdough migrate transport ../myPrimal          # dry-run report
sourdough migrate transport ../myPrimal --apply  # apply changes
```

### Sign and verify binaries

```bash
sourdough sign --generate-key
sourdough sign primals/x86_64-unknown-linux-musl/myprimal
sourdough verify primals/x86_64-unknown-linux-musl/myprimal
```

### Run diagnostics

```bash
sourdough doctor --comprehensive
```

---

## Where to Go Next

- **[Specification](specs/SOURDOUGH_SPECIFICATION.md)** — what sourDough is
- **[Architecture](specs/ARCHITECTURE.md)** — how it's built
- **[Roadmap](specs/ROADMAP.md)** — where it's going
- **[Conventions](CONVENTIONS.md)** — coding standards
- **[What's Next](WHATS_NEXT.md)** — immediate priorities
