use blockchain::BlockChain;
use database::{Database, SqlitePool};
use types::p2p::P2PConfig;

#[tokio::main]
async fn main() {
    let peer_addr = "127.0.0.1:9090".parse().unwrap();
    let config = P2PConfig::default();
    let blockchain = BlockChain::new(
        Database::new(
            SqlitePool::connect("sqlite:main.db").await.unwrap(),
            SqlitePool::connect("sqlite:private.db").await.unwrap(),
        ),
        Some(config),
    );
    // Start the node and connect to a peer
    blockchain.connect_to_peer(peer_addr).await.unwrap();
}
