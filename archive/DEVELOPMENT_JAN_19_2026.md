# 🛠️ Development Guide - sourDough

**Version**: 0.1.0  
**Date**: January 19, 2026  
**Status**: Production Ready

---

## 🎯 Purpose

This guide helps you develop with and contribute to `sourDough`, the reference implementation and starter culture for ecoPrimals.

---

## 📋 Prerequisites

### Required Tools

```bash
# Rust toolchain (1.70+)
rustc --version
cargo --version

# Code coverage
cargo install cargo-llvm-cov

# Code quality
cargo install cargo-audit
rustup component add clippy rustfmt
```

### Development Environment

```bash
# Clone the repository
git clone git@github.com:ecoPrimals/sourDough.git
cd sourDough

# Build everything
cargo build --all-features

# Run all tests
cargo test --all-features

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings -W clippy::all -W clippy::pedantic
cargo fmt --all -- --check
```

---

## 🏗️ Project Structure

```
sourDough/
├── crates/
│   ├── sourdough-core/      # Core library (traits + utilities)
│   │   ├── src/
│   │   │   ├── lifecycle.rs  # PrimalLifecycle trait
│   │   │   ├── health.rs     # PrimalHealth trait
│   │   │   ├── identity.rs   # PrimalIdentity trait (BearDog integration)
│   │   │   ├── discovery.rs  # PrimalDiscovery trait (Songbird integration)
│   │   │   ├── config.rs     # PrimalConfig trait
│   │   │   ├── rpc.rs        # RPC layer (tarpc-based)
│   │   │   ├── types.rs      # Common types (Did, ContentHash, Timestamp)
│   │   │   └── error.rs      # PrimalError type
│   │   └── Cargo.toml
│   │
│   └── sourdough/           # UniBin CLI tool
│       ├── src/
│       │   ├── main.rs      # CLI entry point
│       │   └── commands/    # Command implementations
│       │       ├── scaffold.rs   # Primal scaffolding
│       │       ├── validate.rs   # Standards validation
│       │       ├── doctor.rs     # System health checks
│       │       └── genomebin.rs  # GenomeBin tooling
│       ├── tests/
│       │   └── cli_integration.rs  # 18 integration tests
│       └── Cargo.toml
│
├── genomebin/               # GenomeBin standard tooling
│   ├── scripts/             # Build, test, sign scripts
│   ├── wrapper/             # Self-extraction wrapper
│   ├── services/            # systemd, launchd, rc.d templates
│   └── config/              # Configuration templates
│
├── specs/                   # Specifications
│   ├── SOURDOUGH_SPECIFICATION.md
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
│
└── docs/ (planned)          # Additional documentation
```

---

## 🧪 Testing Strategy

### Unit Tests (90 tests, 98% coverage)

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test --lib --test types

# Run with output
cargo test --lib -- --nocapture
```

### Integration Tests (18 tests)

```bash
# Run CLI integration tests
cargo test --test cli_integration

# Run specific integration test
cargo test --test cli_integration test_scaffold_new_primal
```

### Coverage Analysis

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# View report
open target/llvm-cov/html/index.html

# Check coverage threshold (90% target)
cargo llvm-cov --all-features --workspace --summary-only
```

### Test Requirements

- **Unit tests**: Cover all public API surface
- **Integration tests**: Cover all CLI commands
- **Coverage**: Maintain > 90%
- **No `unwrap()`** in production code (tests OK)
- **Mock isolation**: All mocks in `#[cfg(test)]` modules

---

## 🏃 Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Follow these principles:

- **Primal Sovereignty**: Primals only know themselves, discover others at runtime
- **No Hardcoding**: Use port 0 (OS-assigned), discover endpoints via Songbird
- **RPC First**: All inter-primal communication via tarpc
- **Zero-Copy**: Use `bytes::Bytes` where possible
- **Error Handling**: `thiserror` for libraries, `anyhow` for CLI

### 3. Test Thoroughly

```bash
# Run tests
cargo test --all-features

# Check formatting
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# Check coverage
cargo llvm-cov --all-features --workspace
```

### 4. Document Changes

- **Public API**: Rustdoc comments with examples
- **Errors**: `# Errors` sections
- **Panics**: `# Panics` sections (if any)
- **Examples**: Working code, not `todo!()`

### 5. Submit Pull Request

- Clear description
- Link to relevant issues
- All tests passing
- No clippy warnings
- Coverage maintained

---

## 🎨 Code Style

### Rust Idioms

```rust
// ✅ Good: Use early returns
pub fn process(&self) -> Result<String, PrimalError> {
    let Some(value) = self.get_value() else {
        return Err(PrimalError::not_found("Value not found"));
    };
    
    Ok(value.to_string())
}

// ❌ Bad: Nested matching
pub fn process(&self) -> Result<String, PrimalError> {
    match self.get_value() {
        Some(value) => Ok(value.to_string()),
        None => Err(PrimalError::not_found("Value not found")),
    }
}
```

### Error Handling

