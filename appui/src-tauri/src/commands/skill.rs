use aiter::{api, error::AiterResult};

use crate::get_mem_write_event_sender;

#[tauri::command]
pub async fn skill_add(
    ai: Option<&str>,
    tool_id: &str,
    trigger: Option<&str>,
) -> AiterResult<Option<api::mem::skill::SkillEntity>> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::mem::skill::add(ai, tool_id, trigger, mem_write_event_sender).await
}

#[tauri::command]
pub async fn skill_adds(
    ai: Option<&str>,
    toolset_id: &str,
) -> AiterResult<Vec<api::mem::skill::SkillEntity>> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::mem::skill::adds(ai, toolset_id, mem_write_event_sender).await
}

#[tauri::command]
pub async fn skill_delete(
    ai: Option<&str>,
    id: &str,
) -> AiterResult<Option<api::mem::skill::SkillEntity>> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::mem::skill::delete(ai, id, mem_write_event_sender).await
}

#[tauri::command]
pub async fn skill_list(
    ai: Option<&str>,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<api::mem::skill::SkillEntity>> {
    api::mem::skill::list(ai, search, limit, offset).await
}
