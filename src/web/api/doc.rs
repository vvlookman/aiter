use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{
    web::{Data, Json},
    *,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api,
    web::{get_mem_write_event_sender, AppState, NotifyDigestEvent},
    AiterError,
};

#[derive(Deserialize, Debug)]
struct DocCountPartReqData {
    ai: Option<String>,
    id: String,
}

#[post("/count-part")]
pub async fn count_part(data: web::Json<DocCountPartReqData>) -> Result<impl Responder> {
    let count = api::mem::doc::count_part(data.ai.as_deref(), data.id.as_str()).await?;

    Ok(count.to_string())
}

#[derive(Deserialize, Debug)]
struct DocDeleteReqData {
    ai: Option<String>,
    id: String,
}

#[post("/delete")]
pub async fn delete(data: web::Json<DocDeleteReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;

    let deleted =
        api::mem::doc::delete(data.ai.as_deref(), &data.id, mem_write_event_sender).await?;

    Ok(Json(deleted))
}

#[derive(Deserialize, Debug)]
struct DocGetPartReqData {
    ai: Option<String>,
    id: String,
    index: u64,
}

#[post("/get-part")]
pub async fn get_part(data: web::Json<DocGetPartReqData>) -> Result<impl Responder> {
    let content =
        api::mem::doc::get_part_as_text(data.ai.as_deref(), data.id.as_str(), data.index).await?;

    Ok(content)
}

#[derive(Debug, MultipartForm)]
struct DocLearnReqForm {
    #[multipart]
    file: TempFile,
    ai: Text<String>,
    filename: Text<String>,
}

#[post("/learn")]
pub async fn learn(
    data: Data<AppState>,
    MultipartForm(form): MultipartForm<DocLearnReqForm>,
) -> Result<impl Responder> {
    let ai = if form.ai.0.trim().is_empty() || form.ai.0.trim() == "~" {
        None
    } else {
        Some(form.ai.0)
    };

    let mem_write_event_sender = get_mem_write_event_sender(ai.as_deref()).await?;

    let filename = form.filename.0.trim();
    if filename.is_empty() {
        return Err(AiterError::Invalid("The filename cannot be empty".to_string()).into());
    }

    // Read
    let read_result = api::learn::read_doc(
        ai.as_deref(),
        form.file.file.path(),
        Some(filename),
        None,
        false,
        mem_write_event_sender,
        None,
    )
    .await?;

    // Notify to digest, it will be put into the digest queue
    if !read_result.doc_exists {
        let event = NotifyDigestEvent {
            ai: ai.clone(),
            doc_id: read_result.doc_id.clone(),
        };

        if let Err(err) = data.notify_digest_event_sender.send(event).await {
            return Err(AiterError::from(err).into());
        }
    }

    if let Some(doc) = api::mem::doc::get(ai.as_deref(), &read_result.doc_id).await? {
        Ok(Json(json!({
            "doc": doc,
            "doc_exists": read_result.doc_exists,
        })))
    } else {
        Err(AiterError::NotExists(format!("Doc '{}' not exists", read_result.doc_id)).into())
    }
}

#[derive(Deserialize, Debug)]
struct DocListReqData {
    ai: Option<String>,
    search: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
}

#[post("/list")]
pub async fn list(data: web::Json<DocListReqData>) -> Result<impl Responder> {
    let items = api::mem::doc::list(
        data.ai.as_deref(),
        data.search.as_deref().unwrap_or_default(),
        data.limit.unwrap_or(21),
        data.offset.unwrap_or(0),
    )
    .await?;

    Ok(Json(items))
}

#[derive(Deserialize, Debug)]
struct DocListByIdsReqData {
    ai: Option<String>,
    ids: Vec<String>,
}

#[post("/list-by-ids")]
pub async fn list_by_ids(data: web::Json<DocListByIdsReqData>) -> Result<impl Responder> {
    let items = api::mem::doc::list_by_ids(data.ai.as_deref(), data.ids.as_slice()).await?;

    Ok(Json(items))
}

#[derive(Deserialize, Debug)]
struct DocListDigestingIdsReqData {
    ai: Option<String>,
    limit: Option<u64>,
}

#[post("/list-digesting-ids")]
pub async fn list_digesting_ids(
    data: web::Json<DocListDigestingIdsReqData>,
) -> Result<impl Responder> {
    let items =
        api::mem::doc::list_digesting(data.ai.as_deref(), data.limit.unwrap_or(100)).await?;

    Ok(Json(
        items.into_iter().map(|item| item.id).collect::<Vec<_>>(),
    ))
}
