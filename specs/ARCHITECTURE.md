# sourDough Architecture

**Version**: 0.1.0
**Date**: April 3, 2026
**Type**: Reference Implementation (Nascent Budding Primal)

---

## Overview

sourDough is both a library and a tool:

- **Library** (`sourdough-core`): Core primal traits + JSON-RPC 2.0 IPC + tarpc RPC
- **Tool** (`sourdough` UniBin): Primal scaffolding, validation, genomeBin, diagnostics
- **Library** (`sourdough-genomebin`): Pure Rust genomeBin operations

sourDough scaffolds new primals that are **self-contained and independent**. The
generated primal receives inlined core traits and has zero dependency on sourDough
after creation.

---

## Crate Structure

```
sourDough/
  Cargo.toml                            Workspace manifest
  crates/
    sourdough-core/src/
      lib.rs                            Re-exports (66 lines)
      lifecycle.rs                      PrimalLifecycle trait (315 lines)
      health.rs                         PrimalHealth trait (371 lines)
      identity.rs                       PrimalIdentity trait (420 lines)
      discovery.rs                      PrimalDiscovery trait (369 lines)
      config.rs                         PrimalConfig trait (290 lines)
      ipc.rs                            JSON-RPC 2.0 IPC, primary (637 lines)
      rpc.rs                            tarpc RPC, secondary (425 lines)
      error.rs                          Common error types (244 lines)
      types.rs                          Common types: Did, ContentHash, Timestamp (444 lines)
    sourdough/src/
      main.rs                           CLI entry point (125 lines)
      commands/
        mod.rs                          Command module declarations (11 lines)
        scaffold.rs                     new-primal, new-crate (789 lines)
        validate.rs                     primal, unibin, ecobin (279 lines)
        genomebin.rs                    create, test, sign (133 lines)
        doctor.rs                       System health checks (123 lines)
      tests/
        cli_integration.rs              Integration tests (442 lines)
    sourdough-genomebin/src/
      lib.rs                            Public API (72 lines)
      platform.rs                       Runtime platform detection (535 lines)
      validator.rs                      genomeBin validation (553 lines)
      builder.rs                        genomeBin creation (401 lines)
      archive.rs                        Pure Rust tar/gzip (251 lines)
      metadata.rs                       Type-safe metadata parsing (242 lines)
      error.rs                          Structured error types (169 lines)
      examples/
        platform_detection.rs           Platform detection example
        create_and_validate.rs          Create and validate example
  specs/                                Specifications
  archive/                              Fossil record
```

Total: 7,705 lines of Rust across 26 files. Largest file: `scaffold.rs` (789 lines).

---

## Core Trait Architecture

### Trait Hierarchy

```
PrimalLifecycle      Essential: state machine (Created, Starting, Running, Stopping, Stopped, Failed)
PrimalHealth         Essential: observability (Healthy, Degraded, Unhealthy)
PrimalIdentity       Integration: BearDog (DIDs, signing, verification)
PrimalDiscovery      Integration: Songbird (capability advertising, registration)
PrimalConfig         Utility: configuration loading and validation
```

Traits are composable. Primals implement what they need.

### Essential Traits

**PrimalLifecycle** -- every primal has a lifecycle:

```rust
pub enum PrimalState { Created, Starting, Running, Stopping, Stopped, Failed }

pub trait PrimalLifecycle {
    fn state(&self) -> PrimalState;
    async fn start(&mut self) -> Result<(), PrimalError>;
    async fn stop(&mut self) -> Result<(), PrimalError>;
    async fn reload(&mut self) -> Result<(), PrimalError>;
}
```

**PrimalHealth** -- every primal needs health monitoring:

```rust
pub enum HealthStatus { Healthy, Degraded { reason: String }, Unhealthy { reason: String } }

pub trait PrimalHealth {
    fn health_status(&self) -> HealthStatus;
    async fn health_check(&self) -> Result<HealthReport, PrimalError>;
}
```

### Integration Traits

**PrimalIdentity** -- cryptographic identity via BearDog (discovered at runtime):

```rust
pub trait PrimalIdentity {
    fn did(&self) -> &Did;
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError>;
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError>;
}
```

**PrimalDiscovery** -- capability-based discovery via Songbird:

```rust
pub trait PrimalDiscovery {
    fn capabilities(&self) -> Vec<UpaCapability>;
    async fn register(&self) -> Result<ServiceRegistration, PrimalError>;
    async fn announce(&self) -> Result<(), PrimalError>;
}
```

### Utility Traits

