//! Transport utilities for primal socket communication.
//!
//! Provides [`PeekedStream`] for first-byte protocol auto-detection on
//! socket accept. Primals use this to multiplex JSON-RPC 2.0 and BTSP
//! binary framing on the same listener.

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Stream wrapper that replays a single peeked byte before delegating.
///
/// After reading the first byte off a connection for protocol detection
/// (`{` → JSON-RPC, else BTSP binary), this wrapper makes the byte
/// available for the chosen handler to re-read.
///
/// # Example
///
/// ```no_run
/// use sourdough_core::transport::{peek_protocol, Protocol};
///
/// # async fn example(stream: tokio::net::TcpStream) -> std::io::Result<()> {
/// let (protocol, peeked) = peek_protocol(stream).await?;
/// match protocol {
///     Protocol::JsonRpc => { /* handle JSON-RPC with peeked stream */ }
///     Protocol::Binary  => { /* handle BTSP with peeked stream */ }
/// }
/// # Ok(())
/// # }
/// ```
pub struct PeekedStream<S> {
    peeked: Option<u8>,
    inner: S,
}

impl<S> PeekedStream<S> {
    /// Wrap a stream with a single pre-read byte.
    pub const fn new(inner: S, first_byte: u8) -> Self {
        Self {
            peeked: Some(first_byte),
            inner,
        }
    }

    /// Access the inner stream.
    pub const fn inner(&self) -> &S {
        &self.inner
    }

    /// Consume the wrapper, returning the inner stream.
    ///
    /// Any un-read peeked byte is lost.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PeekedStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if let Some(byte) = this.peeked.take() {
            buf.put_slice(&[byte]);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PeekedStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

/// Detected wire protocol from first-byte peek.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    /// JSON-RPC 2.0 (first byte is `{`, 0x7B).
    JsonRpc,
    /// Binary framing (BTSP or other).
    Binary,
}

/// Peek the first byte of a stream to determine the wire protocol.
///
/// Returns the detected [`Protocol`] and a [`PeekedStream`] that replays
/// the consumed byte transparently.
///
/// # Errors
///
/// Returns `io::Error` if the read fails or the connection is closed
/// before any data arrives.
pub async fn peek_protocol<S: AsyncRead + Unpin>(
    mut stream: S,
) -> std::io::Result<(Protocol, PeekedStream<S>)> {
    use tokio::io::AsyncReadExt;

    let mut byte = [0u8; 1];
    let n = stream.read(&mut byte).await?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "connection closed before first byte",
        ));
    }

    let protocol = if byte[0] == b'{' {
        Protocol::JsonRpc
    } else {
        Protocol::Binary
    };

    Ok((protocol, PeekedStream::new(stream, byte[0])))
}

/// Resolve the socket path for a primal using ecosystem conventions.
///
/// Path: `$BIOMEOS_SOCKET_DIR/{name}-{family_id}.sock` (production with family),
/// or `$XDG_RUNTIME_DIR/biomeos/{name}.sock` (development, no family ID).
///
/// Falls back to `/tmp/biomeos/` if neither env var is set.
#[must_use]
pub fn resolve_socket_path(primal_name: &str, family_id: Option<&str>) -> std::path::PathBuf {
    let socket_dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
        format!("{runtime_dir}/biomeos")
    });

    socket_path_in(&socket_dir, primal_name, family_id)
}

/// Build a socket path from explicit components (no env var reads).
#[must_use]
pub fn socket_path_in(
    socket_dir: &str,
    primal_name: &str,
    family_id: Option<&str>,
) -> std::path::PathBuf {
    let filename = family_id
        .filter(|id| !id.is_empty() && *id != "default")
        .map_or_else(
            || format!("{primal_name}.sock"),
            |fid| format!("{primal_name}-{fid}.sock"),
        );

    std::path::PathBuf::from(socket_dir).join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn peeked_stream_replays_json_rpc_byte() {
        let data: &[u8] = b"{\"jsonrpc\":\"2.0\"}";
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(protocol, Protocol::JsonRpc);

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn peeked_stream_replays_binary_byte() {
        let data: &[u8] = &[0x01, 0x02, 0x03, 0x04];
        let (protocol, mut stream) = peek_protocol(data).await.unwrap();

        assert_eq!(protocol, Protocol::Binary);

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn peek_empty_stream_returns_eof() {
        let data: &[u8] = b"";
        let result = peek_protocol(data).await;
        assert!(result.is_err());
    }

    #[test]
    fn socket_path_with_family_id() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", Some("abc123"));
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal-abc123.sock")
        );
    }

    #[test]
    fn socket_path_without_family_id() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", None);
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal.sock")
        );
    }

    #[test]
    fn socket_path_default_family_id_ignored() {
        let path = socket_path_in("/run/user/1000/biomeos", "testprimal", Some("default"));
        assert_eq!(
            path,
            std::path::PathBuf::from("/run/user/1000/biomeos/testprimal.sock")
        );
    }

    #[test]
    fn socket_path_empty_family_id_ignored() {
        let path = socket_path_in("/tmp/biomeos", "myprimal", Some(""));
        assert_eq!(path, std::path::PathBuf::from("/tmp/biomeos/myprimal.sock"));
    }
}
