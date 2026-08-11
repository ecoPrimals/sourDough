//! Core crate templates: `Cargo.toml`, `lib.rs`, `error.rs`, `lifecycle.rs`, `health.rs`, `env_keys.rs`.
//!
//! These templates define the primal DNA — lifecycle, health, and error traits
//! that every primal inherits at scaffold time.

/// Generate the core crate `Cargo.toml` for a scaffolded primal.
pub(in crate::commands::scaffold) fn core_cargo_toml(core_crate_name: &str, name: &str) -> String {
    format!(
        r#"[package]
name = "{core_crate_name}"
description = "Core library for {name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[lints]
workspace = true

[dependencies]
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
thiserror = {{ workspace = true }}
tracing = {{ workspace = true }}
tarpc = {{ workspace = true }}
tokio-serde = {{ workspace = true }}

[dev-dependencies]
tokio = {{ workspace = true, features = ["test-util"] }}
"#,
    )
}

/// Generate the core `lib.rs` with a starter primal implementation.
pub(in crate::commands::scaffold) fn lib_rs(name: &str) -> String {
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! # {name} Core
//!
//! Core library for the {name} primal.
//!
//! Self-contained: all primal DNA (traits, types, patterns) is defined here.
//! This primal discovers other primals at runtime via JSON-RPC 2.0 IPC.
//! High-performance intra-gate callers use the tarpc service (G64 cephalization).

pub mod env_keys;
pub mod error;
pub mod health;
pub mod lifecycle;
pub mod platform_paths;
pub mod platform_signal;
pub mod platform_substrate;
pub mod protocol_negotiation;
pub mod tarpc_service;
pub mod transport;

pub use error::{{PrimalError, PrimalResult}};
pub use health::{{HealthReport, HealthStatus, PrimalHealth}};
pub use lifecycle::{{PrimalLifecycle, PrimalState}};

/// The {name} primal.
pub struct {type_name}Primal {{
    state: PrimalState,
}}

impl {type_name}Primal {{
    /// Create a new primal instance.
    #[must_use]
    pub fn new() -> Self {{
        Self {{
            state: PrimalState::Created,
        }}
    }}
}}

impl Default for {type_name}Primal {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl PrimalLifecycle for {type_name}Primal {{
    fn state(&self) -> PrimalState {{
        self.state
    }}

    async fn start(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_start() {{
            return Err(PrimalError::lifecycle("cannot start from current state"));
        }}
        self.state = PrimalState::Running;
        Ok(())
    }}

    async fn stop(&mut self) -> Result<(), PrimalError> {{
        if !self.state.can_stop() {{
            return Err(PrimalError::lifecycle("cannot stop from current state"));
        }}
        self.state = PrimalState::Stopped;
        Ok(())
    }}
}}

impl PrimalHealth for {type_name}Primal {{
    fn health_status(&self) -> HealthStatus {{
        if self.state.is_running() {{
            HealthStatus::Healthy
        }} else {{
            HealthStatus::Unknown
        }}
    }}

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {{
        Ok(HealthReport::new("{name}", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_lifecycle() {{
        let mut primal = {type_name}Primal::new();
        assert_eq!(primal.state(), PrimalState::Created);

        primal.start().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Running);

        primal.stop().await.unwrap();
        assert_eq!(primal.state(), PrimalState::Stopped);
    }}

    #[tokio::test]
    async fn test_health() {{
        let mut primal = {type_name}Primal::new();
        primal.start().await.unwrap();

        assert!(primal.health_status().is_healthy());

        let report = primal.health_check().await.unwrap();
        assert_eq!(report.name, "{name}");
    }}
}}
"#,
    )
}

/// Environment variable names template.
pub(in crate::commands::scaffold) const ENV_KEYS_RS: &str =
    include_str!("dna/env_keys.rs.tmpl");

/// Common error types template.
pub(in crate::commands::scaffold) const ERROR_RS: &str = include_str!("dna/error.rs.tmpl");

/// Lifecycle state machine template.
pub(in crate::commands::scaffold) const LIFECYCLE_RS: &str =
    include_str!("dna/lifecycle.rs.tmpl");

/// Health check traits template.
pub(in crate::commands::scaffold) const HEALTH_RS: &str = include_str!("dna/health.rs.tmpl");
