//! Ecosystem socket path resolution conventions.
//!
//! Primals use standard socket path patterns for UDS communication:
//! - `$BIOMEOS_SOCKET_DIR/{name}-{family_id}.sock` (production)
//! - `$XDG_RUNTIME_DIR/biomeos/{name}.sock` (development)
//! - `/tmp/biomeos/{name}.sock` (fallback)

use crate::env_keys;
use std::path::PathBuf;

/// Resolve the socket path for a primal using ecosystem conventions.
///
/// Path: `$BIOMEOS_SOCKET_DIR/{name}-{family_id}.sock` (production with family),
/// or `$XDG_RUNTIME_DIR/biomeos/{name}.sock` (development, no family ID).
///
/// Falls back to `/tmp/biomeos/` if neither env var is set.
#[must_use]
pub fn resolve_socket_path(primal_name: &str, family_id: Option<&str>) -> PathBuf {
    let socket_dir = std::env::var(env_keys::BIOMEOS_SOCKET_DIR).unwrap_or_else(|_| {
        let runtime_dir = std::env::var(env_keys::XDG_RUNTIME_DIR)
            .unwrap_or_else(|_| env_keys::FALLBACK_RUNTIME_DIR.to_owned());
        format!("{runtime_dir}/{}", env_keys::SOCKET_DIR_NAME)
    });

    socket_path_in(&socket_dir, primal_name, family_id)
}

/// Build a socket path from explicit components (no env var reads).
#[must_use]
pub fn socket_path_in(socket_dir: &str, primal_name: &str, family_id: Option<&str>) -> PathBuf {
    let filename = family_id
        .filter(|id| !id.is_empty() && *id != "default")
        .map_or_else(
            || format!("{primal_name}.sock"),
            |fid| format!("{primal_name}-{fid}.sock"),
        );

    PathBuf::from(socket_dir).join(filename)
}
