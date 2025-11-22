use std::os::unix::net::SocketAddr;

use axum::{Extension, Json, Router, routing::post};
use blockchain::BlockChain;
use database::{Database, SqlitePool};
use types::{Block, CandidateResult, p2p::P2PConfig};

#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize the logger
    log::info!("Starting an submission node at port 9090...");
    let bind_addr = "127.0.0.1:9090".parse().unwrap();
    let config = P2PConfig::default();
    let blockchain = BlockChain::new(
        Database::new(
            SqlitePool::connect("sqlite://../../data/blockchain.db")
                .await
                .unwrap(),
            SqlitePool::connect("sqlite://../../data/private.db")
                .await
                .unwrap(),
        ),
        Some(config),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9091")
        .await
        .unwrap();
    // Start the node and connect to a peer
    let node = blockchain.start_p2p_server(bind_addr);
    let server = axum::serve(
        listener,
        run_api_server().layer(Extension(blockchain.clone())),
    );

    let res = tokio::join!(node, server);
    match res {
        (Ok(_), Ok(_)) => todo!(),
        (Ok(_), Err(_)) => todo!(),
        (Err(_), Ok(_)) => todo!(),
        (Err(_), Err(_)) => todo!(),
    }
}

async fn submit_result(mut blockchain: Extension<BlockChain>, result: Json<Block>) -> String {
    let block = blockchain.add_block(&result.0).await.unwrap();
    blockchain.announce_block(result.0).await.unwrap();
    format!("Block with index {} submitted successfully!", block)
}

fn run_api_server() -> Router {
    let router = Router::new().route("/submit", post(submit_result));
    router
}
