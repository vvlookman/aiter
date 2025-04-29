use aiter::{api, error::AiterResult};

#[tauri::command]
pub async fn ai_add(name: &str) -> AiterResult<Option<api::ai::AiEntity>> {
    api::ai::add(name).await
}

#[tauri::command]
pub async fn ai_delete(name: &str) -> AiterResult<Option<api::ai::AiEntity>> {
    api::ai::delete(name).await
}

#[tauri::command]
pub async fn ai_list() -> AiterResult<Vec<api::ai::AiEntity>> {
    api::ai::list().await
}

#[tauri::command]
pub async fn ai_rename(name: &str, new_name: &str) -> AiterResult<Option<api::ai::AiEntity>> {
    api::ai::rename(name, new_name).await
}
