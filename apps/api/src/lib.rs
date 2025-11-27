use axum::{
    Extension, Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use blockchain::BlockChain;
use serde::{Deserialize, Serialize};
use types::{Block, CandidateResult, merkle::MerkleTree};

async fn submit_result(mut blockchain: Extension<BlockChain>, result: Json<Block>) -> String {
    let block = blockchain.add_block(&result.0).await.unwrap();
    blockchain.announce_block(result.0).await.unwrap();
    format!("Block with index {} submitted successfully!", block)
}

async fn submit_raw_result(
    mut blockchain: Extension<BlockChain>,
    results: Json<Vec<CandidateResult>>,
) -> String {
    let db = &blockchain.db;
    let height = db.get_height().await.unwrap();

    assert!(results.len() > 0, "No empty results");
    let signer = db.get_private_key().await.unwrap();
    let prev_hash = db.get_block_by_height(height).await.unwrap().hash;

    let tree = MerkleTree::from_election_results_proper(&results);
    let root = tree.get_root_hash();

    let block = Block::new(
        &signer,
        &prev_hash,
        results.0,
        (height + 1) as usize,
        root.unwrap(),
    );
    let height = blockchain.add_block(&block).await.unwrap();
    blockchain.announce_block(block).await.unwrap();
    format!("Block with index {} submitted successfully!", height)
}

async fn block_by_height(
    blockchain: Extension<BlockChain>,
    height: Path<i64>,
) -> impl IntoResponse {
    let db = &blockchain.db;

    let block = db.get_block_by_height(*height).await.unwrap();

    Json(block)
}

async fn positions(blockchain: Extension<BlockChain>) -> impl IntoResponse {
    let db = &blockchain.db;

    let positions = db.positions().await.unwrap();

    Json(positions)
}

async fn parties(blockchain: Extension<BlockChain>) -> impl IntoResponse {
    let db = &blockchain.db;

    let parties = db.parties().await.unwrap();

    Json(parties)
}

async fn counties(blockchain: Extension<BlockChain>) -> impl IntoResponse {
    let db = &blockchain.db;

    let counties = db.counties().await.unwrap();

    Json(counties)
}

async fn constituencies_by_county(
    blockchain: Extension<BlockChain>,
    county: Path<u32>,
) -> impl IntoResponse {
    let db = &blockchain.db;

    let constituencies = db.constituencies_by_county(&county).await.unwrap();

    Json(constituencies)
}

async fn constituencies(blockchain: Extension<BlockChain>) -> impl IntoResponse {
    let db = &blockchain.db;

    let constituencies = db.constituencies().await.unwrap();

    Json(constituencies)
}

async fn wards_by_constituency(
    blockchain: Extension<BlockChain>,
    constituency: Path<u32>,
) -> impl IntoResponse {
    let db = &blockchain.db;

    let wards = db.wards_by_constituency(&constituency).await.unwrap();

    Json(wards)
}

async fn stations_by_ward(blockchain: Extension<BlockChain>, ward: Path<u32>) -> impl IntoResponse {
    let db = &blockchain.db;

    let stations = db.stations_by_ward(&ward).await.unwrap();

    Json(stations)
}

async fn candidates_by_position_type(
    blockchain: Extension<BlockChain>,
    Path((position_type, area_id)): Path<(String, i32)>,
) -> impl IntoResponse {
    let db = &blockchain.db;

    let res = match position_type.as_str() {
        "Mca" => db.candidates_by_ward(&area_id).await.unwrap(),
        "Governor" => db.candidates_by_county(&area_id, "Governor").await.unwrap(),
        "Senator" => db.candidates_by_county(&area_id, "Senator").await.unwrap(),
        "Mp" => db.candidates_by_constituency(&area_id, "Mp").await.unwrap(),
        "WomenRep" => db
            .candidates_by_constituency(&area_id, "WomenRep")
            .await
            .unwrap(),
        _ => db.candidates_national().await.unwrap(),
    };

    Json(res)
}

async fn candidates_by_result(
    blockchain: Extension<BlockChain>,
    Path((position_type, area_id)): Path<(String, i32)>,
) -> impl IntoResponse {
    let db = &blockchain.db;

    let res = match position_type.as_str() {
        "Mca" => db.results_by_ward(&area_id).await.unwrap(),
        "Governor" => db.results_by_county(&area_id, "Governor").await.unwrap(),
        "Senator" => db.results_by_county(&area_id, "Senator").await.unwrap(),
        "Mp" => db.results_by_constituency(&area_id, "Mp").await.unwrap(),
        "WomenRep" => db
            .results_by_constituency(&area_id, "WomenRep")
            .await
            .unwrap(),
        _ => vec![],
    };

    Json(res)
}

async fn live(blockchain: Extension<BlockChain>) -> impl IntoResponse {
    let db = &blockchain.db;

    let res = db.last_five_results().await.unwrap();

    Json(res)
}

pub fn run_api_server() -> Router {
    let router = Router::new()
        .route("/submit", post(submit_result))
        .route("/submit/raw", post(submit_raw_result))
        .route("/block/{height}", get(block_by_height))
        .route("/positions", get(positions))
        .route("/parties", get(parties))
        .route("/counties", get(counties))
        .route(
            "/counties/{county}/constituencies",
            get(constituencies_by_county),
        )
        .route(
            "/constituencies/{constituency}/wards",
            get(wards_by_constituency),
        )
        .route("/wards/{ward}/stations", get(stations_by_ward))
        .route(
            "/candidates/{position_type}/{area_id}",
            get(candidates_by_position_type),
        )
        .route(
            "/candidates/{position_type}/{area_id}/results",
            get(candidates_by_result),
        )
        .route("/live", get(live));
    router
}
