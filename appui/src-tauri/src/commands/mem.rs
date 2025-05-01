use aiter::{api, error::AiterResult};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct MemStatsResult {
    pub size: u64,
}

#[tauri::command]
pub async fn mem_stats(ai: Option<&str>) -> AiterResult<MemStatsResult> {
    let size = api::mem::stats(ai).await?;

    Ok(MemStatsResult { size })
}

#[tauri::command]
pub async fn mem_vacuum(ai: Option<&str>) -> AiterResult<()> {
    api::mem::vacuum(ai).await
}
