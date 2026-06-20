use crate::api::Credentials;
use pow_puzzle::PublicKey;

pub fn authorize_puzzle(
    credentials: &Credentials,
    min_difficulty_bits: u32,
) -> Result<PublicKey, ()> {
    crate::puzzle_auth::verify_registration_public_key(&credentials.public_key, min_difficulty_bits)
        .map_err(|_| ())
}
