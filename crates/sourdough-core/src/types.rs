//! Common types used across all primals.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Content-addressed hash (Blake3, 32 bytes).
///
/// Used for identifying data by its content rather than location.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    /// Create a new content hash from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a content hash from a hex string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not valid hex or not 64 characters.
    pub fn from_hex(s: &str) -> Result<Self, ContentHashError> {
        if s.len() != 64 {
            return Err(ContentHashError::InvalidLength(s.len()));
        }

        let mut bytes = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|_| ContentHashError::InvalidHex)?;
            bytes[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|_| ContentHashError::InvalidHex)?;
        }
        Ok(Self(bytes))
    }

    /// Convert to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Get the raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContentHash({})", &self.to_hex()[..16])
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Error parsing content hash.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContentHashError {
    /// Invalid length (expected 64 hex chars).
    #[error("invalid length: expected 64, got {0}")]
    InvalidLength(usize),

    /// Invalid hex characters.
    #[error("invalid hex characters")]
    InvalidHex,
}

/// Timestamp with nanosecond precision.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    /// Seconds since Unix epoch.
    pub secs: u64,
    /// Nanoseconds within the second.
    pub nanos: u32,
}

impl Timestamp {
    /// Create a timestamp for the current moment.
    #[must_use]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time went backwards");
        Self {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }

    /// Create a timestamp from seconds since epoch.
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self { secs, nanos: 0 }
    }

    /// Create a timestamp from milliseconds since epoch.
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / 1000,
            nanos: ((millis % 1000) * 1_000_000) as u32,
        }
    }

    /// Convert to milliseconds since epoch.
    #[must_use]
    pub const fn as_millis(&self) -> u64 {
        self.secs * 1000 + (self.nanos / 1_000_000) as u64
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timestamp({}.{:09})", self.secs, self.nanos)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ISO 8601 format
        use std::time::{Duration, UNIX_EPOCH};
        let time = UNIX_EPOCH + Duration::new(self.secs, self.nanos);
        write!(f, "{time:?}")
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_roundtrip() {
        let bytes = [42u8; 32];
        let hash = ContentHash::new(bytes);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn timestamp_ordering() {
        let t1 = Timestamp::from_secs(100);
        let t2 = Timestamp::from_secs(200);
        assert!(t1 < t2);
    }
}

