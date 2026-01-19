//! Identity traits for `BearDog` integration.
//!
//! Every primal needs an identity—a way to prove who it is and sign its actions.
//! This module provides the traits for integrating with `BearDog`'s identity system.

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
/// Implement this trait to integrate with `BearDog` for identity and signing.
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
    /// Returns proof of this primal's lineage in the `BearDog` trust hierarchy.
    /// Not all primals need this.
    fn lineage_proof(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<LineageProof>, PrimalError>> + Send {
        async { Ok(None) }
    }
}

/// Proof of lineage in the `BearDog` trust hierarchy.
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

    #[test]
    fn did_web() {
        let did = Did::new("did:web:example.com");
        assert!(did.is_web_did());
        assert!(!did.is_key_did());
    }

    #[test]
    fn did_other_method() {
        let did = Did::new("did:other:abc123");
        assert!(!did.is_key_did());
        assert!(!did.is_web_did());
    }

    #[test]
    fn did_as_str() {
        let did = Did::new("did:key:test");
        assert_eq!(did.as_str(), "did:key:test");
    }

    #[test]
    fn did_from_string() {
        let s = String::from("did:key:test");
        let did: Did = s.into();
        assert_eq!(did.as_str(), "did:key:test");
    }

    #[test]
    fn did_from_str() {
        let did: Did = "did:web:example.com".into();
        assert!(did.is_web_did());
    }

    #[test]
    fn did_display() {
        let did = Did::new("did:key:z6Mk");
        assert_eq!(format!("{did}"), "did:key:z6Mk");
    }

    #[test]
    fn did_debug() {
        let did = Did::new("did:key:z6Mk");
        let debug_str = format!("{did:?}");
        assert!(debug_str.contains("did:key:z6Mk"));
    }

    #[test]
    fn did_clone_and_equality() {
        let did1 = Did::new("did:key:test");
        let did2 = did1.clone();
        assert_eq!(did1, did2);
    }

    #[test]
    fn signature_creation() {
        let sig = Signature::new(
            vec![1, 2, 3, 4],
            "Ed25519",
            "key-123",
        );
        
        assert_eq!(sig.bytes, vec![1, 2, 3, 4]);
        assert_eq!(sig.algorithm, "Ed25519");
        assert_eq!(sig.key_id, "key-123");
    }

    #[test]
    fn signature_debug() {
        let sig = Signature::new(vec![1, 2, 3], "Ed25519", "key-1");
        let debug_str = format!("{sig:?}");
        
        assert!(debug_str.contains("Ed25519"));
        assert!(debug_str.contains("key-1"));
        assert!(debug_str.contains("3 bytes"));
    }

    #[test]
    fn signature_clone_and_equality() {
        let sig1 = Signature::new(vec![1, 2], "Ed25519", "key-1");
        let sig2 = sig1.clone();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn signature_serialization() {
        let sig = Signature::new(vec![1, 2, 3], "Ed25519", "key-123");
        let json = serde_json::to_string(&sig).unwrap();
        let parsed: Signature = serde_json::from_str(&json).unwrap();
        
        assert_eq!(sig.bytes, parsed.bytes);
        assert_eq!(sig.algorithm, parsed.algorithm);
        assert_eq!(sig.key_id, parsed.key_id);
    }

    #[test]
    fn lineage_proof_creation() {
        let proof = LineageProof {
            subject: Did::new("did:key:child"),
            parent: Some(Did::new("did:key:parent")),
            depth: 2,
            proof: vec![1, 2, 3, 4],
        };
        
        assert_eq!(proof.depth, 2);
        assert!(proof.parent.is_some());
    }

    #[test]
    fn lineage_proof_genesis() {
        let proof = LineageProof {
            subject: Did::new("did:key:genesis"),
            parent: None,
            depth: 0,
            proof: vec![],
        };
        
        assert_eq!(proof.depth, 0);
        assert!(proof.parent.is_none());
    }

    #[test]
    fn lineage_proof_serialization() {
        let proof = LineageProof {
            subject: Did::new("did:key:test"),
            parent: Some(Did::new("did:key:parent")),
            depth: 1,
            proof: vec![1, 2, 3],
        };
        
        let json = serde_json::to_string(&proof).unwrap();
        let parsed: LineageProof = serde_json::from_str(&json).unwrap();
        
        assert_eq!(proof.depth, parsed.depth);
    }

    // Mock implementation for testing trait
    struct MockIdentityPrimal {
        did: Did,
    }

    impl MockIdentityPrimal {
        fn new(did: impl Into<String>) -> Self {
            Self {
                did: Did::new(did),
            }
        }
    }

    impl PrimalIdentity for MockIdentityPrimal {
        fn did(&self) -> &Did {
            &self.did
        }

        async fn sign(&self, data: &[u8]) -> Result<Signature, PrimalError> {
            Ok(Signature::new(
                data.to_vec(),
                "Ed25519",
                "mock-key",
            ))
        }

        async fn verify(
            &self,
            data: &[u8],
            signature: &Signature,
            _signer: &Did,
        ) -> Result<bool, PrimalError> {
            Ok(signature.bytes == data)
        }

        async fn lineage_proof(&self) -> Result<Option<LineageProof>, PrimalError> {
            Ok(Some(LineageProof {
                subject: self.did.clone(),
                parent: None,
                depth: 0,
                proof: vec![],
            }))
        }
    }

    #[tokio::test]
    async fn trait_identity() {
        let primal = MockIdentityPrimal::new("did:key:test123");
        
        let did = primal.did();
        assert_eq!(did.as_str(), "did:key:test123");
    }

    #[tokio::test]
    async fn trait_sign() {
        let primal = MockIdentityPrimal::new("did:key:signer");
        
        let data = b"test message";
        let sig = primal.sign(data).await.unwrap();
        
        assert_eq!(sig.bytes, data.to_vec());
        assert_eq!(sig.algorithm, "Ed25519");
    }

    #[tokio::test]
    async fn trait_verify() {
        let primal = MockIdentityPrimal::new("did:key:verifier");
        let signer = Did::new("did:key:signer");
        
        let data = b"test message";
        let sig = Signature::new(data.to_vec(), "Ed25519", "key");
        
        let valid = primal.verify(data, &sig, &signer).await.unwrap();
        assert!(valid);
        
        let different_data = b"different";
        let invalid = primal.verify(different_data, &sig, &signer).await.unwrap();
        assert!(!invalid);
    }

    #[tokio::test]
    async fn trait_lineage_proof() {
        let primal = MockIdentityPrimal::new("did:key:test");
        
        let proof = primal.lineage_proof().await.unwrap();
        assert!(proof.is_some());
        
        let proof = proof.unwrap();
        assert_eq!(proof.depth, 0);
        assert!(proof.parent.is_none());
    }
}
