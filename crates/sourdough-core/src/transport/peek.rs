//! First-byte protocol auto-detection (`PeekedStream`).
//!
//! Evolved to riboCipher-first routing (Wave 111). The accept loop reads
//! the first byte and checks for riboCipher signal prefixes (`0xEC`/`0xED`/`0xEE`)
//! BEFORE legacy peek-and-guess. Legacy connections produce `Protocol::Legacy`
//! for deprecation-period backwards compatibility.

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Stream wrapper that replays peeked bytes before delegating.
///
/// # Example
///
/// ```no_run
/// use sourdough_core::transport::{peek_protocol, Protocol};
///
/// # async fn example(stream: tokio::net::TcpStream) -> std::io::Result<()> {
/// let (protocol, peeked) = peek_protocol(stream).await?;
/// match protocol {
///     Protocol::RiboCipher { .. } => { /* riboCipher-signalled connection */ }
///     Protocol::JsonRpc => { /* legacy JSON-RPC (deprecation period) */ }
///     Protocol::Binary  => { /* legacy binary (deprecation period) */ }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
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

    /// Wrap a stream with no peeked byte (signal was fully consumed).
    pub const fn consumed(inner: S) -> Self {
        Self {
            peeked: None,
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

/// Detected wire protocol from first-byte detection.
///
/// riboCipher-signalled connections are preferred. Legacy connections
/// are supported during the deprecation period (Waves 111-113).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    /// riboCipher-signalled connection (standard — Wave 111+).
    RiboCipher {
        /// The protocol type declared by the signal.
        protocol_type: super::ribocipher::ProtocolType,
    },
    /// Legacy: JSON-RPC 2.0 (first byte is `{`, 0x7B). Deprecated.
    JsonRpc,
    /// Legacy: Binary framing (BTSP or other). Deprecated.
    Binary,
}

/// Peek the first byte of a stream to determine the wire protocol.
///
/// **riboCipher-first**: If the first byte is a signal prefix (`0xEC`),
/// reads the protocol type byte and returns `Protocol::RiboCipher`.
/// For mito/nuclear signals (`0xED`/`0xEE`), returns `RiboCipher` with
/// `NdjsonRpc` as the default (actual decode requires key material).
///
/// For legacy unsignalled connections, falls back to peek-and-guess:
/// `{` → `JsonRpc`, anything else → `Binary`.
///
/// The returned [`PeekedStream`] replays the first byte for legacy
/// connections (handler needs it) or is fully consumed for riboCipher
/// (signal envelope is not part of the payload).
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

    match byte[0] {
        super::ribocipher::SIGNAL_CLEAR => {
            let mut proto = [0u8; 1];
            stream.read_exact(&mut proto).await?;
            let protocol_type = super::ribocipher::ProtocolType::from_byte(proto[0])
                .unwrap_or(super::ribocipher::ProtocolType::NdjsonRpc);
            Ok((
                Protocol::RiboCipher { protocol_type },
                PeekedStream::consumed(stream),
            ))
        }
        super::ribocipher::SIGNAL_MITO => {
            let mut tag = [0u8; 4];
            stream.read_exact(&mut tag).await?;
            Ok((
                Protocol::RiboCipher {
                    protocol_type: super::ribocipher::ProtocolType::NdjsonRpc,
                },
                PeekedStream::consumed(stream),
            ))
        }
        super::ribocipher::SIGNAL_NUCLEAR => {
            let mut payload = [0u8; 6];
            stream.read_exact(&mut payload).await?;
            Ok((
                Protocol::RiboCipher {
                    protocol_type: super::ribocipher::ProtocolType::NdjsonRpc,
                },
                PeekedStream::consumed(stream),
            ))
        }
        b'{' => Ok((Protocol::JsonRpc, PeekedStream::new(stream, byte[0]))),
        _ => Ok((Protocol::Binary, PeekedStream::new(stream, byte[0]))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn ribocipher_clear_jsonrpc() {
        let data: &[u8] = &[0xEC, 0x01, b'h', b'i'];
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert!(matches!(
            proto,
            Protocol::RiboCipher {
                protocol_type: super::super::ribocipher::ProtocolType::NdjsonRpc
            }
        ));
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"hi");
    }

    #[tokio::test]
    async fn ribocipher_clear_btsp() {
        let data: &[u8] = &[0xEC, 0x02, 0xFF];
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert!(matches!(proto, Protocol::RiboCipher { .. }));
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, &[0xFF]);
    }

    #[tokio::test]
    async fn ribocipher_clear_unknown_type_defaults_to_jsonrpc() {
        let data: &[u8] = &[0xEC, 0xFE, b'x'];
        let (proto, _) = peek_protocol(data).await.unwrap();
        assert!(matches!(
            proto,
            Protocol::RiboCipher {
                protocol_type: super::super::ribocipher::ProtocolType::NdjsonRpc
            }
        ));
    }

    #[tokio::test]
    async fn ribocipher_mito_consumes_5_bytes() {
        let data: &[u8] = &[0xED, 0x01, 0x02, 0x03, 0x04, b'p'];
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert!(matches!(proto, Protocol::RiboCipher { .. }));
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"p");
    }

    #[tokio::test]
    async fn ribocipher_nuclear_consumes_7_bytes() {
        let data: &[u8] = &[0xEE, 1, 2, 3, 4, 5, 6, b'n'];
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert!(matches!(proto, Protocol::RiboCipher { .. }));
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"n");
    }

    #[tokio::test]
    async fn legacy_json_replays_brace() {
        let data: &[u8] = b"{\"jsonrpc\":\"2.0\"}";
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert_eq!(proto, Protocol::JsonRpc);
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn legacy_binary_replays_byte() {
        let data: &[u8] = &[0x01, 0x02, 0x03];
        let (proto, mut s) = peek_protocol(data).await.unwrap();
        assert_eq!(proto, Protocol::Binary);
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, data);
    }

    #[tokio::test]
    async fn empty_stream_eof() {
        let data: &[u8] = &[];
        let result = peek_protocol(data).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[tokio::test]
    async fn peeked_stream_consumed_has_no_replay() {
        let data: &[u8] = b"hello";
        let mut stream = PeekedStream::consumed(data);
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn protocol_debug_and_eq() {
        let a = Protocol::JsonRpc;
        let b = Protocol::JsonRpc;
        assert_eq!(a, b);
        let _ = format!("{a:?}");
    }

    #[test]
    fn protocol_ribocipher_variant() {
        let p = Protocol::RiboCipher {
            protocol_type: super::super::ribocipher::ProtocolType::Probe,
        };
        assert_ne!(p, Protocol::JsonRpc);
        assert_ne!(p, Protocol::Binary);
    }
}
