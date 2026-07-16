# sourDough

The nascent primal. The budding primal. The starter culture for ecoPrimals.

**Version**: 0.4.0
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)
**Edition**: Rust 2024

## Purpose

sourDough scaffolds new primals that are **self-contained and independent**. When sourDough
creates a new primal, the result has its own inlined core traits, its own workspace, and
zero runtime dependency on sourDough. Like biological budding, the offspring is complete.

sourDough also serves as:
- **Reference implementation** of ecoPrimals transport and IPC standards
- **Ecosystem tooling** for validation, compliance auditing, and depot management
- **Migration tooling** for evolving existing primals toward ecosystem standards

## Crates

| Crate | Role |
|-------|------|
| `sourdough-core` | Core traits + JSON-RPC 2.0 IPC + TransportEndpoint + IpcClient + riboCipher + CircuitBreaker |
| `sourdough` | UniBin CLI: scaffold, validate (transport, ribocipher, depot, composition), sign, migrate, doctor |
| `sourdough-genomebin` | Pure Rust genomeBin: platform detection, metadata, archive, validation, Ed25519 signing |

## Quick Start

```bash
# Scaffold a new self-contained primal
sourdough scaffold new-primal myPrimal "Purpose of my primal" --output ../myPrimal

# Generate a transport module for an existing primal
sourdough scaffold transport-kit myPrimal

# Validate primal compliance
sourdough validate primal ../myPrimal
sourdough validate transport ../myPrimal
sourdough validate ribocipher ../myPrimal

# Ecosystem-wide transport audit (JSON output for CI)
sourdough validate transport-report --primals-dir ../
sourdough validate transport-report --primals-dir ../ --json

# Depot freshness check
sourdough validate depot primals/ --stale-hours 48

# System health check
sourdough doctor --comprehensive
```

Production binary comes from plasmidBin. For local development:
```bash
cargo build --release
./target/release/sourdough --help
```

## Quality

| Metric | Value |
|--------|-------|
| Tests | 502 passing (zero ignored) |
| Clippy | zero warnings (workspace-level pedantic + nursery) |
| Unsafe | zero (`#![forbid(unsafe_code)]` on all crate roots) |
| C deps | zero (Pure Rust entire dependency tree) |
| Max file | 608 lines (production code) |
| unwrap/expect | zero in library production code |
| Mocks | zero in production (test-only) |
| Cross-arch | `cargo check --target x86_64-pc-windows-gnu` passing |

## Standards Compliance

- **UniBin**: single binary, multiple subcommands
- **ecoBin**: Pure Rust, zero C dependencies, static linking, cross-compilation (4 targets)
- **genomeBin**: Pure Rust platform detection, metadata, archive operations
- **JSON-RPC 2.0**: primary IPC with semantic `domain.verb` method naming
- **riboCipher**: transport signal standard (Wave 111) — signal-first detection, legacy deprecation
- **TransportEndpoint**: canonical wire format (serde tagged: uds/tcp/mesh_relay)
- **MethodGate (JH-0)**: pre-dispatch capability gate on all scaffolded primals
- **Primal sovereignty**: no runtime dependency on sourdough-core; wire format is the contract
- **Capability-based discovery**: env-driven constants, no hardcoded primal names
- **scyBorg Provenance Trio**: AGPL-3.0-or-later (software), ORC (research/data), CC-BY-SA-4.0 (docs)

## Project Layout

```
sourDough/
  Cargo.toml                  Workspace manifest
  compositions.toml           Named compositions (tower, nucleus, full, niche-*)
  crates/
    sourdough-core/           Core traits + IPC + transport + circuit breaker
    sourdough/                UniBin CLI
    sourdough-genomebin/      Pure Rust genomeBin library
  specs/                      Specifications and architecture docs
  sporeprint/                 Validation summary for ecosystem dashboard
  wateringHole/handoffs/      Active handoff documents
  archive/                    Fossil record of past sessions
```

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo deny check
cargo doc --workspace --no-deps
```

## Design Principles

- **Primal sovereignty**: primals know only themselves, discover others at runtime
- **Wire format is the contract**: TransportEndpoint serde format, not library dependency
- **Zero hardcoding**: capability-based discovery, transport injection, no primal name coupling
- **Scaffold independence**: generated primals are complete and self-sufficient
- **Pure Rust**: no C dependencies, no shell scripts, no external tooling
- **Modern idiomatic Rust**: edition 2024, `#[expect(reason)]`, `#![forbid(unsafe_code)]`
- **Cross-architecture**: Windows target parity via `#[cfg(unix)]` guards (Wave 141a)

## Documentation

- [Start Here](START_HERE.md) — new developer guide
- [What's Next](WHATS_NEXT.md) — roadmap and priorities
- [Specification](specs/SOURDOUGH_SPECIFICATION.md)
- [Architecture](specs/ARCHITECTURE.md)
- [Roadmap](specs/ROADMAP.md)
- [Conventions](CONVENTIONS.md)
