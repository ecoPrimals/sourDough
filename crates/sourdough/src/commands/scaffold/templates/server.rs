//! Server crate templates: Cargo.toml, main.rs, server.rs, dispatch.rs, `method_gate.rs`.
//!
//! Generated `{name}-server` crate provides a JSON-RPC 2.0 server with
//! capability wire standard handlers, first-byte peek, socket naming, and
//! pre-dispatch `MethodGate` (JH-0/JH-2 ecosystem standard).

/// Generate the server crate `Cargo.toml`.
pub(in crate::commands::scaffold) fn server_cargo_toml(
    server_crate_name: &str,
    core_crate_name: &str,
    name: &str,
) -> String {
    format!(
        r#"[package]
name = "{server_crate_name}"
description = "JSON-RPC server for {name}"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[lints]
workspace = true

[[bin]]
name = "{name_lower}"
path = "src/main.rs"

[dependencies]
{core_crate_name} = {{ path = "../{core_crate_name}" }}
tokio = {{ workspace = true, features = ["fs"] }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
anyhow = {{ workspace = true }}
tracing = {{ workspace = true }}
tracing-subscriber = {{ workspace = true }}
clap = {{ workspace = true }}
"#,
        name_lower = name.to_lowercase(),
    )
}

/// Generate the server `main.rs` with CLI entry point and transport injection.
pub(in crate::commands::scaffold) fn server_main_rs(name: &str) -> String {
    let type_name = super::super::primal_rust_type_name(name);
    let name_lower = name.to_lowercase();
    let core_ident = format!("{}_core", name_lower.replace('-', "_"));
    format!(
        r#"//! {name} server binary.
//!
//! JSON-RPC 2.0 server with transport injection — the primal does not choose
//! its transport. The launcher or Songbird provides it.

mod announce;
mod dispatch;
mod method_gate;
mod server;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "{name_lower}", about = "{name} primal server")]
struct Cli {{
    /// Family ID for socket naming (production mode).
    #[arg(long, env = "FAMILY_ID")]
    family_id: Option<String>,

    /// Transport endpoint override (JSON, e.g. '{{"transport":"tcp","host":"0.0.0.0","port":7800}}').
    /// When set, the primal binds to this endpoint instead of deriving from socket conventions.
    #[arg(long, env = "TRANSPORT_ENDPOINT")]
    transport_endpoint: Option<String>,
}}

#[tokio::main]
async fn main() -> Result<()> {{
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let mut primal = {core_ident}::{type_name}Primal::new();
    {core_ident}::PrimalLifecycle::start(&mut primal)
        .await
        .map_err(|e| anyhow::anyhow!("startup failed: {{e}}"))?;

    tracing::info!("{name} started");

    let endpoint: Option<server::TransportEndpoint> = cli
        .transport_endpoint
        .as_deref()
        .map(|s| serde_json::from_str(s))
        .transpose()
        .map_err(|e| anyhow::anyhow!("invalid TRANSPORT_ENDPOINT: {{e}}"))?;

    server::run("{name_lower}", cli.family_id.as_deref(), endpoint.as_ref(), &primal).await
}}
"#,
    )
}

