use serde::{Deserialize, Serialize};

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub title: String,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: i32,
    pub title: Option<String>,
    pub logo: Option<String>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct County {
    pub county_code: i32,
    pub county_name: Option<String>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constituency {
    pub constituency_code: i32,
    pub county_code: i32,
    pub constituency_name: Option<String>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ward {
    pub ward_code: i32,
    pub constituency_code: i32,
    pub ward_name: Option<String>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: i64,
    pub ward_code: i32,
    pub reg_center_code: Option<i32>,
    pub station_name: Option<String>,
    pub registered_voters: Option<i32>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateArea {
    pub candidate_id: i32,
    pub area_type: String,
    pub station_id: Option<i32>,
    pub ward_code: Option<i32>,
    pub constituency_code: Option<i32>,
    pub county_code: Option<i32>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub address: String,
    pub time_added: i64,
    pub permanent: bool,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRow {
    pub station_id: i32,
    pub candidate_id: i32,
    pub votes: i32,
    pub block_height: i64,
}
