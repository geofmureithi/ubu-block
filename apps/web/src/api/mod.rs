use types::{
    CandidateResult,
    models::{Constituency, County, Station, Ward},
    results::{Candidate, GeneralResult, LastResultSummary},
};

pub async fn positions() -> Result<Vec<String>, String> {
    let res = gloo_net::http::Request::get("/api/v1/positions")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn counties() -> Result<Vec<County>, String> {
    let res = gloo_net::http::Request::get("/api/v1/counties")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn constituencies(county_id: &str) -> Result<Vec<Constituency>, String> {
    let res = gloo_net::http::Request::get(&format!("/api/v1/counties/{county_id}/constituencies"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn wards(constituency_id: &str) -> Result<Vec<Ward>, String> {
    let res =
        gloo_net::http::Request::get(&format!("/api/v1/constituencies/{constituency_id}/wards"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn stations(ward_id: &str) -> Result<Vec<Station>, String> {
    let res = gloo_net::http::Request::get(&format!("/api/v1/wards/{ward_id}/stations"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn candidates(position_type: &str, area_id: &str) -> Result<Vec<Candidate>, String> {
    let res =
        gloo_net::http::Request::get(&format!("/api/v1/candidates/{position_type}/{area_id}"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn results(position_type: &str, area_id: &str) -> Result<Vec<GeneralResult>, String> {
    let res = gloo_net::http::Request::get(&format!(
        "/api/v1/candidates/{position_type}/{area_id}/results"
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn live() -> Result<Vec<LastResultSummary>, String> {
    let res = gloo_net::http::Request::get(&format!("/api/v1/live"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn submit(results: Vec<CandidateResult>) -> Result<(), String> {
    gloo_net::http::Request::post(&format!("/api/v1/submit/raw"))
        .json(&results)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
