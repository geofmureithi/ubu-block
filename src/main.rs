mod blockchain;
mod crypto;
mod db;
use bincode::serialize;
use clap::Parser;
use sha3::{Digest, Sha3_256};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Pool, Row, SqlitePool};
use tabled::{Table, Tabled};
use tokio::runtime::Runtime;

use crate::blockchain::{Block, BlockChain, CandidateResult, ResultBlock};
use crate::crypto::{sha256_digest, sign_hash};
use crate::db::Database;

#[derive(Parser, Debug)]
#[clap(name = "ubu-block")]
#[clap(bin_name = "ubu-block")]
enum UbuBlock {
    Pull(Pull),
    /// Run a query on the blockchain
    Query {
        #[clap(short)]
        query: String,
    },
    /// Import an existing sqlite file
    Import {
        path: std::path::PathBuf,
    },
    /// Initialize a new blockchain
    Init,

    /// Insert a single block
    Insert {
        station: usize,
        candidate: usize,
        votes: usize,
    },
}

#[derive(clap::Args, Debug)]
#[clap(author, version, about, long_about = "Pull the blockchain from a url")]
struct Pull {
    #[clap(long)]
    manifest_path: Option<std::path::PathBuf>,
}

#[derive(sqlx::FromRow, Debug, Tabled)]
struct VoteResult {
    county: String,
    constituency: String,
    ward: String,
    candidate: String,
    party: String,
    votes: u32,
}

fn main() {
    use std::str::FromStr;
    let ubu = UbuBlock::parse();
    let rt = Runtime::new().expect("tokio runtime can be initialized");
    let main_db = SqliteConnectOptions::from_str("sqlite://data/ubu-block.db").unwrap();
    let private_db = SqliteConnectOptions::from_str("sqlite://data/private.db").unwrap();

    rt.block_on(async move {
        match ubu {
            UbuBlock::Pull(_) => todo!(),
            UbuBlock::Query { query } => {
                // You can only run select queries
                let conn = SqlitePool::connect_with(main_db.read_only(true))
                    .await
                    .unwrap();
                let mut pool = conn.acquire().await.unwrap();
                let res: Vec<VoteResult> = sqlx::query_as(&query)
                    .fetch_all(&mut pool)
                    .await
                    .expect("Could not query");
                println!("{}", Table::new(&res).to_string());
            }
            UbuBlock::Import { path: _ } => todo!(),
            UbuBlock::Init => {
                let main_db = SqlitePool::connect_with(main_db).await.unwrap();
                let private_db = SqlitePool::connect_with(private_db).await.unwrap();
                let mut tx = main_db.begin().await.unwrap();
                sqlx::query(db::MAIN_SETUP).execute(&mut tx).await.unwrap();
                tx.commit().await.unwrap();

                let mut conn = private_db.acquire().await.unwrap();
                sqlx::query(db::PRIV_SETUP)
                    .execute(&mut conn)
                    .await
                    .unwrap();

                let db = Database::new(main_db, private_db);

                // Create keypair
                let private_key = crate::crypto::get_private_key();
                let pub_key = private_key.verifying_key();
                let pub_key_hash = sha256_digest(&pub_key);
                let pub_key_bytes = serialize(&pub_key).unwrap();
                // Save keypair to db
                db.add_public_key(&pub_key_bytes, &pub_key_hash, -1)
                    .await
                    .unwrap();

                db.add_private_key(&pub_key_bytes, &pub_key_hash)
                    .await
                    .unwrap();

                let my_keys = db.get_my_public_key_hashes().await.unwrap();

                assert_eq!(my_keys.len(), 1);

                //assert_eq!(&my_keys.get(0).unwrap(), &pub_key_hash);

                let prev_hash = "1000000000000000000000000000000000000000000000000000000000000001";
                let prev_signature = sign_hash(&private_key, prev_hash);

                let mut blockchain = BlockChain::new(db);
                blockchain
                    .add_block(Block::genesis(
                        prev_hash,
                        &prev_signature,
                        "Geoff",
                        &pub_key_hash,
                    ))
                    .await
                    .unwrap();
            }
            UbuBlock::Insert {
                station,
                candidate,
                votes,
            } => {
                let main = SqlitePool::connect_with(main_db).await.unwrap();
                let private = SqlitePool::connect_with(private_db).await.unwrap();
                let mut chain = BlockChain::new(Database::new(main, private));
                let result = CandidateResult::new(station, candidate, votes);
                let block = Block::new("0000000000000000000000000000000000", vec![result]);
                chain
                    .add_block(block)
                    .await
                    .expect("Block could not be added to Chain");
            }
        };
    });
}
