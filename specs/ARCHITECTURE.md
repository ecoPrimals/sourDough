# 🏗️ sourDough Architecture

**Version**: 0.1.0 (unreleased)  
**Date**: January 19, 2026  
**Type**: Reference Implementation

---

## 🎯 Overview

sourDough is **both** a library and a tool:
- **Library** (`sourdough-core`): Core traits for all primals
- **Tool** (`sourdough` UniBin): Primal management CLI

This dual nature makes sourDough unique: it's infrastructure AND a reference implementation.

---

## 📦 Crate Structure

```
sourDough/
├── Cargo.toml                           # Workspace manifest
├── crates/
│   ├── sourdough-core/                  # Core library (traits + IPC)
│   │   └── src/
│   │       ├── lib.rs                   # Re-exports
│   │       ├── lifecycle.rs             # PrimalLifecycle trait
│   │       ├── health.rs                # PrimalHealth trait
│   │       ├── identity.rs             # PrimalIdentity trait
│   │       ├── discovery.rs            # PrimalDiscovery trait
│   │       ├── config.rs               # PrimalConfig trait
│   │       ├── ipc.rs                  # JSON-RPC 2.0 IPC (primary)
│   │       ├── rpc.rs                  # tarpc RPC (secondary)
│   │       ├── error.rs                # Common error types
│   │       └── types.rs                # Common types
│   ├── sourdough/                       # UniBin CLI
│   │   ├── src/
│   │   │   ├── main.rs                  # Entry point
│   │   │   └── commands/
│   │   │       ├── mod.rs
│   │   │       ├── scaffold.rs          # Scaffold commands
│   │   │       ├── genomebin.rs         # genomeBin commands
│   │   │       ├── validate.rs          # Validation commands
│   │   │       └── doctor.rs            # Health check
│   │   └── tests/
│   │       └── cli_integration.rs
│   └── sourdough-genomebin/             # genomeBin library
│       ├── src/
│       │   ├── lib.rs
│       │   ├── builder.rs              # GenomeBinBuilder
│       │   ├── validator.rs            # Validation
│       │   ├── archive.rs              # Tar/gzip operations
│       │   ├── metadata.rs             # Type-safe metadata
│       │   ├── platform.rs             # Platform detection
│       │   └── error.rs                # Error types
│       └── examples/
├── genomebin/                           # Standard genomeBin scaffolding
├── specs/                               # Specifications
└── archive/                             # Historical session docs
```

---

## 🧬 Core Trait Architecture

### **Trait Hierarchy**

```
PrimalLifecycle      (Essential - lifecycle management)
    ↓
PrimalHealth         (Essential - observability)
    ↓
PrimalIdentity       (Integration - BearDog)
    ↓
PrimalDiscovery      (Integration - Songbird)
    ↓
PrimalConfig         (Utility - configuration)
```

**Principle**: Traits are composable. Implement what you need!

### **Essential Traits** (Required)

Every primal SHOULD implement:

#### **1. PrimalLifecycle**

```rust
pub enum PrimalState {
    Created,      // Just instantiated
    Starting,     // Initialization in progress
    Running,      // Operational
    Stopping,     // Shutdown in progress
    Stopped,      // Cleanly stopped
    Failed,       // Error state
}

pub trait PrimalLifecycle {
    /// Current state
    fn state(&self) -> PrimalState;
    
    /// Start the primal
    async fn start(&mut self) -> Result<(), PrimalError>;
    
    /// Stop the primal gracefully
    async fn stop(&mut self) -> Result<(), PrimalError>;
    
    /// Reload configuration without restart
    async fn reload(&mut self) -> Result<(), PrimalError>;
}
```

**Why**: All primals have a lifecycle. This provides standard interface.

#### **2. PrimalHealth**

```rust
pub enum HealthStatus {
    Healthy,                              // All good
    Degraded { reason: String },          // Partial functionality
    Unhealthy { reason: String },         // Not operational
}

pub struct HealthReport {
    pub name: String,
    pub version: String,
    pub status: HealthStatus,
    pub uptime: Duration,
    pub dependencies: Vec<DependencyHealth>,
    pub metadata: HashMap<String, String>,
}

pub trait PrimalHealth {
    /// Quick health status
    fn health_status(&self) -> HealthStatus;
    
    /// Detailed health check
    async fn health_check(&self) -> Result<HealthReport, PrimalError>;
}
```

