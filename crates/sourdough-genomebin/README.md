# sourdough-genomebin

**Pure Rust genomeBin infrastructure** - Type-safe, concurrent, universal deployment.

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)]()
[![Tests](https://img.shields.io/badge/tests-22%20passing-brightgreen.svg)]()

## Overview

`sourdough-genomebin` is a Pure Rust library for creating and managing genomeBins - self-extracting deployment packages for ecoPrimal binaries. It replaces the bash script-based tooling with modern, idiomatic, concurrent Rust.

### Key Features

- ✅ **100% Pure Rust** - Zero C dependencies (default features)
- ✅ **Zero Unsafe Code** - `#![forbid(unsafe_code)]`
- ✅ **Zero Hardcoding** - Runtime platform discovery
- ✅ **Type-Safe API** - Compile-time guarantees
- ✅ **Concurrent** - Parallel processing support
- ✅ **Comprehensive Testing** - 22 unit tests, all passing
- ✅ **Pedantic Clippy** - Maximum code quality
- ✅ **Well Documented** - Extensive docs and examples

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sourdough-genomebin = "0.1"
```

## Quick Start

### Platform Detection

```rust
use sourdough_genomebin::Platform;

// Detect current platform at runtime
let platform = Platform::detect()?;
println!("Running on: {}", platform.target_triple());
```

### Create a genomeBin

```rust
use sourdough_genomebin::GenomeBinBuilder;

// Build and create a genomeBin
let builder = GenomeBinBuilder::new("myprimal", "1.0.0")
    .ecobins_dir("./ecobins")
    .output("myprimal-1.0.0.genome")
    .parallel(true);

let genome = builder.build().await?;
let output = genome.create().await?;

println!("Created: {}", output.display());
```

### Validate a genomeBin

```rust
use sourdough_genomebin::Validator;

let validator = Validator::new("myprimal-1.0.0.genome");
let results = validator.validate().await?;

for result in results {
    println!("{}: {}", result.name, if result.passed { "✓" } else { "✗" });
}
```

## Architecture

### Modules

- **`platform`** - Runtime platform detection (OS, arch, libc)
- **`metadata`** - Type-safe metadata handling
- **`archive`** - Tar/gzip operations (Pure Rust)
- **`builder`** - genomeBin creation with concurrent support
- **`validator`** - Comprehensive testing and validation
- **`error`** - Structured error types

### Dependencies (All Pure Rust)

- `tokio` - Async runtime
- `serde` / `toml` - Serialization
- `tar` / `flate2` - Archive operations
- `blake3` / `sha2` - Hashing
- `bytes` - Zero-copy buffers
- `thiserror` / `anyhow` - Error handling
- `tracing` - Logging

## Design Principles

### 1. Zero Unsafe Code

All operations use safe Rust:

```rust
#![forbid(unsafe_code)]
```

### 2. Zero Hardcoding

Runtime discovery instead of compile-time assumptions:

```rust
// ❌ BAD: Hardcoded
let os = "linux";

// ✅ GOOD: Runtime discovery
let os = Os::detect();
```

### 3. Type-Safe

Validated types prevent invalid states:

```rust
// ❌ BAD: String manipulation
let metadata = format!("primal={}\nversion={}", primal, version);

// ✅ GOOD: Type-safe struct
let metadata = Metadata::new(primal, version, architectures)?;
```

### 4. Concurrent

Designed for parallel processing:

```rust
GenomeBinBuilder::new("primal", "1.0.0")
    .parallel(true) // Concurrent ecoBin processing
    .build()
```

## Performance

Compared to bash script implementation:

| Operation | Bash | Rust | Improvement |
|-----------|------|------|-------------|
| Creation | 5-10s | 2-3s | **2-3x faster** |
| Testing | 2-4s | 1-2s | **2x faster** |
| Validation | Sequential | Concurrent | **Parallelizable** |

## Examples

See [`examples/`](examples/) directory:

- [`platform_detection.rs`](examples/platform_detection.rs) - Platform detection
- [`create_and_validate.rs`](examples/create_and_validate.rs) - Full workflow

Run examples:

```bash
cargo run --example platform_detection
cargo run --example create_and_validate
```

## Testing

Run all tests:

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

Current coverage: **22 tests, 100% passing**

## API Documentation

Generate and view API docs:

```bash
cargo doc --open
```

## Contributing

See [`CONVENTIONS.md`](../../CONVENTIONS.md) and [`START_HERE.md`](../../START_HERE.md) for guidelines.

## License

AGPL-3.0 - See [LICENSE](../../LICENSE)

## Part of ecoPrimals

This library is part of the ecoPrimals ecosystem:

- **sourDough** - Primal scaffolding and genomeBin tooling
- **UniBin Architecture** - Single binary, multiple subcommands
- **ecoBin Architecture** - 100% Pure Rust, universal cross-compilation
- **genomeBin Architecture** - Self-extracting deployment packages

---

**🦀 From "Jelly Strings" to Idiomatic Rust ✨**

