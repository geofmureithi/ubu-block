use sqlx::{Row, sqlite::SqliteRow};

use bincode::deserialize;
use chrono::{DateTime, Utc};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::Verifier};
use types::{
    Block, ElectionBlockHeader, PubKey,
    models::{Constituency, County, Party, Station, Ward},
    results::{
        Candidate, ConstituencyResult, CountyResult, PositionResult, StationResult, WardResult,
    },
};

pub const PRIV_SETUP: &str = include_str!("../sql/private_db.sql");
pub const MAIN_SETUP: &str = include_str!("../sql/main_db.sql");

pub use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Database {
    pub chain_db: SqlitePool,
    pub private_db: SqlitePool,
}

impl Database {
    pub fn new(chain_db: SqlitePool, private_db: SqlitePool) -> Self {
        Self {
            chain_db,
            private_db,
        }
    }

    pub fn new_in_memory() -> Self {
        let chain_db = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        let private_db = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        Self {
            chain_db,
            private_db,
        }
    }
    pub async fn add_block(&mut self, block: &Block) -> Result<i64, sqlx::Error> {
        let mut tx = self.chain_db.begin().await.unwrap();
        let mut height = block.height as i64;

        // ignore genesis
        if height != 0 {
            height = self.get_height().await? as i64 + 1;
        } else {
            assert!(
                matches!(block.inner, types::BlockType::Genesis),
                "First block must be genesis"
            );
        }

        dbg!(&block);

        sqlx::query("INSERT INTO blockchain (hash, height, prev_hash, sigkey_hash, hash_signature, prev_hash_signature, timestamp, version, merkle_root) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);")
        .bind(&block.hash)
        .bind(height)
        .bind(&block.prev_hash)
        .bind(&block.signature_pub_key_hash)
        .bind(&block.hash_signature)
        .bind(&block.prev_hash_signature)
        .bind(block.timestamp)
        .bind(block.version as i64)
        .bind(&block.merkle_root[..])
        .execute(&mut *tx).await?.last_insert_rowid();

        let results = match &block.inner {
            types::BlockType::Result(results) => results,
            _ => &vec![],
        };

        println!("Adding results: {:?}", results);

        for result in results {
            let query = "INSERT INTO results VALUES(?1, ?2, ?3, ?4);";
            let _res = sqlx::query(query)
                .bind(result.station_id)
                .bind(result.candidate_id)
                .bind(result.votes)
                .bind(height)
                .execute(&mut *tx)
                .await?;
        }

        // self.is_valid().await?; // TODO: Handle invalid chains more gracefully
        tx.commit().await?;

        Ok(height)
    }

