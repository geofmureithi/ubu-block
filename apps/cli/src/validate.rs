use blockchain::BlockChain;
use types::config::Config;

pub(crate) async fn validate_blockchain(config: Config) {
    let blockchain = BlockChain::from_config(config).await;
    assert!(blockchain.is_valid().await.unwrap());
    log::info!("Blockchain is valid!");
}
