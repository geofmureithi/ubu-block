pub mod config;
pub mod crypto;
pub mod error;
pub mod p2p;

use bincode::deserialize;
use chrono::{DateTime, Utc};
use p256::ecdsa::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256 as Sha256};
use sqlx::{FromRow, sqlite::SqliteRow};

use crate::crypto::{sha256_digest, sign_hash};
pub const VERSION: usize = 1;

pub type BlockSigner = (SigningKey, VerifyingKey, PubKey);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub hash: String,
    pub hash_signature: String,
    pub inner: BlockType,
    pub height: usize,
    pub signature_pub_key_hash: String,
    pub timestamp: DateTime<Utc>,
    pub prev_hash: String,
    pub prev_hash_signature: String,
    pub creator: String,
    pub creator_pub_key: String,
    pub version: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BlockType {
    Pending,
    Genesis,
    Result(Vec<CandidateResult>),
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

        let hash = crate::crypto::hash_block(&ElectionBlockHeader {
            previous_hash: hex::decode(prev_hash)
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            // TODO: merkle root of results
            timestamp: Utc::now().timestamp() as u64,
            block_number: height as u64,
            validator_signature: sigkey_hash.clone(),
        });
        let hash_signature = sign_hash(&signer.0, &hash);
        Self {
            hash,
            hash_signature,
            inner: BlockType::Result(results),
            height,
            timestamp: Utc::now(),
            prev_hash: prev_hash.to_string(),
            signature_pub_key_hash: sigkey_hash.to_string(),
            prev_hash_signature,
            version: VERSION,
            creator: creator.to_string(),
            creator_pub_key: sha256_digest(&signer.1),
        }
    }

    pub fn set_results(&mut self, results: Vec<CandidateResult>) {
        if let BlockType::Result(ref mut rb) = self.inner {
            *rb = results;
        } else {
            panic!("Cannot set results on a genesis block");
        }
    }

    pub fn set_pub_key(&mut self, pub_key: PubKey) {
        self.creator = pub_key.creator;
        let public_key: VerifyingKey = deserialize(&pub_key.bytes).unwrap();
        self.creator_pub_key = sha256_digest(&public_key);
    }

    pub fn genesis(signer: &BlockSigner, init_query: String) -> Self {
        let prev_hash = "1000000000000000000000000000000000000000000000000000000000000001";
        let hash = crate::crypto::hash_block(&ElectionBlockHeader {
            previous_hash: hex::decode(prev_hash)
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            // merkle_root: [0u8; 32],
            timestamp: Utc::now().timestamp() as u64,
            block_number: 0,
            validator_signature: sha256_digest(&signer.1),
        });
        let hash_signature = sign_hash(&signer.0, &hash);
        let prev_hash_signature = sign_hash(&signer.0, &init_query);
        let sigkey_hash = sha256_digest(&signer.1);
        Self {
            prev_hash: prev_hash.to_string(),
            hash_signature,
            inner: BlockType::Genesis,
            height: 0,
            signature_pub_key_hash: sigkey_hash,
            timestamp: Utc::now(),
            hash,
            prev_hash_signature,
            creator: "GENESIS".to_string(),
            creator_pub_key: "".to_string(), // No creator for genesis block?
            version: VERSION,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PubKey {
    pub hash: String,
    pub creator: String,
    pub bytes: Vec<u8>,
    pub state: String,
    pub time_added: DateTime<Utc>,
    pub is_revoked: bool,
    pub time_revoked: Option<DateTime<Utc>>,
    pub add_block_height: usize,
    // metadata:
}

impl PubKey {
    pub fn new_dummy() -> Self {
        Self {
            hash: "1000000000000000000000000000000000000000000000000000000000000001".to_string(),
            bytes: vec![],
            state: "".to_string(),
            time_added: Utc::now(),
            is_revoked: false,
            time_revoked: None,
            add_block_height: 0,
            creator: "test-node-1".to_string(),
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Block {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let hash = row.try_get("hash")?;
        let hash_signature = row.try_get("hash_signature")?;
        let height: i64 = row.try_get("height")?;
        let sigkey_hash = row.try_get("sigkey_hash")?;
        let prev_hash = row.try_get("prev_hash")?;
        let prev_hash_signature = row.try_get("prev_hash_signature")?;
        let version: i64 = row.try_get("version")?;
        let timestamp = row.try_get("timestamp")?;

        if height == 0 {
            return Ok(Block {
                hash,
                hash_signature,
                inner: BlockType::Genesis,
                height: height as usize,
                signature_pub_key_hash: sigkey_hash,
                timestamp,
                prev_hash,
                prev_hash_signature,
                creator: Default::default(),
                creator_pub_key: Default::default(),
                version: version.try_into().unwrap(),
            });
        }

        Ok(Block {
            hash,
            hash_signature,
            inner: BlockType::Pending,
            height: height as usize,
            signature_pub_key_hash: sigkey_hash,
            timestamp,
            prev_hash,
            prev_hash_signature,
            creator: Default::default(),
            creator_pub_key: Default::default(),
            version: version.try_into().unwrap(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionBlockHeader {
    pub previous_hash: [u8; 32],
    // pub merkle_root: [u8; 32],        // Hash of all election results
    pub timestamp: u64,
    pub block_number: u64,
    pub validator_signature: String, // Instead of PoW nonce
}

impl ElectionBlockHeader {
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }
}
