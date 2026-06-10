//! Standard JSON-RPC 2.0 method names for the ecoPrimals ecosystem.
//!
//! All methods follow the `domain.verb` semantic naming convention.
//! These constants are the canonical reference — primals implementing
//! these methods should use these exact strings.

/// Health domain methods.
pub mod health {
    /// Full health check.
    pub const CHECK: &str = "health.check";
    /// Liveness probe (is the process alive?).
    pub const LIVENESS: &str = "health.liveness";
    /// Readiness probe (can it serve requests?).
    pub const READINESS: &str = "health.readiness";
}

/// Lifecycle domain methods.
pub mod lifecycle {
    /// Get current state.
    pub const STATE: &str = "lifecycle.state";
    /// Trigger reload.
    pub const RELOAD: &str = "lifecycle.reload";
}

/// Capability domain methods.
pub mod capabilities {
    /// List all capabilities.
    pub const LIST: &str = "capabilities.list";
}

/// Identity domain methods.
pub mod identity {
    /// Get primal DID.
    pub const DID: &str = "identity.did";
}

/// System domain methods.
pub mod system {
    /// Ping for liveness.
    pub const PING: &str = "system.ping";
    /// Get primal version.
    pub const VERSION: &str = "system.version";
}

/// IPC domain methods (songbird-mediated).
pub mod ipc {
    /// Resolve a primal's transport endpoint.
    pub const RESOLVE: &str = "ipc.resolve";
    /// Register capabilities at startup.
    pub const REGISTER: &str = "ipc.register";
}

/// Capability domain methods (mesh relay).
pub mod capability {
    /// Forward a JSON-RPC call through the mesh to a remote peer.
    ///
    /// Envelope: `{ "peer_id": "...", "capability": "...", "request": { ... } }`
    /// songBird routes the inner request to the peer's capability handler.
    pub const CALL: &str = "capability.call";
}

/// Primal domain methods (self-management).
pub mod primal {
    /// Announce to the ecosystem (Neural API registration).
    pub const ANNOUNCE: &str = "primal.announce";
    /// Graceful shutdown.
    pub const SHUTDOWN: &str = "primal.shutdown";
}
