use serde::{Deserialize, Serialize};

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationResult {
    pub station_id: i32,
    pub station_name: String,
    pub ward_code: i32,
    pub ward_name: String,
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub votes: i32,
    pub registered_voters: Option<i32>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardResult {
    pub ward_code: i32,
    pub ward_name: String,
    pub constituency_code: i32,
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub total_votes: i64,
    pub station_count: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstituencyResult {
    pub constituency_code: i32,
    pub constituency_name: String,
    pub county_code: i32,
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub total_votes: i64,
    pub ward_count: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyResult {
    pub county_code: i32,
    pub county_name: String,
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub total_votes: i64,
    pub constituency_count: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalResult {
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub total_votes: i64,
    pub county_count: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSummary {
    pub candidate_id: i32,
    pub candidate_name: String,
    pub gender: String,
    pub party_title: Option<String>,
    pub position_type: String,
    pub total_votes: i64,
    pub stations_reported: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResult {
    pub position_type: String,
    pub candidate_id: i32,
    pub candidate_name: String,
    pub party_title: Option<String>,
    pub total_votes: i64,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyResult {
    pub party_id: i32,
    pub party_title: String,
    pub position_type: String,
    pub total_votes: i64,
    pub candidate_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::FromRow))]
pub struct Candidate {
    pub id: i32,
    pub name: String,
    pub gender: String,
    pub photo: Option<String>,
    pub position_type: String,
    pub party_id: i32,
}
