//! Streaming SHA-256 of a disc file, so a build fingerprint can be checked before trusting
//! fixed locations. Which file to hash and the expected digest must come from an approved
//! spec (request R-0001); nothing here assumes either.
use crate::Disc;
use sha2::{Digest, Sha256};

const CHUNK: usize = 1024 * 1024;

pub type Sha256Digest = [u8; 32];

pub fn sha256_file(disc: &mut Disc, name: &str) -> Result<Sha256Digest, String> {
    let size = disc.size(name)?;
    let mut hasher = Sha256::new();
    disc.stream_range(name, 0, size, CHUNK, |bytes| {
        hasher.update(bytes);
        Ok(())
    })?;
    Ok(hasher.finalize().into())
}

pub fn verify_sha256(disc: &mut Disc, name: &str, expected: &Sha256Digest) -> Result<(), String> {
    let actual = sha256_file(disc, name)?;
    if &actual != expected {
        return Err(format!(
            "{name} fingerprint mismatch: expected {}, found {}",
            to_hex(expected),
            to_hex(&actual)
        ));
    }
    Ok(())
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Accepts the 64 hex digits a spec would print; case-insensitive, no separators.
pub fn parse_sha256(hex: &str) -> Result<Sha256Digest, String> {
    let hex = hex.trim();
    // from_str_radix alone would also accept a leading '+'.
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("A SHA-256 digest is 64 hex digits".into());
    }
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| "A SHA-256 digest is 64 hex digits")?;
    }
    Ok(out)
}
