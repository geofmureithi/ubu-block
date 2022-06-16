use chrono::{DateTime, Utc};

use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::db::Database;

#[derive(Debug, Clone)]
pub struct QueryBlockChain {
    pub chain: Vec<Block>,
    pub length: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CandidateResult {
    pub station_id: usize,
    pub candidate_id: usize,
    pub votes: usize,
}

impl CandidateResult {
    pub fn new(station_id: usize, candidate_id: usize, votes: usize) -> Self {
        Self {
            station_id,
            candidate_id,
            votes,
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
    pub sigkey_hash: String,
    pub timestamp: DateTime<Utc>,
    pub results: Vec<CandidateResult>,
    pub prev_hash: String,
    pub prev_signature: String,
    pub creator: String,
    pub creator_pub_key: String,
    pub version: usize,
}

impl Block {
    pub fn new(prev_hash: &str, results: Vec<CandidateResult>) -> Self {
        let inner = ResultBlock {
            height: 0,
            timestamp: Utc::now(),
            results,
            prev_hash: prev_hash.to_string(),
            sigkey_hash: Default::default(),
            prev_signature: Default::default(),

            version: Default::default(),
            creator: Default::default(),
            creator_pub_key: Default::default(),
        };
        Self {
            hash: crate::crypto::hash_block(&inner),
            hash_signature: Default::default(),
            inner,
        }
    }

    pub fn genesis(
        prev_hash: &str,
        prev_signature: &str,
        creator: &str,
        creator_pub_key: &str,
    ) -> Self {
        let hash_signature = hex::decode("30460221008b8b3b3cfee2493ef58f2f6a1f1768b564f4c9e9a341ad42912cbbcf5c3ec82f022100fbcdfd0258fa1a5b073d18f688c2fb3d8f9a7c59204c6777f2bbf1faeb1eb1ed".to_string()).unwrap();
        let inner = ResultBlock {
            height: 0,
            timestamp: Utc::now(),
            results: vec![],
            prev_hash: prev_hash.to_string(),
            sigkey_hash: creator_pub_key.to_string(),
            prev_signature: prev_signature.to_string(),
            version: 1,
            creator: creator.to_string(),
            creator_pub_key: creator_pub_key.to_string(),
        };
        Self {
            hash: crate::crypto::hash_block(&inner),
            hash_signature: hex::encode(hash_signature),
            inner,
        }
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

    pub fn valid_chain<T: AsRef<[Block]>>(&self, chain: T) -> bool {
        let mut block_peek = chain.as_ref().iter().peekable();
        while let Some(block) = block_peek.next() {
            let next_block = block_peek.peek();
            if next_block.is_none() {
                break;
            }
            // caculate the hash value of each block, and valid it with the current nodes'
            // let to_be_verified = Self::hash(&block);

            // if to_be_verified
            //     .map(|v| next_block.unwrap().prev_hash.ne(&v))
            //     .is_err()
            // {
            //     return false;
            // }

            // if !Self::valid_proof(block.proof, next_block.unwrap().proof) {
            //     return false;
            // }
        }

        true
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
            prev_signature,
            version: 1,
            height: 1,
            sigkey_hash: Default::default(),
            creator: Default::default(),
            creator_pub_key: Default::default(),
        };
        let block = chain.add_block(new_block.clone()).unwrap();

        println!("CHAIN: {:?}", chain);
    }
}
