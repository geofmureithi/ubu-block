use tabled::{Table, Tabled};
use types::config::Config;
#[derive(sqlx::FromRow, Debug, Tabled)]
struct VoteResult {
    county: String,
    constituency: String,
    ward: String,
    candidate: String,
    party: String,
    votes: u32,
}

pub async fn query_blockchain(config: Config, query: &str) {
    let chain_db = database::SqlitePool::connect(&config.main_db)
        .await
        .unwrap();
    let res: Vec<VoteResult> = sqlx::query_as(query)
        .fetch_all(&chain_db)
        .await
        .expect("Could not query");
    println!("{}", Table::new(&res));
}
