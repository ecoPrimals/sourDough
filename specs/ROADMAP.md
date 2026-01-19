# 🗺️ sourDough Roadmap

**Version**: 0.2.0  
**Date**: January 19, 2026  
**Vision**: Complete reference implementation for ecoPrimals

---

## 🎯 Mission

Transform sourDough from scaffolding templates into a **complete reference primal** that demonstrates UniBin, ecoBin, and genomeBin standards while providing tooling for the entire ecosystem.

---

## 📊 Current Status (v0.2.0)

### ✅ **Complete**

- [x] Core traits (`PrimalLifecycle`, `PrimalHealth`, `PrimalIdentity`, `PrimalDiscovery`, `PrimalConfig`)
- [x] `sourdough-core` library (traits + types)
- [x] Basic scaffolding script (`scripts/scaffold.sh`)
- [x] Comprehensive specifications
  - [x] `SOURDOUGH_SPECIFICATION.md`
  - [x] `ARCHITECTURE.md`
  - [x] `ROADMAP.md` (this document)
- [x] genomeBin standard scaffolding (`genomebin/` directory)
- [x] Zero C dependencies (Pure Rust)

### 📝 **In Progress**

- [ ] sourDough UniBin CLI implementation
- [ ] Validation tools
- [ ] genomeBin creation tools

### ⏳ **Planned**

- [ ] ecoBin certification (cross-compilation validation)
- [ ] sourDough genomeBin
- [ ] Integration libraries (`sourdough-genomebin`)
- [ ] biomeOS/neuralAPI connectors

---

## 🚀 Version Roadmap

### **v0.2.0** - Foundation Complete ✅ (Current)

**Status**: Architecture defined, specs complete  
**Timeline**: January 19, 2026  

**Deliverables**:
- ✅ Comprehensive specification
- ✅ Architecture documented
- ✅ Core traits stable
- ✅ genomeBin scaffolding designed
- ✅ Zero C dependencies

**Next**: Implement UniBin CLI

---

### **v0.3.0** - UniBin Implementation 📝 (Next)

**Status**: Implementing  
**Timeline**: ~2-3 weeks  
**Effort**: ~40-60 hours

#### **Goals**

Transform sourDough into a TRUE UniBin with multiple operational modes.

#### **Deliverables**

**1. sourDough UniBin Binary** (~20 hours)
- [ ] Create `crates/sourdough/` (CLI crate)
- [ ] Implement `main.rs` with `clap` CLI parsing
- [ ] Implement subcommand structure
- [ ] Professional `--help` output
- [ ] Version handling

**2. Scaffold Commands** (~8 hours)
- [ ] `sourdough scaffold new-primal <name> "<desc>"`
  - [ ] Generate workspace structure
  - [ ] Generate `Cargo.toml`
  - [ ] Generate core crate with traits
  - [ ] Generate specs/ directory
  - [ ] Generate README
- [ ] `sourdough scaffold new-crate <primal> <crate>`
  - [ ] Generate crate structure
  - [ ] Update workspace `Cargo.toml`

**3. Validate Commands** (~12 hours)
- [ ] `sourdough validate primal <dir>`
  - [ ] Check workspace structure
  - [ ] Check specs/ exists
  - [ ] Check implements `sourdough-core` traits
  - [ ] Report compliance
- [ ] `sourdough validate unibin <dir>`
  - [ ] Check single binary
  - [ ] Check subcommands
  - [ ] Check CLI flags
  - [ ] Report compliance
- [ ] `sourdough validate ecobin <dir>`
  - [ ] Check UniBin compliance
  - [ ] Check `cargo tree` for C deps
  - [ ] Check cross-compilation
  - [ ] Check binary analysis
  - [ ] Report compliance

**4. Doctor Command** (~4 hours)
- [ ] `sourdough doctor`
  - [ ] Check sourDough health
  - [ ] Check dependencies
  - [ ] Check genomeBin tools available
  - [ ] Report status

**5. Testing** (~8 hours)
- [ ] Unit tests for all commands
- [ ] Integration tests
- [ ] CLI acceptance tests
- [ ] Documentation

**Success Criteria**:
- ✅ `sourdough --help` works
- ✅ `sourdough scaffold new-primal` creates valid primal
- ✅ `sourdough validate` checks compliance
- ✅ `sourdough doctor` reports health
- ✅ All tests pass

---

### **v0.4.0** - ecoBin Certification 📝

**Status**: Planned  
**Timeline**: ~1 week  
**Effort**: ~20-30 hours

#### **Goals**

Certify sourDough as TRUE ecoBin with full cross-compilation support.

#### **Deliverables**

**1. Cross-Compilation** (~8 hours)
- [ ] Build for `x86_64-unknown-linux-musl`
- [ ] Build for `aarch64-unknown-linux-musl`
- [ ] Build for `x86_64-apple-darwin`
- [ ] Build for `aarch64-apple-darwin`
- [ ] (Optional) Build for `riscv64gc-unknown-linux-gnu`

