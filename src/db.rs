use core::panic;

use bincode::deserialize;
use chrono::{DateTime, NaiveDateTime, Utc};
use p256::ecdsa::{signature::Verifier, Signature, SigningKey, VerifyingKey};
use sqlx::{sqlite::SqliteRow, FromRow, Row, SqlitePool};

use crate::blockchain::{Block, ResultBlock};

pub const PRIV_SETUP: &str = include_str!("sql/private_db.sql");
pub const MAIN_SETUP: &str = include_str!("sql/main_db.sql");

#[derive(Debug, Clone)]
pub struct Database {
    pub chain_db: SqlitePool,
    pub private_db: SqlitePool,
}

pub const VERSION: usize = 1;

impl Database {
    pub fn new(chain_db: SqlitePool, private_db: SqlitePool) -> Self {
        Self {
            chain_db,
            private_db,
        }
    }
    pub async fn add_block(&mut self, block: Block) -> Result<Block, sqlx::Error> {
        let mut tx = self.chain_db.begin().await.unwrap();
        let height = self.get_height().await?;
        sqlx::query("INSERT INTO blockchain (hash, height, prev_hash, sigkey_hash, hash_signature, prev_hash_signature, timestamp, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);")
        .bind(&block.hash)
        .bind(height + 1)
        .bind(&block.prev_hash)
        .bind(&block.signature_pub_key_hash)
        .bind(&block.hash_signature)
        .bind(&block.prev_hash_signature)
        .bind(&block.timestamp)
        .bind(block.version as i64)
        .execute(&mut tx)
        .await?
        .last_insert_rowid();

        for result in &block.results {
            let query = "INSERT INTO results VALUES(?1, ?2, ?3, ?4);";
            let _res = sqlx::query(query)
                .bind(result.station_id as i64)
                .bind(result.candidate_id as i64)
                .bind(result.votes as i64)
                .bind(height + 1)
                .execute(&mut tx)
                .await?;
        }
        tx.commit().await?;

        Ok(block)
    }

