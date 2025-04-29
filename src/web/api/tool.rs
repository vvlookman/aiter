use actix_web::{web::Json, *};
use serde::Deserialize;

use crate::api;

#[derive(Deserialize, Debug)]
struct ToolDeleteByToolsetReqData {
    toolset_id: String,
}

#[post("/delete-by-toolset")]
pub async fn delete_by_toolset(
    data: web::Json<ToolDeleteByToolsetReqData>,
) -> Result<impl Responder> {
    let tools = api::tool::delete_by_toolset(&data.toolset_id).await?;

    Ok(Json(tools))
}

#[derive(Deserialize, Debug)]
struct ToolGetReqData {
    id: String,
}

#[post("/get")]
pub async fn get(data: web::Json<ToolGetReqData>) -> Result<impl Responder> {
    let tool = api::tool::get(&data.id).await?;

    Ok(Json(tool))
}

#[derive(Deserialize, Debug)]
struct ToolImportReqData {
    r#type: String,
    title: Option<String>,
    options: Vec<String>,
}

#[post("/import")]
pub async fn import(data: web::Json<ToolImportReqData>) -> Result<impl Responder> {
    let tools = api::tool::import(&data.r#type, data.title.as_deref(), &data.options).await?;

    Ok(Json(tools))
}

#[derive(Deserialize, Debug)]
struct ToolListByIdsReqData {
    ids: Vec<String>,
}

#[post("/list-by-ids")]
pub async fn list_by_ids(data: web::Json<ToolListByIdsReqData>) -> Result<impl Responder> {
    let items = api::tool::list_by_ids(data.ids.as_slice()).await?;

    Ok(Json(items))
}

#[post("/list-toolsets")]
pub async fn list_toolsets() -> Result<impl Responder> {
    let toolsets = api::tool::list_toolsets().await?;

    Ok(Json(toolsets))
}

#[derive(Deserialize, Debug)]
struct ToolQueryByToolsetReqData {
    toolset_id: String,
}

#[post("/query-by-toolset")]
pub async fn query_by_toolset(
    data: web::Json<ToolQueryByToolsetReqData>,
) -> Result<impl Responder> {
    let tools = api::tool::query_by_toolset(&data.toolset_id).await?;

    Ok(Json(tools))
}

#[derive(Deserialize, Debug)]
struct ToolTestReqData {
    r#type: String,
    options: Vec<String>,
}

#[post("/parse")]
pub async fn parse(data: web::Json<ToolTestReqData>) -> Result<impl Responder> {
    let tools = api::tool::parse(&data.r#type, None, &data.options).await?;

    Ok(Json(tools))
}
