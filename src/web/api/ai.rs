use actix_web::{web::Json, *};
use serde::Deserialize;

use crate::{api, AiterError};

#[derive(Deserialize, Debug)]
struct AiAddReqData {
    name: String,
}

#[post("/add")]
pub async fn add(data: web::Json<AiAddReqData>) -> Result<impl Responder> {
    if let Some(ai) = api::ai::add(data.name.as_str()).await? {
        Ok(Json(ai))
    } else {
        Err(AiterError::NotExists(format!("AI '{}' not exists", &data.name)).into())
    }
}

#[derive(Deserialize, Debug)]
struct AiDeleteReqData {
    name: String,
}

#[post("/delete")]
pub async fn delete(data: web::Json<AiDeleteReqData>) -> Result<impl Responder> {
    if let Some(deleted) = api::ai::delete(data.name.as_str()).await? {
        Ok(Json(deleted))
    } else {
        Err(AiterError::NotExists(format!("AI '{}' not exists", &data.name)).into())
    }
}

#[post("/list")]
pub async fn list() -> Result<impl Responder> {
    let items = api::ai::list().await?;

    Ok(Json(items))
}

#[derive(Deserialize, Debug)]
struct AiRenameReqData {
    name: String,
    new_name: String,
}

#[post("/rename")]
pub async fn rename(data: web::Json<AiRenameReqData>) -> Result<impl Responder> {
    if let Some(renamed) = api::ai::rename(data.name.as_str(), data.new_name.as_str()).await? {
        Ok(Json(renamed))
    } else {
        Err(AiterError::NotExists(format!("AI '{}' not exists", &data.name)).into())
    }
}
