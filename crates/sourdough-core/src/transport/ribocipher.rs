//! riboCipher transport signal standard (Wave 111).
//!
//! Replaces ad-hoc peek-and-guess protocol detection with intentional
//! signal envelopes. Connections declare their protocol via a signal
//! prefix; servers route deterministically.
//!
//! # Wire Format
//!
//! | Signal byte | Tier | Envelope |
//! |-------------|------|----------|
//! | `0xEC` | Clear (local UDS) | `[0xEC][protocol_type: u8]` (2 bytes) |
//! | `0xED` | Mito-obfuscated (family WAN) | `[0xED][hmac_tag: [u8; 4]]` (5 bytes) |
//! | `0xEE` | Nuclear-sealed (privileged) | `[0xEE][encrypted: [u8; 6]]` (7 bytes) |
//!
//! See: `wateringHole/RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md`

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Signal prefix for clear-tier riboCipher (local UDS, trusted wire).
pub const SIGNAL_CLEAR: u8 = 0xEC;
/// Signal prefix for mito-obfuscated tier (cross-gate WAN, family seed).
pub const SIGNAL_MITO: u8 = 0xED;
/// Signal prefix for nuclear-sealed tier (privileged, per-peer key).
pub const SIGNAL_NUCLEAR: u8 = 0xEE;

/// Protocol types carried within the riboCipher signal envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolType {
    /// Lightweight health probe.
    Probe = 0x00,
    /// Newline-delimited JSON-RPC 2.0 (standard ecosystem IPC).
    NdjsonRpc = 0x01,
    /// BTSP length-prefixed binary handshake.
    BtspBinary = 0x02,
    /// BTSP JSON-line `ClientHello` handshake.
    BtspJsonLine = 0x03,
    /// HTTP/1.1 over UDS (axum/hyper).
    Http = 0x04,
    /// Encrypted resume (post-BTSP session).
    EncryptedResume = 0x05,
    /// Dark Forest Beacon packet.
    DarkForestBeacon = 0x06,
    /// songBird mesh relay frame.
    MeshRelay = 0x07,
}

impl ProtocolType {
    /// Parse a protocol type byte.
    #[must_use]
    pub const fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::Probe),
            0x01 => Some(Self::NdjsonRpc),
            0x02 => Some(Self::BtspBinary),
            0x03 => Some(Self::BtspJsonLine),
            0x04 => Some(Self::Http),
            0x05 => Some(Self::EncryptedResume),
            0x06 => Some(Self::DarkForestBeacon),
            0x07 => Some(Self::MeshRelay),
            _ => None,
        }
    }

    /// The wire byte for this protocol type.
    #[must_use]
    pub const fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Result of riboCipher signal detection on an incoming connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalResult {
    /// Clear-tier signal detected — protocol type decoded directly.
    Clear(ProtocolType),
    /// Mito-obfuscated signal detected — HMAC tag read (decode requires family key).
    Mito {
        /// The 4-byte HMAC tag from the wire.
        tag: [u8; 4],
    },
    /// Nuclear-sealed signal detected — encrypted payload (decode requires nuclear key).
    Nuclear {
        /// The 6-byte encrypted payload from the wire.
        payload: [u8; 6],
    },
    /// No riboCipher signal — legacy unsignalled connection (deprecation period).
    Legacy {
        /// The first byte that was read (needed for legacy routing).
        first_byte: u8,
    },
}

impl SignalResult {
    /// Whether this connection used a riboCipher signal.
    #[must_use]
    pub const fn is_signalled(&self) -> bool {
        !matches!(self, Self::Legacy { .. })
    }

    /// The signal tier, if signalled.
    #[must_use]
    pub const fn tier(&self) -> Option<SignalTier> {
        match self {
            Self::Clear(_) => Some(SignalTier::Clear),
            Self::Mito { .. } => Some(SignalTier::Mito),
            Self::Nuclear { .. } => Some(SignalTier::Nuclear),
            Self::Legacy { .. } => None,
        }
    }
}

/// Signal tier classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalTier {
    /// Clear — local trusted wire.
    Clear,
    /// Mito-obfuscated — family WAN.
    Mito,
    /// Nuclear-sealed — privileged.
    Nuclear,
}

/// Whether a byte is a riboCipher signal prefix.
#[must_use]
pub const fn is_signal_byte(b: u8) -> bool {
    matches!(b, SIGNAL_CLEAR | SIGNAL_MITO | SIGNAL_NUCLEAR)
}

