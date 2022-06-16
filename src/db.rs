use chrono::{DateTime, NaiveDateTime, Utc};
use p256::{
    ecdsa::{signature::Verifier, SigningKey, VerifyingKey},
    NistP256,
};
use sqlx::SqlitePool;

use crate::blockchain::{Block, ResultBlock};

pub const PRIV_SETUP: &str = r#"
CREATE TABLE privkeys (
	pubkey_hash		VARCHAR NOT NULL PRIMARY KEY,
	privkey			VARCHAR NOT NULL,
	time_added		INTEGER NOT NULL
);
"#;

pub const MAIN_SETUP: &str = r#" 
  CREATE TABLE "positions" (
    "title" TEXT PRIMARY KEY
  );
  
  CREATE TABLE "parties" (
    "id" int PRIMARY KEY,
    "title" TEXT ,
    "logo" TEXT 
  );
  
  CREATE TABLE "counties" (
    "county_code" int PRIMARY KEY,
    "county_name" TEXT 
  );
  
  CREATE TABLE "constituencies" (
    "constituency_code" int PRIMARY KEY,
    "county_code" int,
    "constituency_name" TEXT ,
    FOREIGN KEY ("county_code") REFERENCES "counties" ("county_code")
  );
  
  CREATE TABLE "wards" (
    "ward_code" int PRIMARY KEY,
    "constituency_code" int,
    "ward_name" TEXT ,
    FOREIGN KEY ("constituency_code") REFERENCES "constituencies" ("constituency_code")
  );
  
  CREATE TABLE "stations" (
    "id" int PRIMARY KEY,
    "ward_code" int,
    "reg_center_code" integer,
    "station_name" TEXT ,
    "registered_voters" integer,
    FOREIGN KEY ("ward_code") REFERENCES "wards" ("ward_code")
  );

  CREATE TABLE "candidates" (
    "id" int PRIMARY KEY,
    "title" TEXT ,
    "photo" TEXT ,
    "position_type" TEXT ,
    "party_id" int,
    "voting_station" int,
    FOREIGN KEY ("position_type") REFERENCES "positions" ("title"),
    FOREIGN KEY ("party_id") REFERENCES "parties" ("id"),
    FOREIGN KEY ("voting_station") REFERENCES "stations" ("id")
  );

  CREATE TABLE blockchain (
    height				INTEGER NOT NULL UNIQUE,
    sigkey_hash			VARCHAR NOT NULL,
    timestamp		INTEGER NOT NULL,
    hash				VARCHAR NOT NULL PRIMARY KEY,
    hash_signature		VARCHAR NOT NULL,
    prev_hash			VARCHAR NOT NULL,
    prev_hash_signature	VARCHAR NOT NULL,
    version				INTEGER NOT NULL
  );

  CREATE INDEX blockchain_sigkey_hash ON blockchain(sigkey_hash);

  CREATE TABLE pubkeys (
    pubkey_hash		VARCHAR NOT NULL PRIMARY KEY,
    pubkey			VARCHAR NOT NULL,
    state			CHAR NOT NULL,
    time_added		INTEGER NOT NULL,
    time_revoked	INTEGER,
    block_height	INTEGER NOT NULL,
    metadata		VARCHAR -- JSON
  );

  CREATE TABLE peers (
    address			VARCHAR NOT NULL PRIMARY KEY,	-- in the format "address:port", lowercase
    time_added		INTEGER NOT NULL, -- time last seen
    permanent		BOOLEAN NOT NULL DEFAULT 0
  );
  
  CREATE TABLE "results" (
    "station_id" int NOT NULL,
    "candidate_id" int NOT NULL,
    "votes" int NOT NULL,
    "block_height" int NOT NULL,
    FOREIGN KEY ("candidate_id") REFERENCES "candidates" ("id"),
    FOREIGN KEY ("station_id") REFERENCES "stations" ("id")
  );

  INSERT INTO positions Values ("Mca"), ("WomenRep"), ("Mp"), ("Senator"), ("Governor"), ("President");
  INSERT INTO parties Values (1, "ODM", ""), (2, "PNU", "");
  INSERT INTO counties VALUES (22, "Kiambu"), ( 45, "Kisii");
  INSERT INTO constituencies VALUES (113, 22, "Juja"), (261, 45, "Bonchari");
  INSERT INTO wards VALUES (563, 113, "Kalimoni"), (1301, 261, "Bomariba");
  INSERT INTO stations VALUES(022113056303301, 563, 33, "Athi Primary School", 533 );
  INSERT INTO stations VALUES(045261130100402, 1301, 4, "Igonga Primary School ", 685 );

  INSERT INTO candidates VALUES(1, "Mwas", "", "Mp", 1, 022113056303301), (2, "Omosh", "", "Mp", 2, 022113056303301);
  INSERT INTO candidates VALUES(3, "Adhis", "", "Mp", 1, 045261130100402), (4, "Juma", "", "Mp", 2, 045261130100402);
  
  "#;