**2. Binary Validation** (~4 hours)
- [ ] Run `nm` analysis (no C symbols)
- [ ] Run `ldd` analysis (statically linked)
- [ ] Size optimization (strip binaries)
- [ ] Functional testing (all modes work)

**3. Harvest to plasmidBin** (~2 hours)
- [ ] Create `plasmidBin/primals/sourdough/v0.4.0/`
- [ ] Copy all ecoBins
- [ ] Update `plasmidBin/MANIFEST.md`
- [ ] Create release notes

**4. Documentation** (~4 hours)
- [ ] ecoBin certification document
- [ ] Cross-compilation guide
- [ ] Distribution guide

**Success Criteria**:
- ✅ Builds on ALL targets without C dependencies
- ✅ Binaries are statically linked
- ✅ All functional tests pass on all platforms
- ✅ Harvested to `plasmidBin/`
- ✅ Official ecoBin certification

---

### **v0.5.0** - genomeBin Tooling 📝

**Status**: Planned  
**Timeline**: ~2-3 weeks  
**Effort**: ~60-80 hours

#### **Goals**

Implement complete genomeBin creation and management tooling.

#### **Deliverables**

**1. genomeBin Create Command** (~20 hours)
- [ ] `sourdough genomebin create`
  - [ ] Collect ecoBins from directory
  - [ ] Create payload tarball
  - [ ] Embed wrapper script
  - [ ] Create self-extracting archive
  - [ ] Generate checksums
  - [ ] (Optional) GPG signing

**2. genomeBin Test Command** (~12 hours)
- [ ] `sourdough genomebin test <genomebin>`
  - [ ] Test extraction
  - [ ] Test system detection
  - [ ] Test installation (Docker containers)
    - [ ] Ubuntu 22.04 (x86_64)
    - [ ] Ubuntu 22.04 (ARM64)
    - [ ] Debian 12
    - [ ] Alpine Linux
    - [ ] Fedora
  - [ ] Test service creation
  - [ ] Test health check
  - [ ] Test uninstall

**3. genomeBin Sign Command** (~4 hours)
- [ ] `sourdough genomebin sign <genomebin>`
  - [ ] GPG signing
  - [ ] SHA256 checksum
  - [ ] Verification

**4. Standard Scripts** (~20 hours)
- [ ] Implement `genomebin/wrapper/genome-wrapper.sh`
- [ ] Implement `genomebin/wrapper/system-detection.sh`
- [ ] Implement `genomebin/wrapper/install-logic.sh`
- [ ] Implement `genomebin/wrapper/lifecycle.sh`
- [ ] Create service templates (systemd, launchd, rc.d)
- [ ] Create config templates

**5. Testing** (~8 hours)
- [ ] End-to-end genomeBin tests
- [ ] Multi-platform validation
- [ ] Documentation

**Success Criteria**:
- ✅ `sourdough genomebin create` produces valid genomeBin
- ✅ genomeBin installs on all target systems
- ✅ One-command installation works
- ✅ Service integration works
- ✅ Update/rollback works

---

### **v0.6.0** - sourDough genomeBin 📝

**Status**: Planned  
**Timeline**: ~1 week  
**Effort**: ~10-15 hours

#### **Goals**

Create sourDough's own genomeBin (meta: tool uses itself!).

#### **Deliverables**

**1. Create sourDough genomeBin** (~4 hours)
- [ ] Use `sourdough genomebin create` to create `sourdough.genome`
- [ ] Test installation on multiple systems
- [ ] Validate all modes work
- [ ] Sign and distribute

**2. Self-Hosting** (~4 hours)
- [ ] Host at `https://install.sourdough.dev/genome`
- [ ] Create installation endpoint
- [ ] Test one-liner: `curl -sSf https://install.sourdough.dev/genome | sh`

**3. Documentation** (~2 hours)
- [ ] Installation guide
- [ ] User guide
- [ ] FAQ

**Success Criteria**:
- ✅ sourDough installs via genomeBin
- ✅ One-command installation works
- ✅ All sourDough modes work after genomeBin install
- ✅ Meta-circular: tool built with itself!

---

### **v0.7.0** - Integration Libraries 📝

**Status**: Planned  
**Timeline**: ~2-3 weeks  
**Effort**: ~60-80 hours

#### **Goals**

Provide Rust libraries for biomeOS and neuralAPI to programmatically manage primals.

#### **Deliverables**

**1. sourdough-genomebin Crate** (~30 hours)
- [ ] `GenomeBinLauncher` (launch any primal)
  - [ ] `new(primal)` - create launcher
  - [ ] `install()` - install primal
  - [ ] `health_check()` - query health
  - [ ] `update(version)` - update primal
  - [ ] `uninstall()` - remove primal
- [ ] `GenomeBinRegistry` (discover/manage primals)
  - [ ] `list_available()` - query registry
  - [ ] `install(primal, version)` - install from registry
  - [ ] `is_installed(primal)` - check if installed
- [ ] Standard JSON-RPC protocol
  - [ ] `health` - health status
  - [ ] `capabilities` - what primal can do
  - [ ] `install` - install primal
  - [ ] `update` - update version
  - [ ] `rollback` - restore previous
  - [ ] `uninstall` - remove primal

