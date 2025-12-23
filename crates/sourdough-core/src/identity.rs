//! Identity traits for BearDog integration.
//!
//! Every primal needs an identity—a way to prove who it is and sign its actions.
//! This module provides the traits for integrating with BearDog's identity system.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

/// Decentralized Identifier (DID).
///
/// DIDs are the foundation of identity in ecoPrimals. They are:
/// - Self-sovereign (you control your own identity)
/// - Cryptographically verifiable
/// - Decentralized (no central authority)
///
/// # Example
///
/// ```
/// use sourdough_core::Did;
///
/// let did = Did::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
/// assert!(did.as_str().starts_with("did:"));
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(String);

impl Did {
    /// Create a new DID from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the DID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a key-based DID (did:key:...).
    #[must_use]
    pub fn is_key_did(&self) -> bool {
        self.0.starts_with("did:key:")
    }

    /// Check if this is a web-based DID (did:web:...).
    #[must_use]
    pub fn is_web_did(&self) -> bool {
        self.0.starts_with("did:web:")
    }
}

impl std::fmt::Debug for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Did({})", self.0)
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Cryptographic signature.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// The signature bytes.
    pub bytes: Vec<u8>,
    /// The algorithm used (e.g., "Ed25519").
    pub algorithm: String,
    /// The key ID that created this signature.
    pub key_id: String,
}

impl Signature {
    /// Create a new signature.
    #[must_use]
    pub fn new(bytes: Vec<u8>, algorithm: impl Into<String>, key_id: impl Into<String>) -> Self {
        Self {
            bytes,
            algorithm: algorithm.into(),
            key_id: key_id.into(),
        }
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Signature({}, {}, {} bytes)",
            self.algorithm,
            self.key_id,
            self.bytes.len()
        )
    }
}

/// Identity trait for primals.
///
/// Implement this trait to integrate with BearDog for identity and signing.
///
/// # Example
///
/// ```rust,ignore
/// use sourdough_core::{PrimalIdentity, Did, Signature, PrimalError};
///
/// struct MyPrimal {
///     did: Did,
///     // ... BearDog client
/// }
///
/// #[async_trait::async_trait]
/// impl PrimalIdentity for MyPrimal {
///     fn did(&self) -> &Did {
///         &self.did
///     }
///
///     async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError> {
///         // Use BearDog to sign
///     }
///
///     async fn verify(&self, data: &[u8], signature: &Signature, signer: &Did)
///         -> Result<bool, PrimalError>
///     {
///         // Use BearDog to verify
///     }
/// }
/// ```
pub trait PrimalIdentity: Send + Sync {
    /// Get this primal's DID.
    fn did(&self) -> &Did;

    /// Sign data with this primal's identity.
    ///
    /// # Errors
    ///
    /// Returns an error if signing fails.
    fn sign(
        &self,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<Signature, PrimalError>> + Send;

    /// Verify a signature.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails (not if the signature is invalid).
    fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
        signer: &Did,
    ) -> impl std::future::Future<Output = Result<bool, PrimalError>> + Send;

    /// Get lineage proof (optional).
    ///
    /// Returns proof of this primal's lineage in the BearDog trust hierarchy.
    /// Not all primals need this.
    fn lineage_proof(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<LineageProof>, PrimalError>> + Send {
        async { Ok(None) }
    }
}

/// Proof of lineage in the BearDog trust hierarchy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageProof {
    /// The subject of this proof (this primal's DID).
    pub subject: Did,
    /// The parent in the lineage.
    pub parent: Option<Did>,
    /// Depth in the lineage tree (0 = genesis).
    pub depth: u32,
    /// Cryptographic proof.
    pub proof: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_parsing() {
        let did = Did::new("did:key:z6MkTest123");
        assert!(did.is_key_did());
        assert!(!did.is_web_did());
    }
}

