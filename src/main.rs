mod db;
use clap::Parser;
use sqlx::SqlitePool;
use tabled::{Table, Tabled};
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
#[clap(name = "ubu-block")]
#[clap(bin_name = "ubu-block")]
enum UbuBlock {
    Pull(Pull),
    /// Run a query on the blockchain
    Query {
        #[clap(short)]
        q: String,
    },
    /// Add a new block
    Import {
        path: std::path::PathBuf,
    },
    /// Start listening
    Start,
}

#[derive(clap::Args, Debug)]
#[clap(author, version, about, long_about = "Pull the blockchain from a url")]
struct Pull {
    #[clap(long)]
    manifest_path: Option<std::path::PathBuf>,
}

#[derive(sqlx::FromRow, Debug, Tabled)]
struct VoteResult {
    county: String,
    constituency: String,
    ward: String,
    candidate: String,
    party: String,
    votes: u32,
}

fn main() {
    let ubu = UbuBlock::parse();
    let rt = Runtime::new().expect("tokio runtime can be initialized");

    rt.block_on(async move {
        let conn = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let mut pool = conn.acquire().await.unwrap();
        sqlx::query(db::SETUP).execute(&mut pool).await.unwrap();
        match ubu {
            UbuBlock::Pull(_) => todo!(),
            UbuBlock::Query { q } => {
                let res: Vec<VoteResult> = sqlx::query_as(&q)
                    .fetch_all(&mut pool)
                    .await
                    .expect("Could not query");
                println!("{}", Table::new(res).to_string());
            }
            UbuBlock::Import { path: _ } => todo!(),
            UbuBlock::Start => todo!(),
        };
    });
}
