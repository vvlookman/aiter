use std::sync::{Arc, LazyLock};

use aiter::{CHANNEL_BUFFER_DEFAULT, api, error::AiterResult};
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use tauri::async_runtime;
use tokio::sync::{Semaphore, mpsc};

use crate::api::{learn::DigestOptions, mem::MemWriteEvent};

mod commands;

static MEM_WRITE_EVENT_SENDERS: LazyLock<DashMap<String, mpsc::Sender<MemWriteEvent>>> =
    LazyLock::new(DashMap::new);

struct AppState {
    app_config: AppConfig,
    chat_exchanges: Arc<DashSet<String>>,
    notify_digest_event_sender: Option<mpsc::Sender<NotifyDigestEvent>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppConfig {
    digest_batch: usize,
    digest_concurrent: usize,
    digest_deep: bool,
    remote_url: String,
    remote_token: String,
    skip_digest: bool,
}

#[derive(Debug)]
struct NotifyDigestEvent {
    ai: Option<String>,
    doc_id: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    aiter::init().await;

    let app_config = AppConfig {
        digest_batch: api::config::get("AppDigestBatch")
            .await
            .unwrap_or(None)
            .map(|s| s.parse().unwrap_or(2))
            .unwrap_or(2)
            .max(1),
        digest_concurrent: api::config::get("AppDigestConcurrent")
            .await
            .unwrap_or(None)
            .map(|s| s.parse().unwrap_or(8))
            .unwrap_or(8)
            .max(1),
        digest_deep: api::config::get("AppDigestDeep")
            .await
            .unwrap_or(None)
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false),
        remote_url: api::config::get("AppRemoteUrl")
            .await
            .unwrap_or(None)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        remote_token: api::config::get("AppRemoteToken")
            .await
            .unwrap_or(None)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        skip_digest: api::config::get("AppSkipDigest")
            .await
            .unwrap_or(None)
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false),
    };

    let app_state = if app_config.remote_url.is_empty() {
        // Reset all terminated digesting tasks
        let _ = reset_terminated_digesting_tasks().await;

        let (notify_digest_event_sender, notify_digest_event_receiver) =
            mpsc::channel::<NotifyDigestEvent>(CHANNEL_BUFFER_DEFAULT);

        spawn_digest_queue(app_config.clone(), notify_digest_event_receiver);

        // Spawn to process not digested documents
        spawn_process_not_digested(app_config.clone(), notify_digest_event_sender.clone());

        AppState {
            app_config: app_config.clone(),
            chat_exchanges: Arc::new(DashSet::new()),
            notify_digest_event_sender: Some(notify_digest_event_sender),
        }
    } else {
        // If remote URL is set, no local digesting thread will be spawned
        // Check out /src/cli/serve.rs for more details about how the server process digesting
        AppState {
            app_config: app_config.clone(),
            chat_exchanges: Arc::new(DashSet::new()),
            notify_digest_event_sender: None,
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::app_config,
            commands::app_get_remote_url,
            commands::app_get_remote_token,
            commands::app_set_remote,
            commands::core_version,
            commands::is_npx_installed,
            commands::is_uv_installed,
            commands::ai::ai_add,
            commands::ai::ai_delete,
            commands::ai::ai_list,
            commands::ai::ai_rename,
            commands::chat::chat,
            commands::chat::chat_abort,
            commands::chat::chat_clear,
            commands::chat::chat_delete,
            commands::chat::chat_history,
            commands::config::config_get,
            commands::config::config_set,
            commands::doc::doc_count_part,
            commands::doc::doc_delete,
            commands::doc::doc_get_part,
            commands::doc::doc_learn,
            commands::doc::doc_list,
            commands::doc::doc_list_by_ids,
            commands::doc::doc_list_digesting_ids,
            commands::llm::llm_active,
            commands::llm::llm_config,
            commands::llm::llm_delete,
            commands::llm::llm_edit,
            commands::llm::llm_list,
            commands::llm::llm_list_actived_names,
            commands::llm::llm_test_chat,
            commands::mem::mem_stats,
            commands::mem::mem_vacuum,
            commands::skill::skill_add,
            commands::skill::skill_adds,
            commands::skill::skill_delete,
            commands::skill::skill_list,
            commands::tool::tool_delete_by_toolset,
            commands::tool::tool_get,
            commands::tool::tool_import,
            commands::tool::tool_list_by_ids,
            commands::tool::tool_list_toolsets,
            commands::tool::tool_parse,
            commands::tool::tool_query_by_toolset,
        ])
        .run(tauri::generate_context!())
        .expect("Running tauri application error!");
}

