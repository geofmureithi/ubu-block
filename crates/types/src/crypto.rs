use bincode::serialize;
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use serde::Serialize;

use sha3::{Digest, Sha3_256 as Sha256};

use crate::ElectionBlockHeader;

pub fn get_private_key() -> SigningKey {
    SigningKey::random(&mut OsRng)
}

pub fn sign_hash(key: &SigningKey, hash: &str) -> String {
    let hash = hex::decode(hash).unwrap();
    let signature: Signature = key.sign(&hash);
    hex::encode(serialize(&signature).unwrap())
}

pub fn hash_block(block: &ElectionBlockHeader) -> String {
    sha256_digest(block)
}

pub fn sha256_digest<T: Serialize>(data: &T) -> String {
    let block_bytes = serialize(data).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(block_bytes);
    let hashed_block = format!("{:x}", hasher.finalize());
    hashed_block
}

pub fn get_public_key(key: &SigningKey) -> VerifyingKey {
    *key.verifying_key()
}