/// Generate the server `server.rs` with transport injection.
#[expect(clippy::too_many_lines, reason = "static template string")]
pub(in crate::commands::scaffold) fn server_rs(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! Transport-injected server with first-byte protocol detection.
//!
//! The primal does not choose its transport — the launcher or Songbird
//! provides a `TransportEndpoint`. Defaults to UDS from socket conventions.

use anyhow::Result;
use serde::{{Deserialize, Serialize}};
use tokio::io::{{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tracing::{{info, warn}};

/// Structured transport endpoint — wire-compatible with songbird_types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport")]
pub enum TransportEndpoint {{
    /// Unix Domain Socket.
    #[serde(rename = "uds")]
    Uds {{ path: String }},
    /// TCP socket.
    #[serde(rename = "tcp")]
    Tcp {{ host: String, port: u16 }},
    /// Mesh relay (not directly bindable, routed via Songbird).
    #[serde(rename = "mesh_relay")]
    MeshRelay {{ peer_id: String, capability: String }},
}}

/// Resolve the default listen endpoint from ecosystem socket conventions.
fn default_endpoint(primal_name: &str, family_id: Option<&str>) -> TransportEndpoint {{
    let socket_dir = std::env::var({core_ident}::env_keys::BIOMEOS_SOCKET_DIR).unwrap_or_else(|_| {{
        let runtime_dir =
            std::env::var({core_ident}::env_keys::XDG_RUNTIME_DIR).unwrap_or_else(|_| "/tmp".to_owned());
        format!("{{runtime_dir}}/biomeos")
    }});

    let filename = match family_id.filter(|id| !id.is_empty() && *id != "default") {{
        Some(fid) => format!("{{primal_name}}-{{fid}}.sock"),
        None => format!("{{primal_name}}.sock"),
    }};

    TransportEndpoint::Uds {{
        path: std::path::PathBuf::from(&socket_dir)
            .join(&filename)
            .to_string_lossy()
            .into_owned(),
    }}
}}

/// Run the JSON-RPC server on the given (or default) transport endpoint.
pub async fn run(
    primal_name: &str,
    family_id: Option<&str>,
    injected_endpoint: Option<&TransportEndpoint>,
    primal: &{core_ident}::{type_name}Primal,
) -> Result<()> {{
    let endpoint = injected_endpoint
        .cloned()
        .unwrap_or_else(|| default_endpoint(primal_name, family_id));

    let gate = crate::method_gate::MethodGate::permissive();

    match &endpoint {{
        TransportEndpoint::Uds {{ path }} => {{
            let socket_path = std::path::PathBuf::from(path);
            if let Some(parent) = socket_path.parent() {{
                tokio::fs::create_dir_all(parent).await?;
            }}
            let _ = tokio::fs::remove_file(&socket_path).await;

            let listener = tokio::net::UnixListener::bind(&socket_path)?;
            info!("Listening on unix://{{path}}");

            let announce_socket = socket_path.clone();
            let announce_family = family_id.unwrap_or("ecoPrimal").to_owned();
            let announce_name = primal_name.to_owned();
            tokio::spawn(async move {{
                crate::announce::announce_to_biomeos(
                    &announce_name,
                    &announce_socket,
                    &announce_family,
                )
                .await;
            }});

            loop {{
                let (stream, _) = listener.accept().await?;
                handle_connection(stream, primal, &gate).await;
            }}
        }}
        TransportEndpoint::Tcp {{ host, port }} => {{
            let addr = format!("{{host}}:{{port}}");
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            info!("Listening on tcp://{{addr}}");

            loop {{
                let (stream, _) = listener.accept().await?;
                handle_connection(stream, primal, &gate).await;
            }}
        }}
        TransportEndpoint::MeshRelay {{ .. }} => {{
            anyhow::bail!(
                "MeshRelay endpoints are not directly bindable — \
                 register capabilities with Songbird and let it route traffic"
            );
        }}
    }}
}}

async fn handle_connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: S,
    primal: &{core_ident}::{type_name}Primal,
    gate: &crate::method_gate::MethodGate,
) {{
    let mut reader = BufReader::new(stream);
    let first_byte = match reader.fill_buf().await {{
        Ok(buf) if !buf.is_empty() => buf[0],
        Ok(_) => return,
        Err(e) => {{
            warn!("Connection error: {{e}}");
            return;
        }}
    }};

    if first_byte == b'{{' {{
        handle_jsonrpc(reader, primal, gate).await;
    }} else {{
        warn!("BTSP connection detected — not yet implemented");
    }}
}}

async fn handle_jsonrpc<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    mut reader: BufReader<S>,
    primal: &{core_ident}::{type_name}Primal,
    gate: &crate::method_gate::MethodGate,
) {{
    let mut line = String::new();
    loop {{
        line.clear();
        match reader.read_line(&mut line).await {{
            Ok(0) => return,
            Err(e) => {{
                warn!("Read error: {{e}}");
                return;
            }}
            Ok(_) => {{}}
        }}

        let response = crate::dispatch::handle_request(line.trim(), primal, gate);
        let writer = reader.get_mut();
        if let Err(e) = writer.write_all(response.as_bytes()).await {{
            warn!("Write error: {{e}}");
            return;
        }}
        if let Err(e) = writer.write_all(b"\n").await {{
            warn!("Write error: {{e}}");
            return;
        }}
    }}
}}
"#,
    )
}

/// Generate the server `dispatch.rs` with capability wire handlers.
pub(in crate::commands::scaffold) fn dispatch_rs(name: &str) -> String {
    format!("{}{}", dispatch_core(name), dispatch_tests(name),)
}

