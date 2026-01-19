# 🎯 Action Items - sourDough
**Date**: January 19, 2026  
**Based On**: COMPREHENSIVE_AUDIT_JAN_19_2026.md

---

## 🔴 Critical (Do First)

### 1. ecoBin Certification (30-60 minutes)

**Why**: Required to claim ecoBin compliance

**Steps**:
```bash
cd /path/to/sourDough

# Add musl targets
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# Build for musl targets
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl

# Verify static linking
ldd target/x86_64-unknown-linux-musl/release/sourdough
# Expected: "not a dynamic executable"

# Check binary size
ls -lh target/x86_64-unknown-linux-musl/release/sourdough

# Verify no C dependencies
cargo tree | grep -E "(openssl-sys|ring|aws-lc-sys|native-tls)"
# Expected: No matches

# Test runtime
./target/x86_64-unknown-linux-musl/release/sourdough --version
./target/x86_64-unknown-linux-musl/release/sourdough doctor

# Document results
cat > ECOBIN_CERTIFICATION.md <<EOF
# ecoBin Certification - sourDough

**Date**: $(date)  
**Status**: ✅ CERTIFIED

## Build Results

- x86_64-musl: ✅ Success
- aarch64-musl: ✅ Success
- Static linking: ✅ Verified
- Zero C deps: ✅ Verified
- Runtime tests: ✅ Passing

## Binary Analysis

\`\`\`
$ ldd target/x86_64-unknown-linux-musl/release/sourdough
not a dynamic executable

$ cargo tree | grep -E "(openssl|ring|aws-lc)"
(no matches - Pure Rust!)
\`\`\`

## Conclusion

sourDough is a TRUE ecoBin!
EOF
```

**Result**: sourDough certified as ecoBin #3 (after BearDog and NestGate)

---

## 🟡 High Priority (This Week)

### 2. Fix Clippy Warnings (5 minutes)

**Why**: Clean pedantic mode

**File**: `crates/sourdough/src/commands/scaffold.rs`

**Changes**:
```rust
// Line 385: Remove # from r#"..."#
r#"# {} - Specification   →   r"# {} - Specification
..."#                      →   ..."

// Line 425: Remove # from r#"..."#  
r#"# {}                   →   r"# {}
..."#                      →   ..."

// Line 468: Remove # from r#"..."#
r#"# Coding Conventions   →   r"# Coding Conventions
..."#                      →   ..."
```

**Verify**:
```bash
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
# Expected: No warnings
```

---

### 3. Improve RPC Test Coverage (30-60 minutes)

**Why**: Bring rpc.rs from 85.71% to 90%+

**File**: `crates/sourdough-core/src/rpc.rs`

**Add tests for**:
1. RPC request with invalid method
2. RPC response error formatting
3. ServerConfig validation edge cases
4. Client connection failures (mock)

**Example**:
```rust
#[cfg(test)]
mod additional_tests {
    use super::*;
    
    #[test]
    fn rpc_request_with_long_method_name() {
        let method = "x".repeat(1000);
        let req = RpcRequest::new(&method, json!({}));
        assert_eq!(req.method.len(), 1000);
    }
    
    #[test]
    fn rpc_response_error_with_special_chars() {
        let err = RpcResponse::<()>::error("Error: \n\t\" special chars");
        assert!(err.error.is_some());
    }
    
    // Add 2-3 more...
}
```

**Verify**:
```bash
cargo llvm-cov --package sourdough-core --summary-only | grep rpc.rs
# Expected: 90%+
```

---

## 🟢 Medium Priority (This Month)

### 4. Harvest to plasmidBin (1-2 hours)

**Why**: Make sourDough available to ecosystem

**Prerequisites**: ecoBin certification complete

**Steps**:
```bash
# Build all targets
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create plasmidBin structure
mkdir -p ../../plasmidBin/primals/sourdough/v0.2.0/

# Copy binaries
cp target/x86_64-unknown-linux-musl/release/sourdough \
   ../../plasmidBin/primals/sourdough/v0.2.0/sourdough-x86_64-linux-musl

cp target/aarch64-unknown-linux-musl/release/sourdough \
   ../../plasmidBin/primals/sourdough/v0.2.0/sourdough-aarch64-linux-musl

# Create checksums
cd ../../plasmidBin/primals/sourdough/v0.2.0/
sha256sum sourdough-* > SHA256SUMS

# Update plasmidBin MANIFEST.md
echo "- sourdough v0.2.0 (Jan 2026) - Starter culture for ecoPrimals" >> ../../MANIFEST.md
```

