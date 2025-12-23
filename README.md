# 🍞 SourDough — The Primal Starter Culture

**Status:** Nascent / Scaffolding Template  
**Purpose:** Birthing ground for new ecoPrimals  

---

## What is SourDough?

SourDough is not a primal itself—it's the **starter culture** from which new primals are born. Just as sourdough bread requires a living starter to leaven new loaves, SourDough provides the essential patterns, traits, and scaffolding that any new primal needs to integrate with the ecoPrimals ecosystem.

When you create a new primal, you don't start from zero. You start from SourDough.

---

## Philosophy

### Minimal by Design

SourDough provides only what's universal:
- Common traits that all primals implement
- Integration patterns with BearDog (identity)
- Integration patterns with Songbird (discovery)
- Standard configuration and error handling
- Documentation and spec templates

### Agnostic by Necessity

SourDough makes no assumptions about:
- What your primal does
- What data structures it uses
- What protocols it speaks
- What storage it needs

These decisions belong to the primal being created.

### Composable by Nature

Every trait in SourDough is:
- Optional (implement what you need)
- Modular (compose traits together)
- Extensible (add your own)

---

## Structure

```
sourDough/
├── README.md                 # You are here
├── Cargo.toml               # Workspace manifest
├── crates/
│   └── sourdough-core/      # Core traits all primals share
│       ├── src/
│       │   ├── lib.rs
│       │   ├── identity.rs  # BearDog integration traits
│       │   ├── discovery.rs # Songbird integration traits
│       │   ├── config.rs    # Configuration patterns
│       │   ├── error.rs     # Common error types
│       │   ├── health.rs    # Health check traits
│       │   └── lifecycle.rs # Primal lifecycle traits
│       └── Cargo.toml
├── specs/
│   └── SPEC_TEMPLATE.md     # Template for primal specs
├── templates/
│   ├── new-primal/          # Template for scaffolding
│   │   ├── Cargo.toml.tmpl
│   │   ├── src/
│   │   │   └── lib.rs.tmpl
│   │   └── README.md.tmpl
│   └── new-crate/           # Template for adding crates
│       ├── Cargo.toml.tmpl
│       └── src/
│           └── lib.rs.tmpl
├── scripts/
│   └── scaffold.sh          # Scaffold a new primal
└── CONVENTIONS.md           # Coding conventions for primals
```

---

## Usage

### Scaffolding a New Primal

```bash
# From the ecoPrimals root:
./sourDough/scripts/scaffold.sh new-primal rhizoCrypt "Ephemeral Data Graph"

# This creates:
# ../rhizoCrypt/
#   ├── Cargo.toml
#   ├── README.md
#   ├── crates/
#   │   └── rhizocrypt-core/
#   └── specs/
```

### Using SourDough Traits

```rust
// In your new primal's Cargo.toml:
[dependencies]
sourdough-core = { path = "../../sourDough/crates/sourdough-core" }

// In your code:
use sourdough_core::{
    PrimalIdentity,      // BearDog integration
    PrimalDiscovery,     // Songbird integration
    PrimalHealth,        // Health checks
    PrimalLifecycle,     // Start/stop/reload
    PrimalConfig,        // Configuration loading
};

pub struct MyPrimal {
    config: MyConfig,
    // ...
}

impl PrimalLifecycle for MyPrimal {
    async fn start(&mut self) -> Result<(), PrimalError> {
        // Your startup logic
    }
    
    async fn stop(&mut self) -> Result<(), PrimalError> {
        // Your shutdown logic
    }
}

impl PrimalHealth for MyPrimal {
    async fn health_check(&self) -> HealthStatus {
        // Your health check logic
    }
}
```

---

## Core Traits

### `PrimalIdentity` — BearDog Integration

Every primal needs identity:
- DID for the primal instance
- Signing capabilities
- Lineage verification

### `PrimalDiscovery` — Songbird Integration

Every primal needs to be found:
- UPA service registration
- BirdSong broadcasting
- Federation participation

### `PrimalLifecycle` — Start/Stop/Reload

Every primal has a lifecycle:
- Initialization
- Running
- Graceful shutdown
- Configuration reload

### `PrimalHealth` — Observability

Every primal needs health checks:
- Liveness (am I running?)
- Readiness (can I serve?)
- Dependencies (are my deps healthy?)

### `PrimalConfig` — Configuration

Every primal needs configuration:
- File-based (TOML)
- Environment variables
- Runtime overrides

---

## What SourDough Does NOT Provide

SourDough is intentionally minimal. It does NOT provide:

❌ **Data structures** — Your primal defines its own  
❌ **Storage backends** — Choose your own (RocksDB, SQLite, etc.)  
❌ **Network protocols** — Choose your own (gRPC, REST, custom)  
❌ **Business logic** — That's your primal's purpose  
❌ **Domain-specific traits** — Define them in your primal  

---

## Primals Born from SourDough

| Primal | Status | Purpose |
|--------|--------|---------|
| 🔐 rhizoCrypt | Planned | Ephemeral Data Graph |
| 🦴 loamSpine | Planned | Permanent Ledger |
| 🌾 sweetGrass | Planned | Semantic Attribution |

---

## Contributing

SourDough should remain minimal. Before adding anything, ask:

1. **Is this universal?** Does every primal need this?
2. **Is this agnostic?** Does it make no assumptions about the primal's purpose?
3. **Is this composable?** Can primals use only what they need?

If the answer to any of these is "no", it belongs in the specific primal, not in SourDough.

---

*SourDough: The starter that gives rise to everything.*