fn dispatch_core(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! JSON-RPC 2.0 method dispatch with capability wire standard handlers.

use {core_ident}::PrimalHealth;
use crate::method_gate::MethodGate;

const PRIMAL_NAME: &str = "{name}";
const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const METHODS: &[&str] = &[
    "health.liveness",
    "health.readiness",
    "health.check",
    "capabilities.list",
    "btsp.negotiate",
    "primal.announce",
];

/// Dispatch a JSON-RPC request and return the response string.
pub fn handle_request(
    raw: &str,
    primal: &{core_ident}::{type_name}Primal,
    gate: &MethodGate,
) -> String {{
    let req: serde_json::Value = match serde_json::from_str(raw) {{
        Ok(v) => v,
        Err(_) => return error_response(serde_json::Value::Null, -32700, "Parse error"),
    }};

    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = req
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    // JH-0: pre-dispatch capability gate
    if let Err(denial) = gate.check(method) {{
        return error_response(id, denial.code, &denial.message);
    }}

    let result = match method {{
        "health.liveness" => serde_json::json!({{ "alive": true }}),
        "health.readiness" => {{
            let ready = primal.is_ready();
            serde_json::json!({{
                "ready": ready,
                "capabilities": METHODS,
            }})
        }}
        "health.check" => {{
            let status = primal.health_status();
            serde_json::json!({{
                "status": format!("{{status:?}}"),
                "liveness": primal.is_live(),
                "readiness": primal.is_ready(),
            }})
        }}
        "capabilities.list" | "capability.list" => {{
            serde_json::json!({{
                "primal": PRIMAL_NAME,
                "version": PRIMAL_VERSION,
                "methods": METHODS,
                "protocol": "jsonrpc-2.0",
                "transport": ["uds", "tcp"],
            }})
        }}
        "btsp.negotiate" => {{
            // BTSP Phase 3: graceful NULL cipher fallback.
            // Returning "null" cipher means plaintext continues — zero breakage.
            // Evolve to ChaCha20-Poly1305 when ready (see petalTongue reference).
            serde_json::json!({{
                "cipher": "null",
                "server_nonce": null,
            }})
        }}
        "primal.announce" => {{
            // Respond to announce queries with self-description.
            serde_json::json!({{
                "primal": PRIMAL_NAME,
                "version": PRIMAL_VERSION,
                "capabilities": crate::announce::capabilities(),
                "methods": METHODS,
                "signal_tiers": crate::announce::signal_tiers(),
            }})
        }}
        _ => return error_response(id, -32601, "Method not found"),
    }};

    serde_json::json!({{
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }})
    .to_string()
}}

fn error_response(id: serde_json::Value, code: i32, message: &str) -> String {{
    serde_json::json!({{
        "jsonrpc": "2.0",
        "id": id,
        "error": {{ "code": code, "message": message }},
    }})
    .to_string()
}}
"#,
    )
}

fn dispatch_tests(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r##"
#[cfg(test)]
mod tests {{
    use super::*;

    use crate::method_gate::MethodGate;

    fn make_primal() -> {core_ident}::{type_name}Primal {{
        {core_ident}::{type_name}Primal::new()
    }}

    fn make_gate() -> MethodGate {{
        MethodGate::permissive()
    }}

    #[test]
    fn liveness_returns_alive() {{
        let primal = make_primal();
        let gate = make_gate();
        let req = r#"{{"jsonrpc":"2.0","id":1,"method":"health.liveness"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal, &gate)).unwrap();
        assert_eq!(resp["result"]["alive"], true);
    }}

    #[test]
    fn capabilities_list_includes_primal_and_methods() {{
        let primal = make_primal();
        let gate = make_gate();
        let req = r#"{{"jsonrpc":"2.0","id":2,"method":"capabilities.list"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal, &gate)).unwrap();
        assert_eq!(resp["result"]["primal"], PRIMAL_NAME);
        assert!(resp["result"]["methods"].is_array());
    }}

    #[test]
    fn unknown_method_returns_error() {{
        let primal = make_primal();
        let gate = make_gate();
        let req = r#"{{"jsonrpc":"2.0","id":3,"method":"unknown.method"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal, &gate)).unwrap();
        assert_eq!(resp["error"]["code"], -32601);
    }}

    #[test]
    fn invalid_json_returns_parse_error() {{
        let primal = make_primal();
        let gate = make_gate();
        let resp: serde_json::Value =
            serde_json::from_str(&handle_request("not json", &primal, &gate)).unwrap();
        assert_eq!(resp["error"]["code"], -32700);
    }}

    #[test]
    fn btsp_negotiate_returns_null_cipher() {{
        let primal = make_primal();
        let gate = make_gate();
        let req = r#"{{"jsonrpc":"2.0","id":4,"method":"btsp.negotiate","params":{{"session_id":"test","preferred_cipher":"chacha20-poly1305","bond_type":"Covalent"}}}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal, &gate)).unwrap();
        assert_eq!(resp["result"]["cipher"], "null");
    }}
}}
"##,
    )
}

