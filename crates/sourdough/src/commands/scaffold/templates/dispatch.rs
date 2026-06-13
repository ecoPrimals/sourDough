//! Dispatch template: JSON-RPC method routing with capability wire standard handlers.

/// Generate the server `dispatch.rs` with capability wire handlers.
pub(in crate::commands::scaffold) fn dispatch_rs(name: &str) -> String {
    format!("{}{}", dispatch_core(name), dispatch_tests(name))
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
