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
///
/// Respects `PRIMAL_BIND_MODE` for platforms where UDS is unavailable (Android/SELinux).
fn default_endpoint(primal_name: &str, family_id: Option<&str>) -> TransportEndpoint {{
    let bind_mode = std::env::var({core_ident}::env_keys::PRIMAL_BIND_MODE).unwrap_or_default();
    if bind_mode == "tcp_only" {{
        return TransportEndpoint::Tcp {{
            host: "127.0.0.1".to_owned(),
            port: 0,
        }};
    }}

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
            let register_name = primal_name.to_owned();
            let register_endpoint = serde_json::to_value(&endpoint).unwrap_or_default();
            tokio::spawn(async move {{
                crate::announce::announce_to_biomeos(
                    &announce_name,
                    &announce_socket,
                    &announce_family,
                )
                .await;
                crate::announce::register_with_songbird(
                    &register_name,
                    &register_endpoint,
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

/// riboCipher signal bytes (Wave 111 — RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md).
const SIGNAL_CLEAR: u8 = 0xEC;
const SIGNAL_MITO: u8 = 0xED;
const SIGNAL_NUCLEAR: u8 = 0xEE;

async fn handle_connection<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    mut stream: S,
    primal: &{core_ident}::{type_name}Primal,
    gate: &crate::method_gate::MethodGate,
) {{
    use tokio::io::AsyncReadExt;

    let mut first = [0u8; 1];
    match stream.read(&mut first).await {{
        Ok(0) => return,
        Err(e) => {{
            warn!("Connection read error: {{e}}");
            return;
        }}
        Ok(_) => {{}}
    }}

    match first[0] {{
        SIGNAL_CLEAR => {{
            // riboCipher clear signal — read protocol type
            let mut proto = [0u8; 1];
            if stream.read_exact(&mut proto).await.is_err() {{
                return;
            }}
            match proto[0] {{
                0x01 => handle_jsonrpc(BufReader::new(stream), primal, gate).await,
                0x00 => {{
                    // Health probe — respond with empty JSON-RPC success
                    if let Err(e) = tokio::io::AsyncWriteExt::write_all(
                        &mut stream,
                        b"{{\"jsonrpc\":\"2.0\",\"result\":\"ok\",\"id\":0}}\n",
                    ).await {{
                        warn!("Probe write error: {{e}}");
                    }}
                }}
                other => warn!("Unsupported riboCipher protocol type: 0x{{other:02X}}"),
            }}
        }}
        SIGNAL_MITO => {{
            // Mito-obfuscated signal — consume 4-byte HMAC tag, route as JSON-RPC
            let mut tag = [0u8; 4];
            if stream.read_exact(&mut tag).await.is_err() {{
                return;
            }}
            handle_jsonrpc(BufReader::new(stream), primal, gate).await;
        }}
        SIGNAL_NUCLEAR => {{
            // Nuclear-sealed signal — consume 6-byte payload, route as JSON-RPC
            let mut payload = [0u8; 6];
            if stream.read_exact(&mut payload).await.is_err() {{
                return;
            }}
            handle_jsonrpc(BufReader::new(stream), primal, gate).await;
        }}
        b'{{' => {{
            // DEPRECATED: unsignalled legacy JSON-RPC (Wave 111-112 deprecation period)
            warn!("DEPRECATED: unsignalled connection — prepend [0xEC, 0x01] for riboCipher");
            handle_jsonrpc_legacy(stream, primal, gate, b'{{').await;
        }}
        other => {{
            warn!("DEPRECATED: unsignalled connection (first byte 0x{{other:02X}}) — not yet handled");
        }}
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

/// Legacy fallback: first byte was already consumed, prepend it to the line buffer.
async fn handle_jsonrpc_legacy<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
    stream: S,
    primal: &{core_ident}::{type_name}Primal,
    gate: &crate::method_gate::MethodGate,
    first_byte: u8,
) {{
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    // First iteration: prepend the consumed byte
    line.push(first_byte as char);
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

    // Subsequent lines: normal loop
    handle_jsonrpc(reader, primal, gate).await;
}}
"#,
    )
}