/// Generate the server `method_gate.rs` with JH-0/JH-2 pre-dispatch gate.
pub(in crate::commands::scaffold) fn method_gate_rs() -> String {
    format!("{}{}", method_gate_core(), method_gate_tests())
}

#[expect(clippy::too_many_lines, reason = "static template string")]
const fn method_gate_core() -> &'static str {
    r#"//! Pre-dispatch capability gate (JH-0 / JH-2 ecosystem standard).
//!
//! Classifies every JSON-RPC method as Public or Protected and gates
//! dispatch based on the current mode. Ships in Permissive mode (all calls
//! allowed) per the ecoPrimals METHOD_GATE_STANDARD.

use serde::{Deserialize, Serialize};

/// Whether a method is freely callable or requires authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodVisibility {
    /// Callable by any peer without credentials.
    Public,
    /// Requires a valid token / caller identity when the gate is enforcing.
    Protected,
}

/// Gate operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateMode {
    /// All calls allowed regardless of caller identity (JH-0 default).
    Permissive,
    /// Protected methods require valid authentication (JH-2 future).
    Enforcing,
}

/// Resource limits carried in an ionic token (JH-2 prep).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEnvelope {
    /// Maximum memory in MB the token grants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_mb: Option<u64>,
    /// Maximum CPU cores the token grants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_cores: Option<u32>,
    /// Maximum timeout in milliseconds per dispatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_timeout_ms: Option<u64>,
    /// Methods this token may call. Empty = all allowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub method_allowlist: Vec<String>,
}

impl ResourceEnvelope {
    /// Check whether the envelope allows calling `method`.
    pub fn allows_method(&self, method: &str) -> bool {
        self.method_allowlist.is_empty()
            || self.method_allowlist.iter().any(|m| m == method)
    }
}

/// Caller identity and resource context (JH-2 prep).
#[derive(Debug, Clone, Default)]
pub struct CallerContext {
    /// Caller identity (e.g. DID from ionic token).
    pub identity: Option<String>,
    /// Resource envelope from token.
    pub envelope: Option<ResourceEnvelope>,
}

impl CallerContext {
    /// Anonymous caller with no token (permissive-mode default).
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Whether this caller presented a token with an envelope.
    pub fn has_envelope(&self) -> bool {
        self.envelope.is_some()
    }
}

/// Gate denial — contains JSON-RPC error code and message.
#[derive(Debug, Clone)]
pub struct GateDenial {
    /// JSON-RPC error code.
    pub code: i32,
    /// Human-readable message.
    pub message: String,
}

/// Pre-dispatch capability gate.
pub struct MethodGate {
    mode: GateMode,
}

impl MethodGate {
    /// Create a new gate in the given mode.
    pub fn new(mode: GateMode) -> Self {
        Self { mode }
    }

    /// Create a gate in permissive mode (JH-0 default).
    pub fn permissive() -> Self {
        Self::new(GateMode::Permissive)
    }

    /// Current operating mode.
    pub fn mode(&self) -> GateMode {
        self.mode
    }

    /// Check whether a method call should be allowed.
    pub fn check(&self, method: &str) -> Result<(), GateDenial> {
        self.check_with_context(method, &CallerContext::anonymous())
    }

    /// Check method access with full caller context (JH-2).
    pub fn check_with_context(
        &self,
        method: &str,
        ctx: &CallerContext,
    ) -> Result<(), GateDenial> {
        let visibility = classify_method(method);

        match self.mode {
            GateMode::Permissive => {
                if let Some(ref env) = ctx.envelope {
                    if !env.allows_method(method) {
                        return Err(GateDenial {
                            code: -32001,
                            message: format!("Token does not permit method: {method}"),
                        });
                    }
                }
                Ok(())
            }
            GateMode::Enforcing => match visibility {
                MethodVisibility::Public => Ok(()),
                MethodVisibility::Protected => {
                    if ctx.identity.is_none() {
                        return Err(GateDenial {
                            code: -32002,
                            message: "Authentication required for protected method".into(),
                        });
                    }
                    if let Some(ref env) = ctx.envelope {
                        if !env.allows_method(method) {
                            return Err(GateDenial {
                                code: -32001,
                                message: format!("Token does not permit method: {method}"),
                            });
                        }
                    }
                    Ok(())
                }
            },
        }
    }
}

/// Classify a method name into its visibility tier.
///
/// Public: health probes, identity, capabilities, auth, lifecycle status,
/// BTSP negotiation. Everything else: protected.
pub fn classify_method(method: &str) -> MethodVisibility {
    match method {
        "health.liveness" | "health.readiness" | "health.check" => MethodVisibility::Public,
        "identity.get" | "capabilities.list" | "capability.list" | "lifecycle.status" => {
            MethodVisibility::Public
        }
        "btsp.negotiate" | "primal.announce" => MethodVisibility::Public,
        m if m.starts_with("auth.") => MethodVisibility::Public,
        _ => MethodVisibility::Protected,
    }
}
"#
}

