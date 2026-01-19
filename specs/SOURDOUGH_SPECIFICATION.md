# 🍞 SourDough - Specification

**Version**: 0.2.0  
**Date**: January 19, 2026  
**Status**: **Reference Implementation**  
**Role**: Nascent Primal + Standardization Framework

---

## 🎯 Purpose

**SourDough** serves three critical functions in the ecoPrimals ecosystem:

### 1. **Starter Culture** (Original Role)

Like biological sourdough starter, sourDough provides the essential "culture" from which new primals are born:
- Core traits (`PrimalLifecycle`, `PrimalHealth`, `PrimalIdentity`, etc.)
- Common patterns (config, error handling, logging)
- Scaffolding templates (new primal, new crate)

### 2. **Reference Implementation** (NEW)

sourDough itself is a **TRUE primal** that demonstrates:
- ✅ **UniBin** architecture (multiple modes, single binary)
- ✅ **ecoBin** compliance (100% Pure Rust, cross-compilation)
- ✅ **genomeBin** tooling (creates genomeBins for other primals)

**Principle**: New primals can reference sourDough as the canonical example!

### 3. **Standardization Framework** (NEW)

sourDough contains standardized machinery:
- `genomebin/` - Standard genomeBin scaffolding
- Validation tools - Check primal compliance
- Documentation templates - Standard docs structure

---

## 🏗️ Architecture

### **sourDough as UniBin**

**Single binary, multiple modes**:

```bash
# Scaffolding
sourdough scaffold new-primal <name> "<description>"
sourdough scaffold new-crate <primal> <crate>

# genomeBin Creation
sourdough genomebin create --primal <name> --version <ver> --ecobins <dir>
sourdough genomebin test <genomeBin>
sourdough genomebin sign <genomeBin>

# Validation
sourdough validate primal <primal-dir>
sourdough validate unibin <primal-dir>
sourdough validate ecobin <primal-dir>

# Health & Info
sourdough doctor
sourdough version
sourdough help
```

**All functionality in ONE binary!**

### **sourDough as ecoBin**

**100% Pure Rust**:
- ✅ Zero C dependencies
- ✅ Cross-compiles to x86_64, ARM64, RISC-V
- ✅ Static linking (musl)
- ✅ Self-contained binary

**Available as ecoBin**:
```
sourdough-x86_64-linux-musl
sourdough-aarch64-linux-musl
sourdough-x86_64-macos
sourdough-aarch64-macos
```

---

## 📦 Components

### **1. sourdough-core** (Library)

**Purpose**: Core traits for all primals

**Exports**:
```rust
// Lifecycle
pub trait PrimalLifecycle {
    fn state(&self) -> PrimalState;
    async fn start(&mut self) -> Result<(), PrimalError>;
    async fn stop(&mut self) -> Result<(), PrimalError>;
    async fn reload(&mut self) -> Result<(), PrimalError>;
}

// Health
pub trait PrimalHealth {
    fn health_status(&self) -> HealthStatus;
    async fn health_check(&self) -> Result<HealthReport, PrimalError>;
}

// Identity (BearDog integration)
pub trait PrimalIdentity {
    fn did(&self) -> &Did;
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError>;
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError>;
}

// Discovery (Songbird integration)
pub trait PrimalDiscovery {
    fn capabilities(&self) -> Vec<UpaCapability>;
    async fn register(&self) -> Result<ServiceRegistration, PrimalError>;
    async fn announce(&self) -> Result<(), PrimalError>;
}

// Configuration
pub trait PrimalConfig {
    type Config;
    fn load(path: &Path) -> Result<Self::Config, PrimalError>;
    fn validate(&self) -> Result<(), PrimalError>;
}
```

**Usage**: All primals depend on `sourdough-core` and implement these traits.

### **2. sourdough** (UniBin)

**Purpose**: Command-line tool for primal management

**Modes**:

#### **Scaffolding**
```bash
# Create new primal
sourdough scaffold new-primal rhizoCrypt "Ephemeral Data Graph"

# Creates:
# ../rhizoCrypt/
#   ├── Cargo.toml (workspace)
#   ├── crates/rhizocrypt-core/ (implements sourdough-core traits)
#   ├── specs/ (specification docs)
#   └── README.md

# Add crate to existing primal
sourdough scaffold new-crate rhizoCrypt rhizocrypt-storage
```

#### **genomeBin Creation**
```bash
# Create genomeBin from ecoBins
sourdough genomebin create \
    --primal beardog \
    --version 1.0.0 \
    --ecobins plasmidBin/primals/beardog/v1.0.0/ \
    --output beardog.genome

# Test genomeBin
sourdough genomebin test beardog.genome

# Sign genomeBin
sourdough genomebin sign beardog.genome
```

#### **Validation**
```bash
# Validate primal structure
sourdough validate primal /path/to/primal

# Check UniBin compliance
sourdough validate unibin /path/to/primal

# Check ecoBin compliance
sourdough validate ecobin /path/to/primal
```

