use aiter::{api, error::AiterResult};
use serde_json::json;

use crate::{AppState, get_mem_write_event_sender};

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn chat(
    ai: Option<&str>,
    message: &str,
    exchange: &str,
    session: Option<&str>,
    llm_for_chat: Option<&str>,
    llm_for_reasoning: Option<&str>,
    llm_options: Option<Vec<String>>,
    deep: Option<bool>,
    retrace: Option<u64>,
    strict: Option<bool>,
    channel: tauri::ipc::Channel<String>,
    state: tauri::State<'_, AppState>,
) -> AiterResult<()> {
    let ai = ai.map(|s| s.to_string());
    let message = message.to_string();
    let chat_options = api::chat::ChatOptions::default()
        .with_deep(deep.unwrap_or(false))
        .with_exchange(Some(exchange.to_string()))
        .with_llm_for_chat(llm_for_chat.map(|s| s.to_string()))
        .with_llm_for_reasoning(llm_for_reasoning.map(|s| s.to_string()))
        .with_llm_options(llm_options.clone().unwrap_or_default())
        .with_retrace(retrace.unwrap_or(0))
        .with_session(session.map(|s| s.to_string()))
        .with_strict(strict.unwrap_or(false));

    let exchange = exchange.to_string();
    let chat_exchanges = state.chat_exchanges.clone();
    let mem_write_event_sender = get_mem_write_event_sender(ai.as_deref()).await?;

    let handle = tokio::spawn(async move {
        chat_exchanges.insert(exchange.to_string());

        let result = match api::chat::chat(
            ai.as_deref(),
            &message,
            &chat_options,
            mem_write_event_sender,
        )
        .await
        {
            Ok(mut stream) => {
                let mut has_content = false;
                let mut has_reasoning_content = false;

                while let Some(event) = stream.next().await {
                    match event {
                        api::llm::ChatCompletionEvent::CallToolStart(task) => {
                            let json_str = json!({ "call_tool_start": task }).to_string();
                            if channel.send(json_str).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::CallToolEnd(task_id, _result, time) => {
                            let json_str =
                                json!({ "call_tool_end": { "id": task_id, "time": time } })
                                    .to_string();
                            if channel.send(json_str).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::CallToolFail(task_id, error, time) => {
                            let json_str =
                                json!({ "call_tool_fail": { "id": task_id, "error": error, "time": time } })
                                    .to_string();
                            if channel.send(json_str).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::Content(content) => {
                            if !has_content && has_reasoning_content {
                                let json_str = json!({ "reasoning": "\n\n" }).to_string();
                                if channel.send(json_str).is_err() {
                                    break;
                                }
                            }

                            has_content = true;
                            let json_str = json!({ "content": content}).to_string();
                            if channel.send(json_str).is_err() {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::ReasoningContent(content) => {
                            has_reasoning_content = true;
                            let json_str = json!({ "reasoning": content }).to_string();
                            if channel.send(json_str).is_err() {
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

#[tauri::command]
pub async fn chat_abort(exchange: &str, state: tauri::State<'_, AppState>) -> AiterResult<()> {
    state.chat_exchanges.remove(exchange);

    Ok(())
}

#[tauri::command]
pub async fn chat_clear(ai: Option<&str>, session: Option<&str>) -> AiterResult<()> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::chat::clear(ai, session, mem_write_event_sender).await
}

#[tauri::command]
pub async fn chat_delete(
    ai: Option<&str>,
    session: Option<&str>,
    exchange: &str,
) -> AiterResult<()> {
    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

    api::chat::delete(ai, session, exchange, mem_write_event_sender).await
}

#[tauri::command]
pub async fn chat_history(
    ai: Option<&str>,
    session: Option<&str>,
) -> AiterResult<Vec<api::chat::HistoryChatEntity>> {
    api::chat::history(ai, session).await
}
