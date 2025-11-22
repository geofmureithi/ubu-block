use std::path::PathBuf;
mod init;
mod query;
mod validate;

use clap::{CommandFactory, Parser, Subcommand};
use database::Database;
use sqlx::SqlitePool;
use types::{Block, config::Config, merkle::MerkleTree};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate our blockchain
    Validate,
    /// Run a query on the blockchain
    Query {
        #[clap(short)]
        query: String,
    },
    /// Import an existing sqlite file
    Import { path: std::path::PathBuf },
    /// Initialize a new blockchain
    Init {
        /// Set the creator of the blockchain
        #[clap(long)]
        source: String,
    },

    /// Submit a block to a submission node
    Submit {
        node_addr: String,

        station_id: i64,
        candidate_id: i64,
        votes: i64,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config: Config = toml::from_str(
        &std::fs::read_to_string(cli.config.as_path()).expect("Failed to read config file"),
    )
    .expect("Failed to parse config file");

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Validate) => {
            validate::validate_blockchain(config).await;
        }
        Some(Commands::Query { query }) => {
            query::query_blockchain(config, query).await;
        }
        Some(Commands::Import { path }) => {
            println!("Importing from: {}", path.display());
        }
        Some(Commands::Init { source }) => {
            init::init_blockchain(config, source).await;
        }
        Some(Commands::Submit {
            node_addr,
            station_id,
            candidate_id,
            votes,
        }) => {
            log::debug!(
                "Submitting to node at {}: station_id={}, candidate_id={}, votes={}",
                node_addr,
                station_id,
                candidate_id,
                votes
            );

            let private_db = SqlitePool::connect(&config.private_db).await.unwrap();
            let main_db = SqlitePool::connect(&config.main_db).await.unwrap();

            let db = Database::new(main_db.clone(), private_db);

            let height = db.get_height().await.unwrap();

            let results = vec![types::CandidateResult {
                station_id: *station_id,
                candidate_id: *candidate_id,
                votes: *votes,
            }];
            let signer = db.get_private_key().await.unwrap();
            let prev_hash = db.get_block_by_height(height).await.unwrap().hash;

            let tree = MerkleTree::from_election_results_proper(&results);
            let root = tree.get_root_hash();

            let block = Block::new(
                &signer,
                &prev_hash,
                results,
                (height + 1) as usize,
                [0u8; 32],
            );
            let client = reqwest::Client::new();
            match client
                .post(&format!("{}/submit", node_addr))
                .json(&block)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Block submitted successfully");
                    } else {
                        eprintln!("Failed to submit block: {}", response.status());
                    }
                }
                Err(e) => eprintln!("Error submitting block: {}", e),
            }
        }
        None => {
            clap::Command::print_long_help(&mut Cli::command()).unwrap();
        }
    }

    // Continued program logic goes here...
}
