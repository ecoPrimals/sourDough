//! Inlined primal DNA templates — the offspring is self-contained after budding.
//!
//! These templates are the genetic material that sourDough passes to new primals.
//! Each scaffolded primal receives its own copy of core traits, types, and patterns
//! with zero runtime dependency on sourDough.
//!
//! Split by output domain:
//! - [`core`] — `{name}-core` crate templates (Cargo.toml, lib.rs, error, lifecycle, health)
//! - [`server`] — `{name}-server` crate templates (Cargo.toml, main, server)
//! - [`dispatch`] — JSON-RPC method dispatch template
//! - [`method_gate`] — JH-0/JH-2 pre-dispatch capability gate template
//! - [`tarpc_service`] — tarpc binary RPC service definition (G64 cephalization)
//! - [`announce`] — Ecosystem announcement template
//! - [`infra`] — CI workflows and deny.toml

mod announce;
mod core;
mod dispatch;
mod infra;
mod method_gate;
mod server;
mod tarpc_service;

pub(super) use self::announce::announce_rs;
pub(super) use self::core::{
    ENV_KEYS_RS, ERROR_RS, HEALTH_RS, LIFECYCLE_RS, PROTOCOL_NEGOTIATION_RS, TRANSPORT_RS,
    core_cargo_toml, lib_rs,
};
pub(super) use self::dispatch::dispatch_rs;
pub(super) use self::infra::{
    DENY_TOML, NOTIFY_PLASMIDBIN_YML, ci_yml, release_yml, systemd_service,
};
pub(super) use self::method_gate::method_gate_rs;
pub(super) use self::server::{server_cargo_toml, server_main_rs, server_rs};
pub(super) use self::tarpc_service::{tarpc_server_section, tarpc_service_rs};
