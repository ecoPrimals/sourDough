//! tarpc service template for cephalization-era primals.
//!
//! Generates the high-performance binary RPC service definition that
//! complements JSON-RPC for intra-gate composition (G64 convergence).

/// Generate the tarpc service module (`tarpc_service.rs`) for the core crate.
pub(in crate::commands::scaffold) fn tarpc_service_rs(name: &str) -> String {
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! tarpc service definition for high-performance binary IPC.
//!
//! Dual-protocol architecture (G64 Cephalization):
//! - JSON-RPC on `{{name}}.sock` — bootstrap, discovery, diagnostics, browser
//! - tarpc on `{{name}}.tarpc.sock` — intra-gate composition, sub-ms binary framing
//!
//! The tarpc service mirrors the JSON-RPC capability surface but eliminates
//! serde roundtrips for high-frequency callers within the same NUCLEUS.

/// tarpc service trait for {name}.
///
/// This is the binary-framed equivalent of the JSON-RPC dispatch.
/// High-frequency intra-gate callers use this for zero-serde composition.
#[tarpc::service]
pub trait {type_name}Service {{
    /// Health liveness check (mirrors `health.liveness`).
    async fn health_liveness() -> bool;

    /// Health readiness check (mirrors `health.readiness`).
    async fn health_readiness() -> bool;

    /// Full health report (mirrors `health.check`).
    async fn health_check() -> HealthCheckResponse;

    /// List capabilities (mirrors `capabilities.list`).
    async fn capabilities_list() -> CapabilitiesResponse;
}}

/// Response type for `health_check` over tarpc.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {{
    pub status: String,
    pub primal: String,
    pub version: String,
}}

/// Response type for `capabilities_list` over tarpc.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilitiesResponse {{
    pub primal: String,
    pub version: String,
    pub methods: Vec<String>,
}}

/// Connect to a remote {name} primal via its tarpc socket.
///
/// Returns a tarpc client stub ready for sub-ms binary RPC calls.
/// Falls back to `$XDG_RUNTIME_DIR/biomeos/{{name}}.tarpc.sock` if no path given.
///
/// # Errors
///
/// Returns an error if the socket doesn't exist or connection fails.
pub async fn connect(socket_path: Option<&str>) -> Result<{type_name}ServiceClient, std::io::Error> {{
    let default_path = {{
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
        format!("{{runtime_dir}}/biomeos/{name_lower}.tarpc.sock")
    }};
    let path = socket_path.unwrap_or(&default_path);

    let transport = tarpc::serde_transport::unix::connect(
        path,
        tokio_serde::formats::Bincode::default,
    )
    .await?;

    Ok({type_name}ServiceClient::new(tarpc::client::Config::default(), transport).spawn())
}}
"#,
        name_lower = name.to_lowercase(),
    )
}

/// Generate the tarpc server binding code (`tarpc_server.rs`) for the server crate.
#[expect(clippy::too_many_lines, reason = "static template string")]
pub(in crate::commands::scaffold) fn tarpc_server_section(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! tarpc binary RPC listener for intra-gate composition.
//!
//! Dual-socket pattern (G64 Cephalization):
//! - JSON-RPC on `{{name}}.sock` — the lifecycle anchor
//! - tarpc on `{{name}}.tarpc.sock` — spawned as background task

use anyhow::Result;
use futures::StreamExt;
use tarpc::server::{{self, Channel}};
use {core_ident}::tarpc_service::{type_name}Service;
use tracing::{{info, warn}};

/// tarpc socket path — same convention as JSON-RPC but with `.tarpc.sock` suffix.
fn tarpc_endpoint(primal_name: &str, family_id: Option<&str>) -> String {{
    let socket_dir = std::env::var({core_ident}::env_keys::BIOMEOS_SOCKET_DIR).unwrap_or_else(|_| {{
        let runtime_dir =
            std::env::var({core_ident}::env_keys::XDG_RUNTIME_DIR).unwrap_or_else(|_| "/tmp".to_owned());
        format!("{{runtime_dir}}/biomeos")
    }});

    let filename = match family_id.filter(|id| !id.is_empty() && *id != "default") {{
        Some(fid) => format!("{{primal_name}}-{{fid}}.tarpc.sock"),
        None => format!("{{primal_name}}.tarpc.sock"),
    }};

    std::path::PathBuf::from(&socket_dir)
        .join(&filename)
        .to_string_lossy()
        .into_owned()
}}

/// Start the tarpc binary RPC listener alongside the JSON-RPC server.
///
/// This runs as a background task — the JSON-RPC listener remains the
/// primary accept loop and the primal's lifecycle anchor.
pub async fn start_tarpc_listener(
    primal_name: &str,
    family_id: Option<&str>,
    primal: std::sync::Arc<{core_ident}::{type_name}Primal>,
) -> Result<()> {{
    let path = tarpc_endpoint(primal_name, family_id);
    let socket_path = std::path::PathBuf::from(&path);

    if let Some(parent) = socket_path.parent() {{
        tokio::fs::create_dir_all(parent).await?;
    }}
    let _ = tokio::fs::remove_file(&socket_path).await;

    let mut listener =
        tarpc::serde_transport::unix::listen(&socket_path, tokio_serde::formats::Bincode::default)
            .await?;
    info!("tarpc listening on unix://{{path}}");

    tokio::spawn(async move {{
        while let Some(result) = listener.next().await {{
            match result {{
                Ok(transport) => {{
                    let primal = primal.clone();
                    tokio::spawn(async move {{
                        let handler = {type_name}ServiceHandler {{ primal }};
                        server::BaseChannel::with_defaults(transport)
                            .execute(handler.serve())
                            .for_each(|response| async move {{ tokio::spawn(response); }})
                            .await;
                    }});
                }}
                Err(e) => {{
                    warn!("tarpc accept error: {{e}}");
                }}
            }}
        }}
    }});

    Ok(())
}}

/// tarpc service handler — bridges the tarpc trait to the primal implementation.
#[derive(Clone)]
struct {type_name}ServiceHandler {{
    primal: std::sync::Arc<{core_ident}::{type_name}Primal>,
}}

impl {core_ident}::tarpc_service::{type_name}Service for {type_name}ServiceHandler {{
    async fn health_liveness(self, _: tarpc::context::Context) -> bool {{
        use {core_ident}::PrimalHealth;
        self.primal.health_status().is_healthy()
    }}

    async fn health_readiness(self, _: tarpc::context::Context) -> bool {{
        use {core_ident}::PrimalLifecycle;
        self.primal.state().is_running()
    }}

    async fn health_check(
        self,
        _: tarpc::context::Context,
    ) -> {core_ident}::tarpc_service::HealthCheckResponse {{
        use {core_ident}::PrimalHealth;
        {core_ident}::tarpc_service::HealthCheckResponse {{
            status: format!("{{:?}}", self.primal.health_status()),
            primal: "{name}".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }}
    }}

    async fn capabilities_list(
        self,
        _: tarpc::context::Context,
    ) -> {core_ident}::tarpc_service::CapabilitiesResponse {{
        {core_ident}::tarpc_service::CapabilitiesResponse {{
            primal: "{name}".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            methods: vec![
                "health.liveness".to_owned(),
                "health.readiness".to_owned(),
                "health.check".to_owned(),
                "capabilities.list".to_owned(),
            ],
        }}
    }}
}}
"#,
    )
}
