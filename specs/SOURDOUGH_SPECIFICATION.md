# sourDough Specification

**Version**: 0.2.0-dev
**Date**: April 30, 2026
**Status**: Reference Implementation
**Role**: Nascent Primal (Budding Primal) + Standardization Framework

---

## Purpose

sourDough serves three functions in the ecoPrimals ecosystem:

### 1. Nascent Primal (Budding Primal)

sourDough scaffolds new primals that are **self-contained and independent**.
When sourDough creates a new primal, the result has its own inlined core traits,
its own workspace, and zero runtime dependency on sourDough. Like biological
budding, the offspring is complete.

Core traits provided to scaffolded primals:
- `PrimalLifecycle` (state machine: Created, Starting, Running, Stopping, Stopped, Failed)
- `PrimalHealth` (observability: Healthy, Degraded, Unhealthy)
- `PrimalState`, `PrimalError`, `HealthStatus`, `HealthReport`

### 2. Reference Implementation

sourDough itself is a primal demonstrating ecoPrimals standards:
- UniBin architecture (single binary, multiple subcommands)
- ecoBin compliance (Pure Rust, zero C dependencies, cross-compilation)
- JSON-RPC 2.0 primary IPC with semantic `domain.verb` method naming
- tarpc secondary high-throughput IPC
- Capability-based discovery, zero hardcoding
- `#![forbid(unsafe_code)]` on all crates

### 3. Standardization Framework

sourDough provides tooling for the ecosystem:
- Validation tools: check primal, UniBin, ecoBin compliance
- genomeBin library: Pure Rust platform detection, metadata, archive, validation
- Documentation templates for specifications, architecture, roadmaps

---

## Architecture

### sourDough as UniBin

Single binary, multiple modes. Note: sourDough is a meta-primal (tooling/scaffolding),
not a long-running service. The `server --port` subcommand specified by
`UNIBIN_ARCHITECTURE_STANDARD.md` does not apply because sourDough has no
daemon mode — all operations are CLI commands that complete and exit.

```bash
# Scaffolding (output is self-contained, no sourDough dependency)
sourdough scaffold new-primal <name> "<description>" [--output <dir>]
sourdough scaffold new-crate <primal> <crate>

# genomeBin operations (Pure Rust)
sourdough genomebin create --primal <name> --version <ver> --ecobins <dir>
sourdough genomebin test <genomeBin>
sourdough genomebin sign <genomeBin>

# Validation
sourdough validate primal <primal-dir>
sourdough validate unibin <primal-dir>
sourdough validate ecobin <primal-dir>

# Diagnostics
sourdough doctor [--comprehensive]
```

### sourDough as ecoBin

- Zero C dependencies (Pure Rust)
- Cross-compiles to x86_64, ARM64, RISC-V (musl targets)
- Static linking
- Self-contained binary

---

## Components

### 1. sourdough-core (Library)

Core traits for all primals. Note: scaffolded primals receive **inlined copies**
of essential traits, not a dependency on this crate. This crate is sourDough's
own implementation.

**Exports**:

```rust
pub trait PrimalLifecycle {
    fn state(&self) -> PrimalState;
    async fn start(&mut self) -> Result<(), PrimalError>;
    async fn stop(&mut self) -> Result<(), PrimalError>;
    async fn reload(&mut self) -> Result<(), PrimalError>;
}

pub trait PrimalHealth {
    fn health_status(&self) -> HealthStatus;
    async fn health_check(&self) -> Result<HealthReport, PrimalError>;
}

pub trait PrimalIdentity {
    fn did(&self) -> &Did;
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError>;
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError>;
}

pub trait PrimalDiscovery {
    fn capabilities(&self) -> Vec<UpaCapability>;
    async fn register(&self) -> Result<ServiceRegistration, PrimalError>;
    async fn announce(&self) -> Result<(), PrimalError>;
}

pub trait PrimalConfig {
    type Config;
    fn load(path: &Path) -> Result<Self::Config, PrimalError>;
    fn validate(&self) -> Result<(), PrimalError>;
}
```

Also provides:
- JSON-RPC 2.0 IPC (`ipc.rs`): newline-delimited, `domain.verb` naming, circuit breaker
- tarpc RPC (`rpc.rs`): type-safe binary IPC for high-throughput paths
- Common types (`types.rs`): `Did`, `ContentHash`, `Timestamp`

