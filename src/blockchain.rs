use bincode::serialize;
use chrono::{DateTime, Utc};
use sha3::{Digest, Sha3_256 as Sha256};
use std::{
    collections::HashSet,
    time::{self, SystemTime},
};

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

const VERSION: usize = 1;

#[derive(Debug, Clone)]
pub struct Block {
    pub height: usize,
    pub timestamp: DateTime<Utc>,
    pub results: Vec<CandidateResult>,
    pub hash: String,
    pub hash_signature: String,
    pub prev_hash: String,
    pub prev_signature: String,
    pub creator: String,
    pub creator_pub_key: String,
    pub version: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultBlock {
    pub timestamp: DateTime<Utc>,
    pub results: Vec<CandidateResult>,
    pub prev_hash: String,
    pub prev_signature: String,
    pub creator: String,
    pub creator_pub_key: String,
    pub version: usize,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            height: Default::default(),
            timestamp: Utc::now(),
            results: Default::default(),
            hash: Default::default(),
            hash_signature: Default::default(),
            prev_hash: Default::default(),
            prev_signature: Default::default(),
            creator: Default::default(),
            creator_pub_key: Default::default(),
            version: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockChain {
    pub chain: Vec<Block>,
    pub nodes: HashSet<String>,
}

impl Default for BlockChain {
    fn default() -> Self {
        BlockChain {
            chain: vec![],
            nodes: HashSet::new(),
        }
    }
}

impl BlockChain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_block(&mut self, block: ResultBlock) -> Option<&Block> {
        let hash = Self::hash(&block);
        let block = Block {
            height: self.chain.len() + 1,
            timestamp: Utc::now(),
            results: block.results,
            prev_hash: block.prev_hash,
            hash,
            hash_signature: Default::default(),
            prev_signature: block.prev_signature,
            creator: block.creator,
            creator_pub_key: block.creator_pub_key,
            version: VERSION,
        };

        // append new block to chain
        self.chain.push(block);

        // return the last block
        self.chain.last()
    }

    pub fn last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn hash(block: &ResultBlock) -> String {
        let block_bytes = serialize(block).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(block_bytes);
        let hashed_block = format!("{:x}", hasher.finalize());
        hashed_block
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
        let new_block = ResultBlock {
            timestamp: Utc::now(),
            results: vec![result],
            prev_hash: "00000000000000000000000000000000000000000000000000".to_owned(),
            prev_signature: Default::default(),
            creator: Default::default(),
            creator_pub_key: Default::default(),
            version: 1,
        };
        let block = chain.new_block(new_block.clone()).unwrap();

        println!("CHAIN: {:?}", chain);
    }
}
