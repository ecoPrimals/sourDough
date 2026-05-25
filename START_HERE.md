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
cargo test --workspace
cargo build --release  # for local development; production binary from plasmidBin
```

---

## Repository Structure

```
sourDough/
├── Cargo.toml                     Workspace manifest (lints, deps, release profile)
├── crates/
│   ├── sourdough/                 CLI binary (scaffold, sign, verify, validate, layout, genomebin, doctor)
│   │   ├── src/commands/
│   │   │   ├── scaffold/          Primal scaffolding (mod + generators + templates)
│   │   │   ├── validate/          Compliance validation (primal, unibin, ecobin, composition)
│   │   │   ├── sign.rs            Ed25519 binary signing
│   │   │   ├── layout.rs          Triple-first layout validation
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
│           ├── signing.rs         Ed25519 detached signatures
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
sourdough scaffold new-primal myPrimal "Description" --output ../myPrimal
cd ../myPrimal && cargo build && cargo test
```

### Validate compliance

```bash
sourdough validate primal ../myPrimal
sourdough validate unibin ../myPrimal
sourdough validate ecobin ../myPrimal
sourdough validate ecobin primals/x86_64-unknown-linux-musl/myprimal  # binary checks
sourdough validate composition tower --primals-dir primals/
```

### Sign binaries

```bash
sourdough sign primals/x86_64-unknown-linux-musl/myprimal --generate-key
sourdough sign primals/x86_64-unknown-linux-musl/myprimal
sourdough verify primals/x86_64-unknown-linux-musl/myprimal
```

### Deployment tooling

```bash
sourdough scaffold systemd myPrimal --role gate
sourdough layout primals/  # validate triple-first layout
```

### Run diagnostics

```bash
sourdough doctor --comprehensive
```

---

## Where to Go Next

- **[Specification](specs/SOURDOUGH_SPECIFICATION.md)** -- what sourDough is
- **[Architecture](specs/ARCHITECTURE.md)** -- how it's built
- **[Roadmap](specs/ROADMAP.md)** -- where it's going
- **[Conventions](CONVENTIONS.md)** -- coding standards
- **[What's Next](WHATS_NEXT.md)** -- immediate priorities
