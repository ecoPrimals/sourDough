# Wave 157a — sourDough G68 Platform Substrate Abstraction

**Date**: August 7, 2026
**Wave**: 157a
**Status**: COMPLETE

---

## What Shipped

sourDough now provides the **reference implementation** for G68 Platform Substrate Abstraction. Three layers defined, two implemented (L1 + L2), L3 deferred to domain-specific primals.

### sourdough-core Module: `platform_substrate.rs`

| Component | Purpose |
|-----------|---------|
| `platform_link(original, link)` | L1: symlink (Unix), junction/hard-link (Windows), hard-link (other) |
| `is_symlink(path)` | Cross-platform symlink detection |
| `PlatformAccess` enum | L2: OwnerReadWrite, OwnerFull, PublicRead, PublicExecute, Readonly, Custom(u32) |
| `PlatformAccess::apply(path)` | Apply access level (mode on Unix, readonly attr on Windows) |
| `query_access(path)` | Query current access level |
| `ensure_dir_with_access(path, access)` | Create dir + set permissions atomically |
| `ensure_secure_parent(path)` | Ensure parent dir exists with owner-only access |

### Validation Tooling: `sourdough validate platform-substrate`

Detects silicon deism violations in three layers:
- **L1**: Raw `symlink()` outside `platform_substrate` module
- **L2**: Raw `PermissionsExt`/`set_mode()`/`from_mode()` outside platform modules
- **L3**: Raw `rustix`/`libc`/`nix` imports outside cfg-guarded modules

JSON output (`--json`) for CI integration.

### Scaffold Templates

New primals automatically receive `platform_substrate.rs` in their core crate. Zero dependency on `sourdough-core` — reference by example.

---

## Cross-Architecture

| Target | Status |
|--------|--------|
| x86_64-unknown-linux-gnu | PASS |
| x86_64-pc-windows-msvc | PASS |
| aarch64-linux-android | PASS |

---

## Test Coverage

563 tests total (18 new for platform substrate). Zero failures, zero ignored.

---

## Ecosystem Adoption Path

1. **New primals**: Automatically get `platform_substrate.rs` from scaffold.
2. **Existing primals**: Run `sourdough validate platform-substrate .` to find violations.
3. **Fix pattern**: Replace raw APIs with `platform_link()` / `PlatformAccess` / backend traits.
4. **Audit target**: 134+ files across 15 primals (17 L1, 56+ L2, 37 L3).

---

## Files Changed

| File | Change |
|------|--------|
| `crates/sourdough-core/src/platform_substrate.rs` | NEW — G68 reference module |
| `crates/sourdough-core/src/lib.rs` | Register module + re-exports |
| `crates/sourdough/src/commands/validate/platform_substrate.rs` | NEW — validator |
| `crates/sourdough/src/commands/validate/mod.rs` | Register subcommand |
| `crates/sourdough/src/commands/scaffold/templates/core.rs` | Add `PLATFORM_SUBSTRATE_RS` template |
| `crates/sourdough/src/commands/scaffold/templates/mod.rs` | Re-export template |
| `crates/sourdough/src/commands/scaffold/generators.rs` | Write template to scaffolded crate |
| `specs/PLATFORM_SUBSTRATE_SPEC.md` | NEW — full G68 specification |
| `specs/ARCHITECTURE.md` | Updated |
| `STATUS.md` | Updated |
| `WHATS_NEXT.md` | Updated |
| `README.md` | Updated |

---

*Wave 157a — G68 Platform Substrate reference. L1 links + L2 permissions + L3 spec. Validator + scaffold. 563 tests, 15/15 cross-arch.*
