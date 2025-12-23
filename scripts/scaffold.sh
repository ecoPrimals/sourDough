#!/bin/bash
# scaffold.sh - Scaffold a new ecoPrimal
#
# Usage:
#   ./scaffold.sh new-primal <name> "<description>"
#   ./scaffold.sh new-crate <primal> <crate-name>
#
# Examples:
#   ./scaffold.sh new-primal rhizoCrypt "Ephemeral Data Graph"
#   ./scaffold.sh new-crate rhizoCrypt rhizocrypt-storage
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURDOUGH_DIR="$(dirname "$SCRIPT_DIR")"
ECOPRIMALS_DIR="$(dirname "$SOURDOUGH_DIR")"
TEMPLATES_DIR="$SOURDOUGH_DIR/templates"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

usage() {
    echo "Usage:"
    echo "  $0 new-primal <name> \"<description>\""
    echo "  $0 new-crate <primal> <crate-name>"
    echo ""
    echo "Examples:"
    echo "  $0 new-primal rhizoCrypt \"Ephemeral Data Graph\""
    echo "  $0 new-crate rhizoCrypt rhizocrypt-storage"
    exit 1
}

log() {
    echo -e "${CYAN}[scaffold]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Convert camelCase to snake_case
to_snake_case() {
    echo "$1" | sed 's/\([A-Z]\)/_\L\1/g' | sed 's/^_//'
}

# Convert camelCase to kebab-case
to_kebab_case() {
    echo "$1" | sed 's/\([A-Z]\)/-\L\1/g' | sed 's/^-//'
}

scaffold_primal() {
    local name="$1"
    local description="$2"
    local snake_name=$(to_snake_case "$name")
    local kebab_name=$(to_kebab_case "$name")
    local target_dir="$ECOPRIMALS_DIR/$name"
    
    log "Scaffolding new primal: $name"
    log "Description: $description"
    log "Target: $target_dir"
    
    if [ -d "$target_dir" ]; then
        error "Directory already exists: $target_dir"
    fi
    
    # Create directory structure
    log "Creating directory structure..."
    mkdir -p "$target_dir/crates/${kebab_name}-core/src"
    mkdir -p "$target_dir/specs"
    mkdir -p "$target_dir/showcase"
    
    # Create workspace Cargo.toml
    log "Creating Cargo.toml..."
    cat > "$target_dir/Cargo.toml" << EOF
[workspace]
resolver = "2"
members = [
    "crates/${kebab_name}-core",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "AGPL-3.0"
repository = "https://github.com/ecoPrimals/${name}"
authors = ["ecoPrimals Project"]

[workspace.dependencies]
# SourDough core traits
sourdough-core = { path = "../sourDough/crates/sourdough-core" }

# Async runtime
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
EOF
    success "Created Cargo.toml"
    
    # Create core crate Cargo.toml
    log "Creating ${kebab_name}-core crate..."
    cat > "$target_dir/crates/${kebab_name}-core/Cargo.toml" << EOF
[package]
name = "${kebab_name}-core"
description = "${description}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
sourdough-core = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
EOF
    success "Created ${kebab_name}-core/Cargo.toml"
    
    # Create core lib.rs
    cat > "$target_dir/crates/${kebab_name}-core/src/lib.rs" << EOF
//! # ${name}
//!
//! ${description}
//!
//! ## Overview
//!
//! ${name} is part of the ecoPrimals ecosystem.
//!
//! ## Quick Start
//!
//! \`\`\`rust,ignore
//! use ${snake_name}_core::${name};
//!
//! let primal = ${name}::new(config).await?;
//! primal.start().await?;
//! \`\`\`

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState,
    HealthStatus, HealthReport, PrimalError,
};

/// ${name} configuration.
pub use config::${name}Config;

/// ${name} errors.
pub use error::${name}Error;

/// The ${name} primal.
pub struct ${name} {
    config: ${name}Config,
    state: PrimalState,
}

impl ${name} {
    /// Create a new ${name} instance.
    pub fn new(config: ${name}Config) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }
}

impl PrimalLifecycle for ${name} {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("${name} starting...");
        
        // TODO: Initialize resources
        
        self.state = PrimalState::Running;
        tracing::info!("${name} running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("${name} stopping...");
        
        // TODO: Clean up resources
        
        self.state = PrimalState::Stopped;
        tracing::info!("${name} stopped");
        Ok(())
    }
}

impl PrimalHealth for ${name} {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("${name}", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
EOF
    success "Created ${kebab_name}-core/src/lib.rs"
    
    # Create config module
    cat > "$target_dir/crates/${kebab_name}-core/src/config.rs" << EOF
//! ${name} configuration.

use serde::{Deserialize, Serialize};
use sourdough_core::CommonConfig;

/// Configuration for ${name}.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ${name}Config {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,
    
    // TODO: Add ${name}-specific configuration
}

impl Default for ${name}Config {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "${name}".to_string(),
                ..CommonConfig::default()
            },
        }
    }
}
EOF
    success "Created ${kebab_name}-core/src/config.rs"
    
    # Create error module
    cat > "$target_dir/crates/${kebab_name}-core/src/error.rs" << EOF
