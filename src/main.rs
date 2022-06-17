mod blockchain;
mod crypto;
mod db;
use bincode::serialize;
use clap::Parser;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use tabled::{Table, Tabled};
use tokio::runtime::Runtime;

use crate::blockchain::{Block, BlockChain, BlockSigner, CandidateResult};
use crate::crypto::sha256_digest;
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
    Init {
        /// Set the creator of the blockchain
        #[clap(long)]
        creator: String,
    },

    /// Insert a single block
    Insert {
        #[clap(long)]
        station: usize,
        #[clap(long)]
        candidate: usize,
        #[clap(long)]
        votes: usize,
    },
    /// Validate our blockchain
    Validate,
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
    std::env::set_var("RUST_LOG", "debug,sqlx::query=error");
    env_logger::init();
    use std::str::FromStr;
    let ubu = UbuBlock::parse();
    let rt = Runtime::new().expect("tokio runtime can be initialized");
    let main_db = SqliteConnectOptions::from_str("sqlite://data/blockchain.db").unwrap();
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
            UbuBlock::Init { creator } => {
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
                let verify_key = private_key.verifying_key();
                let pub_key_hash = sha256_digest(&verify_key);
                let pub_key_bytes = serialize(&verify_key).unwrap();
                let priv_key_bytes = private_key.to_bytes().as_slice().to_vec();

                // Save keypair to db
                db.add_public_key(&pub_key_bytes, &creator, &pub_key_hash, -1)
                    .await
                    .unwrap();

                db.add_private_key(&priv_key_bytes, &pub_key_hash)
                    .await
                    .unwrap();

                let my_keys = db.get_my_public_key_hashes().await.unwrap();

                assert_eq!(my_keys.len(), 1);

                //assert_eq!(&my_keys.get(0).unwrap(), &pub_key_hash);
                let pub_key = db.get_public_key(&pub_key_hash).await.unwrap();
                let prev_hash = "1000000000000000000000000000000000000000000000000000000000000001";
                // let there be light
                let genesis_block =
                    Block::new(&(private_key, verify_key, pub_key), prev_hash, vec![], 0);
                let mut blockchain = BlockChain::new(db);

                blockchain.add_block(genesis_block).await.unwrap();
                assert!(blockchain.is_valid().await.unwrap());
                log::info!("Blockchain was successfully initialized!");
            }
            UbuBlock::Insert {
                station,
                candidate,
                votes,
            } => {
                let main = SqlitePool::connect_with(main_db).await.unwrap();
                let private = SqlitePool::connect_with(private_db).await.unwrap();
                let mut blockchain = BlockChain::new(Database::new(main, private));
                assert!(blockchain.is_valid().await.unwrap());

                let result = CandidateResult::new(station, candidate, votes);
                let prev_block = blockchain.last_block().await.unwrap();

                let prev_hash = &prev_block.hash;

                let signer: BlockSigner = blockchain.get_private_key().await.unwrap();
                let height = blockchain.get_height().await.unwrap();

                let block = Block::new(&signer, prev_hash, vec![result], (height + 1) as usize);
                blockchain
                    .add_block(block)
                    .await
                    .expect("Block could not be added to Chain");
                assert!(blockchain.is_valid().await.unwrap());
                log::info!("Block was added successfully!");
            }
            UbuBlock::Validate => {
                let main = SqlitePool::connect_with(main_db).await.unwrap();
                let private = SqlitePool::connect_with(private_db).await.unwrap();
                let blockchain = BlockChain::new(Database::new(main, private));
                assert!(blockchain.is_valid().await.unwrap());
                log::info!("Blockchain is valid!");
            }
        };
    });
}
