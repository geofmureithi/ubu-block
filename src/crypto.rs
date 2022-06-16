use bincode::serialize;
use p256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    elliptic_curve::rand_core::OsRng,
};
use serde::Serialize;

use crate::blockchain::ResultBlock;
use sha3::{Digest, Sha3_256 as Sha256};

pub fn get_private_key() -> SigningKey {
    SigningKey::random(&mut OsRng)
}

pub fn sign_hash(key: &SigningKey, hash: &str) -> String {
    let hash = hex::decode(hash).unwrap();
    let signature: Signature = key.sign(&hash);
    hex::encode(signature.to_vec())
}

pub fn hash_block(block: &ResultBlock) -> String {
    sha256_digest(block)
}

pub fn sha256_digest<T: Serialize>(data: &T) -> String {
    let block_bytes = serialize(data).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(block_bytes);
    let hashed_block = format!("{:x}", hasher.finalize());
    hashed_block
}