    pub async fn add_public_key(
        &self,
        pub_key: &Vec<u8>,
        creator: &str,
        hash: &str,
        block_height: i32,
    ) -> Result<i64, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let sql = "INSERT INTO pubkeys(pubkey_hash, creator, pubkey, state, time_added, block_height) VALUES (?, ?, ?, ?, ?, ?)";
        let res = sqlx::query(sql)
            .bind(hash)
            .bind(creator)
            .bind(hex::encode(pub_key))
            .bind("A")
            .bind(Utc::now().timestamp())
            .bind(block_height)
            .execute(&mut pool)
            .await?
            .last_insert_rowid();
        Ok(res)
    }

    pub async fn add_private_key(
        &self,
        priv_key: &Vec<u8>,
        pub_key_hash: &str,
    ) -> Result<i64, sqlx::Error> {
        let mut pool = self.private_db.acquire().await?;
        let sql = "INSERT INTO privkeys(pubkey_hash, privkey, time_added) VALUES (?, ?, ?)";
        let res = sqlx::query(sql)
            .bind(pub_key_hash)
            .bind(hex::encode(priv_key))
            .bind(Utc::now().timestamp())
            .execute(&mut pool)
            .await?
            .last_insert_rowid();
        Ok(res)
    }

    pub async fn get_my_public_key_hashes(&self) -> Result<Vec<String>, sqlx::Error> {
        let mut pool = self.private_db.acquire().await?;
        let sql = "SELECT pubkey_hash FROM privkeys";
        let res: Vec<(String,)> = sqlx::query_as(sql).fetch_all(&mut pool).await?;
        let hashes = res.into_iter().map(|r| r.0).collect();
        Ok(hashes)
    }

    async fn get_private_key_from_db(&self) -> Result<(String, String), sqlx::Error> {
        let sql = "SELECT privkey, pubkey_hash FROM privkeys LIMIT 1";
        let mut pool = self.private_db.acquire().await?;
        let res = sqlx::query_as(sql).fetch_one(&mut pool).await?;
        Ok(res)
    }

    pub async fn get_private_key(&self) -> Result<(SigningKey, VerifyingKey, PubKey), sqlx::Error> {
        let (private_key, public_key_hash) = self.get_private_key_from_db().await?;
        let pub_key = self.get_public_key(&public_key_hash).await?;
        let private_key = SigningKey::from_bytes(&hex::decode(&private_key).unwrap()).unwrap();

        let public_key = deserialize(&pub_key.bytes).unwrap();
        Ok((private_key, public_key, pub_key))
    }

    pub async fn get_public_key(&self, hash: &str) -> Result<PubKey, sqlx::Error> {
        let sql = "SELECT pubkey_hash, pubkey, state, time_added, COALESCE(time_revoked, -1), block_height, creator FROM pubkeys WHERE pubkey_hash = ?1";
        let mut pool = self.chain_db.acquire().await?;
        let res: (String, String, String, DateTime<Utc>, i64, i64, String) =
            sqlx::query_as(sql).bind(hash).fetch_one(&mut pool).await?;
        // Create a NaiveDateTime from the timestamp
        let mut is_revoked = false;
        let mut time_revoked = None;

        if res.4 == -1 {
            let naive = NaiveDateTime::from_timestamp(res.4, 0);
            time_revoked = Some(DateTime::from_utc(naive, Utc));
            is_revoked = true;
        };

        Ok(PubKey {
            hash: res.0,
            bytes: hex::decode(res.1).unwrap(),
            state: res.2,
            time_added: res.3,
            is_revoked,
            time_revoked,
            add_block_height: res.5 as usize,
            creator: res.6,
        })
    }
    pub async fn get_height(&self) -> Result<i64, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let len: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(height), -1) as count FROM blockchain")
                .fetch_one(&mut pool)
                .await?;
        Ok(len.0)
    }

    pub async fn last_block(&self) -> Result<Block, sqlx::Error> {
        let height = self.get_height().await?;
        let block = self.get_block_by_height(height).await?;
        Ok(block)
    }

    pub async fn get_block_by_height(&self, height: i64) -> Result<Block, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let mut block: Block = sqlx::query_as("SELECT * FROM blockchain WHERE height = ?1")
            .bind(height)
            .fetch_one(&mut pool)
            .await?;
        let results = sqlx::query_as("Select * from results where block_height = ?1")
            .bind(height)
            .fetch_all(&mut pool)
            .await?;
        let pub_key = self
            .get_public_key(&block.signature_pub_key_hash)
            .await
            .unwrap();
        block.set_results(results);
        block.set_pub_key(pub_key);
        Ok(block)
    }

    pub async fn is_valid(&self) -> Result<bool, sqlx::Error> {
        let height = self.get_height().await?;
        for index in 0..height {
            let block = self.get_block_by_height(index).await?;
            let hash = &block.hash;
            let hashed = crate::crypto::hash_block(&block.inner);
            if hash != &hashed {
                panic!(
                    "Could not verify block, found {}, block: {:?}",
                    hashed, &block
                )
            }
            let pub_key = self
                .get_public_key(&block.inner.signature_pub_key_hash)
                .await?;
            let verifier: VerifyingKey = deserialize(&pub_key.bytes).unwrap();
            let hash_bytes = hex::decode(&hash).unwrap();
            let signature: Signature = deserialize(&hex::decode(&block.hash_signature).unwrap())
                .expect(&format!("Could not verify block: {}", index));
            verifier.verify(&hash_bytes, &signature).unwrap();
        }
        Ok(true)
    }
}

impl<'r> FromRow<'r, SqliteRow> for Block {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let hash = row.try_get("hash")?;
        let hash_signature = row.try_get("hash_signature")?;
        let height: i64 = row.try_get("height")?;
        let sigkey_hash = row.try_get("sigkey_hash")?;
        let prev_hash = row.try_get("prev_hash")?;
        let prev_hash_signature = row.try_get("prev_hash_signature")?;
        let version: i64 = row.try_get("version")?;
        let timestamp = row.try_get("timestamp")?;

        Ok(Block {
            hash,
            hash_signature,
            inner: ResultBlock {
                height: height as usize,
                signature_pub_key_hash: sigkey_hash,
                timestamp,
                results: vec![], // We still needd to fetch results from results table for block to be valid
                prev_hash,
                prev_hash_signature,
                creator: Default::default(),
                creator_pub_key: Default::default(),
                version: version.try_into().unwrap(),
            },
        })
    }
}

#[allow(dead_code)]
pub struct PubKey {
    hash: String,
    pub creator: String,
    pub bytes: Vec<u8>,
    state: String,
    time_added: DateTime<Utc>,
    is_revoked: bool,
    time_revoked: Option<DateTime<Utc>>,
    add_block_height: usize,
    // metadata:
}
