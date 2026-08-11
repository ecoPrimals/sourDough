//! Platform abstraction DNA templates.
//!
//! These templates give scaffolded primals G68 platform substrate, platform
//! paths, and platform signal handling — self-contained, zero sourDough dep.

/// G68 Platform Substrate module template (L1 links + L2 permissions).
pub(in crate::commands::scaffold) const PLATFORM_SUBSTRATE_RS: &str =
    include_str!("dna/platform_substrate.rs.tmpl");

/// Platform Paths module template (cross-platform directory resolution).
pub(in crate::commands::scaffold) const PLATFORM_PATHS_RS: &str =
    include_str!("dna/platform_paths.rs.tmpl");

/// Platform Signal module template (cross-platform graceful shutdown).
pub(in crate::commands::scaffold) const PLATFORM_SIGNAL_RS: &str =
    include_str!("dna/platform_signal.rs.tmpl");
