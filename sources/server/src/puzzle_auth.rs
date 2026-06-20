use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use pow_puzzle::{PublicKey, PuzzleError, verify_static};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid base64 public_key")]
    InvalidBase64,
    #[error("invalid public key: {0}")]
    InvalidPublicKey(#[from] pow_puzzle::PublicKeyParseError),
    #[error(transparent)]
    Puzzle(#[from] PuzzleError),
}

pub fn decode_public_key_b64(b64: &str) -> Result<PublicKey, AuthError> {
    let raw = B64
        .decode(b64.trim().as_bytes())
        .map_err(|_| AuthError::InvalidBase64)?;
    Ok(PublicKey::try_from(raw.as_slice())?)
}

pub fn verify_registration_public_key(
    b64: &str,
    min_difficulty_bits: u32,
) -> Result<PublicKey, AuthError> {
    let pk = decode_public_key_b64(b64)?;
    verify_static(&pk, min_difficulty_bits)?;
    Ok(pk)
}
