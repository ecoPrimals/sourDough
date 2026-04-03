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
  Cargo.toml                            Workspace manifest + lints + release profile
  crates/
    sourdough-core/src/
      lib.rs                            Re-exports (57 lines)
      lifecycle.rs                      PrimalLifecycle trait (305 lines)
      health.rs                         PrimalHealth trait (371 lines)
      identity.rs                       PrimalIdentity trait (414 lines)
      discovery.rs                      PrimalDiscovery trait (369 lines)
      config.rs                         PrimalConfig trait (290 lines)
      ipc.rs                            JSON-RPC 2.0 IPC, primary (637 lines)
      rpc.rs                            tarpc RPC, secondary (425 lines)
      error.rs                          Common error types (244 lines)
      types.rs                          Common types: ContentHash, Timestamp (444 lines)
    sourdough/src/
      main.rs                           CLI entry point (121 lines)
      commands/
        mod.rs                          Command module declarations (11 lines)
        scaffold/
          mod.rs                        Command dispatch + orchestration (154 lines)
          generators.rs                 File writing logic (220 lines)
          templates.rs                  Inlined primal DNA constants (438 lines)
        validate.rs                     primal, unibin, ecobin (279 lines)
        genomebin.rs                    create, test, sign (133 lines)
        doctor.rs                       System health diagnostics (133 lines)
      tests/
        cli_integration.rs              Integration tests (538 lines)
        e2e_scaffold_lifecycle.rs       E2E scaffold lifecycle (153 lines)
    sourdough-genomebin/src/
      lib.rs                            Public API (68 lines)
      platform.rs                       Runtime platform detection (535 lines)
      validator.rs                      genomeBin validation (553 lines)
      builder.rs                        genomeBin creation (415 lines)
      archive.rs                        Pure Rust tar/gzip (251 lines)
      metadata.rs                       Type-safe metadata parsing (242 lines)
      error.rs                          Structured error types (169 lines)
      examples/
        platform_detection.rs           Platform detection example
        create_and_validate.rs          Create and validate example
  specs/                                Specifications
  archive/                              Fossil record
```

Total: ~8,100 lines of Rust across 29 files. Largest file: `ipc.rs` (637 lines).

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
- `bytes::Bytes` for zero-copy on the wire (custom `rpc_bytes_serde`)
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

Note: sourDough is a meta-primal (tooling/scaffolding), not a long-running
service. It has no `server --port` mode per UNIBIN_ARCHITECTURE_STANDARD.

### Scaffold Independence

When `sourdough scaffold new-primal` creates a new primal:

1. A workspace is generated with its own `-core` crate
2. Core traits (`PrimalLifecycle`, `PrimalHealth`, `PrimalState`, `PrimalError`,
   `HealthStatus`, `HealthReport`) are **inlined** into the generated code
3. The new primal has zero dependency on `sourdough-core`
4. Each `new-crate` within the primal uses a path dependency to the primal's own core
5. Generated workspace includes `[workspace.lints]` for pedantic/nursery/forbid(unsafe)

This is the **budding primal pattern**: like biological budding, the offspring is
complete and independent from creation.

---

## Dependency Strategy

All dependencies are Pure Rust (ecoBin compliant):

```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tarpc = { version = "0.34", features = ["tokio1", "serde1", "serde-transport"] }
bytes = { version = "1.11", features = ["serde"] }
blake3 = "1.5"
thiserror = "2.0"
```

No `ring`, `openssl-sys`, `aws-lc-sys`, `native-tls`, `reqwest`. Crypto operations
are delegated to BearDog at runtime. HTTP is delegated to Songbird when needed.

---

## Lint and Build Configuration

Workspace-level lint enforcement (`[workspace.lints]` in root `Cargo.toml`):

- `rust.unsafe_code = "forbid"` -- no unsafe in any crate
- `clippy::pedantic` + `clippy::nursery` at warn level
- `rust.missing_docs = "warn"` on library crates
- `.cargo/config.toml`: `rustflags = ["-D", "warnings"]`

Release profile: `lto = true`, `codegen-units = 1`, `strip = true`.

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
- doctor diagnostics with genomeBin tooling validation

### Phase 3: Pure Rust genomeBin Library -- COMPLETE

- Platform detection (OS, arch, libc)
- Type-safe metadata parsing
- Pure Rust tar/gzip archive operations
- Comprehensive validation
- Parallel ecoBin processing

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
