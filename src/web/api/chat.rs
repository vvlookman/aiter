use actix_web::{web::Json, *};
use actix_web_lab::sse;
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};

use crate::{
    api, api::chat::ChatOptions, llm::ChatEvent, web::get_mem_write_event_sender,
    CHANNEL_BUFFER_DEFAULT,
};

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
    let (event_sender, mut event_receiver) = mpsc::channel::<ChatEvent>(CHANNEL_BUFFER_DEFAULT);
    let (sse_event_sender, sse_event_receiver) =
        mpsc::channel::<sse::Event>(CHANNEL_BUFFER_DEFAULT);

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

    let sse_event_sender_1 = sse_event_sender.clone();
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                ChatEvent::StreamStart => {}
                ChatEvent::CallToolStart(task) => {
                    let json_str = json!({ "call_tool_start": task }).to_string();
                    if sse_event_sender_1
                        .send(sse::Data::new(json_str).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::CallToolEnd(task_id, _result, time) => {
                    let json_str =
                        json!({ "call_tool_end": { "id": task_id, "time": time } }).to_string();
                    if sse_event_sender_1
                        .send(sse::Data::new(json_str).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::ReasoningStart => {}
                ChatEvent::ReasoningContent(content) => {
                    let json_str = json!({ "reasoning": content }).to_string();
                    if sse_event_sender_1
                        .send(sse::Data::new(json_str).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::ReasoningEnd => {
                    let json_str = json!({ "reasoning": "\n\n" }).to_string();
                    if sse_event_sender_1
                        .send(sse::Data::new(json_str).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::StreamContent(content) => {
                    let json_str = json!({ "content": content}).to_string();
                    if sse_event_sender_1
                        .send(sse::Data::new(json_str).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::StreamEnd => {
                    let _ = sse_event_sender_1
                        .send(sse::Data::new("[DONE]").into())
                        .await;
                    break;
                }
            }
        }
    });

    let sse_event_sender_2 = sse_event_sender.clone();
    if let Ok(mem_write_event_sender) = get_mem_write_event_sender(data.ai.as_deref()).await {
        tokio::spawn(async move {
            if let Err(err) = api::chat::chat(
                ai.as_deref(),
                &message,
                &chat_options,
                mem_write_event_sender,
                Some(event_sender),
            )
            .await
            {
                let json_str = json!({"content": err.to_string()}).to_string();
                let _ = sse_event_sender_2
                    .send(sse::Data::new(json_str).into())
                    .await;

                let _ = sse_event_sender_2
                    .send(sse::Data::new("[DONE]").into())
                    .await;
            }
        });
    } else {
        let json_str = json!({"content": "Spawn mem write error"}).to_string();
        let _ = sse_event_sender_2
            .send(sse::Data::new(json_str).into())
            .await;

        let _ = sse_event_sender_2
            .send(sse::Data::new("[DONE]").into())
            .await;
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
