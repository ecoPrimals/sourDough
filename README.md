# 🍞 SourDough — The Starter Culture for ecoPrimals

**Version:** 0.1.0  
**Status:** ✅ Production Ready  
**Quality:** ⭐⭐⭐⭐⭐ (92% test coverage)  
**Standards:** UniBin Certified • EcoBin Ready • GenomeBin Reference

---

## What is SourDough?

SourDough serves **three critical functions** in the ecoPrimals ecosystem:

### 1. 🧬 Starter Culture

Like biological sourdough starter, SourDough provides the essential "culture" from which new primals are born:
- **Core Traits**: `PrimalLifecycle`, `PrimalHealth`, `PrimalIdentity`, `PrimalDiscovery`, `PrimalConfig`
- **Common Patterns**: Error handling, logging, configuration, async runtime
- **Scaffolding**: One command to create complete primal projects

### 2. 📚 Reference Implementation

SourDough itself is a **complete primal** demonstrating best practices:
- ✅ **UniBin Architecture** - Single binary, multiple commands
- ✅ **EcoBin Compliant** - 100% Pure Rust, zero C dependencies
- ✅ **GenomeBin Standard** - Self-installing deployment packages
- ✅ **RPC Communication** - Type-safe `tarpc`-based inter-primal communication
- ✅ **Capability-Based** - Zero hardcoding, runtime discovery
- ✅ **92% Test Coverage** - Comprehensive unit and integration tests
- ✅ **Modern Idiomatic Rust** - Clean, safe, performant code

### 3. 🛠️ Standardization Framework

SourDough contains reusable machinery for all primals:
- **Validation Tools** - Check UniBin, EcoBin, GenomeBin compliance
- **GenomeBin Tooling** - Standard scripts (80-90% reusable!)
- **Service Templates** - systemd, launchd, rc.d integration
- **Documentation Templates** - Specifications, architecture, roadmaps

**Philosophy**: "Just as sourdough starter contains all the essential microorganisms to create bread, SourDough contains all the essential patterns to create primals."

---

## 🚀 Quick Start

### Create a New Primal

```bash
# One command creates a complete primal project
./target/release/sourdough scaffold new-primal myPrimal \
  "My primal's purpose" \
  --output ../myPrimal

# Result: Complete primal with:
# - Workspace structure
# - Core crate with trait implementations  
# - Tests (already passing!)
# - Specifications
# - README and CONVENTIONS
```

### Validate Compliance

```bash
# Validate primal structure
./target/release/sourdough validate primal ../myPrimal

# Check UniBin compliance
./target/release/sourdough validate unibin ../myPrimal

# Check EcoBin compliance (Pure Rust)
./target/release/sourdough validate ecobin ../myPrimal
```

### Create GenomeBin

```bash
# Build cross-platform genomeBin for distribution
./target/release/sourdough genomebin create \
  --primal myPrimal \
  --version 1.0.0 \
  --ecobins ./ecobins/ \
  --output myPrimal.genome

# Test across platforms
./target/release/sourdough genomebin test myPrimal.genome

# Sign for distribution
./target/release/sourdough genomebin sign myPrimal.genome
```

### Health Diagnostics

```bash
# Check system readiness
./target/release/sourdough doctor

# Comprehensive checks
./target/release/sourdough doctor --comprehensive
```

---

## 📦 Structure