**2. biomeOS Integration** (~15 hours)
- [ ] `genomebin/integration/biomeos-launcher.rs`
- [ ] Examples for biomeOS team
- [ ] Integration tests

**3. neuralAPI Integration** (~15 hours)
- [ ] `genomebin/integration/neuralapi-launcher.rs`
- [ ] Examples for neuralAPI team
- [ ] Integration tests

**Success Criteria**:
- ✅ biomeOS can programmatically launch any primal
- ✅ neuralAPI can manage primal dependencies
- ✅ Standard protocol works across all primals
- ✅ Documentation complete

---

### **v1.0.0** - Production Ready 🎯

**Status**: Future  
**Timeline**: 6+ months  
**Effort**: Ongoing refinement

#### **Goals**

sourDough is the definitive reference implementation and tooling platform.

#### **Requirements**

**1. Stability**
- [ ] All APIs stable (semantic versioning)
- [ ] Backward compatibility guaranteed
- [ ] Comprehensive test coverage (>90%)

**2. Documentation**
- [ ] Complete API documentation
- [ ] Comprehensive guides
- [ ] Video tutorials
- [ ] Case studies

**3. Ecosystem Adoption**
- [ ] All primals use `sourdough-core` traits
- [ ] All ecoBins use sourDough genomeBin tooling
- [ ] biomeOS uses sourDough for primal management
- [ ] neuralAPI uses sourDough for dependencies

**4. Performance**
- [ ] genomeBin creation < 1 minute
- [ ] genomeBin installation < 30 seconds
- [ ] Validation commands < 5 seconds

**5. Reliability**
- [ ] Zero known critical bugs
- [ ] Battle-tested in production
- [ ] Security audit complete

**Success Criteria**:
- ✅ Used by 100% of ecoPrimals
- ✅ Zero regressions in 6 months
- ✅ Community contributors
- ✅ Production deployments

---

## 📈 Milestones Summary

| Version | Milestone | Timeline | Effort | Status |
|---------|-----------|----------|--------|--------|
| v0.2.0 | Foundation Complete | Jan 2026 | - | ✅ Done |
| v0.3.0 | UniBin Implementation | Feb 2026 | 40-60h | 📝 Next |
| v0.4.0 | ecoBin Certification | Mar 2026 | 20-30h | ⏳ Planned |
| v0.5.0 | genomeBin Tooling | Apr 2026 | 60-80h | ⏳ Planned |
| v0.6.0 | sourDough genomeBin | May 2026 | 10-15h | ⏳ Planned |
| v0.7.0 | Integration Libraries | Jun 2026 | 60-80h | ⏳ Planned |
| v1.0.0 | Production Ready | Q4 2026 | Ongoing | 🎯 Goal |

**Total Development**: ~190-265 hours across 6-9 months

---

## 🎯 Immediate Next Steps (v0.3.0)

### **Week 1-2: CLI Foundation**

**Days 1-3**: Set up CLI infrastructure
- [ ] Create `crates/sourdough/` with `clap`
- [ ] Implement argument parsing
- [ ] Set up logging
- [ ] Basic `--help` and `--version`

**Days 4-7**: Scaffold commands
- [ ] Implement `scaffold new-primal`
- [ ] Implement `scaffold new-crate`
- [ ] Test scaffolding
- [ ] Documentation

**Days 8-10**: Validation commands
- [ ] Implement `validate primal`
- [ ] Implement `validate unibin`
- [ ] Implement `validate ecobin`
- [ ] Test validation
- [ ] Documentation

**Days 11-14**: Polish and release
- [ ] Doctor command
- [ ] Integration tests
- [ ] User documentation
- [ ] Release v0.3.0

---

## 🎊 Success Metrics

### **Technical Metrics**

- **Code Coverage**: >85% by v1.0.0
- **Build Time**: <30 seconds (clean build)
- **Binary Size**: <5MB (stripped, musl)
- **Dependencies**: <50 crates total
- **Documentation**: 100% public APIs documented

### **Adoption Metrics**

- **Primal Usage**: 100% of new primals use sourDough scaffolding
- **ecoBin Usage**: 100% of ecoBins use sourDough genomeBin tools
- **Integration**: biomeOS and neuralAPI use sourDough libraries

### **Quality Metrics**

- **Bugs**: <5 open critical bugs at any time
- **Response Time**: Issue triage <24 hours
- **Stability**: No regressions for 3+ months before v1.0.0

---

## 📚 Related Documents

- `SOURDOUGH_SPECIFICATION.md` - What sourDough is
- `ARCHITECTURE.md` - How sourDough is built
- [`DEVELOPMENT.md`](../DEVELOPMENT.md) - How to develop with sourDough ✅

---

**Date**: January 19, 2026  
**Version**: 0.2.0  
**Status**: Foundation complete, UniBin next  
**Timeline**: v1.0.0 by Q4 2026

🍞🧬🦀 **The path to the complete reference primal!** ✨