**Why**: biomeOS and neuralAPI need to monitor primal health.

### **Integration Traits** (Ecosystem)

#### **3. PrimalIdentity** (BearDog Integration)

```rust
pub struct Did(String);  // Decentralized ID
pub struct Signature(Vec<u8>);

pub trait PrimalIdentity {
    /// Get primal's DID
    fn did(&self) -> &Did;
    
    /// Sign data (delegates to BearDog)
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError>;
    
    /// Verify signature (delegates to BearDog)
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError>;
}
```

**Why**: Primals need identity and cryptographic capabilities via BearDog.

**Implementation Pattern**:
```rust
impl PrimalIdentity for MyPrimal {
    fn did(&self) -> &Did {
        &self.did  // Obtained from BearDog at startup
    }
    
    async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError> {
        // Delegate to BearDog via Unix socket
        self.beardog_client.sign(data).await
    }
    
    async fn verify(&self, data: &[u8], sig: &Signature) -> Result<bool, PrimalError> {
        // Delegate to BearDog via Unix socket
        self.beardog_client.verify(data, sig).await
    }
}
```

#### **4. PrimalDiscovery** (Songbird Integration)

```rust
pub struct UpaCapability {
    pub name: String,
    pub version: String,
    pub endpoints: Vec<String>,
}

pub struct ServiceRegistration {
    pub service_id: String,
    pub capabilities: Vec<UpaCapability>,
    pub metadata: HashMap<String, String>,
}

pub trait PrimalDiscovery {
    /// List capabilities this primal provides
    fn capabilities(&self) -> Vec<UpaCapability>;
    
    /// Register with UPA (via Songbird)
    async fn register(&self) -> Result<ServiceRegistration, PrimalError>;
    
    /// Announce via BirdSong
    async fn announce(&self) -> Result<(), PrimalError>;
}
```

**Why**: Primals need to be discoverable in the ecosystem.

**Implementation Pattern**:
```rust
impl PrimalDiscovery for MyPrimal {
    fn capabilities(&self) -> Vec<UpaCapability> {
        vec![
            UpaCapability {
                name: "my-capability".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                endpoints: vec!["/my-endpoint".to_string()],
            }
        ]
    }
    
    async fn register(&self) -> Result<ServiceRegistration, PrimalError> {
        // Delegate to Songbird via Unix socket
        self.songbird_client.register(self.capabilities()).await
    }
    
    async fn announce(&self) -> Result<(), PrimalError> {
        // Broadcast via BirdSong protocol
        self.songbird_client.announce().await
    }
}
```

### **Utility Traits**

#### **5. PrimalConfig**

```rust
pub struct CommonConfig {
    pub name: String,
    pub log_level: String,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

pub trait PrimalConfig {
    type Config;
    
    /// Load configuration from file
    fn load(path: &Path) -> Result<Self::Config, PrimalError>;
    
    /// Validate configuration
    fn validate(&self) -> Result<(), PrimalError>;
}
```

**Why**: Standard configuration interface.

---

## 🔧 sourDough UniBin Architecture

### **Command Structure**

```
sourdough
├── scaffold
│   ├── new-primal <name> "<description>"
│   └── new-crate <primal> <crate>
├── genomebin
│   ├── create --primal <name> --version <ver> --ecobins <dir>
│   ├── test <genomeBin>
│   └── sign <genomeBin>
├── validate
│   ├── primal <dir>
│   ├── unibin <dir>
│   └── ecobin <dir>
├── doctor
├── version
└── help [command]
```

### **Module Architecture**

