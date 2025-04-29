use actix_web::{web::Json, *};
use serde::Deserialize;

use crate::{api, web::get_mem_write_event_sender};

#[derive(Deserialize, Debug)]
struct SkillAddReqData {
    ai: Option<String>,
    tool_id: String,
    trigger: Option<String>,
}

#[post("/add")]
pub async fn add(data: web::Json<SkillAddReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;

    let item = api::mem::skill::add(
        data.ai.as_deref(),
        &data.tool_id,
        data.trigger.as_deref(),
        mem_write_event_sender,
    )
    .await?;

    Ok(Json(item))
}

#[derive(Deserialize, Debug)]
struct SkillAddsReqData {
    ai: Option<String>,
    toolset_id: String,
}

#[post("/adds")]
pub async fn adds(data: web::Json<SkillAddsReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;

    let items =
        api::mem::skill::adds(data.ai.as_deref(), &data.toolset_id, mem_write_event_sender).await?;

    Ok(Json(items))
}

#[derive(Deserialize, Debug)]
struct SkillDeleteReqData {
    ai: Option<String>,
    id: String,
}

#[post("/delete")]
pub async fn delete(data: web::Json<SkillDeleteReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;

    let deleted =
        api::mem::skill::delete(data.ai.as_deref(), &data.id, mem_write_event_sender).await?;

    Ok(Json(deleted))
}

#[derive(Deserialize, Debug)]
struct SkillListReqData {
    ai: Option<String>,
    search: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[post("/list")]
pub async fn list(data: web::Json<SkillListReqData>) -> Result<impl Responder> {
    let items = api::mem::skill::list(
        data.ai.as_deref(),
        data.search.as_deref().unwrap_or_default(),
        data.limit.unwrap_or(21),
        data.offset.unwrap_or(0),
    )
    .await?;

    Ok(Json(items))
}
