use blockchain::BlockChain;
use database::{Database, SqlitePool};
use types::p2p::P2PConfig;

#[tokio::main]
async fn main() {
    // TODO: This should come from genesis
    let init_sql = r#"
    INSERT INTO
    positions
Values
    ("Mca"),
    ("WomenRep"),
    ("Mp"),
    ("Senator"),
    ("Governor"),
    ("President");

INSERT INTO
    parties
Values
    (1, "ODM", ""),
    (2, "PNU", "");

INSERT INTO
    counties
VALUES
    (22, "Kiambu"),
    (45, "Kisii");

INSERT INTO
    constituencies
VALUES
    (113, 22, "Juja"),
    (261, 45, "Bonchari");

INSERT INTO
    wards
VALUES
    (563, 113, "Kalimoni"),
    (1301, 261, "Bomariba");

INSERT INTO
    stations
VALUES
    (
        022113056303301,
        563,
        33,
        "Athi Primary School",
        533
    );

INSERT INTO
    stations
VALUES
    (
        045261130100402,
        1301,
        4,
        "Igonga Primary School",
        685
    );
INSERT INTO
    candidates
VALUES
    (1, "Mwas", "M", "", "Mp", 1, 022113056303301),
    (2, "Omosh", "M", "", "Mp", 2, 022113056303301);

INSERT INTO
    candidates
VALUES
    (3, "Adhis", "F", "", "Mp", 1, 045261130100402),
    (4, "Juma", "F", "", "Mp", 2, 045261130100402);
    "#;
    env_logger::init(); // Initialize the logger
    log::info!("Starting an observer node...");
    let peer_addr = "127.0.0.1:9090".parse().unwrap();
    let config = P2PConfig::default();
    let chain_db = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    let private_db = SqlitePool::connect_lazy("sqlite::memory:").unwrap();

    let blockchain = BlockChain::new(Database::new(chain_db, private_db), Some(config));
    sqlx::query(database::MAIN_SETUP)
        .execute(&blockchain.db.chain_db)
        .await
        .unwrap();
    sqlx::query(init_sql)
        .execute(&blockchain.db.chain_db)
        .await
        .unwrap();

    // Start the node and connect to a peer
    blockchain.connect_to_peer(peer_addr).await.unwrap();
}