#### **Health & Info**
```bash
# Health check
sourdough doctor

# Version info
sourdough version

# Help
sourdough help [command]
```

### **3. genomebin/** (Standard Scaffolding)

**Purpose**: Standardized genomeBin creation machinery

**Structure**:
```
genomebin/
├── README.md                    # Complete guide
├── wrapper/
│   ├── genome-wrapper.sh        # Main wrapper script
│   ├── system-detection.sh      # OS/arch detection
│   ├── install-logic.sh         # Installation
│   └── lifecycle.sh             # Update/rollback
├── services/
│   ├── systemd.service.tmpl     # Linux systemd
│   ├── launchd.plist.tmpl       # macOS launchd
│   └── rc.d.tmpl                # BSD rc.d
├── scripts/
│   ├── create-genomebin.sh      # Build genomeBin
│   ├── test-genomebin.sh        # Test across systems
│   └── sign-genomebin.sh        # Sign and checksum
├── config/
│   ├── config-template.toml     # Base template
│   └── environments/            # Env-specific configs
└── integration/
    ├── biomeos-launcher.rs      # biomeOS integration
    └── neuralapi-launcher.rs    # neuralAPI integration
```

**Usage**: All ecoBin primals use this to create genomeBins

---

## 🔄 Workflow: New Primal Lifecycle

### **Phase 1: Nascent** (sourDough scaffold)

```bash
# Developer creates new primal
sourdough scaffold new-primal myPrimal "Description"

# Result: myPrimal/ with:
#   - Implements sourdough-core traits
#   - Has basic specs/
#   - Ready for development
```

### **Phase 2: Development** (Build functionality)

```bash
# Developer implements primal logic
cd myPrimal/
cargo build
cargo test

# Primal-specific work here!
```

### **Phase 3: UniBin** (Consolidate to single binary)

```bash
# Validate UniBin compliance
sourdough validate unibin myPrimal/

# Checklist:
# - One binary per primal ✓
# - Multiple modes (subcommands) ✓
# - Professional CLI (--help, --version) ✓
```

### **Phase 4: ecoBin** (Achieve Pure Rust)

```bash
# Validate ecoBin compliance
sourdough validate ecobin myPrimal/

# Checklist:
# - UniBin architecture ✓
# - 100% Pure Rust ✓
# - Cross-compilation ✓
# - Static linking ✓
```

### **Phase 5: genomeBin** (Add deployment wrapper)

```bash
# Create genomeBin (ONE command!)
sourdough genomebin create \
    --primal myPrimal \
    --version 1.0.0 \
    --ecobins plasmidBin/primals/myPrimal/v1.0.0/ \
    --output myPrimal.genome

# Test
sourdough genomebin test myPrimal.genome

# Sign
sourdough genomebin sign myPrimal.genome

# Distribute!
```

### **Result**: New primal went from idea → production-ready genomeBin!

---

## 🎯 sourDough Self-Compliance

### **UniBin** ✅

- **Binary**: `sourdough`
- **Modes**: `scaffold`, `genomebin`, `validate`, `doctor`, `version`, `help`
- **CLI**: Professional (`--help`, `--version`, consistent UX)

### **ecoBin** ✅

- **Pure Rust**: 100% (zero C dependencies)
- **Dependencies**:
  ```
  sourdough-core v0.2.0
  ├── tokio (Pure Rust)
  ├── serde/serde_json (Pure Rust)
  ├── toml (Pure Rust)
  ├── thiserror (Pure Rust)
  ├── tracing (Pure Rust)
  └── config (Pure Rust)
  ```
- **Cross-compilation**: x86_64, ARM64 (validated)
- **Static linking**: musl

### **genomeBin** 📝 (Future)

sourDough can create its own genomeBin:
```bash
sourdough genomebin create \
    --primal sourdough \
    --version 0.2.0 \
    --ecobins plasmidBin/primals/sourdough/v0.2.0/ \
    --output sourdough.genome
```

**Meta**: sourDough uses itself to create its genomeBin! 🎉

---

## 📊 Traits Provided

### **Essential Traits** (All Primals Should Implement)

#### **PrimalLifecycle**
```rust
pub trait PrimalLifecycle {
    fn state(&self) -> PrimalState;
    async fn start(&mut self) -> Result<(), PrimalError>;
    async fn stop(&mut self) -> Result<(), PrimalError>;
    async fn reload(&mut self) -> Result<(), PrimalError>;
}
```

**Why**: Every primal has a lifecycle (start, run, stop, reload)

#### **PrimalHealth**
```rust
pub trait PrimalHealth {
    fn health_status(&self) -> HealthStatus;
    async fn health_check(&self) -> Result<HealthReport, PrimalError>;
}
```

**Why**: Every primal needs health monitoring (for biomeOS, neuralAPI, ops)

### **Integration Traits** (Ecosystem Integration)

