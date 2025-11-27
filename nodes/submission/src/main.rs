use api::ui_handler;
use clap::Parser;
use std::path::PathBuf;

use axum::{Extension, Router};
use blockchain::BlockChain;
use database::{Database, SqlitePool};
use types::config::Config;

// cargo run --peers=1,2,3

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize the logger
    log::info!("Starting an submission node at port 9090...");
    log::info!("Starting an http server at port 9091...");

    let cli = Cli::parse();

    let config: Config = toml::from_str(
        &std::fs::read_to_string(cli.config.as_path()).expect("Failed to read config file"),
    )
    .expect("Failed to parse config file");

    let bind_addr = config.node_addr.unwrap().parse().unwrap();
    let blockchain = BlockChain::new(
        Database::new(
            SqlitePool::connect(&config.main_db).await.unwrap(),
            SqlitePool::connect(&config.private_db).await.unwrap(),
        ),
        config.peer_config,
    );
    let listener = tokio::net::TcpListener::bind(config.http_addr.unwrap())
        .await
        .unwrap();
    // Start the node and connect to a peer
    let node = blockchain.start_p2p_server(bind_addr);

    if let Some(peers) = config.peers {
        for peer in peers {
            let blockchain = blockchain.clone();
            tokio::spawn(async move {
                blockchain
                    .connect_to_peer(peer.1.parse().unwrap())
                    .await
                    .unwrap();
            });
        }
    }

    let api_routes = api::run_api_server().layer(Extension(blockchain.clone()));
    let server = axum::serve(
        listener,
        Router::new()
            .nest("/api/v1", api_routes)
            .fallback_service(ui_handler()),
    );

    let res = tokio::join!(node, server);
    match res {
        (Ok(_), Ok(_)) => todo!(),
        (Ok(_), Err(_)) => todo!(),
        (Err(_), Ok(_)) => todo!(),
        (Err(_), Err(_)) => todo!(),
    }
}
