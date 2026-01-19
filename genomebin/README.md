# 🧬 genomeBin Standard Scaffolding

**Status**: Standard Deployment Machinery  
**Purpose**: Standardized genomeBin creation for all ecoBin-certified primals  
**Version**: 1.0.0

---

## 🎯 What Is This?

This directory contains **standardized genomeBin machinery** that is **80-90% reusable** across all primals. Once a primal achieves ecoBin status, they can use this scaffolding to quickly create a genomeBin with minimal primal-specific customization.

**Key Principle**: genomeBin machinery should be **standard**, not per-primal work!

---

## 📊 What's Standardized vs. Primal-Specific

### ✅ **80-90% Standardized** (Use As-Is)

**From this directory**:

1. **System Detection** (100% standard)
   - OS detection (Linux, macOS, BSD, Windows)
   - Architecture detection (x86_64, ARM64, RISC-V, etc.)
   - Init system detection (systemd, launchd, rc.d)
   - Privilege level detection (root vs user)
   
2. **Installation Logic** (100% standard)
   - Binary extraction
   - File installation
   - Permission setting
   - Conflict handling
   
3. **Service Integration** (95% standard)
   - systemd template (standard)
   - launchd template (standard)
   - rc.d template (standard)
   - Service installation logic (standard)
   - Only variable: primal name
   
4. **Configuration Management** (80% standard)
   - Directory creation
   - Config file installation
   - Environment detection
   - Most logic is standard
   
5. **Lifecycle Management** (90% standard)
   - Update system
   - Rollback mechanism
   - Uninstall logic
   - Backup/restore
   
6. **Wrapper Script** (95% standard)
   - Self-extraction logic
   - Error handling
   - User interaction
   - Progress reporting

### ⚙️ **10-20% Primal-Specific** (Customize Per Primal)

**You provide**:

1. **ecoBin Payloads** (100% primal-specific)
   - `primal-x86_64-linux-musl`
   - `primal-aarch64-linux-musl`
   - `primal-x86_64-macos`
   - etc.
   
2. **Configuration Schema** (variable)
   - Default `config.toml` template
   - Environment-specific overrides
   - Primal-specific settings
   
3. **Health Check Details** (uses standard trait)
   - Implements `PrimalHealth` trait (standard)
   - Primal-specific checks (custom)
   - Uses standard reporting format
   
4. **Service Configuration** (minor)
   - Service user/group
   - Additional environment variables
   - Resource limits (if needed)
   
5. **Documentation** (primal-specific)
   - Installation examples
   - Configuration guide
   - Usage instructions

---

## 🏗️ Directory Structure

```
sourDough/genomebin/
├── README.md                    # You are here
├── wrapper/
│   ├── genome-wrapper.sh        # Main wrapper script (STANDARD)
│   ├── system-detection.sh      # System detection (STANDARD)
│   ├── install-logic.sh         # Installation logic (STANDARD)
│   └── lifecycle.sh             # Update/rollback/uninstall (STANDARD)
├── services/
│   ├── systemd.service.tmpl     # systemd template (STANDARD)
│   ├── launchd.plist.tmpl       # launchd template (STANDARD)
│   └── rc.d.tmpl                # rc.d template (STANDARD)
├── scripts/
│   ├── create-genomebin.sh      # Build genomeBin from ecoBins (STANDARD)
│   ├── test-genomebin.sh        # Test across systems (STANDARD)
│   └── sign-genomebin.sh        # Sign and checksum (STANDARD)
├── config/
│   ├── config-template.toml     # Base config template (STANDARD)
│   └── environments/            # Environment configs (CUSTOMIZABLE)
│       ├── development.toml
│       ├── production.toml
│       └── embedded.toml
└── integration/
    ├── biomeos-launcher.rs      # biomeOS integration (STANDARD)
    └── neuralapi-launcher.rs    # neuralAPI integration (STANDARD)
```

---

## 🚀 Usage: From ecoBin to genomeBin

### **Prerequisites**

Your primal MUST be:
- ✅ TRUE ecoBin (certified)
- ✅ Cross-compiled for target architectures
- ✅ Implements `sourdough-core` traits
- ✅ Has `primal doctor` command

### **Step 1: Prepare Your ecoBins**

```bash
# Build all target architectures
cd your-primal/
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Collect binaries
mkdir -p genome-build/ecobins/
cp target/x86_64-unknown-linux-musl/release/yourprimal \
   genome-build/ecobins/yourprimal-x86_64-linux-musl
cp target/aarch64-unknown-linux-musl/release/yourprimal \
   genome-build/ecobins/yourprimal-aarch64-linux-musl
# ... etc
```

### **Step 2: Customize Configuration (Optional)**