async fn get_mem_write_event_sender(ai: Option<&str>) -> AiterResult<mpsc::Sender<MemWriteEvent>> {
    let ai_key = ai.unwrap_or_default();

    if let Some(sender) = MEM_WRITE_EVENT_SENDERS.get(ai_key) {
        Ok(sender.clone())
    } else {
        let sender = api::mem::spawn_mem_write(ai).await?;
        MEM_WRITE_EVENT_SENDERS.insert(ai_key.to_string(), sender.clone());

        Ok(sender)
    }
}

async fn process_not_digested(
    ai: Option<&str>,
    notify_digest_event_sender: mpsc::Sender<NotifyDigestEvent>,
) -> AiterResult<()> {
    let docs = api::mem::doc::list_not_digested(ai).await?;

    for doc in docs {
        notify_digest_event_sender
            .send(NotifyDigestEvent {
                ai: ai.map(|s| s.to_string()),
                doc_id: doc.id,
            })
            .await?;
    }

    Ok(())
}

async fn reset_terminated_digesting_tasks() -> AiterResult<()> {
    for ai in api::ai::list().await? {
        api::mem::doc::reset_not_digested_but_started(Some(&ai.name)).await?;
    }
    api::mem::doc::reset_not_digested_but_started(None).await?;

    Ok(())
}

fn spawn_digest_queue(
    app_config: AppConfig,
    mut notify_digest_event_receiver: mpsc::Receiver<NotifyDigestEvent>,
) {
    let semaphore = Arc::new(Semaphore::new(app_config.digest_batch));

    async_runtime::spawn(async move {
        while let Some(event) = notify_digest_event_receiver.recv().await {
            if app_config.skip_digest {
                continue;
            }

            let ai_key = event.ai.clone().unwrap_or_default();
            let mem_write_event_sender = if let Some(sender) = MEM_WRITE_EVENT_SENDERS.get(&ai_key)
            {
                sender.clone()
            } else {
                let sender = api::mem::spawn_mem_write(event.ai.as_deref())
                    .await
                    .expect("Spawn mem write error");
                MEM_WRITE_EVENT_SENDERS.insert(ai_key.to_string(), sender.clone());

                sender
            };

            if let Ok(permit) = semaphore.clone().acquire_owned().await {
                let app_config = app_config.clone();
                let mem_write_event_sender = mem_write_event_sender.clone();

                async_runtime::spawn(async move {
                    let options = DigestOptions::default()
                        .with_concurrent(app_config.digest_concurrent)
                        .with_deep(app_config.digest_deep);

                    let _ = api::learn::digest_doc(
                        event.ai.as_deref(),
                        &event.doc_id,
                        &options,
                        mem_write_event_sender,
                        None,
                    )
                    .await;

                    drop(permit);
                });
            }
        }
    });
}

fn spawn_process_not_digested(
    app_config: AppConfig,
    notify_digest_event_sender: mpsc::Sender<NotifyDigestEvent>,
) {
    if app_config.skip_digest {
        return;
    }

    async_runtime::spawn(async move {
        let _ = process_not_digested(None, notify_digest_event_sender.clone()).await;

        if let Ok(ai_list) = api::ai::list().await {
            for ai in ai_list {
                let _ =
                    process_not_digested(Some(&ai.name), notify_digest_event_sender.clone()).await;
            }
        }
    });
}
