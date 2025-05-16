use std::collections::HashMap;

use aiter::{
    api,
    error::{AiterError, AiterResult},
};

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

    if let Some(saved) = api::llm::get_by_name(name).await? {
        Ok(Some(saved))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", name)))
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

    if let Some(saved) = api::llm::get_by_name(name).await? {
        Ok(Some(saved))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", name)))
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
    exchange: &str,
    name: &str,
    protocol: &str,
    options: HashMap<String, String>,
    channel: tauri::ipc::Channel<String>,
    state: tauri::State<'_, AppState>,
) -> AiterResult<()> {
    let name = name.to_string();
    let prompt = prompt.to_string();
    let protocol = protocol.to_string();
    let options = options.clone();

    let exchange = exchange.to_string();
    let chat_exchanges = state.chat_exchanges.clone();

    let handle = tokio::spawn(async move {
        chat_exchanges.insert(exchange.to_string());

        let result = match api::llm::stream_test_chat_completion(
            &prompt, &name, &protocol, &options,
        )
        .await
        {
            Ok(mut stream) => {
                let mut has_content = false;
                let mut has_reasoning_content = false;

                while let Some(event) = stream.next().await {
                    match event {
                        api::llm::ChatCompletionEvent::CallToolStart(_task) => {}
                        api::llm::ChatCompletionEvent::CallToolEnd(_task_id, _result, _time) => {}
                        api::llm::ChatCompletionEvent::CallToolFail(_task_id, _error, _time) => {}
                        api::llm::ChatCompletionEvent::Content(delta) => {
                            if !has_content && has_reasoning_content {
                                if channel.send("\n\n".to_string()).is_err() {
                                    break;
                                }
                            }

                            has_content = true;
                            if channel.send(delta).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::ReasoningContent(delta) => {
                            has_reasoning_content = true;
                            if channel.send(delta).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::Error(err) => {
                            let _ = channel.send(err.to_string());
                            break;
                        }
                    }

                    // Aborted
                    if !chat_exchanges.contains(&exchange) {
                        break;
                    }
                }

                Ok(())
            }
            Err(err) => Err(err),
        };

        chat_exchanges.remove(&exchange);

        result
    });

    match handle.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(err.into()),
    }
}
