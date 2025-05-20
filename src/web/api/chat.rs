use actix_web::{web::Json, *};
use actix_web_lab::sse;
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};

use crate::{CHANNEL_BUFFER_DEFAULT, api, api::chat::ChatOptions, web::get_mem_write_event_sender};

#[derive(Deserialize, Debug)]
struct ChatReqData {
    ai: Option<String>,
    message: String,
    exchange: String,
    session: Option<String>,
    llm_for_chat: Option<String>,
    llm_for_reasoning: Option<String>,
    llm_options: Option<Vec<String>>,
    deep: Option<bool>,
    retrace: Option<u64>,
    strict: Option<bool>,
}

#[post("/")]
pub async fn index(data: web::Json<ChatReqData>) -> impl Responder {
    let ai = data.ai.clone();
    let message = data.message.clone();
    let chat_options = ChatOptions::default()
        .with_deep(data.deep.unwrap_or(false))
        .with_exchange(Some(data.exchange.clone()))
        .with_llm_for_chat(data.llm_for_chat.clone())
        .with_llm_for_reasoning(data.llm_for_reasoning.clone())
        .with_llm_options(data.llm_options.clone().unwrap_or_default())
        .with_retrace(data.retrace.unwrap_or(0))
        .with_session(data.session.clone())
        .with_strict(data.strict.unwrap_or(false));

    let (sse_event_sender, sse_event_receiver) =
        mpsc::channel::<sse::Event>(CHANNEL_BUFFER_DEFAULT);

    if let Ok(mem_write_event_sender) = get_mem_write_event_sender(data.ai.as_deref()).await {
        tokio::spawn(async move {
            match api::chat::chat(
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
                                if sse_event_sender
                                    .send(sse::Data::new(json_str).into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            api::llm::ChatCompletionEvent::CallToolEnd(task_id, _result, time) => {
                                let json_str =
                                    json!({ "call_tool_end": { "id": task_id, "time": time } })
                                        .to_string();
                                if sse_event_sender
                                    .send(sse::Data::new(json_str).into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            api::llm::ChatCompletionEvent::CallToolFail(task_id, error, time) => {
                                let json_str =
                                    json!({ "call_tool_fail": { "id": task_id, "error": error, "time": time } })
                                        .to_string();
                                if sse_event_sender
                                    .send(sse::Data::new(json_str).into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            api::llm::ChatCompletionEvent::Content(content) => {
                                if !has_content && has_reasoning_content {
                                    let json_str = json!({ "reasoning": "\n\n" }).to_string();
                                    if sse_event_sender
                                        .send(sse::Data::new(json_str).into())
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }

                                has_content = true;
                                let json_str = json!({ "content": content}).to_string();
                                if sse_event_sender
                                    .send(sse::Data::new(json_str).into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            api::llm::ChatCompletionEvent::ReasoningContent(content) => {
                                has_reasoning_content = true;
                                let json_str = json!({ "reasoning": content }).to_string();
                                if sse_event_sender
                                    .send(sse::Data::new(json_str).into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            api::llm::ChatCompletionEvent::Error(err) => {
                                let json_str = json!({"content": err.to_string()}).to_string();
                                let _ =
                                    sse_event_sender.send(sse::Data::new(json_str).into()).await;

                                let _ =
                                    sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
                            }
                        }
                    }

                    let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
                }
                Err(err) => {
                    let json_str = json!({"content": err.to_string()}).to_string();
                    let _ = sse_event_sender.send(sse::Data::new(json_str).into()).await;

                    let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
                }
            }
        });
    } else {
        let json_str = json!({"content": "Spawn mem write error"}).to_string();
        let _ = sse_event_sender.send(sse::Data::new(json_str).into()).await;

        let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
    }

    sse::Sse::from_infallible_receiver(sse_event_receiver)
        .with_retry_duration(Duration::from_secs(10))
}

#[derive(Deserialize, Debug)]
struct ChatClearReqData {
    ai: Option<String>,
    session: String,
}

#[post("/clear")]
pub async fn clear(data: web::Json<ChatClearReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;
    api::chat::clear(
        data.ai.as_deref(),
        Some(data.session.as_str()),
        mem_write_event_sender,
    )
    .await?;

    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize, Debug)]
struct ChatDeleteReqData {
    ai: Option<String>,
    session: String,
    exchange: String,
}

#[post("/delete")]
pub async fn delete(data: web::Json<ChatDeleteReqData>) -> Result<impl Responder> {
    let mem_write_event_sender = get_mem_write_event_sender(data.ai.as_deref()).await?;
    api::chat::delete(
        data.ai.as_deref(),
        Some(data.session.as_str()),
        data.exchange.as_str(),
        mem_write_event_sender,
    )
    .await?;

    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize, Debug)]
struct ChatHistoryReqData {
    ai: Option<String>,
    session: String,
}

#[post("/history")]
pub async fn history(data: web::Json<ChatHistoryReqData>) -> Result<impl Responder> {
    let items = api::chat::history(data.ai.as_deref(), Some(data.session.as_str())).await?;

    Ok(Json(items))
}
