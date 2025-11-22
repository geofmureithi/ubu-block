use bincode::serialize;
use blockchain::BlockChain;
use database::Database;
use sqlx::SqlitePool;
use types::{
    Block,
    config::Config,
    crypto::{self, sha256_digest},
};

pub async fn init_blockchain(config: Config, init_query_path: &str) {
    let main_db = SqlitePool::connect(&config.main_db).await.unwrap();
    let private_db = SqlitePool::connect(&config.private_db).await.unwrap();
    let mut tx = main_db.begin().await.unwrap();
    sqlx::query(database::MAIN_SETUP)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let mut conn = private_db.acquire().await.unwrap();
    sqlx::query(database::PRIV_SETUP)
        .execute(&mut *conn)
        .await
        .unwrap();

    let db = Database::new(main_db.clone(), private_db);

    // Create keypair
    let private_key = crypto::get_private_key();
    let verify_key = *private_key.verifying_key();
    let pub_key_hash = sha256_digest(&verify_key);
    let pub_key_bytes = serialize(&verify_key).unwrap();
    let priv_key_bytes = private_key.to_bytes().as_slice().to_vec();

    // Save keypair to db
    db.add_public_key(&pub_key_bytes, "genesis", &pub_key_hash, -1)
        .await
        .unwrap();

    db.add_private_key(&priv_key_bytes, &pub_key_hash)
        .await
        .unwrap();

    let my_keys = db.get_my_public_key_hashes().await.unwrap();

    assert_eq!(my_keys.len(), 1);

    //assert_eq!(&my_keys.get(0).unwrap(), &pub_key_hash);
    let pub_key = db.get_public_key(&pub_key_hash).await.unwrap();
    let init_query = std::fs::read_to_string(init_query_path).unwrap();
    let init_query_hash = sha256_digest(&init_query);
    // let there be light
    let genesis_block = Block::genesis(&(private_key, verify_key, pub_key), init_query_hash);
    let mut blockchain = BlockChain::new(db, None);
    sqlx::query(&init_query).execute(&main_db).await.unwrap();
    blockchain.add_block(&genesis_block).await.unwrap();
    assert!(blockchain.is_valid().await.unwrap());
    log::info!("Blockchain was successfully initialized!");
}
