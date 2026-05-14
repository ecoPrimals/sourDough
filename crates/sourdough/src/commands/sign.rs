//! Ed25519 binary signing command.
//!
//! Signs any file with a detached Ed25519 signature (.sig sidecar).
//! This is the v0.3.0 interface per the deployment internalization contract.

use anyhow::{Context, Result};
use sourdough_genomebin::signing;
use std::path::{Path, PathBuf};

/// Sign a binary or artifact with Ed25519.
///
/// Produces a detached `.sig` sidecar file alongside the original.
pub(crate) fn run(path: &Path, key_path: Option<&Path>, generate_key: bool) -> Result<()> {
    let default_key = PathBuf::from("signing.key");
    let key_file = key_path.unwrap_or(&default_key);

    if generate_key {
        return generate_keypair(key_file);
    }

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    if !key_file.exists() {
        anyhow::bail!(
            "Signing key not found: {}\n\
             Run `sourdough sign --generate-key` to create a keypair, \
             or specify --key <path>",
            key_file.display()
        );
    }

    let signing_key = load_signing_key(key_file)?;
    let signature = signing::sign_file(path, &signing_key)
        .with_context(|| format!("Failed to sign {}", path.display()))?;

    signing::write_signature(&signature, path)
        .with_context(|| format!("Failed to write signature for {}", path.display()))?;

    let sig_path = signing::signature_path_for(path);
    crate::success(&format!("Signed: {}", path.display()));
    crate::info(&format!("Signature: {}", sig_path.display()));

    Ok(())
}

/// Verify a signed binary.
pub(crate) fn verify(path: &Path, pub_key_path: Option<&Path>) -> Result<()> {
    let default_pub = PathBuf::from("signing.pub");
    let pub_key_file = pub_key_path.unwrap_or(&default_pub);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    if !pub_key_file.exists() {
        anyhow::bail!(
            "Verifying key not found: {}\nSpecify --pub-key <path>",
            pub_key_file.display()
        );
    }

    let verifying_key = signing::read_verifying_key(pub_key_file)
        .with_context(|| format!("Failed to read key from {}", pub_key_file.display()))?;

    let signature = signing::read_signature(path)
        .with_context(|| format!("No .sig sidecar found for {}", path.display()))?;

    let valid = signing::verify_file(path, &signature, &verifying_key)
        .with_context(|| format!("Verification failed for {}", path.display()))?;

    if valid {
        crate::success(&format!("Verified: {}", path.display()));
        Ok(())
    } else {
        crate::error(&format!("INVALID signature: {}", path.display()));
        anyhow::bail!("Signature verification failed");
    }
}

fn generate_keypair(key_file: &Path) -> Result<()> {
    let pub_file = key_file.with_extension("pub");

    if key_file.exists() {
        anyhow::bail!(
            "Key already exists: {}\nRemove it first if you want to regenerate.",
            key_file.display()
        );
    }

    let (signing_key, verifying_key) = signing::generate_keypair();

    let key_bytes = signing_key.to_bytes();
    let hex: String = key_bytes.iter().fold(String::new(), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    });
    std::fs::write(key_file, format!("{hex}\n"))
        .with_context(|| format!("Failed to write signing key to {}", key_file.display()))?;

    signing::write_verifying_key(&verifying_key, &pub_file)
        .with_context(|| format!("Failed to write public key to {}", pub_file.display()))?;

    crate::success(&format!("Generated signing key: {}", key_file.display()));
    crate::success(&format!("Generated public key: {}", pub_file.display()));
    crate::info("Keep the signing key secure. Distribute the public key for verification.");

    Ok(())
}

fn load_signing_key(key_file: &Path) -> Result<ed25519_dalek::SigningKey> {
    let content = std::fs::read_to_string(key_file)
        .with_context(|| format!("Failed to read signing key from {}", key_file.display()))?;
    let hex = content.trim();
    let bytes = hex_decode(hex)?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Signing key must be 32 bytes (64 hex chars)"))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&arr))
}

fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        anyhow::bail!("Invalid hex: odd number of characters");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .with_context(|| format!("Invalid hex at position {i}"))
        })
        .collect()
}
