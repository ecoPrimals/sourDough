# ecoPrimals Coding Conventions

These conventions apply to all primals in the ecoPrimals ecosystem. Consistency enables AI agents and humans to work effectively across the codebase.

---

## 1. Project Structure

### Standard Primal Layout

```
primalName/
├── Cargo.toml              # Workspace manifest
├── README.md               # Overview, quick start
├── STATUS.md               # Current status, grades
├── WHATS_NEXT.md          # Roadmap, next steps
├── START_HERE.md          # New developer guide
├── crates/
│   ├── primalname-core/   # Core library (required)
│   ├── primalname-cli/    # CLI tool (optional)
│   └── primalname-*/      # Additional crates
├── specs/
│   └── *.md               # Specifications
├── showcase/
│   └── */                 # Demonstrations
└── tests/
    └── integration/       # Integration tests
```

### Crate Naming

- Use kebab-case: `primalname-core`, `primalname-storage`
- Core crate is always `primalname-core`
- CLI crate is always `primalname-cli`

---

## 2. Rust Conventions

### Code Style

- **Edition**: 2024
- **Linting**: `#![warn(clippy::all, clippy::pedantic)]`
- **Docs**: `#![warn(missing_docs)]`
- **Format**: `cargo fmt` (rustfmt defaults)

### Error Handling

```rust
// Use thiserror for library errors
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("description: {0}")]
    Variant(String),
}

// Use anyhow for application errors (CLI)
fn main() -> anyhow::Result<()> {
    // ...
}
```

### Async

- Use `tokio` runtime
- Prefer `async fn` over `impl Future`
- Use `Send + Sync` bounds for trait objects

### Logging

```rust
use tracing::{info, warn, error, debug, trace};

// Structured logging
info!(user = %user_id, action = "login", "user logged in");
```

---

## 3. API Conventions

### RPC Communication (Primary)

- Use `tarpc` for type-safe inter-primal communication
- Define service traits with `#[tarpc::service]`
- All RPC methods are async
- Use zero-copy types (`bytes::Bytes`) for large payloads
- Implement `PrimalRpc` trait for standard methods

```rust
#[tarpc::service]
pub trait MyPrimalRpc {
    async fn my_method(data: Vec<u8>) -> Result<Response, String>;
}
```

### REST APIs (Optional)

- Versioned paths: `/api/v1/...`
- JSON responses
- Standard HTTP status codes
- OpenAPI 3.0 specs

### Common Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | Health check |
| `GET /ready` | Readiness check |
| `GET /metrics` | Prometheus metrics |

### Service Discovery

- **Zero hardcoding** - no primal names, service names, ports, or endpoints
- Use port 0 for OS-assigned ephemeral ports
- Register with discovery service on startup (discovered via universal adapter)
- Discover other services at runtime via universal adapter
- Primal knows only itself, discovers everything else like an infant learning its environment

---

## 4. Configuration

### File Format

- TOML for configuration files
- Environment variables for secrets
- Prefix: `{PRIMALNAME}_`

### Standard Config Structure

```toml
# Common configuration
name = "primalname"
instance_id = "auto"
log_level = "info"
data_dir = "./data"
listen_addr = "0.0.0.0"
listen_port = 0  # OS assigns ephemeral port, discovered via Songbird

# BearDog integration (discovered at runtime)
# No hardcoded endpoints - use service discovery

# Songbird integration (discovered at runtime)
# No hardcoded endpoints - use service discovery

# Primal-specific configuration
[primal]
# ...
```

---

## 5. Documentation

### Required Files

| File | Purpose |
|------|---------|
| `README.md` | Overview, quick start |
| `STATUS.md` | Current status |
| `WHATS_NEXT.md` | Roadmap |
| `START_HERE.md` | New developer guide |

### Code Documentation

- All public items must have doc comments
- Include examples for complex functions
- Link to related items

```rust
/// Brief description.
///
/// Longer description with details.
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 42);
/// ```
///
/// # Errors
///
/// Returns an error if...
///
/// # Panics
///
/// Panics if...
pub fn my_function(x: i32) -> Result<i32, Error> {
    // ...
}
```

---

## 6. Testing

### Test Organization

```
crates/primalname-core/
├── src/
│   └── lib.rs
└── tests/
    └── integration.rs     # Integration tests

tests/
└── e2e/                   # End-to-end tests
```

### Test Naming

```rust
#[test]
fn test_function_name_expected_behavior() {
    // ...
}

#[tokio::test]
async fn test_async_function() {
    // ...
}
```

### Coverage Target

- Unit tests: 80%+ coverage
- Integration tests: Critical paths
- E2E tests: User flows

---

## 7. Version Control

### Commit Messages

```
type(scope): description

body (optional)

footer (optional)
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

### Branch Naming

- `main`: Stable
- `dev`: Development
- `feat/description`: Feature branches
- `fix/description`: Bug fixes

---

## 8. Dependencies

### Workspace Dependencies

Define in workspace `Cargo.toml`:

```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
# ...
```

### Version Policy

- Use workspace versions
- Pin major versions
- Update quarterly

### Required Dependencies

All primals should use:
- `sourdough-core` — Common traits and RPC layer
- `tokio` — Async runtime
- `tarpc` — RPC communication
- `serde` — Serialization
- `bytes` — Zero-copy buffers
- `thiserror` — Error handling (libraries)
- `anyhow` — Error handling (applications)
- `tracing` — Logging

---

## 9. Security

### Secrets

- Never commit secrets
- Use environment variables
- Integrate with BearDog for crypto

### Input Validation

- Validate all external input
- Use strong types
- Fail fast, fail safe

### Unsafe Code

- Minimize `unsafe`
- Document all `unsafe` blocks
- Gate platform-specific code

---

## 10. Performance

### General Guidelines

- Profile before optimizing
- Prefer zero-copy where possible
- Use streaming for large data
- Cache expensive operations

### Memory

- Avoid unnecessary allocations
- Use `Cow` for conditional ownership
- Consider arena allocation for hot paths

### Async

- Don't block the runtime
- Use `spawn_blocking` for CPU-bound work
- Batch small operations

---

## 11. Integration

### SourDough Traits

All primals should implement:

| Trait | Purpose |
|-------|---------|
| `PrimalLifecycle` | Start/stop/reload |
| `PrimalHealth` | Health checks |
| `PrimalConfig` | Configuration |

Optional traits:
| Trait | Purpose |
|-------|---------|
| `PrimalIdentity` | BearDog integration |
| `PrimalDiscovery` | Songbird integration |

### Identity Service Integration

- Use DIDs for identity
- Sign important actions via identity service (discovered at runtime)
- Verify external signatures
- No hardcoded identity service names or endpoints

### Discovery Service Integration

- Register with discovery service on startup (discovered via universal adapter)
- Deregister on shutdown
- Discover other services at runtime
- **Zero compile-time dependencies** on other primals or services
- Use capability-based addressing (no hardcoded ports)
- Infant-like awakening: start with zero knowledge, learn environment

---

*Consistency is the foundation of collaboration.*

