use std::collections::HashMap;

use actix_web::{web::Json, *};
use actix_web_lab::sse;
use serde::Deserialize;
use serde_json::json;
use tokio::{sync::mpsc, time::Duration};

use crate::{api, llm::ChatEvent, AiterError, CHANNEL_BUFFER_DEFAULT};

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
    let (event_sender, mut event_receiver) = mpsc::channel::<ChatEvent>(CHANNEL_BUFFER_DEFAULT);
    let (sse_event_sender, sse_event_receiver) =
        mpsc::channel::<sse::Event>(CHANNEL_BUFFER_DEFAULT);

    let sse_event_sender_1 = sse_event_sender.clone();
    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                ChatEvent::StreamStart => {}
                ChatEvent::CallToolStart(_task) => {}
                ChatEvent::CallToolEnd(_task_id, _result, _time) => {}
                ChatEvent::CallToolError(_task_id, _error) => {}
                ChatEvent::ReasoningStart => {}
                ChatEvent::ReasoningContent(delta) => {
                    if sse_event_sender_1
                        .send(sse::Data::new(delta).into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::ReasoningEnd => {
                    if sse_event_sender_1
                        .send(sse::Data::new("\n\n").into())
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                ChatEvent::StreamContent(delta) => {
                    if sse_event_sender_1
                        .send(sse::Data::new(delta).into())
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

    let prompt = data.prompt.to_string();
    let name = data.name.to_string();
    let protocol = data.protocol.to_string();
    let options = data.options.clone();

    let sse_event_sender_2 = sse_event_sender.clone();
    tokio::spawn(async move {
        if let Err(err) =
            api::llm::test_chat(&prompt, &name, &protocol, &options, Some(event_sender)).await
        {
            let _ = sse_event_sender_2
                .send(sse::Data::new(err.to_string()).into())
                .await;

            let _ = sse_event_sender_2
                .send(sse::Data::new("[DONE]").into())
                .await;
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
