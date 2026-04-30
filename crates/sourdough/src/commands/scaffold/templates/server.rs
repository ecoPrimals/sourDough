//! Server crate templates: Cargo.toml, main.rs, server.rs, dispatch.rs.
//!
//! Generated `{name}-server` crate provides a JSON-RPC 2.0 server with
//! capability wire standard handlers, first-byte peek, and socket naming.

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

/// Generate the server `main.rs` with CLI entry point.
pub(in crate::commands::scaffold) fn server_main_rs(name: &str) -> String {
    let type_name = super::super::primal_rust_type_name(name);
    let name_lower = name.to_lowercase();
    let core_ident = format!("{}_core", name_lower.replace('-', "_"));
    format!(
        r#"//! {name} server binary.
//!
//! JSON-RPC 2.0 server with capability wire standard handlers.

mod dispatch;
mod server;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "{name_lower}", about = "{name} primal server")]
struct Cli {{
    /// Family ID for socket naming (production mode).
    #[arg(long, env = "FAMILY_ID")]
    family_id: Option<String>,
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

    server::run("{name_lower}", cli.family_id.as_deref(), &primal).await
}}
"#,
    )
}

/// Generate the server `server.rs` with UDS listener + first-byte peek.
pub(in crate::commands::scaffold) fn server_rs(name: &str) -> String {
    let core_ident = format!("{}_core", name.to_lowercase().replace('-', "_"));
    let type_name = super::super::primal_rust_type_name(name);
    format!(
        r#"//! Unix domain socket server with first-byte protocol detection.

use anyhow::Result;
use tokio::io::{{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::net::UnixListener;
use tracing::{{info, warn}};

/// Run the JSON-RPC server on a Unix domain socket.
pub async fn run(
    primal_name: &str,
    family_id: Option<&str>,
    primal: &{core_ident}::{type_name}Primal,
) -> Result<()> {{
    let socket_dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {{
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
        format!("{{runtime_dir}}/biomeos")
    }});
    tokio::fs::create_dir_all(&socket_dir).await?;

    let filename = match family_id.filter(|id| !id.is_empty() && *id != "default") {{
        Some(fid) => format!("{{primal_name}}-{{fid}}.sock"),
        None => format!("{{primal_name}}.sock"),
    }};
    let socket_path = std::path::PathBuf::from(&socket_dir).join(&filename);

    // Clean up stale socket
    let _ = tokio::fs::remove_file(&socket_path).await;

    let listener = UnixListener::bind(&socket_path)?;
    info!("Listening on {{}}", socket_path.display());

    loop {{
        let (stream, _addr) = listener.accept().await?;

        let mut reader = BufReader::new(stream);
        let first_byte = match reader.fill_buf().await {{
            Ok(buf) if !buf.is_empty() => buf[0],
            Ok(_) => continue,
            Err(e) => {{
                warn!("Connection error: {{e}}");
                continue;
            }}
        }};

        if first_byte == b'{{' {{
            // JSON-RPC 2.0
            handle_jsonrpc(reader, primal).await;
        }} else {{
            // BTSP binary framing (not yet implemented)
            warn!("BTSP connection detected — not yet implemented");
        }}
    }}
}}

async fn handle_jsonrpc(
    mut reader: BufReader<tokio::net::UnixStream>,
    primal: &{core_ident}::{type_name}Primal,
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

        let response = crate::dispatch::handle_request(line.trim(), primal);
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

const PRIMAL_NAME: &str = "{name}";
const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

const METHODS: &[&str] = &[
    "health.liveness",
    "health.readiness",
    "health.check",
    "capabilities.list",
];

/// Dispatch a JSON-RPC request and return the response string.
pub fn handle_request(
    raw: &str,
    primal: &{core_ident}::{type_name}Primal,
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
                "transport": ["uds"],
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

    fn make_primal() -> {core_ident}::{type_name}Primal {{
        {core_ident}::{type_name}Primal::new()
    }}

    #[test]
    fn liveness_returns_alive() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":1,"method":"health.liveness"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["result"]["alive"], true);
    }}

    #[test]
    fn capabilities_list_includes_primal_and_methods() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":2,"method":"capabilities.list"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["result"]["primal"], PRIMAL_NAME);
        assert!(resp["result"]["methods"].is_array());
    }}

    #[test]
    fn unknown_method_returns_error() {{
        let primal = make_primal();
        let req = r#"{{"jsonrpc":"2.0","id":3,"method":"unknown.method"}}"#;
        let resp: serde_json::Value = serde_json::from_str(&handle_request(req, &primal)).unwrap();
        assert_eq!(resp["error"]["code"], -32601);
    }}

    #[test]
    fn invalid_json_returns_parse_error() {{
        let primal = make_primal();
        let resp: serde_json::Value =
            serde_json::from_str(&handle_request("not json", &primal)).unwrap();
        assert_eq!(resp["error"]["code"], -32700);
    }}
}}
"##,
    )
}
