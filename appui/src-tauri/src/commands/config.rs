use aiter::{api, error::AiterResult};

#[tauri::command]
pub async fn config_get(key: &str) -> AiterResult<Option<String>> {
    api::config::get(key).await
}

#[tauri::command]
pub async fn config_set(key: &str, value: &str) -> AiterResult<()> {
    api::config::set(key, value).await
}
