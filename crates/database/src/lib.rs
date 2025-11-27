use sqlx::{Row, sqlite::SqliteRow};

use bincode::deserialize;
use chrono::{DateTime, Utc};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::Verifier};
use types::{
    Block, ElectionBlockHeader, PubKey,
    models::{Constituency, County, Party, Station, Ward},
    results::{Candidate, GeneralResult, LastResultSummary},
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
    ) -> Result<Vec<GeneralResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, GeneralResult>(
            "WITH per_candidate AS (
                SELECT
                    c.id AS candidate_id,
                    c.name AS candidate_name,
                    COALESCE(p.title, 'Independent') AS party_title,
                    CAST(AVG(r.votes) AS INTEGER) AS votes,
                    AVG(r.votes * r.votes) AS sq_votes,
                    AVG(r.votes) AS avg_votes
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE s.id = ?
                GROUP BY c.id
            )
            SELECT
                candidate_id,
                candidate_name,
                party_title,
                votes,
                CAST(CASE
                    WHEN sq_votes - (avg_votes * avg_votes) > 0
                    THEN (sq_votes - (avg_votes * avg_votes))
                    ELSE 0
                END AS INTEGER) AS sd
            FROM per_candidate
            ORDER BY votes DESC;
            ",
        )
        .bind(station_id)
        .fetch_all(&self.chain_db)
        .await?;

        // Calculate square root in Rust
        let results: Vec<GeneralResult> = results
            .into_iter()
            .map(|mut r| {
                r.sd = (r.sd as f64).sqrt() as u32;
                r
            })
            .collect();

        Ok(results)
    }

    pub async fn results_by_ward(
        &self,
        ward_code: &i32,
    ) -> Result<Vec<GeneralResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, GeneralResult>(
                "WITH station_candidate_agg AS (
                    SELECT
                        s.id AS station_id,
                        c.id AS candidate_id,
                        AVG(r.votes) AS avg_votes,
                        AVG(r.votes * r.votes) AS avg_sq_votes
                    FROM results r
                    JOIN stations s ON r.station_id = s.id
                    JOIN candidates c ON r.candidate_id = c.id
                    JOIN wards w ON s.ward_code = w.ward_code
                    WHERE w.ward_code = ?
                    GROUP BY s.id, c.id
                ),
                candidate_summary AS (
                    SELECT
                        candidate_id,
                        SUM(avg_votes) AS votes,
                        AVG(avg_sq_votes) AS avg_sq_votes,
                        AVG(avg_votes) AS overall_avg_votes,
                        COUNT(*) AS station_count
                    FROM station_candidate_agg
                    GROUP BY candidate_id
                )
                SELECT
                    c.id AS candidate_id,
                    c.name AS candidate_name,
                    COALESCE(p.title, 'Independent') AS party_title,
                    CAST(candidate_summary.votes AS INTEGER) AS votes,
                    CAST(CASE
                        WHEN station_count > 1 AND (avg_sq_votes - (overall_avg_votes * overall_avg_votes)) > 0
                        THEN (avg_sq_votes - (overall_avg_votes * overall_avg_votes))
                        ELSE 0
                    END AS INTEGER) AS sd
                FROM candidate_summary
                JOIN candidates c ON c.id = candidate_summary.candidate_id
                LEFT JOIN parties p ON p.id = c.party_id
                ORDER BY candidate_summary.votes DESC;
                "
            )
            .bind(ward_code)
            .fetch_all(&self.chain_db)
            .await?;

        // Calculate square root in Rust
        let results: Vec<GeneralResult> = results
            .into_iter()
            .map(|mut r| {
                r.sd = (r.sd as f64).sqrt() as u32;
                r
            })
            .collect();

        Ok(results)
    }

    pub async fn results_by_constituency(
        &self,
        constituency_code: &i32,
        position_type: &str,
    ) -> Result<Vec<GeneralResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, GeneralResult>(
            "WITH station_candidate_agg AS (
                    SELECT
                        s.id AS station_id,
                        c.id AS candidate_id,
                        AVG(r.votes) AS avg_votes,
                        AVG(r.votes * r.votes) AS avg_sq_votes
                    FROM results r
                    JOIN stations s ON r.station_id = s.id
                    JOIN wards w ON s.ward_code = w.ward_code
                    JOIN constituencies con ON w.constituency_code = con.constituency_code
                    JOIN candidates c ON r.candidate_id = c.id
                    WHERE con.constituency_code = ? AND c.position_type = ?
                    GROUP BY s.id, c.id
                ),
                candidate_summary AS (
                    SELECT
                        candidate_id,
                        SUM(avg_votes) AS total_votes,
                        AVG(avg_sq_votes) AS avg_sq_votes,
                        AVG(avg_votes) AS overall_avg_votes,
                        COUNT(*) AS station_count
                    FROM station_candidate_agg
                    GROUP BY candidate_id
                )
                SELECT
                    c.id AS candidate_id,
                    c.name AS candidate_name,
                    COALESCE(p.title, 'Independent') AS party_title,
                    CAST(cs.total_votes AS INTEGER) AS votes,
                    CAST(CASE
                        WHEN station_count > 1 AND (avg_sq_votes - (overall_avg_votes * overall_avg_votes)) > 0
                        THEN (avg_sq_votes - (overall_avg_votes * overall_avg_votes))
                        ELSE 0
                    END AS INTEGER) AS sd
                FROM candidate_summary cs
                JOIN candidates c ON c.id = cs.candidate_id
                LEFT JOIN parties p ON p.id = c.party_id
                ORDER BY cs.total_votes DESC;
                ",
        )
        .bind(constituency_code)
        .bind(position_type)
        .fetch_all(&self.chain_db)
        .await?;

        // Calculate square root in Rust
        let results: Vec<GeneralResult> = results
            .into_iter()
            .map(|mut r| {
                r.sd = (r.sd as f64).sqrt() as u32;
                r
            })
            .collect();

        Ok(results)
    }

    pub async fn results_by_county(
        &self,
        county_code: &i32,
        position_type: &str,
    ) -> Result<Vec<GeneralResult>, sqlx::Error> {
        let results = sqlx::query_as::<_, GeneralResult>(
            "WITH station_candidate_agg AS (
                SELECT
                    s.id AS station_id,
                    c.id AS candidate_id,
                    AVG(r.votes) AS avg_votes,
                    AVG(r.votes * r.votes) AS avg_sq_votes
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN wards w ON s.ward_code = w.ward_code
                JOIN constituencies con ON w.constituency_code = con.constituency_code
                JOIN counties co ON con.county_code = co.county_code
                JOIN candidates c ON r.candidate_id = c.id
                WHERE co.county_code = ? AND c.position_type = ?
                GROUP BY s.id, c.id
            ),
            candidate_summary AS (
                SELECT
                    candidate_id,
                    SUM(avg_votes) AS total_votes,
                    AVG(avg_sq_votes) AS avg_sq_votes,
                    AVG(avg_votes) AS overall_avg_votes,
                    COUNT(*) AS station_count
                FROM station_candidate_agg
                GROUP BY candidate_id
            )
            SELECT
                c.id AS candidate_id,
                c.name AS candidate_name,
                COALESCE(p.title, 'Independent') AS party_title,
                CAST(cs.total_votes AS INTEGER) AS votes,
                CAST(CASE
                    WHEN station_count > 1 AND (avg_sq_votes - (overall_avg_votes * overall_avg_votes)) > 0
                    THEN (avg_sq_votes - (overall_avg_votes * overall_avg_votes))
                    ELSE 0
                END AS INTEGER) AS sd
            FROM candidate_summary cs
            JOIN candidates c ON c.id = cs.candidate_id
            LEFT JOIN parties p ON c.party_id = p.id
            ORDER BY cs.total_votes DESC;
            ",
        )
        .bind(county_code)
        .bind(position_type)
        .fetch_all(&self.chain_db)
        .await?;

        // Calculate square root in Rust
        let results: Vec<GeneralResult> = results
            .into_iter()
            .map(|mut r| {
                r.sd = (r.sd as f64).sqrt() as u32;
                r
            })
            .collect();

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
    pub async fn last_five_results(&self) -> Result<Vec<LastResultSummary>, sqlx::Error> {
        let mut results = sqlx::query_as::<_, LastResultSummary>(
            "WITH latest_stations AS (
                SELECT DISTINCT station_id
                FROM results
                ORDER BY station_id DESC
                LIMIT 5
            ),
            station_candidate_votes AS (
                SELECT
                    s.id AS station_id,
                    s.station_name,
                    c.id AS candidate_id,
                    c.name AS candidate_name,
                    COALESCE(p.title, 'Independent') AS party_title,
                    c.position_type,
                    r.votes
                FROM results r
                JOIN stations s ON r.station_id = s.id
                JOIN candidates c ON r.candidate_id = c.id
                LEFT JOIN parties p ON c.party_id = p.id
                WHERE s.id IN (SELECT station_id FROM latest_stations)
            ),
            station_candidate_agg AS (
                SELECT
                    station_id,
                    station_name,
                    candidate_id,
                    candidate_name,
                    party_title,
                    position_type,
                    AVG(votes) AS avg_votes,
                    AVG(votes * votes) AS avg_sq_votes
                FROM station_candidate_votes
                GROUP BY station_id, candidate_id
            ),
            station_totals AS (
                SELECT
                    station_id,
                    position_type,
                    SUM(avg_votes) AS total_votes
                FROM station_candidate_agg
                GROUP BY station_id, position_type
            ),
            station_sd AS (
                SELECT
                    station_id,
                    position_type,
                    CAST(CASE
                        WHEN COUNT(*) > 1 AND (AVG(avg_sq_votes) - AVG(avg_votes) * AVG(avg_votes)) > 0
                        THEN (AVG(avg_sq_votes) - AVG(avg_votes) * AVG(avg_votes))
                        ELSE 0
                    END AS INTEGER) AS sd_squared
                FROM station_candidate_agg
                GROUP BY station_id, position_type
            ),
            ranked_candidates AS (
                SELECT
                    sca.station_id,
                    sca.station_name,
                    sca.candidate_id,
                    sca.candidate_name,
                    sca.party_title,
                    sca.position_type,
                    CAST(sca.avg_votes AS INTEGER) AS votes,
                    (sca.avg_votes * 100.0 / st.total_votes) AS percentage,
                    ROW_NUMBER() OVER (PARTITION BY sca.station_id, sca.position_type ORDER BY sca.avg_votes DESC) AS rank
                FROM station_candidate_agg sca
                JOIN station_totals st ON sca.station_id = st.station_id AND sca.position_type = st.position_type
            ),
            final_results AS (
                SELECT
                    rc1.position_type,
                    rc1.station_id,
                    rc1.station_name,
                    rc1.candidate_id AS candidate1_id,
                    rc1.candidate_name AS candidate1_name,
                    rc1.party_title AS candidate1_party,
                    rc1.votes AS candidate1_votes,
                    rc1.percentage AS candidate1_percentage,
                    rc2.candidate_id AS candidate2_id,
                    rc2.candidate_name AS candidate2_name,
                    rc2.party_title AS candidate2_party,
                    rc2.votes AS candidate2_votes,
                    rc2.percentage AS candidate2_percentage,
                    sd.sd_squared AS sd
                FROM ranked_candidates rc1
                LEFT JOIN ranked_candidates rc2
                    ON rc1.station_id = rc2.station_id
                    AND rc1.position_type = rc2.position_type
                    AND rc2.rank = 2
                JOIN station_sd sd
                    ON rc1.station_id = sd.station_id
                    AND rc1.position_type = sd.position_type
                WHERE rc1.rank = 1
            )
            SELECT * FROM final_results
            ORDER BY station_id DESC, position_type;
            "
        )
        .fetch_all(&self.chain_db)
        .await?;

        // Calculate square root for SD in Rust
        for result in &mut results {
            result.sd = (result.sd as f64).sqrt() as u32;
        }

        Ok(results)
    }
}