```bash
# Copy standard config template
cp ../../sourDough/genomebin/config/config-template.toml \
   genome-build/config.toml

# Customize for your primal
nano genome-build/config.toml

# Customize environment-specific configs (optional)
mkdir -p genome-build/environments/
cp ../../sourDough/genomebin/config/environments/* \
   genome-build/environments/
```

### **Step 3: Create genomeBin**

```bash
# Use standard script
../../sourDough/genomebin/scripts/create-genomebin.sh \
    --primal yourprimal \
    --version 1.0.0 \
    --ecobins genome-build/ecobins/ \
    --config genome-build/config.toml \
    --output yourprimal.genome

# This creates:
#   yourprimal.genome           (self-installing)
#   yourprimal.genome.sha256    (checksum)
#   yourprimal.genome.asc       (signature, if GPG configured)
```

### **Step 4: Test**

```bash
# Test on different systems
../../sourDough/genomebin/scripts/test-genomebin.sh \
    yourprimal.genome

# This tests:
#   - Ubuntu 22.04 (x86_64)
#   - Ubuntu 22.04 (ARM64)
#   - Debian 12
#   - Alpine
#   - Fedora
#   - (macOS if available)
```

### **Step 5: Publish**

```bash
# Sign
../../sourDough/genomebin/scripts/sign-genomebin.sh \
    yourprimal.genome

# Upload to your distribution server
scp yourprimal.genome* user@cdn.yourprimal.dev:/releases/v1.0.0/
```

**That's it!** You have a genomeBin! 🎉

---

## 📋 What You Get

### **User Installation Experience**

```bash
# User runs ONE command:
curl -sSf https://install.yourprimal.dev/genome | sh

# genomeBin does EVERYTHING:
# 1. Detects: Linux + ARM64
# 2. Extracts: yourprimal-aarch64-linux-musl
# 3. Installs: /usr/local/bin/yourprimal
# 4. Configures: /etc/yourprimal/config.toml (smart defaults)
# 5. Creates service: /etc/systemd/system/yourprimal.service
# 6. Starts: systemctl start yourprimal
# 7. Validates: yourprimal doctor
# 8. Reports: "✅ YourPrimal v1.0.0 installed successfully!"

# ZERO manual configuration!
```

### **Service Management**

```bash
# Native system integration
systemctl status yourprimal    # Linux (systemd)
launchctl list | grep yourprimal  # macOS (launchd)
service yourprimal status      # BSD (rc.d)
```

### **Lifecycle Management**

```bash
# Update
yourprimal.genome update

# Rollback (if needed)
yourprimal.genome rollback

# Uninstall
yourprimal.genome uninstall [--keep-data] [--purge]
```

---

## 🧬 biomeOS Integration

### **Programmatic Primal Launching**

The genomeBin standard includes **biomeOS integration** for programmatic launching:

```rust
// From biomeOS or neuralAPI:
use sourdough_genomebin::GenomeBinLauncher;

// Launch a primal programmatically
let launcher = GenomeBinLauncher::new("beardog")
    .version("0.9.0")
    .architecture(Architecture::detect())
    .install_mode(InstallMode::System)  // or User
    .config_override(custom_config)
    .build()?;

// Install
launcher.install().await?;

// Result:
// - beardog installed
// - Service created and started
// - Health check passed
// - Ready to use!

// Query status
let status = launcher.status().await?;
println!("BearDog: {}", status.health);  // "Healthy"

// Update
launcher.update("0.10.0").await?;

// Uninstall
launcher.uninstall(UninstallMode::KeepData).await?;
```

### **neuralAPI Integration**

```rust
// From neuralAPI:
use sourdough_genomebin::GenomeBinRegistry;

// Discover available primals
let registry = GenomeBinRegistry::new("https://registry.ecoprimals.dev")?;
let available = registry.list_available().await?;

// Install required primal
if !available.contains("toadstool") {
    registry.install("toadstool", "latest").await?;
}

// Launch with dependency resolution
let launcher = registry.launch("toadstool")
    .with_dependencies(true)  // Auto-install beardog, songbird if needed
    .await?;

// Result: ToadStool + all dependencies running!
```

### **Standard Protocol**

All genomeBins support:
- ✅ JSON-RPC control interface (via Unix socket)
- ✅ Health check endpoint (`/health`)
- ✅ Status endpoint (`/status`)
- ✅ Capabilities endpoint (`/capabilities`)
- ✅ Lifecycle control (`/install`, `/update`, `/uninstall`)

This enables biomeOS to:
1. Discover what primals are needed
2. Install them programmatically
3. Monitor their health
4. Update them automatically
5. Uninstall when not needed

---

## 📊 Standard vs. Per-Primal Effort

### **Before genomeBin Standard** (Per-Primal Work)

