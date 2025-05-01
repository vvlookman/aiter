use actix_web::{web::Json, *};
use serde::Deserialize;
use serde_json::json;

use crate::api;

#[derive(Deserialize, Debug)]
struct MemStatsReqData {
    ai: Option<String>,
}

#[post("/stats")]
pub async fn stats(data: web::Json<MemStatsReqData>) -> Result<impl Responder> {
    let size = api::mem::stats(data.ai.as_deref()).await?;

    Ok(Json(json!({
        "size": size,
    })))
}

#[derive(Deserialize, Debug)]
struct MemVacuumReqData {
    ai: Option<String>,
}

#[post("/vacuum")]
pub async fn vacuum(data: web::Json<MemVacuumReqData>) -> Result<impl Responder> {
    api::mem::vacuum(data.ai.as_deref()).await?;

    Ok(Json(json!({ "ok": true })))
}
