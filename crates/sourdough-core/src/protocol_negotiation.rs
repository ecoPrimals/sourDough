//! G65 Protocol Negotiation — single-socket protocol selection.
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc at connection time.
//! This is the **Phase 3** evolution of cephalization: a single socket serves both protocols,
//! eliminating the dual-socket pattern (`.sock` + `.tarpc.sock`).
//!
//! ## Wire Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! [Connection proceeds with selected protocol]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the client doesn't send a `PROTOCOLS:` line, the server assumes JSON-RPC.
//! This means legacy clients (Phase 1/2) continue to work without modification.
//!
//! ## Preference Order
//!
//! The server picks the first protocol from the client's list that it also supports.
//! Clients should list protocols in order of preference (typically `tarpc,jsonrpc`
//! for maximum performance with graceful fallback).

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};

/// RPC protocol variants supported by the ecoPrimals ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 — text-based, human-readable, backward-compatible default.
    #[default]
    JsonRpc,
    /// tarpc — binary, type-safe, high-performance intra-gate protocol.
    Tarpc,
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

impl IpcProtocol {
    /// Wire name used in the `PROTOCOLS:` / `PROTOCOL:` lines.
    #[must_use]
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }

    /// Parse a protocol from its wire name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" | "binary" => Some(Self::Tarpc),
            _ => None,
        }
    }

    /// All protocols this build supports (tarpc preferred).
    #[must_use]
    pub fn all_supported() -> Vec<Self> {
        vec![Self::Tarpc, Self::JsonRpc]
    }
}

/// Client's protocol negotiation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationRequest {
    /// Protocols the client supports, in preference order.
    pub supported: Vec<IpcProtocol>,
}

impl NegotiationRequest {
    /// Create a request listing the given protocols.
    #[must_use]
    pub const fn new(supported: Vec<IpcProtocol>) -> Self {
        Self { supported }
    }

    /// Request preferring tarpc, falling back to JSON-RPC.
    #[must_use]
    pub fn prefer_tarpc() -> Self {
        Self {
            supported: vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        }
    }

    /// Serialize to wire format: `"PROTOCOLS: tarpc,jsonrpc\n"`
    #[must_use]
    pub fn to_wire(&self) -> String {
        let names: Vec<&str> = self.supported.iter().map(IpcProtocol::wire_name).collect();
        format!("PROTOCOLS: {}\n", names.join(","))
    }

    /// Parse from wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the line doesn't start with `PROTOCOLS: ` or has no valid protocols.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let body = trimmed
            .strip_prefix("PROTOCOLS: ")
            .ok_or(NegotiationError::InvalidRequest)?;

        let supported: Vec<IpcProtocol> = body
            .split(',')
            .filter_map(|s| IpcProtocol::parse(s.trim()))
            .collect();

        if supported.is_empty() {
            return Err(NegotiationError::NoValidProtocols);
        }

        Ok(Self { supported })
    }
}

/// Server's protocol selection response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationResponse {
    /// The protocol the server selected.
    pub selected: IpcProtocol,
}

impl NegotiationResponse {
    /// Create a response selecting the given protocol.
    #[must_use]
    pub const fn new(selected: IpcProtocol) -> Self {
        Self { selected }
    }

    /// Serialize to wire format: `"PROTOCOL: tarpc\n"`
    #[must_use]
    pub fn to_wire(&self) -> String {
        format!("PROTOCOL: {}\n", self.selected.wire_name())
    }

    /// Parse from wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the line doesn't match the expected format.
    pub fn from_wire(line: &str) -> Result<Self, NegotiationError> {
        let trimmed = line.trim();
        let name = trimmed
            .strip_prefix("PROTOCOL: ")
            .ok_or(NegotiationError::InvalidResponse)?;

        let selected = IpcProtocol::parse(name).ok_or(NegotiationError::UnknownProtocol)?;

        Ok(Self { selected })
    }
}

