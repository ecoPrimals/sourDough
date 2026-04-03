# sourDough

The nascent primal. The budding primal. The starter culture for ecoPrimals.

**Version**: 0.1.0 (unreleased)
**License**: AGPL-3.0-or-later (scyBorg Provenance Trio)
**Edition**: Rust 2024

## Purpose

sourDough scaffolds new primals that are **self-contained and independent**. When sourDough
creates a new primal, the result has its own inlined core traits, its own workspace, and
zero runtime dependency on sourDough. Like biological budding, the offspring is complete.

sourDough also serves as a reference implementation demonstrating ecoPrimals standards:
UniBin, ecoBin, genomeBin, JSON-RPC 2.0 IPC, and capability-based discovery.

## Crates

| Crate | Role |
|-------|------|
| `sourdough-core` | Core traits library: lifecycle, health, identity, discovery, config, JSON-RPC 2.0 IPC, tarpc RPC |
| `sourdough` | UniBin CLI: scaffold, validate, genomebin, doctor |
| `sourdough-genomebin` | Pure Rust genomeBin: platform detection, metadata, archive, validation |

## Quick Start

```bash
cargo build --release

# Scaffold a new self-contained primal
./target/release/sourdough scaffold new-primal myPrimal "Purpose of my primal" --output ../myPrimal

# Validate primal compliance
./target/release/sourdough validate primal ../myPrimal

# System health check
./target/release/sourdough doctor
```

Scaffolded primals include their own core traits (`PrimalLifecycle`, `PrimalHealth`,
`PrimalState`, `PrimalError`) inlined directly. No dependency on sourDough after creation.

## Quality

| Metric | Value |
|--------|-------|
| Tests | 229 passing |
| Coverage | 94.40% (llvm-cov, target: 90%) |
| Clippy | zero warnings (pedantic + nursery, `-D warnings`) |
| Unsafe | zero (`#![forbid(unsafe_code)]` on all crates) |
| C deps | zero (Pure Rust) |
| Max file | < 800 lines (target: 1000) |

## Standards Compliance

- **UniBin**: single binary, multiple subcommands
- **ecoBin**: Pure Rust, zero C dependencies, static linking, cross-compilation
- **genomeBin**: Pure Rust platform detection, metadata, archive operations
- **JSON-RPC 2.0**: primary IPC with semantic `domain.verb` method naming
- **tarpc**: secondary high-throughput binary IPC path
- **scyBorg Provenance Trio**: AGPL-3.0-or-later (software), ORC (research/data), CC-BY-SA-4.0 (docs)

## Project Layout

```
sourDough/
  Cargo.toml                  Workspace manifest
  crates/
    sourdough-core/           Core traits + IPC
    sourdough/                UniBin CLI
    sourdough-genomebin/      Pure Rust genomeBin library
  specs/
    ARCHITECTURE.md           Technical architecture
    SOURDOUGH_SPECIFICATION.md  Full specification
    ROADMAP.md                Evolution roadmap
    EPHEMERAL_PRIMAL_SCAFFOLDING.md  Session-as-primal spec
  archive/                    Fossil record of past sessions
```

## Development

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -W clippy::pedantic -W clippy::nursery -D warnings
cargo fmt --all -- --check
cargo llvm-cov --workspace
cargo doc --workspace --no-deps
```

## Design Principles

- **Primal sovereignty**: primals know only themselves, discover others at runtime
- **Zero hardcoding**: OS-assigned ports, capability-based discovery, no primal name coupling
- **Scaffold independence**: generated primals are complete and self-sufficient
- **Pure Rust**: no C dependencies, no shell scripts, no external tooling
- **Modern idiomatic Rust**: edition 2024, `#[expect(reason)]`, `#![forbid(unsafe_code)]`

## Documentation

- [Specification](specs/SOURDOUGH_SPECIFICATION.md)
- [Architecture](specs/ARCHITECTURE.md)
- [Roadmap](specs/ROADMAP.md)
- [Ephemeral Primal Scaffolding](specs/EPHEMERAL_PRIMAL_SCAFFOLDING.md)
- [Conventions](CONVENTIONS.md)