/// Detect the riboCipher signal from the first bytes of a connection.
///
/// Reads the first byte. If it's a signal prefix (`0xEC`/`0xED`/`0xEE`),
/// reads the remainder of the signal envelope. Otherwise returns a
/// `Legacy` result for deprecation-period fallback routing.
///
/// # Errors
///
/// Returns `io::Error` if reading from the stream fails.
pub async fn detect_signal<S: AsyncRead + Unpin>(stream: &mut S) -> std::io::Result<SignalResult> {
    let mut first = [0u8; 1];
    let n = stream.read(&mut first).await?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "connection closed before first byte",
        ));
    }

    match first[0] {
        SIGNAL_CLEAR => {
            let mut proto = [0u8; 1];
            stream.read_exact(&mut proto).await?;
            let protocol_type = ProtocolType::from_byte(proto[0]).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown riboCipher protocol type: 0x{:02X}", proto[0]),
                )
            })?;
            Ok(SignalResult::Clear(protocol_type))
        }
        SIGNAL_MITO => {
            let mut tag = [0u8; 4];
            stream.read_exact(&mut tag).await?;
            Ok(SignalResult::Mito { tag })
        }
        SIGNAL_NUCLEAR => {
            let mut payload = [0u8; 6];
            stream.read_exact(&mut payload).await?;
            Ok(SignalResult::Nuclear { payload })
        }
        other => Ok(SignalResult::Legacy { first_byte: other }),
    }
}

/// Send a clear-tier riboCipher signal before a payload.
///
/// Prepends `[0xEC, protocol_type]` to the connection. This is what
/// clients send before their first protocol message.
///
/// # Errors
///
/// Returns `io::Error` if writing to the stream fails.
pub async fn send_clear_signal<S: AsyncWrite + Unpin>(
    stream: &mut S,
    protocol_type: ProtocolType,
) -> std::io::Result<()> {
    stream
        .write_all(&[SIGNAL_CLEAR, protocol_type.as_byte()])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn detect_clear_jsonrpc() {
        let mut data = Cursor::new(vec![0xEC, 0x01]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(result, SignalResult::Clear(ProtocolType::NdjsonRpc));
        assert!(result.is_signalled());
        assert_eq!(result.tier(), Some(SignalTier::Clear));
    }

    #[tokio::test]
    async fn detect_clear_btsp_binary() {
        let mut data = Cursor::new(vec![0xEC, 0x02]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(result, SignalResult::Clear(ProtocolType::BtspBinary));
    }

    #[tokio::test]
    async fn detect_clear_probe() {
        let mut data = Cursor::new(vec![0xEC, 0x00]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(result, SignalResult::Clear(ProtocolType::Probe));
    }

    #[tokio::test]
    async fn detect_mito_signal() {
        let mut data = Cursor::new(vec![0xED, 0xAA, 0xBB, 0xCC, 0xDD]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(
            result,
            SignalResult::Mito {
                tag: [0xAA, 0xBB, 0xCC, 0xDD]
            }
        );
        assert!(result.is_signalled());
        assert_eq!(result.tier(), Some(SignalTier::Mito));
    }

    #[tokio::test]
    async fn detect_nuclear_signal() {
        let mut data = Cursor::new(vec![0xEE, 1, 2, 3, 4, 5, 6]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(
            result,
            SignalResult::Nuclear {
                payload: [1, 2, 3, 4, 5, 6]
            }
        );
        assert!(result.is_signalled());
        assert_eq!(result.tier(), Some(SignalTier::Nuclear));
    }

    #[tokio::test]
    async fn detect_legacy_json() {
        let mut data = Cursor::new(vec![b'{', b'"']);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(result, SignalResult::Legacy { first_byte: b'{' });
        assert!(!result.is_signalled());
        assert_eq!(result.tier(), None);
    }

    #[tokio::test]
    async fn detect_legacy_binary() {
        let mut data = Cursor::new(vec![0x01, 0x02]);
        let result = detect_signal(&mut data).await.unwrap();
        assert_eq!(result, SignalResult::Legacy { first_byte: 0x01 });
    }

    #[tokio::test]
    async fn detect_empty_stream_eof() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let result = detect_signal(&mut data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn detect_unknown_protocol_type() {
        let mut data = Cursor::new(vec![0xEC, 0xFF]);
        let result = detect_signal(&mut data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn send_clear_signal_writes_two_bytes() {
        let mut buf = Vec::new();
        send_clear_signal(&mut buf, ProtocolType::NdjsonRpc)
            .await
            .unwrap();
        assert_eq!(buf, vec![0xEC, 0x01]);
    }

    #[tokio::test]
    async fn send_clear_signal_btsp() {
        let mut buf = Vec::new();
        send_clear_signal(&mut buf, ProtocolType::BtspBinary)
            .await
            .unwrap();
        assert_eq!(buf, vec![0xEC, 0x02]);
    }

    #[test]
    fn protocol_type_roundtrip() {
        for byte in 0x00..=0x07u8 {
            let pt = ProtocolType::from_byte(byte).unwrap();
            assert_eq!(pt.as_byte(), byte);
        }
    }

    #[test]
    fn is_signal_byte_classification() {
        assert!(is_signal_byte(0xEC));
        assert!(is_signal_byte(0xED));
        assert!(is_signal_byte(0xEE));
        assert!(!is_signal_byte(0xEB));
        assert!(!is_signal_byte(0xEF));
        assert!(!is_signal_byte(b'{'));
        assert!(!is_signal_byte(0x00));
    }
}