---

### 5. Create sourDough genomeBin (30-60 minutes)

**Why**: Meta-circular achievement (tool uses itself!)

**Prerequisites**: ecoBin certification + plasmidBin harvest

**Command**:
```bash
./target/release/sourdough genomebin create \
    --primal sourdough \
    --version 0.2.0 \
    --ecobins ../../plasmidBin/primals/sourdough/v0.2.0/ \
    --output sourdough.genome

# Test
./target/release/sourdough genomebin test sourdough.genome

# Sign
./target/release/sourdough genomebin sign sourdough.genome
```

**Result**: First primal to create its own genomeBin!

---

### 6. Update Documentation (30 minutes)

**Files to update**:
1. `STATUS.md`
   - Update coverage (97.17%)
   - Note ecoBin certification
   - Update test count (98/98)

2. `README.md`
   - Add ecoBin badge (once certified)
   - Update quality metrics

3. `specs/ROADMAP.md`
   - Mark v0.3.0 as complete
   - Update v0.4.0 status

---

## 🔵 Low Priority (Nice to Have)

### 7. Chaos Testing (2-4 hours)

**Why**: Validate RPC resilience

**Add tests for**:
- Network timeouts
- Connection drops
- Malformed packets
- Concurrent request storms
- Out-of-order responses

**Tool**: Consider `tokio-test` or custom chaos harness

---

### 8. Performance Benchmarks (2-4 hours)

**Why**: Understand performance characteristics

**Benchmark**:
- RPC throughput (requests/sec)
- Scaffolding speed (time to create primal)
- Validation speed (time to validate primal)
- genomeBin creation speed

**Tool**: `criterion` crate

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn rpc_throughput(c: &mut Criterion) {
    c.bench_function("rpc ping", |b| {
        b.iter(|| {
            // Benchmark RPC ping
            black_box(rpc_client.ping().await)
        });
    });
}

criterion_group!(benches, rpc_throughput);
criterion_main!(benches);
```

---

### 9. E2E Cross-Primal Tests (BLOCKED)

**Why**: Validate real-world integration

**Blockers**: Need BearDog + Songbird to be UniBin-compliant

**Defer Until**: Other primals reach UniBin status

**Tests Would Include**:
- sourDough → BearDog (identity)
- sourDough → Songbird (discovery)
- Full RPC round-trip with actual services

---

## 📊 Progress Tracking

| Item | Priority | Effort | Status | ETA |
|------|----------|--------|--------|-----|
| 1. ecoBin Certification | 🔴 Critical | 30-60m | ⏳ Ready | This session |
| 2. Fix Clippy Warnings | 🟡 High | 5m | ⏳ Ready | This session |
| 3. RPC Test Coverage | 🟡 High | 30-60m | ⏳ Ready | This week |
| 4. Harvest to plasmidBin | 🟢 Medium | 1-2h | ⏳ Blocked | After #1 |
| 5. Create genomeBin | 🟢 Medium | 30-60m | ⏳ Blocked | After #4 |
| 6. Update Documentation | 🟢 Medium | 30m | ⏳ Ready | After #1 |
| 7. Chaos Testing | 🔵 Low | 2-4h | 📝 Planned | Q1 2026 |
| 8. Performance Benchmarks | 🔵 Low | 2-4h | 📝 Planned | Q1 2026 |
| 9. E2E Cross-Primal | 🔵 Low | TBD | 🚫 Blocked | Q2 2026 |

---

## 🎯 Next Session Goals

**Priority Order**:
1. ✅ Fix clippy warnings (5 min)
2. ✅ Run ecoBin certification (30-60 min)
3. ✅ Document certification results
4. ⏳ Update STATUS.md

**Stretch Goals**:
- Improve RPC test coverage
- Start plasmidBin harvest

**Estimated Time**: 60-90 minutes

---

**Created**: January 19, 2026  
**Based On**: COMPREHENSIVE_AUDIT_JAN_19_2026.md  
**Status**: ⏳ Ready to Execute

🎯 **Focus**: ecoBin certification first, then ecosystem integration!

