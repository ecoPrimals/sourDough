//! Announce template: Neural API `primal.announce` startup logic.
//!
//! Generated `announce.rs` provides fire-and-forget registration with
//! biomeOS's adaptive routing layer (Wave 42 ecosystem standard).

/// Generate the server `announce.rs` with Neural API `primal.announce` startup logic.
#[expect(clippy::too_many_lines, reason = "static template string")]
pub(in crate::commands::scaffold) fn announce_rs(name: &str) -> String {
    let name_lower = name.to_lowercase();
    format!(
        r#"//! Neural API self-announcement (Wave 42 `primal.announce` standard).
//!
//! On startup, the primal announces itself to biomeOS's Neural API for
//! adaptive routing. This is fire-and-forget — if biomeOS is unavailable,
//! the primal operates normally without routing intelligence.

use std::path::{{Path, PathBuf}};
use tokio::io::{{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::net::UnixStream;
use tracing::{{info, warn}};

const PRIMAL_NAME: &str = "{name_lower}";
const PRIMAL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Announce this primal to biomeOS's Neural API socket.
///
/// Discovers the neural-api socket via tiered lookup, connects, and sends
/// a `primal.announce` JSON-RPC call. Logs a warning and returns gracefully
/// if biomeOS is unreachable.
pub async fn announce_to_biomeos(primal_name: &str, socket: &Path, family: &str) {{
    let Some(neural_socket) = discover_neural_api_socket(family) else {{
        info!("biomeOS neural-api socket not found — skipping announce");
        return;
    }};

    let payload = serde_json::json!({{
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": {{
            "primal": primal_name,
            "socket": socket.display().to_string(),
            "pid": std::process::id(),
            "capabilities": capabilities(),
            "methods": crate::dispatch::METHODS,
            "signal_tiers": signal_tiers(),
            "cost_hints": cost_hints(),
            "latency_estimates": latency_estimates(),
            "version": PRIMAL_VERSION,
        }},
        "id": 1,
    }});

    match UnixStream::connect(&neural_socket).await {{
        Ok(stream) => {{
            let mut reader = BufReader::new(stream);
            let writer = reader.get_mut();
            let msg = format!("{{}}\n", payload);
            if let Err(e) = writer.write_all(msg.as_bytes()).await {{
                warn!("Failed to send primal.announce: {{e}}");
                return;
            }}
            // Read response (best-effort)
            let mut response = String::new();
            match reader.read_line(&mut response).await {{
                Ok(0) | Err(_) => warn!("No response to primal.announce"),
                Ok(_) => info!("Announced to Neural API: {{response}}"),
            }}
        }}
        Err(e) => {{
            info!("biomeOS neural-api not reachable ({{e}}) — primal operates standalone");
        }}
    }}
}}

/// Discover biomeOS neural-api socket via tiered lookup.
fn discover_neural_api_socket(family: &str) -> Option<PathBuf> {{
    // Tier 1: explicit env override
    if let Ok(path) = std::env::var("NEURAL_API_SOCKET") {{
        let p = PathBuf::from(&path);
        if p.exists() {{
            return Some(p);
        }}
    }}
    // Tier 2: XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {{
        let p = PathBuf::from(format!("{{runtime_dir}}/biomeos/neural-api-{{family}}.sock"));
        if p.exists() {{
            return Some(p);
        }}
    }}
    // Tier 3: /tmp fallback
    let p = PathBuf::from(format!("/tmp/biomeos/neural-api-{{family}}.sock"));
    if p.exists() {{
        return Some(p);
    }}
    None
}}

/// Register capabilities with Songbird for discovery by other primals.
///
/// This enables `ipc.resolve` — when another primal needs to find us,
/// songbird will return our endpoint.
pub async fn register_with_songbird(primal_name: &str, endpoint_json: &serde_json::Value) {{
    let Some(songbird_socket) = discover_songbird_socket() else {{
        info!("Songbird socket not found — skipping ipc.register");
        return;
    }};

    let payload = serde_json::json!({{
        "jsonrpc": "2.0",
        "method": "ipc.register",
        "params": {{
            "primal": primal_name,
            "capabilities": capabilities(),
            "endpoint": endpoint_json,
            "version": PRIMAL_VERSION,
        }},
        "id": 1,
    }});

    match UnixStream::connect(&songbird_socket).await {{
        Ok(stream) => {{
            let mut reader = BufReader::new(stream);
            let writer = reader.get_mut();
            let msg = format!("{{}}\n", payload);
            if let Err(e) = writer.write_all(msg.as_bytes()).await {{
                warn!("Failed to send ipc.register to songbird: {{e}}");
                return;
            }}
            let mut response = String::new();
            match reader.read_line(&mut response).await {{
                Ok(0) | Err(_) => warn!("No response from songbird ipc.register"),
                Ok(_) => info!("Registered with Songbird: {{response}}"),
            }}
        }}
        Err(e) => {{
            info!("Songbird not reachable ({{e}}) — primal operates without discovery");
        }}
    }}
}}

/// Discover songbird socket via standard biomeOS resolution.
fn discover_songbird_socket() -> Option<PathBuf> {{
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {{
        let p = PathBuf::from(format!("{{runtime_dir}}/biomeos/songbird.sock"));
        if p.exists() {{
            return Some(p);
        }}
    }}
    let p = PathBuf::from("/tmp/biomeos/songbird.sock");
    if p.exists() {{
        return Some(p);
    }}
    None
}}

/// Capability domains this primal serves.
///
/// Update this list as you add capabilities to your primal.
/// These are domain names (e.g. "crypto", "storage"), not method names.
pub fn capabilities() -> &'static [&'static str] {{
    // TODO: Replace with your primal's actual capability domains.
    &[]
}}

/// Composition tiers this primal participates in.
///
/// Valid values: "tower", "node", "nest", "meta".
pub fn signal_tiers() -> &'static [&'static str] {{
    // TODO: Replace with your primal's actual tier membership.
    &[]
}}

/// Cost hints per capability domain (arbitrary units, lower = cheaper).
fn cost_hints() -> serde_json::Value {{
    // TODO: Replace with your primal's actual cost estimates.
    serde_json::json!({{}})
}}

/// Latency estimates per capability domain (milliseconds).
fn latency_estimates() -> serde_json::Value {{
    // TODO: Replace with your primal's actual latency estimates.
    serde_json::json!({{}})
}}
"#,
    )
}