// INSERT INTO blockchain (hash, height, prev_hash, sigkey_hash, hash_signature, prev_hash_signature, timestamp, version) VALUES (
//   '9a0ff19183d1525a36de803047de4b73eb72506be8c81296eb463476a5c2d9e2',
//   0,
//   1000000000000000000000000000000000000000000000000000000000000001,
//   '1:a3c07ef6cbee246f231a61ff36bbcd8e8563723e3703eb345ecdd933d7709ae2',
//   '30460221008b8b3b3cfee2493ef58f2f6a1f1768b564f4c9e9a341ad42912cbbcf5c3ec82f022100fbcdfd0258fa1a5b073d18f688c2fb3d8f9a7c59204c6777f2bbf1faeb1eb1ed',
//   '3046022100db037ae6cb3c6e37cbc8ec592ba7eed2e6d18e6a3caedc4e2e81581eb97acb67022100d46d8ed27b5d78a8509b1eb8549c9b6b8f1c0a134c0c7af23bb93ab8cc842e2d',
//   1655371828,
//   1);

// INSERT INTO results VALUES( 022113056303301, 1, 52, 1);
// INSERT INTO results VALUES( 022113056303301, 2, 99, 1);
// INSERT INTO results VALUES( 045261130100402, 3, 172, 1);
// INSERT INTO results VALUES( 022113056303301, 1, 56, 1);

// pub(crate) struct Database {
//     db: String,
// }

// // table positions
// enum Position {
//     Mca,
//     WomenRep,
//     Mp,
//     Senator,
//     Governor,
//     President,
// }

// // table stations
// struct PollingStation {
//     county: String,
//     county_code: String,
//     ward: String,
//     ward_code: String,
//     station_code: u64,
//     station_name: String,
//     constituency: String,
//     registered_voters: u32,
// }

// // table candidates
// struct Candidate {
//     name: String,
//     party: String,
//     role: Position,
// }

// // table results
// struct StationResult {
//     station: PollingStation,
//     candidate: Candidate,
//     votes: u64,
// }

#[derive(Debug, Clone)]
pub struct Database {
    pub chain_db: SqlitePool,
    pub private_db: SqlitePool,
}

const VERSION: usize = 1;

impl Database {
    pub fn new(chain_db: SqlitePool, private_db: SqlitePool) -> Self {
        Self {
            chain_db,
            private_db,
        }
    }
    pub async fn add_block(&mut self, block: Block) -> Result<Block, sqlx::Error> {
        let mut tx = self.chain_db.begin().await.unwrap();
        let len: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(height), -1) as count FROM blockchain")
                .fetch_one(&mut tx)
                .await
                .unwrap();

        let id = sqlx::query(
        r#"
        INSERT INTO blockchain (hash, height, prev_hash, sigkey_hash, hash_signature, prev_hash_signature, timestamp, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
        "#
    )
    .bind(&block.hash)
    .bind(len.0 + 1)
    .bind(&block.prev_hash)
    .bind(&block.sigkey_hash)
    .bind(&block.hash_signature)
    .bind(&block.prev_signature)
    .bind(&block.timestamp.timestamp())
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
                .bind(id)
                .execute(&mut tx)
                .await?;
        }

        tx.commit().await?;

        Ok(block)
    }

    pub async fn add_public_key(
        &self,
        pub_key: &Vec<u8>,
        hash: &str,
        block_height: i32,
    ) -> Result<i64, sqlx::Error> {
        let mut pool = self.chain_db.acquire().await?;
        let sql = "INSERT INTO pubkeys(pubkey_hash, pubkey, state, time_added, block_height) VALUES (?, ?, ?, ?, ?)";
        let res = sqlx::query(sql)
            .bind(hash)
            .bind(hex::encode(pub_key))
            .bind("A")
            .bind(Utc::now().timestamp())
            .bind(block_height)
            .execute(&mut pool)
            .await?
            .last_insert_rowid();
        Ok(res)
    }

    pub async fn add_private_key(&self, pub_key: &Vec<u8>, hash: &str) -> Result<i64, sqlx::Error> {
        let mut pool = self.private_db.acquire().await?;
        let sql = "INSERT INTO privkeys(pubkey_hash, privkey, time_added) VALUES (?, ?, ?)";
        let res = sqlx::query(sql)
            .bind(hash)
            .bind(hex::encode(pub_key))
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

    async fn get_private_key_from_db(&self) -> Result<(Vec<u8>, String), sqlx::Error> {
        let sql = "SELECT pubkey_hash, privkey FROM privkeys LIMIT 1";
        let mut pool = self.private_db.acquire().await?;
        let res = sqlx::query_as(sql).fetch_one(&mut pool).await?;
        Ok(res)
    }

    pub async fn get_private_key(&self) -> Result<(SigningKey, VerifyingKey), sqlx::Error> {
        let (private_key, public_key_hash) = self.get_private_key_from_db().await?;
        let pub_key = self.get_public_key(&&public_key_hash).await?;
        let private_key = SigningKey::from_bytes(&private_key).unwrap();

        let public_key = VerifyingKey::from_sec1_bytes(&pub_key.bytes).unwrap();
        Ok((private_key, public_key))
    }

    pub async fn get_public_key(&self, hash: &str) -> Result<PubKey, sqlx::Error> {
        let sql = "SELECT pubkey_hash, pubkey, state, time_added, COALESCE(time_revoked, -1), block_height FROM pubkeys WHERE pubkey_hash=?";
        let mut pool = self.chain_db.acquire().await?;
        let res: (String, String, String, DateTime<Utc>, i64, i64) =
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
        })
    }
    // pub fn last_block(&self) -> Option<&Block> {
    //     //self.chain_db.last()
    // }
}

pub struct PubKey {
    hash: String,
    bytes: Vec<u8>,
    state: String,
    time_added: DateTime<Utc>,
    is_revoked: bool,
    time_revoked: Option<DateTime<Utc>>,
    add_block_height: usize,
    // metadata:
}
