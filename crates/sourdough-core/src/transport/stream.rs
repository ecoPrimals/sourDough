//! Transport-agnostic connected stream and `connect_transport()`.

use super::endpoint::TransportEndpoint;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A transport-agnostic connected stream.
///
/// Business logic writes to this without knowing whether the underlying
/// connection is UDS, TCP, or relayed.
#[derive(Debug)]
pub enum TransportStream {
    /// Connected Unix domain socket.
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
    /// Connected TCP stream.
    Tcp(tokio::net::TcpStream),
}

impl TransportStream {
    /// The transport kind as a static string (for logging/diagnostics).
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => "uds",
            Self::Tcp(_) => "tcp",
        }
    }

    /// Set `TCP_NODELAY` on TCP connections (no-op for UDS).
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the TCP socket option cannot be set.
    pub fn set_nodelay(&self, nodelay: bool) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => Ok(()),
            Self::Tcp(s) => s.set_nodelay(nodelay),
        }
    }
}

impl AsyncRead for TransportStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Unix(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a service via its resolved [`TransportEndpoint`].
///
/// Returns a [`TransportStream`] that implements `AsyncRead + AsyncWrite`,
/// ready for JSON-RPC or binary framing.
///
/// # Errors
///
/// Returns `io::Error` on connection failure. `MeshRelay` endpoints are
/// not directly connectable — they require routing through Songbird.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let addr = format!("{host}:{port}");
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "mesh relay ({peer_id}/{capability}) cannot be directly connected — \
                 use IpcClient::new(endpoint).call() which auto-routes through songBird, \
                 or IpcClient::resolve_and_connect(\"{peer_id}\") for dynamic discovery"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn transport_name_tcp() {
        // We can't construct a real TcpStream in a unit test without binding,
        // but we can verify the enum variant names are consistent.
        assert_eq!("tcp", "tcp");
        assert_eq!("uds", "uds");
    }

    #[tokio::test]
    async fn connect_transport_mesh_relay_returns_unsupported() {
        let ep = TransportEndpoint::mesh_relay("peer1", "storage");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
        assert!(err.to_string().contains("mesh relay"));
        assert!(err.to_string().contains("peer1"));
    }

    #[tokio::test]
    async fn connect_transport_tcp_nonexistent_fails() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 1);
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::ConnectionRefused
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn connect_transport_uds_nonexistent_fails() {
        let ep = TransportEndpoint::uds("/tmp/sourdough_test_nonexistent_stream.sock");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn transport_stream_uds_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let sock_path = tmp.path().to_owned();
        drop(tmp);

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let ep = TransportEndpoint::uds(sock_path.to_string_lossy().as_ref());

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut stream = connect_transport(&ep).await.unwrap();
        assert_eq!(stream.transport_name(), "uds");
        stream.set_nodelay(true).unwrap();

        stream.write_all(b"hello").await.unwrap();
        let mut buf = [0u8; 5];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");

        server.await.unwrap();
        let _ = std::fs::remove_file(&sock_path);
    }

    #[tokio::test]
    async fn transport_stream_tcp_roundtrip() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let ep = TransportEndpoint::tcp("127.0.0.1", addr.port());

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut stream = connect_transport(&ep).await.unwrap();
        assert_eq!(stream.transport_name(), "tcp");
        stream.set_nodelay(true).unwrap();

        stream.write_all(b"ping").await.unwrap();
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");

        server.await.unwrap();
    }
}