### 2. sourdough (UniBin CLI)

Command-line tool for primal management. Scaffolding produces self-contained
primals with inlined trait definitions and no external sourdough-core dependency.

### 3. sourdough-genomebin (Pure Rust Library)

Pure Rust genomeBin operations:

| Module | Purpose |
|--------|---------|
| `platform.rs` | Runtime OS/arch/libc detection, target triple generation |
| `metadata.rs` | Type-safe TOML metadata parsing |
| `archive.rs` | Pure Rust tar/gzip operations |
| `builder.rs` | genomeBin creation pipeline |
| `validator.rs` | Comprehensive genomeBin validation |
| `error.rs` | Structured error types |

---

## Scaffold Independence

When sourDough scaffolds a new primal:

1. A workspace is generated with the primal's own `-core` crate
2. Core traits are **inlined** into the generated code (not imported from sourdough-core)
3. Each additional crate uses a path dependency to the primal's own core
4. Generated `Cargo.toml` uses granular tokio features
5. Generated `CONVENTIONS.md` and `README.md` reference self-contained structure

The scaffolded primal is immediately buildable and testable with `cargo build`
and `cargo test`, with zero knowledge of sourDough's existence.

---

## Workflow: New Primal Lifecycle

### Phase 1: Nascent (sourDough scaffold)

```bash
sourdough scaffold new-primal myPrimal "Description" --output ../myPrimal
# Result: self-contained workspace with inlined core traits
```

### Phase 2: Development

```bash
cd myPrimal/
cargo build    # Builds immediately, no external dependencies on sourDough
cargo test     # Tests pass out of the box
```

### Phase 3: UniBin (single binary)

```bash
sourdough validate unibin myPrimal/
```

### Phase 4: ecoBin (Pure Rust)

```bash
sourdough validate ecobin myPrimal/
```

### Phase 5: genomeBin (deployment wrapper)

```bash
sourdough genomebin create --primal myPrimal --version 1.0.0 --ecobins ./ecobins/ --output myPrimal.genome
sourdough genomebin test myPrimal.genome
sourdough genomebin sign myPrimal.genome
```

---

## sourDough Self-Compliance

### UniBin

- Binary: `sourdough`
- Modes: scaffold, genomebin, validate, doctor
- CLI: `--help`, `--version`, consistent UX

### ecoBin

- Pure Rust: zero C dependencies
- Dependencies: tokio, serde, tarpc, bytes, blake3 (pure), thiserror, clap, tracing
- Cross-compilation: x86_64, ARM64 (musl targets)

### Quality

- 229 tests, 94.40% coverage (llvm-cov)
- `#![forbid(unsafe_code)]` on all crates
- `clippy::pedantic` + `clippy::nursery` clean (`-D warnings`)
- All `#[expect(reason)]`, zero `#[allow()]`

---

## Validation Criteria

### UniBin Validation (`sourdough validate unibin`)

- Single binary exists
- Multiple modes via subcommands
- `--help` and `--version` flags work
- Consistent CLI UX

### ecoBin Validation (`sourdough validate ecobin`)

- UniBin validation passes
- `cargo tree` shows zero C dependencies
- Cross-compilation works (musl targets)
- Binary is statically linked

### genomeBin Validation (`sourdough genomebin test`)

- ecoBin validation passes
- genomeBin self-extracts correctly
- System detection works (OS, arch, init)
- Health check passes after install

---

## Related Standards

- `wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md`
- `wateringHole/CAPABILITY_BASED_DISCOVERY_STANDARD.md`

---

## Future Evolution

### Near Term

- Cross-compilation validation (musl targets)
- genomeBin signing (Pure Rust, sequoia-openpgp)

### Medium Term

- EphemeralOwner<T> for short-lived primals (see EPHEMERAL_PRIMAL_SCAFFOLDING.md)
- biomeOS integration library
- neuralAPI integration library

### Long Term

- Complete integration platform
- Ephemeral-to-permanent primal promotion
- Ephemeral mesh coordination

---

**Date**: April 30, 2026
**Version**: 0.2.0-dev
**Status**: Reference Implementation
