//! Ed25519 detached signature support for genomeBin artifacts.
//!
//! Provides binary authenticity beyond BLAKE3 checksums by producing
//! detached `.sig` sidecar files using Ed25519 signatures. This is
//! pure Rust (zero C dependencies), aligned with the ecosystem roadmap:
//! BLAKE3 checksums → Ed25519 signatures → BearDog-derived keys.
//!
//! # Signing scheme
//!
//! Rather than signing raw file bytes (which would require loading
//! potentially large genomeBin artifacts into memory), we sign the
//! BLAKE3 hash of the file content. Verification re-hashes the file
//! and checks the signature against the hash.
//!
//! # Example
//!
//! ```rust,no_run
//! use sourdough_genomebin::signing::{generate_keypair, sign_file, verify_file};
//! use std::path::Path;
//!
//! # fn example() -> sourdough_genomebin::Result<()> {
//! let (signing_key, verifying_key) = generate_keypair();
//!
//! // Sign an artifact
//! let sig = sign_file(Path::new("myprimal.genome"), &signing_key)?;
//!
//! // Verify it
//! assert!(verify_file(Path::new("myprimal.genome"), &sig, &verifying_key)?);
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;

use crate::error::{GenomeBinError, Result};

/// Generate a new Ed25519 signing keypair using OS randomness.
#[must_use]
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign a file by computing its BLAKE3 hash and signing that.
///
/// Returns the Ed25519 signature over the 32-byte BLAKE3 digest.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn sign_file(path: &Path, key: &SigningKey) -> Result<Signature> {
    let hash = blake3_hash_file(path)?;
    Ok(key.sign(hash.as_bytes()))
}

/// Verify an Ed25519 signature against a file's BLAKE3 hash.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn verify_file(path: &Path, signature: &Signature, key: &VerifyingKey) -> Result<bool> {
    let hash = blake3_hash_file(path)?;
    Ok(key.verify(hash.as_bytes(), signature).is_ok())
}

/// Write a detached signature to a `.sig` sidecar file.
///
/// The sidecar is placed at `{original_path}.sig` and contains
/// the raw 64-byte Ed25519 signature encoded as hex.
///
/// # Errors
///
/// Returns an error if the sidecar file cannot be written.
pub fn write_signature(signature: &Signature, artifact_path: &Path) -> Result<()> {
    let sig_path = signature_path_for(artifact_path);
    let hex = hex_encode(signature.to_bytes());
    std::fs::write(&sig_path, format!("{hex}\n"))?;
    Ok(())
}

/// Read a detached signature from a `.sig` sidecar file.
///
/// # Errors
///
/// Returns an error if the sidecar doesn't exist or contains invalid data.
pub fn read_signature(artifact_path: &Path) -> Result<Signature> {
    let sig_path = signature_path_for(artifact_path);
    let content = std::fs::read_to_string(&sig_path)?;
    let bytes = hex_decode(content.trim())?;
    Signature::from_slice(&bytes)
        .map_err(|e| GenomeBinError::Validation(format!("invalid Ed25519 signature: {e}")))
}

/// Write a verifying (public) key to a file as hex.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn write_verifying_key(key: &VerifyingKey, path: &Path) -> Result<()> {
    let hex = hex_encode(key.to_bytes());
    std::fs::write(path, format!("{hex}\n"))?;
    Ok(())
}

/// Read a verifying (public) key from a hex-encoded file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the key is invalid.
pub fn read_verifying_key(path: &Path) -> Result<VerifyingKey> {
    let content = std::fs::read_to_string(path)?;
    let bytes = hex_decode(content.trim())?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| GenomeBinError::Validation("verifying key must be 32 bytes".into()))?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|e| GenomeBinError::Validation(format!("invalid Ed25519 verifying key: {e}")))
}

/// Derive the `.sig` sidecar path for a given artifact.
#[must_use]
pub fn signature_path_for(artifact_path: &Path) -> std::path::PathBuf {
    let mut sig = artifact_path.as_os_str().to_owned();
    sig.push(".sig");
    std::path::PathBuf::from(sig)
}

fn blake3_hash_file(path: &Path) -> Result<blake3::Hash> {
    let content = std::fs::read(path)?;
    Ok(blake3::hash(&content))
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().fold(
        String::with_capacity(bytes.as_ref().len() * 2),
        |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        },
    )
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err(GenomeBinError::Validation(
            "hex string has odd length".into(),
        ));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| GenomeBinError::Validation(format!("invalid hex: {e}")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn keypair_generation_produces_valid_pair() {
        let (signing, verifying) = generate_keypair();
        assert_eq!(signing.verifying_key(), verifying);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (signing_key, verifying_key) = generate_keypair();
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, b"test artifact content").unwrap();

        let sig = sign_file(file.path(), &signing_key).unwrap();
        assert!(verify_file(file.path(), &sig, &verifying_key).unwrap());
    }

    #[test]
    fn verification_fails_with_wrong_key() {
        let (signing_key, _) = generate_keypair();
        let (_, wrong_verifying) = generate_keypair();
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, b"content").unwrap();

        let sig = sign_file(file.path(), &signing_key).unwrap();
        assert!(!verify_file(file.path(), &sig, &wrong_verifying).unwrap());
    }

    #[test]
    fn verification_fails_with_tampered_content() {
        let (signing_key, verifying_key) = generate_keypair();
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"original").unwrap();

        let sig = sign_file(file.path(), &signing_key).unwrap();
        std::fs::write(file.path(), b"tampered").unwrap();
        assert!(!verify_file(file.path(), &sig, &verifying_key).unwrap());
    }

    #[test]
    fn write_read_signature_roundtrip() {
        let (signing_key, _) = generate_keypair();
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, b"artifact").unwrap();

        let sig = sign_file(file.path(), &signing_key).unwrap();
        write_signature(&sig, file.path()).unwrap();

        let loaded = read_signature(file.path()).unwrap();
        assert_eq!(sig, loaded);
    }

    #[test]
    fn write_read_verifying_key_roundtrip() {
        let (_, verifying_key) = generate_keypair();
        let key_file = NamedTempFile::new().unwrap();

        write_verifying_key(&verifying_key, key_file.path()).unwrap();
        let loaded = read_verifying_key(key_file.path()).unwrap();
        assert_eq!(verifying_key, loaded);
    }

    #[test]
    fn signature_path_appends_sig_extension() {
        let path = Path::new("/tmp/myprimal.genome");
        assert_eq!(
            signature_path_for(path),
            std::path::PathBuf::from("/tmp/myprimal.genome.sig")
        );
    }

    #[test]
    fn hex_encode_decode_roundtrip() {
        let data = [0xde, 0xad, 0xbe, 0xef, 0x01, 0x23];
        let encoded = hex_encode(data);
        assert_eq!(encoded, "deadbeef0123");
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
