# genomeBin Rust Evolution Plan

**Date**: January 19, 2026  
**Status**: 🔄 **PLANNING - Ready to Evolve from Bash to Rust**

---

## 🎯 **Vision**

**Current State**: genomeBin infrastructure is ~1,112 lines of bash scripts  
**Problem**: "Jelly strings" - fragile string manipulation, no type safety, limited concurrency  
**Goal**: Evolve to modern, idiomatic, concurrent Rust implementation

**Philosophy**: "First solution works, Rust solution scales and is maintainable"

---

## 📊 **Current Implementation Analysis**

### **Bash Scripts** (1,112 lines total)

```
scripts/create-genomebin.sh     238 lines  - Archive creation
scripts/test-genomebin.sh       252 lines  - Testing & validation
scripts/sign-genomebin.sh       133 lines  - GPG signing
wrapper/genome-wrapper.sh       245 lines  - Self-extraction
wrapper/system-detection.sh     244 lines  - Platform detection
```

### **Problems with Current Bash Approach**

1. **String Manipulation** ("Jelly Strings")
   - Complex awk/sed/grep pipelines
   - Error-prone string parsing
   - Difficult to test edge cases

2. **Type Safety**
   - No compile-time guarantees
   - Runtime errors only
   - Hidden bugs (like the extraction bug we fixed)

3. **Concurrency**
   - Sequential processing only
   - No parallel archive creation
   - Slow for multi-architecture builds

4. **Error Handling**
   - `set -euo pipefail` is crude
   - No structured error types
   - Difficult to recover gracefully

5. **Testing**
   - Integration tests only
   - No unit tests
   - Hard to mock dependencies

6. **Portability**
   - Bash version dependencies
   - Platform-specific commands (grep -a, etc.)
   - Complex escape sequences

---

## 🦀 **Rust Evolution Strategy**

### **Phase 1: Core Library** ✨ **HIGH PRIORITY**

**Create**: `crates/sourdough-genomebin/` (new crate)

**Modules**:

```rust
sourdough-genomebin/
├── src/
│   ├── lib.rs              // Public API
│   ├── builder.rs          // GenomeBin builder (replaces create-genomebin.sh)
│   ├── extractor.rs        // Self-extraction logic (replaces wrapper parts)
│   ├── validator.rs        // Testing & validation (replaces test-genomebin.sh)
│   ├── signer.rs           // GPG signing (replaces sign-genomebin.sh)
│   ├── platform.rs         // System detection (replaces system-detection.sh)
│   ├── metadata.rs         // metadata.toml handling
│   ├── archive.rs          // Tar/gzip operations (using tar-rs, flate2)
│   └── error.rs            // Structured error types
└── Cargo.toml
```

**Benefits**:
- ✅ Type-safe API
- ✅ Unit testable
- ✅ Zero string manipulation bugs
- ✅ Concurrent archive creation
- ✅ Better error messages
- ✅ Reusable library (other primals can use it)

---

### **Phase 2: CLI Integration** ✨ **HIGH PRIORITY**

**Update**: `crates/sourdough/src/commands/genomebin.rs`

**Current** (delegates to bash):
```rust
async fn create_genomebin(...) -> Result<()> {
    let script_path = find_genomebin_script("create-genomebin.sh")?;
    let status = std::process::Command::new(&script_path)
        .arg("--primal").arg(&primal)
        // ... more args
        .status()?;
    // ...
}
```

**Evolved** (uses Rust library):
```rust
async fn create_genomebin(...) -> Result<()> {
    use sourdough_genomebin::GenomeBinBuilder;
    
    let genome = GenomeBinBuilder::new(primal, version)
        .ecobins_dir(ecobins)
        .output(output)
        .build()
        .await?;
    
    genome.create().await?;
    genome.test().await?;
    
    info!("✅ genomeBin created: {}", genome.path().display());
    Ok(())
}
```

**Benefits**:
- ✅ No external script dependencies
- ✅ Better error handling
- ✅ Progress reporting
- ✅ Cancellation support
- ✅ Async/concurrent operations

---

### **Phase 3: Self-Extraction Wrapper** 🔄 **MEDIUM PRIORITY**