/// Errors during protocol negotiation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NegotiationError {
    /// Line does not start with `PROTOCOLS: `.
    #[error("invalid negotiation request (expected PROTOCOLS: ...)")]
    InvalidRequest,
    /// Line does not start with `PROTOCOL: `.
    #[error("invalid negotiation response (expected PROTOCOL: ...)")]
    InvalidResponse,
    /// None of the listed protocols are recognized.
    #[error("no valid protocols in request")]
    NoValidProtocols,
    /// Protocol name not recognized.
    #[error("unknown protocol name")]
    UnknownProtocol,
    /// I/O error during negotiation.
    #[error("negotiation I/O error: {0}")]
    Io(String),
    /// Timeout waiting for negotiation.
    #[error("negotiation timed out")]
    Timeout,
}

/// Select the best protocol: first from `client_prefs` that `server_supports` also contains.
///
/// Falls back to `JsonRpc` if no intersection (JSON-RPC is always implicitly supported).
#[must_use]
pub fn select_protocol(
    client_prefs: &[IpcProtocol],
    server_supports: &[IpcProtocol],
) -> IpcProtocol {
    for proto in client_prefs {
        if server_supports.contains(proto) {
            return *proto;
        }
    }
    IpcProtocol::JsonRpc
}

/// Client-side negotiation: send preferences, receive server's selection.
///
/// # Errors
///
/// Returns `NegotiationError` on I/O failure or invalid response.
pub async fn negotiate_client<T>(
    transport: &mut T,
    supported: &[IpcProtocol],
) -> Result<IpcProtocol, NegotiationError>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let request = NegotiationRequest::new(supported.to_vec());
    let wire = request.to_wire();

    debug!("client sending: {:?}", wire.trim());
    transport
        .write_all(wire.as_bytes())
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;
    transport
        .flush()
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    let mut reader = BufReader::new(transport);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    let response = NegotiationResponse::from_wire(&line)?;
    info!("negotiated protocol: {}", response.selected);
    Ok(response.selected)
}

