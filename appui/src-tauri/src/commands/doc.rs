use std::{env, fs};

use aiter::{
    api,
    error::{AiterError, AiterResult},
};
use serde::Serialize;
use ulid::Ulid;

use crate::{get_mem_write_event_sender, AppState, NotifyDigestEvent};

#[tauri::command]
pub async fn doc_count_part(ai: Option<&str>, id: &str) -> AiterResult<u64> {
    api::mem::doc::count_part(ai, id).await
}

#[tauri::command]
pub async fn doc_delete(
    ai: Option<&str>,
    id: &str,
) -> AiterResult<Option<api::mem::doc::DocEntity>> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::mem::doc::delete(ai, id, mem_write_event_sender).await
}

#[tauri::command]
pub async fn doc_get_part(ai: Option<&str>, id: &str, index: u64) -> AiterResult<Option<String>> {
    api::mem::doc::get_part_as_text(ai, id, index).await
}

#[derive(Clone, Serialize)]
pub struct DocLearnResult {
    pub doc: api::mem::doc::DocEntity,
    pub doc_exists: bool,
}

#[tauri::command]
pub async fn doc_learn(
    ai: Option<&str>,
    file_data: Vec<u8>,
    file_name: &str,
    state: tauri::State<'_, AppState>,
) -> AiterResult<DocLearnResult> {
    // Write to temp file
    let file_path = env::temp_dir().join(format!("aiter-{}", Ulid::new()));
    fs::write(&file_path, &file_data)?;

    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    // Read
    let read_result = api::learn::read_doc(
        ai,
        &file_path,
        Some(file_name),
        None,
        false,
        mem_write_event_sender,
        None,
    )
    .await?;
    fs::remove_file(&file_path)?;

    // Notify to digest, it will be put into the digest queue
    if !read_result.doc_exists {
        if let Some(event_sender) = &state.notify_digest_event_sender {
            let event = NotifyDigestEvent {
                ai: ai.map(|s| s.to_string()),
                doc_id: read_result.doc_id.clone(),
            };

            if let Err(err) = event_sender.send(event).await {
                return Err(AiterError::from(err));
            }
        }
    }

    if let Some(doc) = api::mem::doc::get(ai, &read_result.doc_id).await? {
        Ok(DocLearnResult {
            doc,
            doc_exists: read_result.doc_exists,
        })
    } else {
        Err(AiterError::NotExists(format!(
            "Doc '{}' not exists",
            read_result.doc_id
        )))
    }
}

#[tauri::command]
pub async fn doc_list(
    ai: Option<&str>,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<api::mem::doc::DocEntity>> {
    api::mem::doc::list(ai, search, limit, offset).await
}

#[tauri::command]
pub async fn doc_list_by_ids(
    ai: Option<&str>,
    ids: Vec<String>,
) -> AiterResult<Vec<api::mem::doc::DocEntity>> {
    api::mem::doc::list_by_ids(ai, &ids).await
}

#[tauri::command]
pub async fn doc_list_digesting_ids(ai: Option<&str>, limit: u64) -> AiterResult<Vec<String>> {
    let items = api::mem::doc::list_digesting(ai, limit).await?;

    Ok(items.into_iter().map(|item| item.id).collect::<Vec<_>>())
}