Each primal team had to:
- [ ] Write wrapper script (~500 lines)
- [ ] Create system detection (~200 lines)
- [ ] Create service templates (~150 lines)
- [ ] Write installation logic (~300 lines)
- [ ] Write update/rollback (~400 lines)
- [ ] Write uninstall (~150 lines)
- [ ] Test on multiple systems (~8 hours)
- [ ] Debug edge cases (~8 hours)

**Total**: ~1700 lines of code + 16 hours work **PER PRIMAL**

### **After genomeBin Standard** (Reuse)

Each primal team only needs to:
- [x] Use standard wrapper (0 lines, provided)
- [x] Use standard detection (0 lines, provided)
- [x] Use standard templates (0 lines, provided)
- [x] Use standard logic (0 lines, provided)
- [x] Use standard lifecycle (0 lines, provided)
- [x] Customize config template (~50 lines)
- [x] Run standard script (~1 command)
- [x] Test with standard tool (~1 command)

**Total**: ~50 lines of config + 2 commands + 1 hour work **PER PRIMAL**

**Savings**: ~1650 lines + 15 hours **PER PRIMAL**! 🎉

---

## 🌟 Benefits

### **For Primal Teams**

- ✅ **Minimal effort**: 1 hour vs. 16 hours
- ✅ **Standard UX**: Consistent across all primals
- ✅ **Proven tooling**: Battle-tested, not experimental
- ✅ **Automatic updates**: New features added to standard
- ✅ **Less maintenance**: Standard handles edge cases

### **For Users**

- ✅ **Consistent**: Same installation for all primals
- ✅ **Simple**: ONE command, ZERO configuration
- ✅ **Reliable**: Standard tested across all systems
- ✅ **Professional**: Consumer-grade experience

### **For Ecosystem**

- ✅ **Interoperability**: biomeOS/neuralAPI can launch any primal
- ✅ **Discoverability**: Standard protocol for capabilities
- ✅ **Composability**: Primals work together seamlessly
- ✅ **Evolution**: Update standard once, benefits all

---

## 🎯 Current Status

### **Standardized Components** (Ready)

- [x] Wrapper script (genome-wrapper.sh)
- [x] System detection (system-detection.sh)
- [x] Service templates (systemd, launchd, rc.d)
- [x] Installation logic (install-logic.sh)
- [x] Lifecycle management (lifecycle.sh)
- [x] Build script (create-genomebin.sh)
- [x] Test script (test-genomebin.sh)
- [x] Sign script (sign-genomebin.sh)

### **Integration Components** (Planned)

- [ ] biomeOS launcher (Rust library)
- [ ] neuralAPI launcher (Rust library)
- [ ] Registry protocol (JSON-RPC spec)
- [ ] Dependency resolution (graph)

### **Documentation** (In Progress)

- [x] This README
- [ ] Primal team guide
- [ ] biomeOS integration guide
- [ ] neuralAPI integration guide

---

## 🚀 Next Steps

### **Phase 1: Create First genomeBin** (BearDog)

1. BearDog team uses this scaffolding
2. Creates first genomeBin
3. Validates on multiple systems
4. Provides feedback on standard

### **Phase 2: Refine Standard**

1. Incorporate BearDog learnings
2. Update standard components
3. Document best practices
4. Create examples

### **Phase 3: Scale to All ecoBins**

1. NestGate genomeBin
2. ToadStool genomeBin
3. biomeOS genomeBin
4. (Squirrel when ecoBin-ready)

### **Phase 4: biomeOS Integration**

1. Create biomeOS launcher library
2. Integrate with spore deployment
3. Enable programmatic primal launching
4. Test full deployment workflow

---

## 📚 Resources

### **Standards**

- wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md (full spec)
- wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md (prerequisite)
- wateringHole/UNIBIN_ARCHITECTURE_STANDARD.md (foundation)

### **Tools**

- sourDough/genomebin/scripts/create-genomebin.sh
- sourDough/genomebin/scripts/test-genomebin.sh
- sourDough/genomebin/scripts/sign-genomebin.sh

### **Templates**

- sourDough/genomebin/wrapper/genome-wrapper.sh
- sourDough/genomebin/services/*.tmpl
- sourDough/genomebin/config/*.toml

---

## 💡 Philosophy

**UniBin**: Per-primal architecture (each primal structures their binary)  
**ecoBin**: Per-primal purity (each primal achieves Pure Rust)  
**genomeBin**: **STANDARD machinery** (all primals use same deployment system)

**Why?**
- UniBin varies: Different CLI modes per primal
- ecoBin varies: Different dependencies per primal
- genomeBin SAME: Deployment is universal!

**Result**: Spend time on YOUR primal's functionality, not deployment plumbing!

---

**Date**: January 19, 2026  
**Status**: Standard Scaffolding (Ready)  
**Version**: 1.0.0  
**Next**: First genomeBin (BearDog recommended)

🧬🌍🦀 **Standard deployment for all ecoPrimals!** ✨