```rust
// src/main.rs
fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Scaffold(args) => commands::scaffold::run(args),
        Commands::GenomeBin(args) => commands::genomebin::run(args),
        Commands::Validate(args) => commands::validate::run(args),
        Commands::Doctor => commands::doctor::run(),
        Commands::Version => commands::version::run(),
    }
}

// src/commands/scaffold.rs
pub fn run(args: ScaffoldArgs) -> Result<()> {
    match args {
        ScaffoldArgs::NewPrimal { name, description } => {
            create_primal_structure(&name, &description)?;
            generate_cargo_toml(&name)?;
            generate_core_crate(&name)?;
            generate_specs(&name, &description)?;
            Ok(())
        }
        ScaffoldArgs::NewCrate { primal, crate_name } => {
            create_crate_structure(&primal, &crate_name)?;
            Ok(())
        }
    }
}

// src/commands/genomebin.rs
pub fn run(args: GenomeBinArgs) -> Result<()> {
    match args {
        GenomeBinArgs::Create { primal, version, ecobins, output } => {
            collect_ecobins(&ecobins)?;
            create_payload(&primal, &version)?;
            create_wrapper(&primal)?;
            sign_genomebin(&output)?;
            Ok(())
        }
        GenomeBinArgs::Test { genomebin } => {
            test_extraction(&genomebin)?;
            test_installation(&genomebin)?;
            Ok(())
        }
        GenomeBinArgs::Sign { genomebin } => {
            sign_with_gpg(&genomebin)?;
            create_checksum(&genomebin)?;
            Ok(())
        }
    }
}

// src/commands/validate.rs
pub fn run(args: ValidateArgs) -> Result<()> {
    match args {
        ValidateArgs::Primal { dir } => {
            check_cargo_workspace(&dir)?;
            check_specs(&dir)?;
            check_traits(&dir)?;
            report_results()
        }
        ValidateArgs::UniBin { dir } => {
            check_single_binary(&dir)?;
            check_subcommands(&dir)?;
            check_cli_flags(&dir)?;
            report_results()
        }
        ValidateArgs::EcoBin { dir } => {
            check_unibin_compliance(&dir)?;
            check_dependencies_pure_rust(&dir)?;
            check_cross_compilation(&dir)?;
            check_binary_analysis(&dir)?;
            report_results()
        }
    }
}

// src/commands/doctor.rs
pub fn run() -> Result<()> {
    check_sourdough_health()?;
    check_dependencies()?;
    check_genomebin_tools()?;
    report_health()
}
```

---

## 🧬 genomeBin Library Architecture

### **GenomeBinLauncher** (biomeOS Integration)

```rust
// sourdough-genomebin/src/launcher.rs

pub struct GenomeBinLauncher {
    primal: String,
    version: String,
    architecture: Architecture,
    install_mode: InstallMode,
    config_override: Option<Config>,
}

impl GenomeBinLauncher {
    pub fn new(primal: impl Into<String>) -> Self {
        Self {
            primal: primal.into(),
            version: "latest".to_string(),
            architecture: Architecture::detect(),
            install_mode: InstallMode::System,
            config_override: None,
        }
    }
    
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    pub async fn install(&self) -> Result<(), GenomeBinError> {
        // 1. Download genomeBin
        let genomebin = self.download().await?;
        
        // 2. Extract correct ecoBin for architecture
        let ecobin = genomebin.extract_for(self.architecture)?;
        
        // 3. Install binary
        ecobin.install(self.install_mode).await?;
        
        // 4. Create service
        let service = Service::create(&self.primal, self.install_mode)?;
        service.start().await?;
        
        // 5. Health check
        self.health_check().await?;
        
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<HealthStatus, GenomeBinError> {
        // Call primal's health endpoint via Unix socket
        let client = UnixSocketClient::connect(&self.primal).await?;
        let health = client.call("health", ()).await?;
        Ok(health)
    }
    
    pub async fn uninstall(&self, mode: UninstallMode) -> Result<(), GenomeBinError> {
        // Stop service
        self.stop().await?;
        
        // Remove binary
        std::fs::remove_file(&self.binary_path)?;
        
        // Optionally remove data
        if mode == UninstallMode::Full {
            std::fs::remove_dir_all(&self.data_dir)?;
        }
        
        Ok(())
    }
}
```

### **GenomeBinRegistry** (neuralAPI Integration)

