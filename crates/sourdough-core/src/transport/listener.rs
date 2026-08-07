//! Transport-agnostic listener (G66 server-side transport abstraction).
//!
//! Mirrors [`super::TransportStream`] for the accept side. Business logic
//! calls `listener.accept()` and gets back a [`super::TransportStream`]
//! without knowing the underlying mechanism.

use super::endpoint::TransportEndpoint;
use super::stream::TransportStream;
use std::io;

/// A transport-agnostic listener that accepts incoming connections.
///
/// On Unix: can bind to UDS or TCP.
/// On non-Unix: only TCP is available (UDS binding returns an error).
#[derive(Debug)]
pub enum TransportListener {
    /// Unix domain socket listener.
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    /// TCP listener.
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Accept the next incoming connection.
    ///
    /// Returns a [`TransportStream`] implementing `AsyncRead + AsyncWrite`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if accept fails.
    pub async fn accept(&self) -> io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Unix(stream))
            }
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Tcp(stream))
            }
        }
    }

    /// The transport kind (for logging/diagnostics).
    #[must_use]
    pub const fn transport_name(&self) -> &'static str {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => "uds",
            Self::Tcp(_) => "tcp",
        }
    }

    /// Whether the listener accepts local-only connections (UDS or TCP localhost).
    #[must_use]
    pub fn is_local(&self) -> bool {
        match self {
            #[cfg(unix)]
            Self::Unix(_) => true,
            Self::Tcp(l) => l
                .local_addr()
                .map(|a| a.ip().is_loopback())
                .unwrap_or(false),
        }
    }

    /// The local address/path this listener is bound to (for diagnostics).
    #[must_use]
    pub fn local_endpoint(&self) -> TransportEndpoint {
        match self {
            #[cfg(unix)]
            Self::Unix(l) => {
                let path = l
                    .local_addr()
                    .ok()
                    .and_then(|a| a.as_pathname().map(|p| p.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "<abstract>".to_owned());
                TransportEndpoint::Uds { path }
            }
            Self::Tcp(l) => {
                let addr = l
                    .local_addr()
                    .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0)));
                TransportEndpoint::Tcp {
                    host: addr.ip().to_string(),
                    port: addr.port(),
                }
            }
        }
    }
}

/// Bind a listener based on the given [`TransportEndpoint`].
///
/// On Unix, UDS endpoints create the socket (removing stale files first).
/// On non-Unix, UDS endpoints return an error.
/// TCP endpoints bind on the specified host:port.
/// `MeshRelay` endpoints cannot be bound (they require songBird routing).
///
/// # Errors
///
/// Returns `io::Error` on bind failure or unsupported endpoint type.
pub async fn bind_transport(endpoint: &TransportEndpoint) -> io::Result<TransportListener> {
    match endpoint {
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let socket_path = std::path::PathBuf::from(path);
            if let Some(parent) = socket_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::remove_file(&socket_path);
            let listener = tokio::net::UnixListener::bind(&socket_path)?;
            Ok(TransportListener::Unix(listener))
        }
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("UDS not available on this platform for {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let addr = format!("{host}:{port}");
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            Ok(TransportListener::Tcp(listener))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "mesh relay ({peer_id}/{capability}) cannot be bound — \
                 register capabilities with songBird and let it route traffic"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn bind_tcp_and_accept() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 0);
        let listener = bind_transport(&ep).await.unwrap();
        assert_eq!(listener.transport_name(), "tcp");
        assert!(listener.is_local());

        let local_ep = listener.local_endpoint();
        let (host, port) = local_ep.tcp_addr().unwrap();
        assert_eq!(host, "127.0.0.1");
        assert!(port > 0);

        let connect_ep = TransportEndpoint::tcp("127.0.0.1", port);
        let server = tokio::spawn(async move {
            let mut stream = listener.accept().await.unwrap();
            assert_eq!(stream.transport_name(), "tcp");
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut client = super::super::stream::connect_transport(&connect_ep)
            .await
            .unwrap();
        client.write_all(b"echo").await.unwrap();
        let mut buf = [0u8; 4];
        client.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"echo");

        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn bind_uds_and_accept() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let sock_path = tmp.path().to_owned();
        drop(tmp);

        let ep = TransportEndpoint::uds(sock_path.to_string_lossy().as_ref());
        let listener = bind_transport(&ep).await.unwrap();
        assert_eq!(listener.transport_name(), "uds");
        assert!(listener.is_local());

        let server = tokio::spawn(async move {
            let mut stream = listener.accept().await.unwrap();
            assert_eq!(stream.transport_name(), "uds");
            let mut buf = [0u8; 3];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut client = super::super::stream::connect_transport(&ep).await.unwrap();
        client.write_all(b"hey").await.unwrap();
        let mut buf = [0u8; 3];
        client.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hey");

        server.await.unwrap();
        let _ = std::fs::remove_file(&sock_path);
    }

    #[tokio::test]
    async fn bind_mesh_relay_returns_unsupported() {
        let ep = TransportEndpoint::mesh_relay("peer", "cap");
        let result = bind_transport(&ep).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[tokio::test]
    async fn local_endpoint_tcp_reflects_bound_addr() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 0);
        let listener = bind_transport(&ep).await.unwrap();
        let local = listener.local_endpoint();
        assert!(local.tcp_addr().is_some());
        assert!(local.tcp_addr().unwrap().1 > 0);
    }
}
