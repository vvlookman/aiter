use std::process::Command;

use aiter::api;

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
    state.app_config
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
