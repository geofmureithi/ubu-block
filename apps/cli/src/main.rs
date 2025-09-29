use std::path::PathBuf;
mod init;
mod query;
mod validate;

use clap::{Parser, Subcommand};
use types::config::Config;

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
        None => {}
    }

    // Continued program logic goes here...
}