//! ${name} error types.

use thiserror::Error;

/// Errors specific to ${name}.
#[derive(Debug, Error)]
pub enum ${name}Error {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
    
    // TODO: Add ${name}-specific errors
    
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
EOF
    success "Created ${kebab_name}-core/src/error.rs"
    
    # Create README
    cat > "$target_dir/README.md" << EOF
# ${name}

${description}

## Status

🌱 **Nascent** — Scaffolded from SourDough

## Quick Start

\`\`\`bash
# Build
cargo build

# Test
cargo test

# Run
cargo run
\`\`\`

## Architecture

\`\`\`
${name}/
├── Cargo.toml           # Workspace manifest
├── crates/
│   └── ${kebab_name}-core/  # Core library
├── specs/               # Specifications
└── showcase/            # Demonstrations
\`\`\`

## Integration

${name} integrates with the ecoPrimals ecosystem via SourDough traits:

- \`PrimalLifecycle\` — Start/stop/reload
- \`PrimalHealth\` — Health checks
- \`PrimalIdentity\` — BearDog integration (TODO)
- \`PrimalDiscovery\` — Songbird integration (TODO)

## License

AGPL-3.0

---

*Born from SourDough, growing into an ecoPrimal.*
EOF
    success "Created README.md"
    
    # Create .gitignore
    cat > "$target_dir/.gitignore" << EOF
/target/
Cargo.lock
*.swp
*.swo
.DS_Store
EOF
    success "Created .gitignore"
    
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✓ Primal '${name}' scaffolded successfully!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. cd $target_dir"
    echo "  2. cargo build"
    echo "  3. Edit specs/ to define your primal"
    echo "  4. Implement the core logic"
    echo ""
}

scaffold_crate() {
    local primal="$1"
    local crate_name="$2"
    local primal_dir="$ECOPRIMALS_DIR/$primal"
    local crate_dir="$primal_dir/crates/$crate_name"
    
    log "Scaffolding new crate: $crate_name in $primal"
    
    if [ ! -d "$primal_dir" ]; then
        error "Primal directory not found: $primal_dir"
    fi
    
    if [ -d "$crate_dir" ]; then
        error "Crate already exists: $crate_dir"
    fi
    
    mkdir -p "$crate_dir/src"
    
    # Create Cargo.toml
    cat > "$crate_dir/Cargo.toml" << EOF
[package]
name = "${crate_name}"
description = "TODO: Add description"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[dependencies]
sourdough-core = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
EOF
    success "Created $crate_name/Cargo.toml"
    
    # Create lib.rs
    cat > "$crate_dir/src/lib.rs" << EOF
//! # ${crate_name}
//!
//! TODO: Add description

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

// TODO: Implement crate
EOF
    success "Created $crate_name/src/lib.rs"
    
    # Update workspace Cargo.toml
    warn "Remember to add '$crate_name' to workspace members in $primal_dir/Cargo.toml"
    
    echo ""
    success "Crate '$crate_name' scaffolded in $primal"
}

# Main
if [ $# -lt 2 ]; then
    usage
fi

case "$1" in
    new-primal)
        if [ $# -lt 3 ]; then
            usage
        fi
        scaffold_primal "$2" "$3"
        ;;
    new-crate)
        if [ $# -lt 3 ]; then
            usage
        fi
        scaffold_crate "$2" "$3"
        ;;
    *)
        usage
        ;;
esac

