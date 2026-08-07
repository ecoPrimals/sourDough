# G68: Platform Substrate Abstraction

**Version**: 1.0.0
**Date**: August 7, 2026
**Status**: Active
**Owner**: sourDough (reference by example)

---

## Philosophy

`#[cfg(unix)]` is silicon deism wearing a mask. G66 solved transport. G68 solves the rest.

**The test**: "Does this primal do *less* on Windows, or the *same thing differently*?"
If less → silicon deism. If differently → platform abstraction.

---

## Three Abstraction Layers

### L1: Links

| Platform | Mechanism |
|----------|-----------|
| Unix | `symlink(original, link)` |
| Windows | `symlink_file` / `symlink_dir` (fallback: `hard_link`) |
| Other | `hard_link` |

**API**: `platform_link(original: &Path, link: &Path) -> io::Result<()>`

### L2: Permissions

| Variant | Unix mode | Windows |
|---------|-----------|---------|
| `OwnerReadWrite` | 0o600 | not-readonly |
| `OwnerFull` | 0o700 | not-readonly |
| `PublicRead` | 0o644 | not-readonly |
| `PublicExecute` | 0o755 | not-readonly |
| `Readonly` | 0o400 | readonly |
| `Custom(u32)` | exact bits | no-op |

**API**: `PlatformAccess.apply(path)`, `query_access(path)`

### L3: Device Backends

Domain-specific (VFIO, mmap, DRM, etc.). Pattern:

```rust
pub trait DeviceBackend: Send + Sync {
    async fn open(&self, config: &Config) -> Result<Handle>;
    async fn read(&self, handle: &Handle, buf: &mut [u8]) -> Result<usize>;
    async fn write(&self, handle: &Handle, data: &[u8]) -> Result<usize>;
}

#[cfg(unix)]
mod linux_backend { /* ... */ }

#[cfg(windows)]
mod windows_backend { /* ... */ }
```

---

## Audit Results (ecosystem-wide)

| Layer | Files | Primals | Fix |
|-------|-------|---------|-----|
| L1 | 17 | 10 | `platform_link()` |
| L2 | 56+ | 13 | `PlatformAccess` |
| L3 | 37 | 3 (toadStool, rustChip, cellMembrane) | Backend trait |

Total: **134+ files across 15 primals**.

---

## Convergence Pattern

sourDough provides the reference (by example). Primals converge independently:

1. Each primal adds `platform_substrate.rs` to their core crate (scaffolded automatically for new primals).
2. Replace raw `symlink()` → `platform_link()`.
3. Replace raw `set_mode()` / `PermissionsExt` → `PlatformAccess.apply()`.
4. Replace raw `rustix`/`libc` → platform-gated backend traits.
5. Validate with `sourdough validate platform-substrate <path>`.

---

## Validation Tooling

```bash
# Check a single primal
sourdough validate platform-substrate /path/to/primal

# JSON output for CI
sourdough validate platform-substrate /path/to/primal --json
```

Checks:
- L1 violations: raw `symlink` outside `platform_substrate` module
- L2 violations: raw `PermissionsExt`/`set_mode`/`from_mode` outside `platform_substrate`
- L3 violations: raw `rustix`/`libc`/`nix` imports outside platform-gated modules
- Positive detection: presence of `PlatformAccess`, `platform_link`, etc.

---

## Relationship to Other Goals

| Goal | Scope | Relationship |
|------|-------|-------------|
| G66 | Transport (sockets, streams) | G68 extends beyond transport |
| G68 | Filesystem + device I/O | Completes platform neutrality |
| G64 | Protocol (tarpc/JSON-RPC) | Orthogonal |
| G65 | Protocol negotiation | Orthogonal |

---

*G68 — Platform substrate abstraction. 134+ files, 3 layers. sourDough leads by example.*