```rust
// sourdough-genomebin/src/registry.rs

pub struct GenomeBinRegistry {
    registry_url: String,
    cache_dir: PathBuf,
}

impl GenomeBinRegistry {
    pub fn new(url: impl Into<String>) -> Result<Self, GenomeBinError> {
        Ok(Self {
            registry_url: url.into(),
            cache_dir: PathBuf::from("/var/cache/genomebin"),
        })
    }
    
    pub async fn list_available(&self) -> Result<Vec<PrimalInfo>, GenomeBinError> {
        // Query registry for available primals
        let client = reqwest::Client::new();
        let response = client.get(&format!("{}/primals", self.registry_url)).send().await?;
        let primals: Vec<PrimalInfo> = response.json().await?;
        Ok(primals)
    }
    
    pub async fn install(
        &self,
        primal: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<GenomeBinLauncher, GenomeBinError> {
        let launcher = GenomeBinLauncher::new(primal)
            .version(version);
        launcher.install().await?;
        Ok(launcher)
    }
    
    pub async fn is_installed(&self, primal: impl AsRef<str>) -> Result<bool, GenomeBinError> {
        let binary_path = self.cache_dir.join(primal.as_ref());
        Ok(binary_path.exists() && binary_path.is_file())
    }
}
```

---

## 📊 Dependency Strategy

### **sourDough Dependencies** (ecoBin Compliant)

```toml
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }

# Async
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
config = "0.14"
```

**All Pure Rust!** ✅

### **Rationale**

- No `ring` (crypto delegated to BearDog)
- No `reqwest` (HTTP delegated to Songbird when needed)
- No `rusqlite` (not needed for sourDough's role)
- No `dirs` (use `etcetera`)

**Result**: sourDough is TRUE ecoBin!

---

## 🎯 Reference Implementation Principles

### **1. Simplicity**

sourDough should be:
- Easy to understand (new devs can read the code)
- Well-documented (every module has clear purpose)
- Minimal (no unnecessary complexity)

### **2. Compliance**

sourDough MUST follow its own standards:
- ✅ UniBin architecture
- ✅ ecoBin compliance (Pure Rust)
- ✅ Implements `sourdough-core` traits
- ✅ Can create its own genomeBin

### **3. Consistency**

sourDough demonstrates:
- Standard project structure
- Standard CLI patterns
- Standard error handling
- Standard configuration

### **4. Generality**

sourDough's patterns should apply to:
- Simple primals (single purpose)
- Complex primals (multiple capabilities)
- System primals (biomeOS, Songbird)
- Application primals (ToadStool, petalTongue)

---

## 🚀 Evolution Strategy

### **Phase 1: Core Traits** ✅ COMPLETE

- [x] `PrimalLifecycle`
- [x] `PrimalHealth`
- [x] `PrimalIdentity`
- [x] `PrimalDiscovery`
- [x] `PrimalConfig`

### **Phase 2: UniBin CLI** 📝 IN PROGRESS

- [ ] Implement `sourdough` CLI binary
- [ ] Scaffold commands
- [ ] genomeBin commands
- [ ] Validation commands
- [ ] Doctor command

### **Phase 3: ecoBin Compliance** 📝 NEXT

- [ ] Verify zero C dependencies
- [ ] Cross-compile to x86_64, ARM64
- [ ] Binary analysis validation
- [ ] Harvest to `plasmidBin/`

### **Phase 4: genomeBin Tooling** 📝 FUTURE

- [ ] Implement genomeBin creation scripts
- [ ] Implement genomeBin testing
- [ ] Implement genomeBin signing
- [ ] Create sourDough's own genomeBin

### **Phase 5: Integration Libraries** 📝 FUTURE

- [ ] `sourdough-genomebin` crate
- [ ] biomeOS launcher
- [ ] neuralAPI registry
- [ ] Standard protocol implementation

---

## 🎊 Summary

**sourDough Architecture**:
- **Library**: Core traits for all primals
- **Tool**: UniBin CLI for primal management
- **Framework**: genomeBin standardization
- **Reference**: Example for all other primals

**Key Principles**:
- Simplicity over complexity
- Standards over custom solutions
- Composability over monoliths
- Reference implementation that follows its own rules

---

**Date**: January 19, 2026  
**Version**: 0.1.0 (unreleased)  
**Status**: Reference Implementation (evolving)  
**Next**: Implement sourDough UniBin CLI

🍞🧬🦀 **The foundation for ALL ecoPrimals!** ✨