```rust
// ✅ Good: Specific error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ❌ Bad: String errors
fn do_something() -> Result<(), String> {
    // ...
}
```

### Async Code

```rust
// ✅ Good: Use tokio runtime
#[tokio::main]
async fn main() -> Result<()> {
    // ...
}

#[tokio::test]
async fn test_async_function() {
    // ...
}
```

### File Size Limit

- **Maximum**: 1000 lines per file
- **Current**: All files < 550 lines ✅
- **If too large**: Smart refactoring (by concern, not arbitrary splits)

---

## 🔧 Common Tasks

### Scaffolding a New Primal

```bash
# Create new primal project
cargo run -- scaffold new-primal my_primal --description "My awesome primal"

# The scaffold creates:
# - my_primal/
#   - Cargo.toml (workspace with sourdough-core dependency)
#   - crates/my-primal-core/ (trait implementations)
#   - specs/ (SPEC.md, ARCHITECTURE.md, ROADMAP.md)
```

### Running Validation

```bash
# Validate primal standards compliance
cargo run -- validate primal ./my_primal

# Validate UniBin compliance
cargo run -- validate unibin ./my_primal

# Validate EcoBin compliance
cargo run -- validate ecobin ./my_primal
```

### System Health Check

```bash
# Check development environment
cargo run -- doctor

# Comprehensive check
cargo run -- doctor --comprehensive
```

---

## 🔬 Advanced Topics

### Implementing PrimalRpc

```rust
use sourdough_core::rpc::PrimalRpc;

#[derive(Clone)]
struct MyPrimalServer {
    // ...
}

#[tarpc::server]
impl PrimalRpc for MyPrimalServer {
    async fn health(self, _: context::Context) -> Result<HealthReport, String> {
        // Return health status
        Ok(self.get_health().await.map_err(|e| e.to_string())?)
    }
    
    async fn state(self, _: context::Context) -> Result<PrimalState, String> {
        Ok(self.get_state().await)
    }
    
    async fn did(self, _: context::Context) -> Result<Did, String> {
        Ok(self.identity.did())
    }
    
    async fn ping(self, _: context::Context) -> Result<String, String> {
        Ok("pong".to_string())
    }
}
```

### Zero-Copy Optimization

```rust
use bytes::Bytes;

// ✅ Good: Use Bytes for large payloads
pub async fn send_data(&self, data: Bytes) -> Result<()> {
    // No copy when sending
    self.transport.send(data).await
}

// ❌ Bad: Clone Vec<u8>
pub async fn send_data(&self, data: Vec<u8>) -> Result<()> {
    self.transport.send(data.clone()).await  // Unnecessary copy!
}
```

### Discovery Integration

```rust
use sourdough_core::discovery::{PrimalDiscovery, ServiceRegistration};

impl PrimalDiscovery for MyPrimal {
    fn registration(&self) -> ServiceRegistration {
        ServiceRegistration::new(&self.name, &self.version, &self.endpoint)
            .with_capability(UpaCapability::new("compute", "1.0", "tarpc"))
            .with_health_endpoint("/health")
    }
    
    async fn register(&self) -> Result<RegistrationHandle, PrimalError> {
        // Register with Songbird via UDP multicast
        self.songbird_client.register(self.registration()).await
    }
}
```

---

## 📊 Quality Metrics

### Current Status

```
Test Coverage:      92.13% ✅ (Target: 90%)
Tests Passing:      98/98 ✅
Clippy Warnings:    0 ✅
File Size Max:      521 lines ✅ (Target: < 1000)
Unsafe Code:        0 ✅
```

### Maintaining Quality

```bash
# Before committing
./scripts/pre-commit-check.sh  # (TODO: create this)

# Or manually:
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo llvm-cov --all-features --workspace
```

---

## 🐛 Debugging

### Logging

```rust
use tracing::{info, debug, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting my primal");
    debug!(port = self.port, "Listening on port");
    
    // ...
}
```

### Running with Logs

```bash
# Info level
RUST_LOG=info cargo run

# Debug level
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=sourdough_core::rpc=trace cargo run
```

---

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [tarpc Documentation](https://docs.rs/tarpc/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### EcoPrimals Documentation

- [INTER_PRIMAL_INTERACTIONS.md](../wateringHole/INTER_PRIMAL_INTERACTIONS.md)
- [UNIBIN_ARCHITECTURE_STANDARD.md](../wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md)
- [ECOBIN_ARCHITECTURE_STANDARD.md](../wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md)
- [GENOMEBIN_ARCHITECTURE_STANDARD.md](../wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md)

---

## 🤝 Contributing

1. Read [CONVENTIONS.md](./CONVENTIONS.md)
2. Follow this development guide
3. Maintain test coverage > 90%
4. Write clear commit messages
5. Document all public APIs

---

## ❓ Getting Help

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Chat**: ecoPrimals Discord (link TBD)

---

**Happy Coding!** 🍞🧬🦀

*sourDough: The starter culture for ecoPrimals*

