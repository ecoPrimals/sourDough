//! Platform-aware server binding strategy.
//!
//! On most platforms, UDS is preferred (lowest latency, filesystem-secured).
//! On Android/SELinux, UDS bind fails in non-standard paths — `TcpOnly` is required.
//!
//! Primals read `PRIMAL_BIND_MODE` at startup to determine their binding strategy.

use crate::env_keys;

/// Bind mode for server-side socket creation.
///
/// Read from `PRIMAL_BIND_MODE` env var at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindMode {
    /// Bind UDS only (default on Linux/macOS).
    Uds,
    /// Bind TCP only (required on Android/SELinux, containers with restricted mounts).
    TcpOnly,
    /// Bind both UDS and TCP (useful for development/migration).
    Both,
}

impl BindMode {
    /// Read bind mode from the `PRIMAL_BIND_MODE` environment variable.
    ///
    /// Returns `Uds` if unset or unrecognized (safe default).
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var(env_keys::PRIMAL_BIND_MODE)
            .unwrap_or_default()
            .as_str()
        {
            "tcp_only" => Self::TcpOnly,
            "both" => Self::Both,
            _ => Self::Uds,
        }
    }

    /// Whether UDS binding is allowed in this mode.
    #[must_use]
    pub const fn allows_uds(self) -> bool {
        matches!(self, Self::Uds | Self::Both)
    }

    /// Whether TCP binding is allowed in this mode.
    #[must_use]
    pub const fn allows_tcp(self) -> bool {
        matches!(self, Self::TcpOnly | Self::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_mode_default_is_uds() {
        let mode = BindMode::Uds;
        assert!(mode.allows_uds());
        assert!(!mode.allows_tcp());
    }

    #[test]
    fn bind_mode_tcp_only() {
        let mode = BindMode::TcpOnly;
        assert!(!mode.allows_uds());
        assert!(mode.allows_tcp());
    }

    #[test]
    fn bind_mode_both() {
        let mode = BindMode::Both;
        assert!(mode.allows_uds());
        assert!(mode.allows_tcp());
    }
}
