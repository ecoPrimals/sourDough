+++
title = "sourDough Validation Summary"
description = "Ecosystem scaffolding meta-primal — nascent primal generation, binary signing, deployment validation. 281 tests, 95%+ coverage."
date = 2026-05-20

[taxonomies]
primals = ["sourdough"]
springs = []
+++

## Status

- **Gate**: CLEAR (meta-primal — scaffolding/tooling, MethodGate N/A for sourDough itself)
- **Phase**: N/A (CLI tool, no runtime IPC server)
- **Edition**: 2024
- **Tests**: 281 passing (152 unit, 31 CLI integration, 2 e2e, 8 doc, 88 genomebin)
- **Coverage**: 95%+ (llvm-cov, target: 90%)
- **Source**: 37 files, max 750 lines
- **Clippy**: 0 warnings (`pedantic` + `nursery`, `-D warnings`)
- **Unsafe**: zero (`forbid(unsafe_code)` workspace-level)
- **deny.toml**: ring, openssl, native-tls, aws-lc-sys banned; cc allowed for blake3 wrapper only

## Capabilities

| Capability | Description |
|-----------|-------------|
| `scaffold` | Generate self-contained primals (budding primal pattern) |
| `sign` | Ed25519 detached binary signatures |
| `verify` | Signature verification against public keys |
| `validate` | Primal, UniBin, ecoBin (project + binary), composition compliance |
| `layout` | Triple-first binary layout validation |
| `genomebin` | Pure Rust genomeBin: create, test, sign, platform detect |
| `doctor` | System health diagnostics |

## CLI Surface (sourdough UniBin)

- `sourdough scaffold new-primal` — full primal workspace with JSON-RPC server + MethodGate + CI/CD
- `sourdough scaffold systemd` — hardened systemd service unit generation
- `sourdough sign` / `sourdough verify` — Ed25519 binary signing
- `sourdough validate primal|unibin|ecobin|composition` — compliance validation
- `sourdough layout` — triple-first directory structure validation
- `sourdough genomebin create|test|sign` — genomeBin archive operations
- `sourdough doctor` — toolchain and environment diagnostics

## Crates (3)

| Crate | Role | LOC |
|-------|------|-----|
| `sourdough-core` | Core traits: lifecycle, health, identity, discovery, config, IPC, RPC, PeekedStream | ~3,800 |
| `sourdough` | UniBin CLI: scaffold, sign, validate, layout, genomebin, doctor | ~4,200 |
| `sourdough-genomebin` | Pure Rust genomeBin: platform, archive, metadata, signing, validation | ~2,500 |

## Composition Role

sourDough is a **build-time meta-primal** — not part of runtime compositions.
It generates primals that participate in compositions. Every NUCLEUS primal was
scaffolded by sourDough or follows the patterns it defines.

## Downstream Consumers

- plasmidBin (genomeBin CI/CD pipeline)
- All 13 NUCLEUS primals (scaffolded patterns)
- primalSpring (deployment internalization contract: v0.3.0 shipped)
- cellMembrane (systemd service units)

## Degradation

sourDough is a CLI tool — no runtime degradation mode. If sourDough is
unavailable, new primals cannot be scaffolded but existing primals are unaffected.
