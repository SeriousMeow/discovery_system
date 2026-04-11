use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub [u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StaticPuzzleSolution(pub [u8; 32]);

impl StaticPuzzleSolution {
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0)
    }
}

impl From<StaticPuzzleSolution> for PublicKey {
    fn from(s: StaticPuzzleSolution) -> Self {
        s.public_key()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PuzzleError {
    #[error("public key must be exactly 32 bytes")]
    InvalidPublicKeyLength,
    #[error(
        "static puzzle not satisfied: need at least {required} leading zero bits, got {achieved}"
    )]
    NotSatisfied { required: u32, achieved: u32 },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PublicKeyParseError {
    #[error("public key must be exactly 32 bytes")]
    InvalidLength,
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = PublicKeyParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let a: &[u8; 32] = value
            .try_into()
            .map_err(|_| PublicKeyParseError::InvalidLength)?;
        Ok(Self(*a))
    }
}

impl FromStr for PublicKey {
    type Err = PublicKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(
            hex::decode(s)
                .map_err(|_| PublicKeyParseError::InvalidLength)?
                .as_slice(),
        )
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PublicKey")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeId").field(&hex::encode(self.0)).finish()
    }
}

impl fmt::Debug for StaticPuzzleSolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StaticPuzzleSolution")
            .field(&hex::encode(self.0))
            .finish()
    }
}

#[must_use]
pub fn double_sha256(public_key: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(public_key);
    Sha256::digest(first).into()
}

#[must_use]
pub fn leading_zero_bits(digest: &[u8; 32]) -> u32 {
    let mut count = 0u32;
    for byte in digest {
        if *byte == 0 {
            count += 8;
        } else {
            count += u32::from(byte.leading_zeros());
            break;
        }
    }
    count
}

#[must_use]
pub fn derive_node_id(public_key: &PublicKey) -> NodeId {
    NodeId(Sha256::digest(public_key.0).into())
}

#[must_use]
pub fn node_id_string(node_id: &NodeId) -> String {
    hex::encode(node_id.0)
}

#[must_use]
pub fn public_key_hex_id(pk: &PublicKey) -> String {
    hex::encode(pk.0)
}

pub fn verify_static(public_key: &PublicKey, min_difficulty_bits: u32) -> Result<u32, PuzzleError> {
    let digest = double_sha256(&public_key.0);
    let achieved = leading_zero_bits(&digest);
    if achieved < min_difficulty_bits {
        Err(PuzzleError::NotSatisfied {
            required: min_difficulty_bits,
            achieved,
        })
    } else {
        Ok(achieved)
    }
}

pub fn generate_static_solution(
    min_difficulty_bits: u32,
) -> Result<(StaticPuzzleSolution, u32), PuzzleError> {
    use rand::RngCore;
    let mut rng = rand::rngs::OsRng;
    let mut buf = [0u8; 32];
    loop {
        rng.fill_bytes(&mut buf);
        let pk = PublicKey(buf);
        match verify_static(&pk, min_difficulty_bits) {
            Ok(achieved) => return Ok((StaticPuzzleSolution(buf), achieved)),
            Err(PuzzleError::NotSatisfied { .. }) => {}
            Err(e @ PuzzleError::InvalidPublicKeyLength) => return Err(e),
        }
    }
}
