use std::process::Command;

use aiter::{api, error::AiterResult};

use crate::{AppConfig, AppState};

pub mod ai;
pub mod chat;
pub mod config;
pub mod doc;
pub mod llm;
pub mod mem;
pub mod skill;
pub mod tool;

#[tauri::command]
pub fn app_config(state: tauri::State<'_, AppState>) -> AppConfig {
    state.app_config.clone()
}

#[tauri::command]
pub async fn app_get_remote_url() -> AiterResult<Option<String>> {
    api::config::get("AppRemoteUrl").await
}

#[tauri::command]
pub async fn app_get_remote_token() -> AiterResult<Option<String>> {
    api::config::get("AppRemoteToken").await
}

#[tauri::command]
pub async fn app_set_remote(
    remote_url: Option<&str>,
    remote_token: Option<&str>,
) -> AiterResult<()> {
    api::config::set("AppRemoteUrl", remote_url.unwrap_or_default()).await?;
    api::config::set("AppRemoteToken", remote_token.unwrap_or_default()).await?;

    Ok(())
}

#[tauri::command]
pub async fn core_version() -> String {
    api::sys::get_version().await
}

#[tauri::command]
pub fn is_npx_installed() -> bool {
    let output = Command::new("npx").arg("--version").output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[tauri::command]
pub fn is_uv_installed() -> bool {
    let output = Command::new("uv").arg("--version").output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