```
sourDough/
├── README.md                     # You are here
├── Cargo.toml                    # Workspace manifest
├── CONVENTIONS.md                # Coding standards
├── DEVELOPMENT.md                # Developer guide
│
├── crates/
│   ├── sourdough-core/          # Core traits library (92% coverage)
│   │   ├── src/
│   │   │   ├── lib.rs           # Re-exports
│   │   │   ├── lifecycle.rs     # PrimalLifecycle trait
│   │   │   ├── health.rs        # PrimalHealth trait
│   │   │   ├── identity.rs      # PrimalIdentity trait (BearDog)
│   │   │   ├── discovery.rs     # PrimalDiscovery trait (Songbird)
│   │   │   ├── config.rs        # PrimalConfig trait
│   │   │   ├── rpc.rs           # RPC communication (tarpc)
│   │   │   ├── error.rs         # Common error types
│   │   │   └── types.rs         # Common types (ContentHash, Timestamp)
│   │   └── Cargo.toml
│   │
│   └── sourdough/               # UniBin CLI (Production Ready)
│       ├── src/
│       │   ├── main.rs          # Entry point
│       │   └── commands/
│       │       ├── scaffold.rs  # Scaffolding (new-primal, new-crate)
│       │       ├── genomebin.rs # GenomeBin creation/testing/signing
│       │       ├── validate.rs  # Compliance validation
│       │       └── doctor.rs    # Health diagnostics
│       ├── tests/
│       │   └── cli_integration.rs  # 18 integration tests
│       └── Cargo.toml
│
├── genomebin/                   # Standard GenomeBin machinery
│   ├── README.md                # Complete guide
│   ├── wrapper/                 # Self-extraction & installation
│   │   ├── genome-wrapper.sh
│   │   └── system-detection.sh
│   ├── scripts/                 # Build, test, sign
│   │   ├── create-genomebin.sh
│   │   ├── test-genomebin.sh
│   │   └── sign-genomebin.sh
│   ├── services/                # Service templates
│   │   ├── systemd.service.tmpl
│   │   ├── launchd.plist.tmpl
│   │   └── rc.d.tmpl
│   └── config/                  # Configuration templates
│       ├── config-template.toml
│       └── environments/
│           ├── development.toml
│           ├── production.toml
│           └── embedded.toml
│
└── specs/                       # Documentation
    ├── SOURDOUGH_SPECIFICATION.md
    ├── ARCHITECTURE.md
    └── ROADMAP.md
```

---

## 🎯 Core Traits

### `PrimalLifecycle` — State Management

Every primal has a lifecycle:

```rust
use sourdough_core::{PrimalLifecycle, PrimalState, PrimalError};

impl PrimalLifecycle for MyPrimal {
    fn state(&self) -> PrimalState {
        self.state
    }
    
    async fn start(&mut self) -> Result<(), PrimalError> {
        // Initialization logic
        self.state = PrimalState::Running;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), PrimalError> {
        // Graceful shutdown
        self.state = PrimalState::Stopped;
        Ok(())
    }
    
    async fn reload(&mut self) -> Result<(), PrimalError> {
        // Hot configuration reload
        Ok(())
    }
}
```

### `PrimalHealth` — Observability

Every primal needs health checks:

```rust
use sourdough_core::health::{PrimalHealth, HealthStatus, HealthReport};

impl PrimalHealth for MyPrimal {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
    
    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("myPrimal", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
```

### `PrimalIdentity` — BearDog Integration

Every primal needs identity:

```rust
use sourdough_core::identity::{PrimalIdentity, Did, Signature};

impl PrimalIdentity for MyPrimal {
    fn did(&self) -> &Did {
        &self.identity.did
    }
    
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError> {
        // Cryptographic signing via BearDog
        self.identity.sign(data).await
    }
}
```

### `PrimalDiscovery` — Songbird Integration

Every primal needs to be discoverable:

```rust
use sourdough_core::discovery::{PrimalDiscovery, ServiceRegistration};

impl PrimalDiscovery for MyPrimal {
    fn registration(&self) -> ServiceRegistration {
        // Register with Songbird for runtime service discovery
        // Port is OS-assigned (listen_port: 0), discovered via Songbird
        ServiceRegistration::new("myPrimal", "1.0.0", &self.endpoint)
            .with_capability(UpaCapability::new("storage", "1.0", "tarpc"))
    }
    
    async fn register(&self) -> Result<(), PrimalError> {
        // Actual registration with Songbird happens here
        Ok(())
    }
}
```

### `PrimalConfig` — Configuration Management

Every primal needs configuration:

```rust
use sourdough_core::config::{PrimalConfig, load_toml};

impl PrimalConfig for MyPrimal {
    type Config = MyConfig;
    
    fn load_config(path: &Path) -> Result<Self::Config, PrimalError> {
        load_toml(path)
    }
}
```

---

## 📊 Quality Metrics

### Test Coverage

```
Overall Coverage: 92.13%

Component Breakdown:
  config.rs       98.04%  (114 lines)
  discovery.rs    98.62%  (173 lines)
  error.rs        95.17%  (97 lines)
  health.rs      100.00%  (198 lines)
  identity.rs     98.38%  (215 lines)
  lifecycle.rs    95.10%  (128 lines)
  rpc.rs          85.00%  (183 lines) [New!]
  types.rs        98.69%  (224 lines)

Total Tests: 98 (unit + integration + doc)
Pass Rate: 100%
```