/// Server-side negotiation: read client request, select best, respond.
///
/// Returns `None` if the first line is not a `PROTOCOLS:` request (backward compat — assume JSON-RPC).
///
/// # Errors
///
/// Returns `NegotiationError` on I/O failure or malformed request.
pub async fn negotiate_server<T>(
    transport: &mut T,
    server_supported: &[IpcProtocol],
    timeout_ms: u64,
) -> Result<Option<IpcProtocol>, NegotiationError>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(transport);
    let mut line = String::new();

    let read_result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        reader.read_line(&mut line),
    )
    .await;

    match read_result {
        Ok(Ok(n)) if n > 0 => {
            if line.trim().starts_with("PROTOCOLS: ") {
                let request = NegotiationRequest::from_wire(&line)?;
                let selected = select_protocol(&request.supported, server_supported);

                let response = NegotiationResponse::new(selected);
                reader
                    .get_mut()
                    .write_all(response.to_wire().as_bytes())
                    .await
                    .map_err(|e| NegotiationError::Io(e.to_string()))?;
                reader
                    .get_mut()
                    .flush()
                    .await
                    .map_err(|e| NegotiationError::Io(e.to_string()))?;

                info!("server negotiated: {selected}");
                Ok(Some(selected))
            } else {
                warn!("no protocol negotiation, assuming JSON-RPC");
                Ok(None)
            }
        }
        Ok(Err(e)) => {
            warn!("negotiation read error: {e}");
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_protocol_display() {
        assert_eq!(IpcProtocol::JsonRpc.to_string(), "jsonrpc");
        assert_eq!(IpcProtocol::Tarpc.to_string(), "tarpc");
    }

    #[test]
    fn ipc_protocol_parse() {
        assert_eq!(IpcProtocol::parse("jsonrpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::parse("json-rpc"), Some(IpcProtocol::JsonRpc));
        assert_eq!(IpcProtocol::parse("tarpc"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::parse("binary"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::parse("unknown"), None);
    }

    #[test]
    fn ipc_protocol_serde_roundtrip() {
        for proto in [IpcProtocol::JsonRpc, IpcProtocol::Tarpc] {
            let json = serde_json::to_string(&proto).unwrap();
            let back: IpcProtocol = serde_json::from_str(&json).unwrap();
            assert_eq!(proto, back);
        }
    }

    #[test]
    fn negotiation_request_wire_roundtrip() {
        let req = NegotiationRequest::prefer_tarpc();
        let wire = req.to_wire();
        assert_eq!(wire, "PROTOCOLS: tarpc,jsonrpc\n");
        let parsed = NegotiationRequest::from_wire(&wire).unwrap();
        assert_eq!(req, parsed);
    }

    #[test]
    fn negotiation_request_single_protocol() {
        let req = NegotiationRequest::new(vec![IpcProtocol::JsonRpc]);
        assert_eq!(req.to_wire(), "PROTOCOLS: jsonrpc\n");
    }

    #[test]
    fn negotiation_request_invalid_prefix() {
        let err = NegotiationRequest::from_wire("INVALID: foo\n").unwrap_err();
        assert_eq!(err, NegotiationError::InvalidRequest);
    }

    #[test]
    fn negotiation_request_no_valid_protocols() {
        let err = NegotiationRequest::from_wire("PROTOCOLS: foo,bar\n").unwrap_err();
        assert_eq!(err, NegotiationError::NoValidProtocols);
    }

    #[test]
    fn negotiation_response_wire_roundtrip() {
        let resp = NegotiationResponse::new(IpcProtocol::Tarpc);
        let wire = resp.to_wire();
        assert_eq!(wire, "PROTOCOL: tarpc\n");
        let parsed = NegotiationResponse::from_wire(&wire).unwrap();
        assert_eq!(resp, parsed);
    }

    #[test]
    fn negotiation_response_invalid() {
        let err = NegotiationResponse::from_wire("STATUS: ok\n").unwrap_err();
        assert_eq!(err, NegotiationError::InvalidResponse);
    }

    #[test]
    fn select_protocol_prefers_client_order() {
        let client = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::Tarpc);
    }

    #[test]
    fn select_protocol_server_only_jsonrpc() {
        let client = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = &[IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn select_protocol_no_intersection_falls_back() {
        let client = &[IpcProtocol::Tarpc];
        let server = &[IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::JsonRpc);
    }

    #[tokio::test]
    async fn negotiate_duplex_tarpc_preferred() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(4096);

        let server_supported = IpcProtocol::all_supported();
        let server_task = tokio::spawn(async move {
            negotiate_server(&mut server_stream, &server_supported, 1000).await
        });

        let client_result = negotiate_client(
            &mut client_stream,
            &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        )
        .await
        .unwrap();
        assert_eq!(client_result, IpcProtocol::Tarpc);

        let server_result = server_task.await.unwrap().unwrap();
        assert_eq!(server_result, Some(IpcProtocol::Tarpc));
    }

    #[tokio::test]
    async fn negotiate_duplex_jsonrpc_only() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(4096);

        let server_task = tokio::spawn(async move {
            negotiate_server(&mut server_stream, &[IpcProtocol::JsonRpc], 1000).await
        });

        let client_result = negotiate_client(
            &mut client_stream,
            &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        )
        .await
        .unwrap();
        assert_eq!(client_result, IpcProtocol::JsonRpc);

        let server_result = server_task.await.unwrap().unwrap();
        assert_eq!(server_result, Some(IpcProtocol::JsonRpc));
    }

    #[tokio::test]
    async fn negotiate_server_non_protocol_line_returns_none() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(4096);

        tokio::spawn(async move {
            client_stream
                .write_all(b"{\"jsonrpc\":\"2.0\"}\n")
                .await
                .unwrap();
            client_stream.flush().await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let result = negotiate_server(&mut server_stream, &IpcProtocol::all_supported(), 200)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
