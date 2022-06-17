use bincode::deserialize;
use chrono::{DateTime, Utc};
use p256::ecdsa::{SigningKey, VerifyingKey};

use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::{
    crypto::{sha256_digest, sign_hash},
    db::{Database, PubKey, VERSION},
};

pub type BlockSigner = (SigningKey, VerifyingKey, PubKey);

#[derive(Debug, Clone)]
pub struct QueryBlockChain {
    pub chain: Vec<Block>,
    pub length: usize,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct CandidateResult {
    pub station_id: i64,
    pub candidate_id: i64,
    pub votes: i64,
}

impl CandidateResult {
    pub fn new(station_id: usize, candidate_id: usize, votes: usize) -> Self {
        Self {
            station_id: station_id as i64,
            candidate_id: candidate_id as i64,
            votes: votes as i64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub hash: String,
    pub hash_signature: String,
    pub inner: ResultBlock,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultBlock {
    pub height: usize,
    pub signature_pub_key_hash: String,
    pub timestamp: DateTime<Utc>,
    pub results: Vec<CandidateResult>,
    pub prev_hash: String,
    pub prev_hash_signature: String,
    pub creator: String,
    pub creator_pub_key: String,
    pub version: usize,
}

impl Block {
    pub fn new(
        signer: &BlockSigner,
        prev_hash: &str,
        results: Vec<CandidateResult>,
        height: usize,
    ) -> Self {
        let prev_hash_signature = sign_hash(&signer.0, prev_hash);
        let sigkey_hash = sha256_digest(&signer.1);
        let creator = &signer.2.creator;

        let inner = ResultBlock {
            height,
            timestamp: Utc::now(),
            results,
            prev_hash: prev_hash.to_string(),
            signature_pub_key_hash: sigkey_hash.to_string(),
            prev_hash_signature,
            version: VERSION,
            creator: creator.to_string(),
            creator_pub_key: sha256_digest(&signer.1),
        };
        let hash = crate::crypto::hash_block(&inner);
        let hash_signature = sign_hash(&signer.0, &hash);
        Self {
            hash,
            hash_signature,
            inner,
        }
    }

    pub(crate) fn set_results(&mut self, results: Vec<CandidateResult>) {
        self.inner.results = results;
    }

    pub(crate) fn set_pub_key(&mut self, pub_key: PubKey) {
        self.inner.creator = pub_key.creator;
        let public_key: VerifyingKey = deserialize(&pub_key.bytes).unwrap();
        self.inner.creator_pub_key = sha256_digest(&public_key);
    }
}

impl Deref for Block {
    type Target = ResultBlock;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct BlockChain {
    db: Database,
    #[allow(dead_code)]
    nodes: HashSet<String>,
}

impl Deref for BlockChain {
    type Target = Database;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for BlockChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl BlockChain {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            nodes: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::blockchain::ResultBlock;

    use super::{BlockChain, CandidateResult};

    #[test]
    fn test_blocks() {
        let mut chain = BlockChain::new();
        let result = CandidateResult {
            station_id: 1,
            candidate_id: 1,
            votes: 20,
        };
        let prev_hash = "00000000000000000000000000000000000000000000000000".to_owned();
        let key = crate::crypto::get_private_key(1);
        let prev_signature = crate::crypto::sign_hash(&key, &prev_hash);
        let new_block = ResultBlock {
            timestamp: Utc::now(),
            results: vec![result],
            prev_hash,
            prev_hash_signature: prev_signature,
            version: 1,
            height: 1,
            signature_pub_key_hash: Default::default(),
            creator: Default::default(),
            creator_pub_key: Default::default(),
        };
        let block = chain.add_block(new_block.clone()).unwrap();

        println!("CHAIN: {:?}", chain);
    }
}