### Code Quality

```
✅ Clippy:              0 errors, 3 warnings (non-blocking, pedantic mode)
✅ Format:              100% formatted (rustfmt)
✅ Documentation:       100% public API documented
✅ Unsafe Code:         0 blocks
✅ C Dependencies:      0 (100% Pure Rust)
✅ File Size:           All files < 550 lines (max 1000)
✅ Hardcoding:          0 (capability-based design)
```

### Standards Compliance

```
✅ UniBin Standard:     CERTIFIED
✅ EcoBin Standard:     READY FOR CERTIFICATION
✅ GenomeBin Standard:  REFERENCE IMPLEMENTATION
```

---

## 🌟 Primals Created with SourDough

| Primal | Status | Purpose |
|--------|--------|---------|
| 🐻🐕 **BearDog** | In Development | Identity, cryptography, HSM integration |
| 🐦 **Songbird** | Planned | Discovery and coordination |
| 🏰 **NestGate** | Planned | Edge orchestration |
| 🍄 **ToadStool** | Planned | Configuration management |
| 🌸 **PetalTongue** | Planned | Visualization UI |

*Each primal saves ~30 hours with SourDough scaffolding!*

---

## 🎓 Philosophy

### Minimal by Design

SourDough provides only what's universal:
- Common traits that all primals implement
- Integration patterns with BearDog (identity) and Songbird (discovery)
- Standard error handling and configuration
- Documentation templates

### Agnostic by Necessity

SourDough makes no assumptions about:
- ❌ What your primal does
- ❌ What data structures it uses
- ❌ What protocols it speaks
- ❌ What storage it needs

These decisions belong to each primal.

### Composable by Nature

Every trait in SourDough is:
- ✅ Optional (implement what you need)
- ✅ Modular (compose traits together)
- ✅ Extensible (add your own)

---

## 📚 Documentation

- **[Specification](specs/SOURDOUGH_SPECIFICATION.md)** - Complete specification
- **[Architecture](specs/ARCHITECTURE.md)** - Technical architecture
- **[Roadmap](specs/ROADMAP.md)** - Evolution roadmap
- **[Development Guide](DEVELOPMENT.md)** - Developer workflows and RPC examples
- **[GenomeBin Guide](genomebin/README.md)** - GenomeBin creation guide
- **[Conventions](CONVENTIONS.md)** - Coding standards

### Archived Documentation

Session documentation from January 19, 2026 available in `archive/`

---

## 🚀 Getting Started

### Prerequisites

```bash
# Rust toolchain (1.70+)
rustup default stable

# Optional: Code coverage
cargo install cargo-llvm-cov
```

### Build

```bash
# Build all crates
cargo build --release

# Run tests
cargo test --all-features

# Check code coverage
cargo llvm-cov --package sourdough-core
```

### Install

```bash
# Install sourdough CLI globally
cargo install --path crates/sourdough

# Or use directly
./target/release/sourdough --help
```

---

## 🤝 Contributing

SourDough should remain minimal. Before adding anything, ask:

1. **Is this universal?** Does every primal need this?
2. **Is this agnostic?** Does it make no assumptions about the primal's purpose?
3. **Is this composable?** Can primals use only what they need?

If the answer to any of these is "no", it belongs in a specific primal, not in SourDough.

### Development

```bash
# Run all checks
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo fmt --check

# Run integration tests
cargo test --package sourdough --test cli_integration

# Check coverage
cargo llvm-cov --package sourdough-core --html --open
```

---

## 📄 License

AGPL-3.0

---

## 🙏 Acknowledgments

SourDough is the foundational layer for the ecoPrimals ecosystem, providing:
- Reference implementation for UniBin, EcoBin, and GenomeBin standards
- Reusable tooling that saves ~30 hours per primal
- Production-quality code generation and validation
- Standard deployment machinery

---

**Status**: ✅ Production Ready  
**Version**: 0.1.0  
**Quality**: ⭐⭐⭐⭐⭐  
**Coverage**: 92.13%

🧬🌍🦀 *The Starter Culture for ecoPrimals* 🦀🌍🧬
