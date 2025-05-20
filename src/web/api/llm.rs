use std::collections::HashMap;

use actix_web::{web::Json, *};
use actix_web_lab::sse;
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};

use crate::{AiterError, CHANNEL_BUFFER_DEFAULT, api};

#[derive(Deserialize, Debug)]
struct LlmActiveReqData {
    r#type: String,
    name: String,
}

#[post("/active")]
pub async fn active(data: web::Json<LlmActiveReqData>) -> Result<impl Responder> {
    api::llm::active(data.r#type.as_str(), data.name.as_str()).await?;

    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize, Debug)]
struct LlmConfigReqData {
    name: String,
    r#type: String,
    protocol: String,
    options: HashMap<String, String>,
}

#[post("/config")]
pub async fn config(data: web::Json<LlmConfigReqData>) -> Result<impl Responder> {
    let r#type = if data.r#type.is_empty() {
        None
    } else {
        Some(data.r#type.as_str())
    };

    let protocol = if data.protocol.is_empty() {
        None
    } else {
        Some(data.protocol.as_str())
    };

    api::llm::config(data.name.as_str(), r#type, protocol, &data.options).await?;

    if let Some(saved) = api::llm::get_by_name(&data.name).await? {
        Ok(Json(mask_sensitive(&saved)))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", &data.name)).into())
    }
}

#[derive(Deserialize, Debug)]
struct LlmDeleteReqData {
    name: String,
}

#[post("/delete")]
pub async fn delete(data: web::Json<LlmDeleteReqData>) -> Result<impl Responder> {
    if let Some(deleted) = api::llm::delete(data.name.as_str()).await? {
        Ok(Json(mask_sensitive(&deleted)))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", &data.name)).into())
    }
}

#[derive(Deserialize, Debug)]
struct LlmEditReqData {
    old_name: String,
    name: String,
    protocol: String,
    options: HashMap<String, String>,
}

#[post("/edit")]
pub async fn edit(data: web::Json<LlmEditReqData>) -> Result<impl Responder> {
    let protocol = if data.protocol.is_empty() {
        None
    } else {
        Some(data.protocol.as_str())
    };

    api::llm::config(data.old_name.as_str(), None, protocol, &data.options).await?;

    if data.old_name != data.name {
        api::llm::rename(data.old_name.as_str(), data.name.as_str()).await?;
    }

    if let Some(saved) = api::llm::get_by_name(&data.name).await? {
        Ok(Json(mask_sensitive(&saved)))
    } else {
        Err(AiterError::NotExists(format!("LLM '{}' not exists", &data.name)).into())
    }
}

#[post("/list")]
pub async fn list() -> Result<impl Responder> {
    let items = api::llm::list().await?;
    let masked_items: Vec<_> = items.iter().map(mask_sensitive).collect();

    Ok(Json(masked_items))
}

#[post("/list-actived-names")]
pub async fn list_actived_names() -> Result<impl Responder> {
    let names = api::llm::list_actived_names().await?;

    Ok(Json(names))
}

#[derive(Deserialize, Debug)]
struct LlmTestChatReqData {
    prompt: String,
    name: String,
    protocol: String,
    options: HashMap<String, String>,
}

#[post("/test-chat")]
pub async fn test_chat(data: web::Json<LlmTestChatReqData>) -> impl Responder {
    let prompt = data.prompt.to_string();
    let name = data.name.to_string();
    let protocol = data.protocol.to_string();
    let options = data.options.clone();

    let (sse_event_sender, sse_event_receiver) =
        mpsc::channel::<sse::Event>(CHANNEL_BUFFER_DEFAULT);

    tokio::spawn(async move {
        match api::llm::stream_test_chat_completion(&prompt, &name, &protocol, &options).await {
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
                                if sse_event_sender
                                    .send(sse::Data::new("\n\n").into())
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }

                            has_content = true;
                            if sse_event_sender
                                .send(sse::Data::new(delta).into())
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::ReasoningContent(delta) => {
                            has_reasoning_content = true;
                            if sse_event_sender
                                .send(sse::Data::new(delta).into())
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        api::llm::ChatCompletionEvent::Error(err) => {
                            let _ = sse_event_sender
                                .send(sse::Data::new(format!("\n{err}")).into())
                                .await;

                            let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
                        }
                    }
                }

                let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
            }
            Err(err) => {
                let _ = sse_event_sender
                    .send(sse::Data::new(format!("\n{err}")).into())
                    .await;

                let _ = sse_event_sender.send(sse::Data::new("[DONE]").into()).await;
            }
        }
    });

    sse::Sse::from_infallible_receiver(sse_event_receiver)
        .with_retry_duration(Duration::from_secs(10))
}

// Do not return sensitive information via Web API
fn mask_sensitive(llm: &api::llm::LlmEntity) -> api::llm::LlmEntity {
    let mut llm = llm.clone();

    if llm.protocol == "openai" {
        if let Some(s) = llm.options.get("api_key") {
            llm.options
                .insert("api_key".to_string(), "*".repeat(s.len()));
        }
    }

    llm
}
