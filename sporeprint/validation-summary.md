+++
title = "sourDough Validation Summary"
description = "Ecosystem scaffolding meta-primal — transport standard, primal generation, binary signing, compliance tooling. 321 tests, zero development debt."
date = 2026-06-10

[taxonomies]
primals = ["sourdough"]
springs = []
+++

## Status

- **Gate**: CLEAR (meta-primal — scaffolding/tooling, not a runtime daemon)
- **Phase**: N/A (CLI tool, no runtime IPC server)
- **Edition**: 2024
- **Tests**: 321 passing (161 core, 60 CLI, 87 genomebin, 11 doc, 2 e2e)
- **Coverage**: target 90%+
- **Source**: all production files < 800 lines
- **Clippy**: 0 warnings (`pedantic` + `nursery`, `-D warnings`)
- **Unsafe**: zero (`forbid(unsafe_code)` workspace-level)
- **deny.toml**: ring, openssl, native-tls, aws-lc-sys banned; entire dep tree Pure Rust

## Capabilities

| Capability | Description |
|-----------|-------------|
| `scaffold` | Generate self-contained primals (budding primal pattern) |
| `scaffold transport-kit` | Generate self-contained transport module for existing primals |
| `sign` / `verify` | Ed25519 detached binary signatures |
| `validate transport` | Single primal transport compliance audit |
| `validate transport-report` | Ecosystem batch audit (--json for CI) |
| `validate depot` | Binary freshness detection (--json, --source) |
| `validate composition` | Composition binary presence (tower/nucleus/full/niche) |
| `validate primal/unibin/ecobin` | Standard compliance checks |
| `migrate transport` | Migration tool for existing primals |
| `layout` | Triple-first binary layout validation |
| `genomebin` | Pure Rust genomeBin: create, test, sign, platform detect |
| `doctor` | System health diagnostics |

## Core Library (sourdough-core)

| Module | Purpose |
|--------|---------|
| `transport` | TransportEndpoint, connect_transport(), TransportStream, PeekedStream |
| `ipc` | JSON-RPC 2.0 types, IpcClient, IpcError, Capability, HealthProbe |
| `methods` | Canonical domain.verb method constants (health, lifecycle, ipc, primal, system) |
| `circuit_breaker` | CircuitBreaker resilience pattern |
| `env_keys` | Centralized environment variable name constants |
| `rpc` | Binary RPC (secondary, high-throughput) |
| `config` | CommonConfig, ConfigLoader, ConfigWatcher |
| `health` | HealthStatus, PrimalHealth, HealthReport |
| `lifecycle` | PrimalLifecycle, PrimalState |
| `identity` | DID, PrimalIdentity, Signature |
| `discovery` | PrimalDiscovery, ServiceRegistration |
| `types` | ContentHash, Timestamp (ISO 8601) |
| `error` | PrimalError (typed, thiserror) |

## Composition Role

sourDough is a **build-time meta-primal** — not part of runtime compositions.
It generates primals that participate in compositions. Every NUCLEUS primal was
scaffolded by sourDough or follows the patterns it defines.

## Downstream Consumers

- All 13 NUCLEUS primals (scaffolded patterns, transport wire format)
- cellMembrane (depot management, validate depot --json, systemd units)
- primalSpring (deployment contract, validate transport-report)
- plasmidBin (genomeBin CI/CD pipeline)

## Degradation

sourDough is a CLI tool — no runtime degradation mode. If sourDough is
unavailable, new primals cannot be scaffolded but existing primals are unaffected.
