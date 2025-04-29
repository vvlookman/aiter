use aiter::{api, error::AiterResult};

#[tauri::command]
pub async fn tool_delete_by_toolset(toolset_id: &str) -> AiterResult<Vec<api::tool::ToolEntity>> {
    api::tool::delete_by_toolset(toolset_id).await
}

#[tauri::command]
pub async fn tool_get(id: &str) -> AiterResult<Option<api::tool::ToolEntity>> {
    api::tool::get(id).await
}

#[tauri::command]
pub async fn tool_import(
    r#type: &str,
    title: Option<&str>,
    options: Vec<String>,
) -> AiterResult<Vec<api::tool::ToolEntity>> {
    api::tool::import(r#type, title, &options).await
}

#[tauri::command]
pub async fn tool_list_by_ids(ids: Vec<String>) -> AiterResult<Vec<api::tool::ToolEntity>> {
    api::tool::list_by_ids(&ids).await
}

#[tauri::command]
pub async fn tool_list_toolsets() -> AiterResult<Vec<api::tool::ToolsetEntity>> {
    api::tool::list_toolsets().await
}

#[tauri::command]
pub async fn tool_parse(r#type: &str, options: Vec<String>) -> AiterResult<Vec<api::tool::Tool>> {
    api::tool::parse(r#type, None, &options).await
}

#[tauri::command]
pub async fn tool_query_by_toolset(toolset_id: &str) -> AiterResult<Vec<api::tool::ToolEntity>> {
    api::tool::query_by_toolset(toolset_id).await
}