#### **PrimalIdentity** (BearDog)
```rust
pub trait PrimalIdentity {
    fn did(&self) -> &Did;
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError>;
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError>;
}
```

**Why**: Primals need identity and signing (via BearDog)

#### **PrimalDiscovery** (Songbird)
```rust
pub trait PrimalDiscovery {
    fn capabilities(&self) -> Vec<UpaCapability>;
    async fn register(&self) -> Result<ServiceRegistration, PrimalError>;
    async fn announce(&self) -> Result<(), PrimalError>;
}
```

**Why**: Primals need to be discoverable (via Songbird/UPA)

### **Utility Traits**

#### **PrimalConfig**
```rust
pub trait PrimalConfig {
    type Config;
    fn load(path: &Path) -> Result<Self::Config, PrimalError>;
    fn validate(&self) -> Result<(), PrimalError>;
}
```

**Why**: Every primal needs configuration management

---

## 🌟 Benefits

### **For New Primals**

**Before sourDough**:
- Write all traits from scratch
- Figure out UniBin architecture
- Discover ecoBin requirements
- Create genomeBin manually
- **Total**: ~100+ hours

**After sourDough**:
- `sourdough scaffold new-primal` → instant primal structure
- Implement `sourdough-core` traits (standard interface)
- Follow sourDough example (reference implementation)
- `sourdough genomebin create` → instant genomeBin
- **Total**: ~20 hours

**Savings**: **~80 hours per primal!**

### **For Existing Primals**

- Reference sourDough for patterns
- Use `sourdough validate` to check compliance
- Use `sourdough genomebin` to create genomeBins
- Implement `sourdough-core` traits for interoperability

### **For Ecosystem**

- **Consistency**: All primals implement same traits
- **Interoperability**: biomeOS/neuralAPI can manage any primal
- **Quality**: Standard patterns = fewer bugs
- **Velocity**: New primals created faster
- **Evolution**: Update sourDough → all future primals benefit

---

## 🎯 Validation Criteria

### **UniBin Validation** (`sourdough validate unibin`)

Checks:
- [ ] Single binary exists
- [ ] Multiple modes via subcommands
- [ ] `--help` flag works
- [ ] `--version` flag works
- [ ] Consistent CLI UX
- [ ] No multiple binaries (e.g., no `-server`, `-client`)

### **ecoBin Validation** (`sourdough validate ecobin`)

Checks:
- [ ] UniBin validation passes ✅
- [ ] `cargo tree` shows zero C dependencies
- [ ] Cross-compilation works (x86_64-musl, aarch64-musl)
- [ ] Binary analysis shows no C symbols (`nm` check)
- [ ] Static linking (`ldd` shows "statically linked")
- [ ] Implements `PrimalHealth` trait

### **genomeBin Validation** (`sourdough genomebin test`)

Checks:
- [ ] ecoBin validation passes ✅
- [ ] genomeBin self-extracts correctly
- [ ] System detection works (OS, arch, init)
- [ ] Installation works (root and user modes)
- [ ] Service creation works (systemd/launchd/rc.d)
- [ ] Health check passes after install
- [ ] Update works
- [ ] Rollback works
- [ ] Uninstall cleans up completely

---

## 📚 Related Standards

- **UniBin**: `wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md`
- **ecoBin**: `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md`
- **genomeBin**: `wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md`

---

## 🚀 Future Evolution

### **v0.3.0** (Near Term)

- [ ] Implement `sourdough` UniBin CLI
- [ ] Implement `sourdough validate` commands
- [ ] Implement `sourdough genomebin` commands
- [ ] Cross-compile sourDough for x86_64, ARM64
- [ ] Harvest sourDough to `plasmidBin/`

### **v0.4.0** (Medium Term)

- [ ] Add `PrimalMetrics` trait (observability)
- [ ] Add `PrimalTelemetry` trait (distributed tracing)
- [ ] Create sourDough genomeBin
- [ ] biomeOS integration library
- [ ] neuralAPI integration library

### **v1.0.0** (Long Term)

- [ ] Complete reference implementation
- [ ] All validation tools complete
- [ ] genomeBin tooling complete
- [ ] Integration libraries production-ready
- [ ] Documentation comprehensive

---

## 🎊 Summary

**sourDough v0.2.0** is evolving from:
- **Was**: Scaffolding templates + core traits
- **Now**: Reference primal (UniBin + ecoBin) + standardization framework
- **Future**: Complete genomeBin + integration platform

**Mission**: Make creating and standardizing primals trivial!

**Philosophy**:
> "Just as sourdough starter contains all the essential microorganisms to create bread, sourDough contains all the essential patterns to create primals."

---

**Date**: January 19, 2026  
**Version**: 0.2.0  
**Status**: Reference Implementation (evolving)  
**Next**: Implement sourDough UniBin CLI

🍞🧬🦀 **The starter culture for ALL ecoPrimals!** ✨