**PrimalConfig** -- standard configuration interface:

```rust
pub trait PrimalConfig {
    type Config;
    fn load(path: &Path) -> Result<Self::Config, PrimalError>;
    fn validate(&self) -> Result<(), PrimalError>;
}
```

---

## IPC Architecture

### JSON-RPC 2.0 (Primary)

`sourdough-core/src/ipc.rs` implements the primary IPC protocol:

- Newline-delimited JSON-RPC 2.0 over raw streams (Unix sockets, TCP)
- Semantic method naming: `domain.verb` (e.g., `lifecycle.start`, `health.check`)
- Request/response/notification/batch support
- Circuit breaker pattern for resilience

### tarpc (Secondary)

`sourdough-core/src/rpc.rs` provides high-throughput binary IPC:

- Type-safe service definitions via tarpc
- Used when JSON-RPC 2.0 overhead is unacceptable
- Same semantic contract, different wire format

### Protocol Selection

| Use case | Protocol |
|----------|----------|
| General IPC, tooling, debugging | JSON-RPC 2.0 |
| High-throughput data transfer | tarpc |
| Health checks, capability queries | JSON-RPC 2.0 |
| Bulk operations, streaming | tarpc |

---

## UniBin CLI Architecture

```
sourdough
  scaffold
    new-primal <name> "<description>" [--output <dir>]
    new-crate <primal> <crate>
  genomebin
    create --primal <name> --version <ver> --ecobins <dir>
    test <genomeBin>
    sign <genomeBin>
  validate
    primal <dir>
    unibin <dir>
    ecobin <dir>
  doctor [--comprehensive]
```

### Scaffold Independence

When `sourdough scaffold new-primal` creates a new primal:

1. A workspace is generated with its own `-core` crate
2. Core traits (`PrimalLifecycle`, `PrimalHealth`, `PrimalState`, `PrimalError`,
   `HealthStatus`, `HealthReport`) are **inlined** into the generated code
3. The new primal has zero dependency on `sourdough-core`
4. Each `new-crate` within the primal uses a path dependency to the primal's own core

This is the **budding primal pattern**: like biological budding, the offspring is
complete and independent from creation.

---

## Dependency Strategy

All dependencies are Pure Rust (ecoBin compliant):

```toml
[workspace.dependencies]
tokio = { version = "1.43", features = ["macros", "rt-multi-thread", "signal", "net", "io-util", "time"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
thiserror = "2.0"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tarpc = { version = "0.35", features = ["tokio1", "serde-transport"] }
bytes = "1.9"
blake3 = { version = "1.5", features = ["pure"] }
```

No `ring`, `openssl-sys`, `aws-lc-sys`, `native-tls`, `reqwest`. Crypto operations
are delegated to BearDog at runtime. HTTP is delegated to Songbird when needed.

---

## Reference Implementation Principles

1. **Simplicity**: easy to read, well-documented, minimal complexity
2. **Compliance**: sourDough follows its own standards (UniBin, ecoBin, `#![forbid(unsafe_code)]`)
3. **Sovereignty**: primals know only themselves, discover others at runtime
4. **Zero hardcoding**: OS-assigned ports, capability-based discovery
5. **Pure Rust**: no C dependencies, no shell scripts

---

## Evolution Phases

### Phase 1: Core Traits -- COMPLETE

- PrimalLifecycle, PrimalHealth, PrimalIdentity, PrimalDiscovery, PrimalConfig
- JSON-RPC 2.0 IPC, tarpc RPC
- Common types (Did, ContentHash, Timestamp, PrimalError)

### Phase 2: UniBin CLI -- COMPLETE

- scaffold new-primal and new-crate with self-contained output
- validate primal, unibin, ecobin
- genomebin create, test, sign
- doctor diagnostics

### Phase 3: Pure Rust genomeBin Library -- COMPLETE

- Platform detection (OS, arch, libc)
- Type-safe metadata parsing
- Pure Rust tar/gzip archive operations
- Comprehensive validation

### Phase 4: Cross-Compilation and Signing -- IN PROGRESS

- Cross-compile to x86_64-musl, aarch64-musl
- genomeBin signing via Pure Rust sequoia-openpgp
- Binary analysis and static linking validation

### Phase 5: Integration Libraries -- PLANNED

- EphemeralOwner<T> for short-lived primals (see EPHEMERAL_PRIMAL_SCAFFOLDING.md)
- biomeOS launcher integration
- neuralAPI registry integration

---

**Date**: April 3, 2026
**Version**: 0.1.0
**Status**: Reference Implementation
