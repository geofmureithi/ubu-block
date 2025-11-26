use axum::{
    Extension, Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use blockchain::BlockChain;
use types::Block;

async fn submit_result(mut blockchain: Extension<BlockChain>, result: Json<Block>) -> String {
    let block = blockchain.add_block(&result.0).await.unwrap();
    blockchain.announce_block(result.0).await.unwrap();
    format!("Block with index {} submitted successfully!", block)
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

pub fn run_api_server() -> Router {
    let router = Router::new()
        .route("/submit", post(submit_result))
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
        );
    router
}