**Two Approaches**:

#### **Approach A: Keep Bash Wrapper** (Simpler)
- Keep minimal bash wrapper (50-100 lines)
- Extract to temp dir
- Execute embedded Rust binary for installation
- Rust binary handles: system detection, binary selection, installation

#### **Approach B: Pure Rust Wrapper** (Ideal, Complex)
- Self-extracting Rust binary
- Embedded tar.gz as `include_bytes!()` or appended
- Extract using Rust (tar-rs, flate2)
- Challenge: Cross-platform shebang replacement

**Recommendation**: Start with **Approach A**, evolve to **Approach B** later

---

## 📦 **Rust Crate Design**

### **sourdough-genomebin Library**

```toml
[package]
name = "sourdough-genomebin"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
tar = "0.4"
flate2 = "1.0"
blake3 = "1.5"
sha2 = "0.10"
bytes = "1.5"

# Optional: GPG signing
gpgme = { version = "0.11", optional = true }

# Platform detection
sysinfo = "0.30"

[features]
default = ["signing"]
signing = ["gpgme"]
```

### **Core Types**

```rust
/// GenomeBin builder with fluent API
pub struct GenomeBinBuilder {
    primal: String,
    version: String,
    ecobins_dir: PathBuf,
    output: PathBuf,
    config: Option<PathBuf>,
}

/// Represents a genomeBin package
pub struct GenomeBin {
    metadata: Metadata,
    ecobins: Vec<EcoBin>,
    config: Option<Config>,
    path: PathBuf,
}

/// Platform detection
pub struct Platform {
    os: Os,
    arch: Arch,
    libc: LibC,
    target: String,
}

/// Archive operations
pub struct Archive {
    path: PathBuf,
    checksum: Blake3Hash,
}
```

---

## 🚀 **Implementation Phases**

### **Phase 1: Foundation** (2-3 hours)

**Tasks**:
1. ✅ Create `crates/sourdough-genomebin/` crate
2. ✅ Implement basic types (Metadata, Platform, EcoBin)
3. ✅ Implement platform detection (replace system-detection.sh)
4. ✅ Add comprehensive tests

**Deliverable**: Platform detection as Rust library

---

### **Phase 2: Archive Creation** (3-4 hours)

**Tasks**:
1. ✅ Implement `GenomeBinBuilder`
2. ✅ Replace create-genomebin.sh logic
3. ✅ Use tar-rs for archive creation
4. ✅ Use flate2 for compression
5. ✅ Concurrent ecoBin processing
6. ✅ Add progress reporting

**Deliverable**: Rust-based genomeBin creation

---

### **Phase 3: Validation** (2-3 hours)

**Tasks**:
1. ✅ Implement `GenomeBinValidator`
2. ✅ Replace test-genomebin.sh logic
3. ✅ Unit tests for each validation
4. ✅ Better error messages

**Deliverable**: Rust-based genomeBin testing

---

### **Phase 4: Signing** (1-2 hours)

**Tasks**:
1. ✅ Implement GPG signing via gpgme
2. ✅ Replace sign-genomebin.sh
3. ✅ Signature verification

**Deliverable**: Rust-based genomeBin signing

---

### **Phase 5: CLI Integration** (1-2 hours)

**Tasks**:
1. ✅ Update `crates/sourdough/src/commands/genomebin.rs`
2. ✅ Remove bash script dependencies
3. ✅ Add progress indicators
4. ✅ Better error reporting

**Deliverable**: Pure Rust genomeBin CLI

---

### **Phase 6: Wrapper Evolution** (4-6 hours, optional)

**Tasks**:
1. 🔄 Design Rust-based self-extraction
2. 🔄 Implement binary appending/extraction
3. 🔄 Cross-platform testing
4. 🔄 Fallback to bash if needed

**Deliverable**: Optional pure Rust wrapper

---

## 💪 **Benefits of Rust Evolution**

### **Type Safety**
```rust
// Before (bash):
PRIMAL_NAME=""  # Could be anything
VERSION=""      # Could be invalid

# After (Rust):
pub struct GenomeBin {
    primal: PrimalName,    // Validated at construction
    version: SemVer,       // Type-safe version
}
```

