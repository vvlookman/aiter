use std::collections::HashMap;

use aiter::{
    api,
    error::{AiterError, AiterResult},
    CHANNEL_BUFFER_DEFAULT,
};
use tokio::sync::mpsc;
use ulid::Ulid;

use crate::AppState;

#[tauri::command]
pub async fn llm_active(r#type: &str, name: &str) -> AiterResult<()> {
    api::llm::active(r#type, name).await
}

#[tauri::command]
pub async fn llm_config(
    name: &str,
    r#type: &str,
    protocol: &str,
    options: HashMap<String, String>,
) -> AiterResult<Option<api::llm::LlmEntity>> {
    let r#type = if r#type.is_empty() {
        None
    } else {
        Some(r#type)
    };

    let protocol = if protocol.is_empty() {
        None
    } else {
        Some(protocol)
    };

    api::llm::config(name, r#type, protocol, &options).await?;

    if let Some(saved) = api::llm::get_by_name(&name).await? {
        Ok(Some(saved))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", &name)).into())
    }
}

#[tauri::command]
pub async fn llm_delete(name: &str) -> AiterResult<Option<api::llm::LlmEntity>> {
    api::llm::delete(name).await
}

#[tauri::command]
pub async fn llm_edit(
    old_name: &str,
    name: &str,
    protocol: &str,
    options: HashMap<String, String>,
) -> AiterResult<Option<api::llm::LlmEntity>> {
    let protocol = if protocol.is_empty() {
        None
    } else {
        Some(protocol)
    };

    api::llm::config(old_name, None, protocol, &options).await?;

    if old_name != name {
        api::llm::rename(old_name, name).await?;
    }

    if let Some(saved) = api::llm::get_by_name(&name).await? {
        Ok(Some(saved))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", &name)).into())
    }
}

#[tauri::command]
pub async fn llm_list() -> AiterResult<Vec<api::llm::LlmEntity>> {
    api::llm::list().await
}

#[tauri::command]
pub async fn llm_list_actived_names() -> AiterResult<HashMap<String, String>> {
    api::llm::list_actived_names().await
}

#[tauri::command]
pub async fn llm_test_chat(
    prompt: &str,
    name: &str,
    protocol: &str,
    options: HashMap<String, String>,
    channel: tauri::ipc::Channel<String>,
    state: tauri::State<'_, AppState>,
) -> AiterResult<()> {
    let (event_sender, mut event_receiver) =
        mpsc::channel::<api::llm::ChatEvent>(CHANNEL_BUFFER_DEFAULT);

    let exchange = Ulid::new().to_string();
    state
        .chat_event_senders
        .insert(exchange.clone(), event_sender.clone());

    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                api::llm::ChatEvent::StreamStart => {}
                api::llm::ChatEvent::CallToolStart(_task) => {}
                api::llm::ChatEvent::CallToolEnd(_task, _result, _time) => {}
                api::llm::ChatEvent::ReasoningStart => {}
                api::llm::ChatEvent::ReasoningContent(delta) => {
                    if channel.send(delta).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::ReasoningEnd => {
                    if channel.send("\n\n".to_string()).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::StreamContent(delta) => {
                    if channel.send(delta).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::StreamEnd => {
                    break;
                }
            }
        }
        event_receiver.close();
    });

    let name = name.to_string();
    let prompt = prompt.to_string();
    let protocol = protocol.to_string();
    let options = options.clone();

    let handle = tokio::spawn(async move {
        api::llm::test_chat(&prompt, &name, &protocol, &options, Some(event_sender)).await
    });

    let result = match handle.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => {
            if let AiterError::Interrupted(_) = err {
                Ok(())
            } else {
                Err(err)
            }
        }
        Err(err) => Err(err.into()),
    };

    state.chat_event_senders.remove(&exchange);

    result
}
