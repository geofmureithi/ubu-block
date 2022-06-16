mod blockchain;
mod db;
use clap::Parser;
use sqlx::{Row, SqlitePool};
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
        query: String,
    },
    /// Import an existing sqlite file
    Import {
        path: std::path::PathBuf,
    },
    /// Start listening
    Init,

    /// Insert a single block
    Insert {
        station: i64,
        candidate: i32,
        votes: i32,
        source: String,
    },
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
        let conn = SqlitePool::connect("sqlite://data/ubu-block.db")
            .await
            .unwrap();
        let mut pool = conn.acquire().await.unwrap();

        match ubu {
            UbuBlock::Pull(_) => todo!(),
            UbuBlock::Query { query } => {
                let res: Vec<VoteResult> = sqlx::query_as(&query)
                    .fetch_all(&mut pool)
                    .await
                    .expect("Could not query");
                println!("{}", Table::new(&res).to_string());
            }
            UbuBlock::Import { path: _ } => todo!(),
            UbuBlock::Init => {
                sqlx::query(db::SETUP).execute(&mut pool).await.unwrap();
            }
            UbuBlock::Insert {
                station,
                candidate,
                votes,
                source,
            } => {
                let query = "INSERT INTO results VALUES(NULL, ?1, ?2, ?3, ?4);";
                let _res = sqlx::query(query)
                    .bind(station)
                    .bind(candidate)
                    .bind(votes)
                    .bind(source)
                    .execute(&mut pool)
                    .await
                    .expect("Could add block");
            }
        };
    });
}
