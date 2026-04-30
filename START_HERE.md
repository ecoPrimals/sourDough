# Start Here

New to sourDough? This guide gets you oriented.

---

## What is sourDough?

sourDough is the **nascent primal** for ecoPrimals. It has three jobs:

1. **Scaffold new primals** that are self-contained and independent
2. **Serve as reference implementation** of ecoPrimals standards
3. **Provide ecosystem tooling** for validation and genomeBin operations

---

## Prerequisites

- Rust 2024 edition (rustc 1.87+)
- `cargo-llvm-cov` for coverage (optional but recommended)

---

## Build and Test

```bash
cargo build --release
cargo test --workspace
```

---

## Repository Structure

```
sourDough/
├── Cargo.toml                     Workspace manifest (lints, deps, release profile)
├── crates/
│   ├── sourdough/                 CLI binary (scaffold, validate, genomebin, doctor)
│   │   ├── src/commands/
│   │   │   ├── scaffold/          Primal scaffolding (mod + generators + templates)
│   │   │   ├── validate.rs        Compliance validation
│   │   │   ├── genomebin.rs       genomeBin CLI commands
│   │   │   └── doctor.rs          Health diagnostics
│   │   └── tests/                 Integration + e2e tests
│   ├── sourdough-core/            Core traits library
│   │   └── src/
│   │       ├── lifecycle.rs       PrimalLifecycle trait + PrimalState
│   │       ├── health.rs          PrimalHealth trait + HealthReport
│   │       ├── identity.rs        PrimalIdentity trait + DID types
│   │       ├── discovery.rs       PrimalDiscovery trait
│   │       ├── config.rs          PrimalConfig trait + CommonConfig
│   │       ├── ipc.rs             JSON-RPC 2.0 IPC (primary)
│   │       ├── rpc.rs             Binary RPC (secondary, high-throughput)
│   │       ├── transport.rs       PeekedStream, socket path resolution
│   │       ├── error.rs           PrimalError types
│   │       └── types.rs           ContentHash, Timestamp
│   └── sourdough-genomebin/       Pure Rust genomeBin operations
│       └── src/
│           ├── platform.rs        Runtime OS/arch detection
│           ├── builder.rs         genomeBin creation pipeline
│           ├── validator.rs       genomeBin validation
│           ├── metadata.rs        TOML metadata handling
│           ├── archive.rs         tar/gzip operations
│           └── error.rs           Error types
├── specs/                         Specifications and architecture docs
├── CONVENTIONS.md                 Coding standards
├── STATUS.md                      Current compliance status
├── WHATS_NEXT.md                  Roadmap and next steps
└── CHANGELOG.md                   Version history
```

---

## Key Concepts

### Budding Primal Pattern

When sourDough scaffolds a new primal, the offspring is **self-contained**:
all core traits are inlined into the generated code. No compile-time or
runtime dependency on sourDough.

### IPC Architecture

- **JSON-RPC 2.0** (primary): semantic `domain.verb` method naming, newline-delimited
- **Binary RPC** (secondary): type-safe binary IPC for high-throughput paths
- `bytes::Bytes` for zero-copy wire format

### Primal Sovereignty

Primals know only themselves. They discover other primals at runtime via
capability-based addressing. No hardcoded service names, ports, or endpoints.

---

## Common Tasks

### Scaffold a new primal

```bash
./target/release/sourdough scaffold new-primal myPrimal "Description" --output ../myPrimal
cd ../myPrimal && cargo build && cargo test
```

### Validate compliance

```bash
./target/release/sourdough validate primal ../myPrimal
./target/release/sourdough validate unibin ../myPrimal
./target/release/sourdough validate ecobin ../myPrimal
```

### Run diagnostics

```bash
./target/release/sourdough doctor --comprehensive
```

---

## Where to Go Next

- **[Specification](specs/SOURDOUGH_SPECIFICATION.md)** -- what sourDough is
- **[Architecture](specs/ARCHITECTURE.md)** -- how it's built
- **[Roadmap](specs/ROADMAP.md)** -- where it's going
- **[Conventions](CONVENTIONS.md)** -- coding standards
- **[What's Next](WHATS_NEXT.md)** -- immediate priorities