### **Concurrency**
```rust
// Process multiple architectures concurrently
let tasks: Vec<_> = ecobins.iter()
    .map(|ecobin| tokio::spawn(async move {
        ecobin.validate().await
    }))
    .collect();

let results = futures::future::join_all(tasks).await;
```

### **Error Handling**
```rust
#[derive(Debug, thiserror::Error)]
pub enum GenomeBinError {
    #[error("Invalid primal name: {0}")]
    InvalidPrimal(String),
    
    #[error("EcoBin not found for target {target}")]
    EcoBinNotFound { target: String },
    
    #[error("Archive creation failed: {0}")]
    ArchiveError(#[from] std::io::Error),
}
```

### **Testing**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn platform_detection_x86_64_linux() {
        let platform = Platform::detect();
        assert_eq!(platform.arch, Arch::X86_64);
        assert_eq!(platform.os, Os::Linux);
    }
    
    #[tokio::test]
    async fn genomebin_builder_creates_valid_archive() {
        let genome = GenomeBinBuilder::new("test", "1.0.0")
            .ecobins_dir(test_dir())
            .build()
            .await
            .unwrap();
        
        assert!(genome.path().exists());
        assert!(genome.validate().await.is_ok());
    }
}
```

---

## 📈 **Performance Improvements**

### **Current Bash** (Sequential)
```bash
# Sequential processing
for file in "${ECOBIN_FILES[@]}"; do
    cp "$file" "$TEMP_DIR/ecobins/"  # One at a time
done
tar -czf "$PAYLOAD_TAR" ...          # Single-threaded
```

**Time**: ~5-10 seconds for multi-arch genomeBin

### **Rust** (Concurrent)
```rust
// Parallel processing
let tasks: Vec<_> = ecobin_files.iter()
    .map(|file| tokio::spawn(async move {
        tokio::fs::copy(file, temp_dir.join(file.name())).await
    }))
    .collect();

futures::future::try_join_all(tasks).await?;

// Parallel compression (flate2 with rayon)
Archive::builder()
    .compression_level(6)
    .parallel(true)
    .build()?;
```

**Expected Time**: ~2-3 seconds (2-3x faster)

---

## 🎓 **Learning from Bash Bugs**

### **Extraction Bug** (We Fixed)
**Bash**:
```bash
# Bug: awk outputs wrapper code, not binary
awk '/EMBEDDED_PAYLOAD/ {found=1; next} found' "$0" | tar -xzf -
```

**Rust** (Type-safe):
```rust
pub fn extract_payload(archive: &Path) -> Result<Vec<u8>> {
    let file = File::open(archive)?;
    let reader = BufReader::new(file);
    
    // Type-safe: find marker, skip to next line
    let marker = b"# === EMBEDDED_PAYLOAD ===";
    let mut found = false;
    
    for line in reader.lines() {
        if found {
            // Read remaining as binary
            return read_to_end();
        }
        if line?.as_bytes() == marker {
            found = true;
        }
    }
    
    Err(GenomeBinError::PayloadNotFound)
}
```

**Benefits**:
- Compile-time guarantees
- Unit testable
- Clear error handling
- No silent failures

---

## 🗺️ **Migration Path**

### **Incremental Evolution** (Recommended)

```
Phase 1: Rust library exists alongside bash scripts
  ├─ Bash scripts remain functional (backward compat)
  └─ Rust library used optionally (--use-rust flag)

Phase 2: Rust becomes default, bash is fallback
  ├─ CLI uses Rust by default
  └─ Bash scripts available for testing

Phase 3: Bash scripts moved to archive/legacy/
  ├─ Rust is only implementation
  └─ Bash kept for reference/comparison

Phase 4: Pure Rust ecosystem
  └─ All genomeBin operations in Rust
