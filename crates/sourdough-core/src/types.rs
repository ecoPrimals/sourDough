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
            let hex_str = std::str::from_utf8(chunk).map_err(|_| ContentHashError::InvalidHex)?;
            bytes[i] = u8::from_str_radix(hex_str, 16).map_err(|_| ContentHashError::InvalidHex)?;
        }
        Ok(Self(bytes))
    }

    /// Convert to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.iter().fold(String::with_capacity(64), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(&mut s, "{b:02x}");
            s
        })
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
    ///
    /// # Panics
    ///
    /// Panics if system time is before Unix epoch (1970-01-01). This should never
    /// happen on any modern system.
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
    #[expect(
        clippy::cast_possible_truncation,
        reason = "truncation is safe: (millis % 1000) * 1_000_000 < u32::MAX"
    )]
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / 1000,
            // Safe truncation: (millis % 1000) is always < 1000, so * 1_000_000 < u32::MAX
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
    fn content_hash_from_hex_valid() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let hash = ContentHash::from_hex(hex).unwrap();

        assert_eq!(hash.as_bytes()[0], 0x01);
        assert_eq!(hash.as_bytes()[1], 0x23);
        assert_eq!(hash.as_bytes()[31], 0xef);
    }

    #[test]
    fn content_hash_from_hex_invalid_length() {
        let result = ContentHash::from_hex("too_short");
        assert!(result.is_err());

        match result {
            Err(ContentHashError::InvalidLength(len)) => assert_eq!(len, 9),
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn content_hash_from_hex_invalid_chars() {
        let invalid = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let result = ContentHash::from_hex(invalid);
        assert!(result.is_err());

        match result {
            Err(ContentHashError::InvalidHex) => {}
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn content_hash_to_hex() {
        let bytes = [255u8; 32];
        let hash = ContentHash::new(bytes);
        let hex = hash.to_hex();

        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hex, "f".repeat(64));
    }

    #[test]
    fn content_hash_as_bytes() {
        let bytes = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let hash = ContentHash::new(bytes);

        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn content_hash_display() {
        let bytes = [15u8; 32];
        let hash = ContentHash::new(bytes);
        let display = format!("{hash}");

        assert_eq!(display.len(), 64);
        assert_eq!(display, "0f".repeat(32));
    }

    #[test]
    fn content_hash_debug() {
        let bytes = [15u8; 32];
        let hash = ContentHash::new(bytes);
        let debug = format!("{hash:?}");

        assert!(debug.contains("ContentHash"));
        assert!(debug.contains("0f"));
    }

    #[test]
    fn content_hash_clone_and_copy() {
        let hash1 = ContentHash::new([1u8; 32]);
        let hash2 = hash1; // Copy
        let hash3 = hash1; // Copy (not clone, since ContentHash implements Copy)

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn content_hash_serialization() {
        let hash = ContentHash::new([42u8; 32]);
        let json = serde_json::to_string(&hash).unwrap();
        let parsed: ContentHash = serde_json::from_str(&json).unwrap();

        assert_eq!(hash, parsed);
    }

    #[test]
    fn content_hash_error_display() {
        let err = ContentHashError::InvalidLength(10);
        assert_eq!(err.to_string(), "invalid length: expected 64, got 10");

        let err = ContentHashError::InvalidHex;
        assert_eq!(err.to_string(), "invalid hex characters");
    }

    #[test]
    fn timestamp_ordering() {
        let t1 = Timestamp::from_secs(100);
        let t2 = Timestamp::from_secs(200);
        assert!(t1 < t2);
    }

    #[test]
    fn timestamp_now() {
        let t1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let t2 = Timestamp::now();

        assert!(t2 > t1);
    }

    #[test]
    fn timestamp_from_secs() {
        let ts = Timestamp::from_secs(1_234_567_890);

        assert_eq!(ts.secs, 1_234_567_890);
        assert_eq!(ts.nanos, 0);
    }

    #[test]
    fn timestamp_from_millis() {
        let ts = Timestamp::from_millis(1500);

        assert_eq!(ts.secs, 1);
        assert_eq!(ts.nanos, 500_000_000);
    }

    #[test]
    fn timestamp_as_millis() {
        let ts = Timestamp {
            secs: 10,
            nanos: 500_000_000,
        };

        assert_eq!(ts.as_millis(), 10_500);
    }

    #[test]
    fn timestamp_millis_roundtrip() {
        let millis = 123_456_789_u64;
        let ts = Timestamp::from_millis(millis);
        let converted = ts.as_millis();

        // Allow for small precision loss due to nanosecond truncation
        #[expect(
            clippy::cast_possible_wrap,
            reason = "test compares u64 millis via i64 abs_diff; values stay in comparable range"
        )]
        let diff = (converted as i64).abs_diff(millis as i64);
        assert!(diff < 2);
    }

    #[test]
    fn timestamp_default() {
        let ts = Timestamp::default();

        // Default should be recent (within last minute)
        assert!(ts.secs > 0);
    }

    #[test]
    fn timestamp_display() {
        let ts = Timestamp::from_secs(0);
        let display = format!("{ts}");

        // Should contain time information
        assert!(!display.is_empty());
    }

    #[test]
    fn timestamp_debug() {
        let ts = Timestamp {
            secs: 123,
            nanos: 456_789_000,
        };
        let debug = format!("{ts:?}");

        assert!(debug.contains("123"));
        assert!(debug.contains("456789000"));
    }

    #[test]
    fn timestamp_equality() {
        let ts1 = Timestamp {
            secs: 100,
            nanos: 500,
        };
        let ts2 = Timestamp {
            secs: 100,
            nanos: 500,
        };
        let ts3 = Timestamp {
            secs: 100,
            nanos: 501,
        };

        assert_eq!(ts1, ts2);
        assert_ne!(ts1, ts3);
    }

    #[test]
    fn timestamp_clone_and_copy() {
        let ts1 = Timestamp::now();
        let ts2 = ts1; // Copy
        let ts3 = ts1; // Copy (not clone, since Timestamp implements Copy)

        assert_eq!(ts1, ts2);
        assert_eq!(ts1, ts3);
    }

    #[test]
    fn timestamp_serialization() {
        let ts = Timestamp {
            secs: 1_234_567_890,
            nanos: 123_456_789,
        };

        let json = serde_json::to_string(&ts).unwrap();
        let parsed: Timestamp = serde_json::from_str(&json).unwrap();

        assert_eq!(ts, parsed);
    }

    #[test]
    fn timestamp_hash() {
        use std::collections::HashSet;

        let ts1 = Timestamp::from_secs(100);
        let ts2 = Timestamp::from_secs(200);
        let ts3 = Timestamp::from_secs(100);

        let mut set = HashSet::new();
        set.insert(ts1);
        set.insert(ts2);
        set.insert(ts3);

        assert_eq!(set.len(), 2); // ts1 and ts3 are the same
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn content_hash_hex_roundtrip(bytes in proptest::collection::vec(any::<u8>(), 32..=32)) {
                let arr: [u8; 32] = bytes.try_into().unwrap();
                let hash = ContentHash::new(arr);
                let hex = hash.to_hex();
                let parsed = ContentHash::from_hex(&hex).unwrap();
                prop_assert_eq!(hash, parsed);
            }

            #[test]
            fn timestamp_millis_roundtrip(secs in 0u64..=4_000_000_000u64, millis_frac in 0u64..1000u64) {
                let total_millis = secs * 1000 + millis_frac;
                let ts = Timestamp::from_millis(total_millis);
                let back = ts.as_millis();
                prop_assert_eq!(total_millis, back);
            }

            #[test]
            fn timestamp_ordering_consistent(a_secs in 0u64..=1_000_000u64, b_secs in 0u64..=1_000_000u64) {
                let a = Timestamp::from_secs(a_secs);
                let b = Timestamp::from_secs(b_secs);
                prop_assert_eq!(a_secs < b_secs, a < b);
                prop_assert_eq!(a_secs == b_secs, a == b);
            }
        }
    }
}