#[expect(clippy::too_many_lines, reason = "static template string")]
const fn method_gate_tests() -> &'static str {
    r#"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissive_allows_all() {
        let gate = MethodGate::permissive();
        assert!(gate.check("health.liveness").is_ok());
        assert!(gate.check("some.protected.method").is_ok());
        assert!(gate.check("custom.domain.verb").is_ok());
    }

    #[test]
    fn enforcing_allows_public() {
        let gate = MethodGate::new(GateMode::Enforcing);
        assert!(gate.check("health.liveness").is_ok());
        assert!(gate.check("health.readiness").is_ok());
        assert!(gate.check("health.check").is_ok());
        assert!(gate.check("identity.get").is_ok());
        assert!(gate.check("capabilities.list").is_ok());
        assert!(gate.check("capability.list").is_ok());
        assert!(gate.check("lifecycle.status").is_ok());
        assert!(gate.check("btsp.negotiate").is_ok());
        assert!(gate.check("auth.check").is_ok());
        assert!(gate.check("auth.mode").is_ok());
    }

    #[test]
    fn enforcing_denies_anonymous_on_protected() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let err = gate.check("custom.method").unwrap_err();
        assert_eq!(err.code, -32002);
    }

    #[test]
    fn enforcing_allows_authenticated() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let ctx = CallerContext {
            identity: Some("did:key:z6Mk_test".into()),
            envelope: Some(ResourceEnvelope::default()),
        };
        assert!(gate.check_with_context("custom.method", &ctx).is_ok());
    }

    #[test]
    fn permissive_enforces_token_allowlist() {
        let gate = MethodGate::permissive();
        let ctx = CallerContext {
            identity: Some("did:key:z6Mk_test".into()),
            envelope: Some(ResourceEnvelope {
                method_allowlist: vec!["health.liveness".into()],
                ..ResourceEnvelope::default()
            }),
        };
        assert!(gate.check_with_context("health.liveness", &ctx).is_ok());
        let err = gate.check_with_context("custom.method", &ctx).unwrap_err();
        assert_eq!(err.code, -32001);
    }

    #[test]
    fn enforcing_denies_method_not_in_allowlist() {
        let gate = MethodGate::new(GateMode::Enforcing);
        let ctx = CallerContext {
            identity: Some("did:key:z6Mk_test".into()),
            envelope: Some(ResourceEnvelope {
                method_allowlist: vec!["health.liveness".into()],
                ..ResourceEnvelope::default()
            }),
        };
        let err = gate.check_with_context("custom.method", &ctx).unwrap_err();
        assert_eq!(err.code, -32001);
    }

    #[test]
    fn classify_public_methods() {
        let public = [
            "health.liveness",
            "health.readiness",
            "health.check",
            "identity.get",
            "capabilities.list",
            "capability.list",
            "lifecycle.status",
            "btsp.negotiate",
            "primal.announce",
            "auth.check",
            "auth.mode",
            "auth.peer_info",
        ];
        for m in &public {
            assert_eq!(classify_method(m), MethodVisibility::Public, "{m}");
        }
    }

    #[test]
    fn classify_protected_methods() {
        let protected = ["custom.method", "data.store", "compute.run", "unknown"];
        for m in &protected {
            assert_eq!(classify_method(m), MethodVisibility::Protected, "{m}");
        }
    }

    #[test]
    fn resource_envelope_empty_allows_all() {
        let env = ResourceEnvelope::default();
        assert!(env.allows_method("anything"));
    }

    #[test]
    fn resource_envelope_restricts_to_allowlist() {
        let env = ResourceEnvelope {
            method_allowlist: vec!["health.liveness".into()],
            ..ResourceEnvelope::default()
        };
        assert!(env.allows_method("health.liveness"));
        assert!(!env.allows_method("custom.method"));
    }

    #[test]
    fn caller_context_anonymous() {
        let ctx = CallerContext::anonymous();
        assert!(ctx.identity.is_none());
        assert!(!ctx.has_envelope());
    }

    #[test]
    fn gate_mode_serde() {
        assert_eq!(
            serde_json::to_string(&GateMode::Permissive).unwrap(),
            "\"permissive\""
        );
        assert_eq!(
            serde_json::to_string(&GateMode::Enforcing).unwrap(),
            "\"enforcing\""
        );
    }
}
"#
}