```

---

## 📝 **Proposed File Structure**

```
sourDough/
├── crates/
│   ├── sourdough-genomebin/     # NEW: Core genomeBin library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── builder.rs       # GenomeBin creation
│   │   │   ├── extractor.rs     # Self-extraction
│   │   │   ├── validator.rs     # Testing
│   │   │   ├── signer.rs        # Signing
│   │   │   ├── platform.rs      # Platform detection
│   │   │   ├── metadata.rs      # Metadata handling
│   │   │   ├── archive.rs       # Tar/gzip ops
│   │   │   └── error.rs         # Error types
│   │   ├── tests/
│   │   │   ├── builder_tests.rs
│   │   │   └── integration/
│   │   └── Cargo.toml
│   │
│   ├── sourdough/               # UPDATED: Use genomebin library
│   │   └── src/commands/genomebin.rs
│   │
│   └── sourdough-core/          # Existing
│
├── genomebin/
│   ├── legacy/                  # MOVED: Bash scripts (reference)
│   │   ├── create-genomebin.sh
│   │   ├── test-genomebin.sh
│   │   └── sign-genomebin.sh
│   │
│   ├── wrapper/                 # MINIMAL: Thin bash wrapper
│   │   └── genome-wrapper.sh    # (50-100 lines, calls Rust)
│   │
│   └── README.md                # UPDATED: Rust-first docs
```

---

## 🎯 **Success Metrics**

### **Code Quality**
- ✅ Zero unsafe code
- ✅ 95%+ test coverage
- ✅ Zero clippy warnings (pedantic)
- ✅ Full type safety

### **Performance**
- ✅ 2-3x faster genomeBin creation
- ✅ Concurrent processing
- ✅ Streaming compression

### **Maintainability**
- ✅ Unit testable components
- ✅ Clear error messages
- ✅ Self-documenting API
- ✅ Reusable library

### **Reliability**
- ✅ No string parsing bugs
- ✅ Compile-time guarantees
- ✅ Better error recovery
- ✅ Graceful degradation

---

## 🚦 **Recommendation: START NOW**

### **Immediate Next Steps**

1. **Create the crate** (30 minutes)
   ```bash
   cd crates/
   cargo new --lib sourdough-genomebin
   ```

2. **Implement platform detection** (2 hours)
   - Replace system-detection.sh with Rust
   - Add comprehensive tests
   - Proves the concept

3. **Benchmark** (30 minutes)
   - Compare Rust vs Bash platform detection
   - Measure accuracy and performance

4. **Continue incrementally** (ongoing)
   - One module at a time
   - Maintain bash scripts during transition
   - Test extensively

---

## 📊 **Estimated Timeline**

**Total Effort**: 15-20 hours (2-3 days)

```
Day 1: Foundation + Platform Detection (4-6 hours)
  - Create crate
  - Implement Platform module
  - Tests

Day 2: Archive Creation + Validation (6-8 hours)
  - Implement GenomeBinBuilder
  - Implement Validator
  - Integration tests

Day 3: Signing + CLI Integration (4-6 hours)
  - Implement Signer
  - Update CLI to use Rust
  - End-to-end testing
```

---

## 💡 **Why This Matters**

### **Ecosystem Impact**

1. **Reusability**
   - Other primals can use `sourdough-genomebin` library
   - Publish to crates.io
   - Standard genomeBin implementation

2. **Reliability**
   - Type safety prevents bugs
   - Better testing coverage
   - Fewer production issues

3. **Performance**
   - Faster CI/CD pipelines
   - Concurrent processing
   - Better resource usage

4. **Maintainability**
   - Clear, documented code
   - Easy to extend
   - Standard Rust patterns

---

## 🎉 **Vision: Pure Rust Ecosystem**

**Goal**: sourDough as 100% Rust reference implementation

```
✅ Core traits         - Rust
✅ CLI commands        - Rust
✅ RPC communication   - Rust (tarpc)
✅ Scaffolding         - Rust (generates Rust)
✅ Validation          - Rust
🔄 genomeBin infra    - Rust (in progress)
```

**Result**: True ecoBin reference - Pure Rust from top to bottom!

---

**Status**: 🚀 **READY TO BEGIN**  
**Priority**: HIGH (improves reliability, performance, maintainability)  
**Complexity**: Medium (well-defined scope, incremental approach)

🧬🦀✨ **Let's evolve to idiomatic, concurrent Rust!** ✨🦀🧬

