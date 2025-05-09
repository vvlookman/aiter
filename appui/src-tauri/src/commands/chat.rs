use aiter::{
    api,
    error::{AiterError, AiterResult},
    CHANNEL_BUFFER_DEFAULT,
};
use serde_json::json;
use tokio::sync::mpsc;

use crate::{get_mem_write_event_sender, AppState};

#[tauri::command]
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
    let (event_sender, mut event_receiver) =
        mpsc::channel::<api::llm::ChatEvent>(CHANNEL_BUFFER_DEFAULT);

    state
        .chat_event_senders
        .insert(exchange.to_string(), event_sender.clone());

    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                api::llm::ChatEvent::StreamStart => {}
                api::llm::ChatEvent::CallToolStart(task) => {
                    let json_str = json!({ "call_tool_start": task }).to_string();
                    if channel.send(json_str).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::CallToolEnd(task_id, _result, time) => {
                    let json_str =
                        json!({ "call_tool_end": { "id": task_id, "time": time } }).to_string();
                    if channel.send(json_str).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::CallToolError(task_id, error) => {
                    let json_str =
                        json!({ "call_tool_error": { "id": task_id, "error": error } }).to_string();
                    if channel.send(json_str).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::ReasoningStart => {}
                api::llm::ChatEvent::ReasoningContent(content) => {
                    let json_str = json!({ "reasoning": content }).to_string();
                    if channel.send(json_str).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::ReasoningEnd => {
                    let json_str = json!({ "reasoning": "\n\n" }).to_string();
                    if channel.send(json_str).is_err() {
                        break;
                    }
                }
                api::llm::ChatEvent::StreamContent(content) => {
                    let json_str = json!({ "content": content}).to_string();
                    if channel.send(json_str).is_err() {
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

    let mem_write_event_sender = get_mem_write_event_sender(ai).await?;

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

    let handle = tokio::spawn(async move {
        api::chat::chat(
            ai.as_deref(),
            &message,
            &chat_options,
            mem_write_event_sender,
            Some(event_sender),
        )
        .await
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

    state.chat_event_senders.remove(exchange);

    result
}

#[tauri::command]
pub async fn chat_abort(
    exchange: Option<&str>,
    state: tauri::State<'_, AppState>,
) -> AiterResult<()> {
    if let Some(exchange) = exchange {
        if let Some((_, event_sender)) = state.chat_event_senders.remove(exchange) {
            event_sender.send(api::llm::ChatEvent::StreamEnd).await?;
        }
    }

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