    pub async fn add_public_key(
        &self,
        pub_key: &[u8],
        creator: &str,
        pubkey_hash: &str,
        block_height: i32,
    ) -> Result<i64, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let sql = "INSERT INTO pubkeys(pubkey_hash, creator, pubkey, state, time_added, block_height) VALUES (?, ?, ?, ?, ?, ?)";
        let res = sqlx::query(sql)
            .bind(pubkey_hash)
            .bind(creator)
            .bind(hex::encode(pub_key))
            .bind("A")
            .bind(Utc::now().timestamp())
            .bind(block_height)
            .execute(&mut *pool)
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
            .execute(&mut *pool)
            .await?
            .last_insert_rowid();
        Ok(res)
    }

    pub async fn get_my_public_key_hashes(&self) -> Result<Vec<String>, sqlx::Error> {
        let mut pool = self.private_db.acquire().await?;
        let sql = "SELECT pubkey_hash FROM privkeys";
        let res: Vec<(String,)> = sqlx::query_as(sql).fetch_all(&mut *pool).await?;
        let hashes = res.into_iter().map(|r| r.0).collect();
        Ok(hashes)
    }

    async fn get_private_key_from_db(&self) -> Result<(String, String), sqlx::Error> {
        let sql = "SELECT privkey, pubkey_hash FROM privkeys LIMIT 1";
        let mut pool = self.private_db.acquire().await?;
        let res = sqlx::query_as(sql).fetch_one(&mut *pool).await?;
        Ok(res)
    }

    pub async fn get_private_key(&self) -> Result<(SigningKey, VerifyingKey, PubKey), sqlx::Error> {
        let (private_key, public_key_hash) = self.get_private_key_from_db().await?;
        let pub_key = self.get_public_key(&public_key_hash).await?;
        let private_key = SigningKey::from_slice(&hex::decode(&private_key).unwrap()).unwrap();

        let public_key = deserialize(&pub_key.bytes).unwrap();
        Ok((private_key, public_key, pub_key))
    }

    pub async fn get_public_key(&self, hash: &str) -> Result<PubKey, sqlx::Error> {
        let sql = "SELECT pubkey_hash, pubkey, state, time_added, COALESCE(time_revoked, -1), block_height, creator FROM pubkeys WHERE pubkey_hash = ?1";
        let mut pool = self.chain_db.acquire().await?;
        let res: (String, String, String, DateTime<Utc>, i64, i64, String) =
            sqlx::query_as(sql).bind(hash).fetch_one(&mut *pool).await?;
        // Create a NaiveDateTime from the timestamp
        let mut is_revoked = false;
        let mut time_revoked = None;

        if res.4 == -1 {
            let naive = DateTime::from_timestamp(res.4, 0)
                .expect("Failed to create NaiveDateTime")
                .naive_utc();
            time_revoked = Some(DateTime::from_naive_utc_and_offset(naive, Utc));
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
                .fetch_one(&mut *pool)
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
            .fetch_one(&mut *pool)
            .await?;
        let results = sqlx::query_as("Select * from results where block_height = ?1")
            .bind(height)
            .fetch_all(&mut *pool)
            .await?;
        let pub_key = self
            .get_public_key(&block.signature_pub_key_hash)
            .await
            .unwrap();
        block.set_results(results);
        block.set_pub_key(pub_key);
        Ok(block)
    }

    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Block, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let mut block: Block = sqlx::query_as("SELECT * FROM blockchain WHERE hash_signature = ?1")
            .bind(hash)
            .fetch_one(&mut *pool)
            .await?;
        let results = sqlx::query_as("Select * from results where block_height = ?1")
            .bind(block.height as i64)
            .fetch_all(&mut *pool)
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
            dbg!(index);
            let block = self.get_block_by_height(index).await?;

            let prev_hash = if index == 0 {
                [0u8; 32]
            } else {
                self.get_block_by_height(index - 1)
                    .await?
                    .hash
                    .as_bytes()
                    .try_into()
                    .unwrap()
            };

            // 1. Reconstruct the header using the stored merkle_root
            let header = ElectionBlockHeader {
                block_number: block.height as i64,
                merkle_root: block.merkle_root, // already stored, no recomputation
                previous_hash: prev_hash,
                validator_signature: block.signature_pub_key_hash.clone(),
                timestamp: block.timestamp.timestamp() as i64,
            };

            // 2. Re-hash header and compare
            let calculated_hash = types::crypto::hash_block(&header);
            if calculated_hash != block.hash {
                return Err(sqlx::Error::Protocol(
                    format!("Block hash mismatch at index {}", index).into(),
                ));
            }

            // 3. Verify signature
            let pub_key = self.get_public_key(&block.signature_pub_key_hash).await?;
            let verifier: VerifyingKey = deserialize(&pub_key.bytes).unwrap();
            let signature_bytes = hex::decode(&block.hash_signature).unwrap();
            let signature: Signature = deserialize(&signature_bytes).unwrap();
            verifier
                .verify(calculated_hash.as_bytes(), &signature)
                .unwrap();

            // 4. Verify chain linkage
            if block.prev_hash != hex::encode(prev_hash) {
                return Err(sqlx::Error::Protocol(
                    format!("Previous hash mismatch at index {}", index).into(),
                ));
            }
        }

        Ok(true)
    }

    // TODO: Optimize this to a single query
    pub async fn get_blocks_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<Block>, sqlx::Error> {
        dbg!(start, end);
        let mut blocks = Vec::new();
        let mut pool = self.chain_db.acquire().await?;
        let raw_blocks: Vec<Block> =
            sqlx::query_as("SELECT * FROM blockchain WHERE height >= ?1 AND height <= ?2")
                .bind(start as i64)
                .bind(end as i64)
                .fetch_all(&mut *pool)
                .await?;
        for mut block in raw_blocks {
            let results = sqlx::query_as("Select * from results where block_height = ?1")
                .bind(block.height as i64)
                .fetch_all(&mut *pool)
                .await?;
            let pub_key = self.get_public_key(&block.signature_pub_key_hash).await?;
            block.set_results(results);
            block.set_pub_key(pub_key);
            blocks.push(block);
        }
        Ok(blocks)
    }

    pub async fn positions(&self) -> Result<Vec<String>, sqlx::Error> {
        let results = sqlx::query("Select * from positions")
            .fetch_all(&self.chain_db)
            .await?
            .into_iter()
            .map(|s: SqliteRow| s.get_unchecked(0))
            .collect();

        Ok(results)
    }
    pub async fn parties(&self) -> Result<Vec<Party>, sqlx::Error> {
        let results = sqlx::query_as("Select * from parties")
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn counties(&self) -> Result<Vec<County>, sqlx::Error> {
        let results = sqlx::query_as("Select * from counties")
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }
    pub async fn constituencies(&self) -> Result<Vec<String>, sqlx::Error> {
        let results = sqlx::query("Select * from constituencies")
            .fetch_all(&self.chain_db)
            .await?
            .into_iter()
            .map(|s: SqliteRow| s.get_unchecked(0))
            .collect();

        Ok(results)
    }

    pub async fn constituencies_by_county(
        &self,
        county_id: &u32,
    ) -> Result<Vec<Constituency>, sqlx::Error> {
        let results = sqlx::query_as("SELECT * FROM constituencies WHERE county_code = ?")
            .bind(county_id)
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn wards_by_constituency(
        &self,
        constituency_code: &u32,
    ) -> Result<Vec<Ward>, sqlx::Error> {
        let results = sqlx::query_as("SELECT * FROM wards WHERE constituency_code = ?")
            .bind(constituency_code)
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn wards(&self) -> Result<Vec<String>, sqlx::Error> {
        let results = sqlx::query("Select * from wards")
            .fetch_all(&self.chain_db)
            .await?
            .into_iter()
            .map(|s: SqliteRow| s.get_unchecked(0))
            .collect();

        Ok(results)
    }

    pub async fn stations_by_ward(&self, ward_id: &u32) -> Result<Vec<Station>, sqlx::Error> {
        let results = sqlx::query_as("SELECT * FROM stations WHERE ward_code = ?")
            .bind(ward_id)
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn stations(&self) -> Result<Vec<String>, sqlx::Error> {
        let results = sqlx::query("Select * from stations LIMIT 100")
            .fetch_all(&self.chain_db)
            .await?
            .into_iter()
            .map(|s: SqliteRow| s.get_unchecked(3))
            .collect();

        Ok(results)
    }

    pub async fn results_by_station(
        &self,
        station_id: i64,
    ) -> Result<Vec<StationResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, StationResult>(
            "SELECT
                    s.id as station_id,
                    s.station_name,
                    s.ward_code,
                    w.ward_name,
                    c.id as candidate_id,
                    c.name as candidate_name,
                    p.title as party_title,
                    c.position_type,
                    r.votes,
                    s.registered_voters
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN wards w ON s.ward_code = w.ward_code
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE s.id = ?
                ORDER BY c.position_type, r.votes DESC",
        )
        .bind(station_id)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn results_by_ward(&self, ward_code: i32) -> Result<Vec<WardResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, WardResult>(
                "SELECT
                    w.ward_code,
                    w.ward_name,
                    w.constituency_code,
                    c.id as candidate_id,
                    c.name as candidate_name,
                    p.title as party_title,
                    c.position_type,
                    SUM(r.votes) as total_votes,
                    COUNT(DISTINCT s.id) as station_count
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN wards w ON s.ward_code = w.ward_code
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE w.ward_code = ?
                GROUP BY w.ward_code, w.ward_name, w.constituency_code, c.id, c.name, p.title, c.position_type
                ORDER BY c.position_type, total_votes DESC"
            )
            .bind(ward_code)
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn results_by_constituency(
        &self,
        constituency_code: i32,
    ) -> Result<Vec<ConstituencyResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, ConstituencyResult>(
                "SELECT
                    con.constituency_code,
                    con.constituency_name,
                    con.county_code,
                    c.id as candidate_id,
                    c.name as candidate_name,
                    p.title as party_title,
                    c.position_type,
                    SUM(r.votes) as total_votes,
                    COUNT(DISTINCT w.ward_code) as ward_count
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN wards w ON s.ward_code = w.ward_code
                JOIN constituencies con ON w.constituency_code = con.constituency_code
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE con.constituency_code = ?
                GROUP BY con.constituency_code, con.constituency_name, con.county_code, c.id, c.name, p.title, c.position_type
                ORDER BY c.position_type, total_votes DESC"
            )
            .bind(constituency_code)
            .fetch_all(&self.chain_db)
            .await?;

        Ok(results)
    }

    pub async fn results_by_county(
        &self,
        county_code: i32,
    ) -> Result<Vec<CountyResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, CountyResult>(
            "SELECT
                    co.county_code,
                    co.county_name,
                    c.id as candidate_id,
                    c.name as candidate_name,
                    p.title as party_title,
                    c.position_type,
                    SUM(r.votes) as total_votes,
                    COUNT(DISTINCT con.constituency_code) as constituency_count
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN wards w ON s.ward_code = w.ward_code
                JOIN constituencies con ON w.constituency_code = con.constituency_code
                JOIN counties co ON con.county_code = co.county_code
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE co.county_code = ?
                GROUP BY co.county_code, co.county_name, c.id, c.name, p.title, c.position_type
                ORDER BY c.position_type, total_votes DESC",
        )
        .bind(county_code)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn results_by_position(
        &self,
        position_type: &str,
    ) -> Result<Vec<PositionResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, PositionResult>(
            "SELECT
                    c.position_type,
                    c.id as candidate_id,
                    c.name as candidate_name,
                    p.title as party_title,
                    SUM(r.votes) as total_votes
                FROM results r
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE c.position_type = ?
                GROUP BY c.position_type, c.id, c.name, p.title
                ORDER BY total_votes DESC",
        )
        .bind(position_type)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn candidates_by_station(
        &self,
        station_id: i32,
    ) -> Result<Vec<Candidate>, sqlx::Error> {
        let results = sqlx::query_as::<_, Candidate>(
            r#"
            SELECT c.*
            FROM candidates c
            JOIN candidate_areas ca ON ca.candidate_id = c.id
            WHERE ca.area_type = 'station'
              AND ca.station_id = ?
            "#,
        )
        .bind(station_id)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn candidates_by_ward(&self, ward_code: &i32) -> Result<Vec<Candidate>, sqlx::Error> {
        let results = sqlx::query_as::<_, Candidate>(
            r#"
            SELECT c.*
            FROM candidates c
            JOIN stations st ON c.voting_station = st.id
            WHERE c.position_type = 'Mca'
              AND st.ward_code = ?
            "#,
        )
        .bind(ward_code)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn candidates_by_constituency(
        &self,
        constituency_code: &i32,
        position_type: &str,
    ) -> Result<Vec<Candidate>, sqlx::Error> {
        let results = sqlx::query_as::<_, Candidate>(
            r#"
            SELECT c.*
            FROM candidates c
            INNER JOIN stations st ON c.voting_station = st.id
            INNER JOIN wards w ON w.ward_code = st.ward_code
            INNER JOIN constituencies cts ON cts.constituency_code = w.constituency_code
            WHERE c.position_type = ?
              AND cts.constituency_code = ?
            "#,
        )
        .bind(position_type)
        .bind(constituency_code)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn candidates_by_county(
        &self,
        county_code: &i32,
        position_type: &str,
    ) -> Result<Vec<Candidate>, sqlx::Error> {
        let results = sqlx::query_as::<_, Candidate>(
            r#"
            SELECT c.*
            FROM candidates c
            INNER JOIN stations st ON c.voting_station = st.id
            INNER JOIN wards w ON w.ward_code = st.ward_code
            INNER JOIN constituencies cts ON cts.constituency_code = ward.constituency_code
            INNER JOIN counties ct ON ct.county_code = cts.county_code
            WHERE c.position_type = ?
              AND cts.constituency_code = ?
            "#,
        )
        .bind(position_type)
        .bind(county_code)
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }

    pub async fn candidates_national(&self) -> Result<Vec<Candidate>, sqlx::Error> {
        let results = sqlx::query_as::<_, Candidate>(
            r#"
            SELECT c.*
            FROM candidates c
            WHERE c.position_type = 'President'
            "#,
        )
        .fetch_all(&self.chain_db)
        .await?;

        Ok(results)
    }
}
