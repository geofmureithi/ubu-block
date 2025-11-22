use blockchain::BlockChain;

#[tokio::main]
// Example usage function
async fn main() {
    let db1 = Database::new_in_memory();
    let db2 = Database::new_in_memory();

    // Start first node on port 9000
    let node1 = BlockChain::new(db1, Some(Default::default()));
    let blockchain = node1.clone();
    tokio::spawn(async move {
        node1
            .start_p2p_server("127.0.0.1:9000".parse().unwrap())
            .await
    });

    // Start second node on port 9009
    let node2 = BlockChain::new(db2, Some(Default::default()));
    tokio::spawn(async move {
        node2
            .start_p2p_server("127.0.0.1:9009".parse().unwrap())
            .await
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // Connect to other peers
    let peer_addr = "127.0.0.1:9009".parse().unwrap();
    blockchain.connect_to_peer(peer_addr).await.unwrap();

    // Create and announce a new block
    let key = types::crypto::get_private_key();
    let pub_key = types::PubKey::new_dummy();
    let signer = (key.clone(), types::crypto::get_public_key(&key), pub_key);

    let results = vec![types::CandidateResult::new(1, 1, 100)];
    let block = Block::new(
        &signer,
        "1000000000000000000000000000000000000000000000000000000000000001",
        results,
        1,
    );

    blockchain.announce_block(block).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
