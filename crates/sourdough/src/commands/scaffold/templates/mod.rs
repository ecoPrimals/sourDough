//! Inlined primal DNA templates — the offspring is self-contained after budding.
//!
//! These templates are the genetic material that sourDough passes to new primals.
//! Each scaffolded primal receives its own copy of core traits, types, and patterns
//! with zero runtime dependency on sourDough.
//!
//! Split by output domain:
//! - [`core`] — `{name}-core` crate templates (Cargo.toml, lib.rs, error, lifecycle, health)
//! - [`server`] — `{name}-server` crate templates (Cargo.toml, main, server, dispatch)
//! - [`infra`] — CI workflows and deny.toml

mod core;
mod infra;
mod server;

pub(super) use self::core::{ERROR_RS, HEALTH_RS, LIFECYCLE_RS, core_cargo_toml, lib_rs};
pub(super) use self::infra::{DENY_TOML, NOTIFY_PLASMIDBIN_YML, ci_yml, release_yml};
pub(super) use self::server::{dispatch_rs, server_cargo_toml, server_main_rs, server_rs};
