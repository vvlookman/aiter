use std::collections::HashMap;

use tokio::sync::mpsc::Sender;

use crate::{
    db,
    error::*,
    llm,
    llm::provider::{open_ai::OpenAiProvider, *},
    LLM_CHAT_TEMPERATURE_DEFAULT,
};

pub static SUPPORTED_TYPES: &[&str] = &["chat", "reasoning"];
pub static SUPPORTED_PROTOCOLS: &[&str] = &["openai"];

pub type ChatCompletionOptions = llm::ChatCompletionOptions;
pub type ChatEvent = llm::ChatEvent;
pub type ChatFunction = llm::ChatFunction;
pub type ChatFunctionCall = llm::ChatFunctionCall;
pub type ChatMessage = llm::ChatMessage;
pub type Role = llm::Role;

pub type LlmEntity = db::core::llm::LlmEntity;

pub async fn active(r#type: &str, name: &str) -> AiterResult<()> {
    match r#type {
        "chat" => db::core::config::set(&db::core::config::ConfigKey::ActiveChatLlm, name).await,
        "reasoning" => {
            db::core::config::set(&db::core::config::ConfigKey::ActiveReasoningLlm, name).await
        }
        _ => Err(AiterError::Invalid(format!(
            "Invalid LLM type '{}'",
            r#type
        ))),
    }
}

pub async fn chat_completion(
    message: &str,
    history: &[ChatMessage],
    chat_completion_options: &ChatCompletionOptions,
    chat_llm_name: Option<&str>,
    event_sender: Option<Sender<ChatEvent>>,
) -> AiterResult<ChatMessage> {
    let name = if chat_llm_name.is_none() {
        get_actived_name("chat").await?
    } else {
        chat_llm_name.map(|s| s.to_string())
    };

    if let Some(name) = name {
        if let Some(LlmEntity {
            protocol, options, ..
        }) = get_by_name(&name).await?
        {
            if protocol == "openai" {
                if let (Some(base_url), Some(api_key), Some(model)) = (
                    options.get("base_url"),
                    options.get("api_key"),
                    options.get("model"),
                ) {
                    let mut messages = history.to_vec();
                    messages.push(ChatMessage {
                        role: Role::User,
                        content: message.to_string(),
                        reasoning: None,
                    });

                    return OpenAiProvider::new(base_url, api_key, model)
                        .chat_completion(&messages, chat_completion_options, event_sender)
                        .await;
                } else {
                    for k in ["base_url", "api_key", "model"] {
                        if !options.contains_key(k) {
                            return Err(AiterError::Invalid(format!(
                                "Missing required option {}",
                                k
                            )));
                        }
                    }
                }
            }
        }
    }

    Err(AiterError::Invalid("No Chat LLM".to_string()))
}

pub async fn chat_function_calls(
    functions: &[ChatFunction],
    message: &str,
    history: &[ChatMessage],
    chat_llm_name: Option<&str>,
) -> AiterResult<Vec<ChatFunctionCall>> {
    let name = if chat_llm_name.is_none() {
        get_actived_name("chat").await?
    } else {
        chat_llm_name.map(|s| s.to_string())
    };

    if let Some(name) = name {
        if let Some(LlmEntity {
            protocol, options, ..
        }) = get_by_name(&name).await?
        {
            if protocol == "openai" {
                if let (Some(base_url), Some(api_key), Some(model)) = (
                    options.get("base_url"),
                    options.get("api_key"),
                    options.get("model"),
                ) {
                    let mut messages = history.to_vec();
                    messages.push(ChatMessage {
                        role: Role::User,
                        content: message.to_string(),
                        reasoning: None,
                    });

                    return OpenAiProvider::new(base_url, api_key, model)
                        .chat_function_calls(&messages, functions)
                        .await;
                } else {
                    for k in ["base_url", "api_key", "model"] {
                        if !options.contains_key(k) {
                            return Err(AiterError::Invalid(format!(
                                "Missing required option {}",
                                k
                            )));
                        }
                    }
                }
            }
        }
    }

    Err(AiterError::Invalid("No Chat LLM".to_string()))
}

pub async fn config(
    name: &str,
    r#type: Option<&str>,
    protocol: Option<&str>,
    options: &HashMap<String, String>,
) -> AiterResult<()> {
    if name.is_empty() {
        return Err(AiterError::Invalid("LLM name cannot be empty".to_string()));
    } else if name.starts_with('~') || name.starts_with('@') {
        return Err(AiterError::Invalid(
            "LLM name cannot start with ~ or @, they are reserved.".to_string(),
        ));
    }

    if let Some(row) = db::core::llm::get_by_name(name).await? {
        let mut merged_options = row.options.clone();
        for (k, v) in options {
            merged_options.insert(k.to_string(), v.to_string());
        }
        db::core::llm::upsert(
            name,
            r#type.unwrap_or(&row.r#type),
            protocol.unwrap_or(&row.protocol),
            &merged_options,
        )
        .await?;
    } else {
        if let (Some(r#type), Some(protocol)) = (r#type, protocol) {
            db::core::llm::upsert(name, r#type, protocol, options).await?;

            #[allow(clippy::single_match)]
            match r#type {
                "chat" => {
                    if db::core::config::get(&db::core::config::ConfigKey::ActiveChatLlm)
                        .await?
                        .is_none()
                    {
                        db::core::config::set(&db::core::config::ConfigKey::ActiveChatLlm, name)
                            .await?;
                    }
                }
                _ => (),
            }
        } else {
            return Err(AiterError::Invalid(
                "LLM's type and protocol are required".to_string(),
            ));
        }
    }

    Ok(())
}

pub async fn delete(name: &str) -> AiterResult<Option<LlmEntity>> {
    let item = db::core::llm::get_by_name(name).await?;

    db::core::llm::delete(name).await?;

    Ok(item)
}

pub async fn get_actived_name(r#type: &str) -> AiterResult<Option<String>> {
    match r#type {
        "chat" => db::core::config::get(&db::core::config::ConfigKey::ActiveChatLlm).await,
        "reasoning" => {
            db::core::config::get(&db::core::config::ConfigKey::ActiveReasoningLlm).await
        }
        _ => Ok(None),
    }
}

pub async fn get_by_name(name: &str) -> AiterResult<Option<LlmEntity>> {
    db::core::llm::get_by_name(name).await
}

pub async fn list() -> AiterResult<Vec<LlmEntity>> {
    db::core::llm::list().await
}

pub async fn list_actived_names() -> AiterResult<HashMap<String, String>> {
    let mut map: HashMap<String, String> = HashMap::new();

    if let Some(name) = db::core::config::get(&db::core::config::ConfigKey::ActiveChatLlm).await? {
        map.insert("chat".to_string(), name);
    }

    if let Some(name) =
        db::core::config::get(&db::core::config::ConfigKey::ActiveReasoningLlm).await?
    {
        map.insert("reasoning".to_string(), name);
    }

    Ok(map)
}

pub async fn rename(name: &str, new_name: &str) -> AiterResult<()> {
    if new_name.is_empty() {
        return Err(AiterError::Invalid("LLM name cannot be empty".to_string()));
    } else if new_name.starts_with('~') || new_name.starts_with('@') {
        return Err(AiterError::Invalid(
            "LLM name cannot start with ~ or @, they are reserved.".to_string(),
        ));
    }

    db::core::llm::rename(name, new_name).await?;

    if let Some(active_name) =
        db::core::config::get(&db::core::config::ConfigKey::ActiveChatLlm).await?
    {
        if active_name == name {
            db::core::config::set(&db::core::config::ConfigKey::ActiveChatLlm, new_name).await?;
        }
    }

    if let Some(active_name) =
        db::core::config::get(&db::core::config::ConfigKey::ActiveReasoningLlm).await?
    {
        if active_name == name {
            db::core::config::set(&db::core::config::ConfigKey::ActiveReasoningLlm, new_name)
                .await?;
        }
    }

    Ok(())
}

pub async fn test_chat(
    prompt: &str,
    name: &str,
    protocol: &str,
    options: &HashMap<String, String>,
    event_sender: Option<Sender<ChatEvent>>,
) -> AiterResult<String> {
    if protocol == "openai" {
        if let (Some(base_url), Some(api_key), Some(model)) = (
            options.get("base_url"),
            options.get("api_key"),
            options.get("model"),
        ) {
            // If api_key is all *, means it is masked, so read from database
            let mut api_key_unmask = api_key.to_string();
            if api_key.trim().trim_matches('*').is_empty() {
                if let Ok(Some(llm)) = get_by_name(name).await {
                    if let Some(api_key) = llm.options.get("api_key") {
                        api_key_unmask = api_key.to_string();
                    }
                }
            }

            let messages = vec![ChatMessage {
                role: Role::User,
                content: prompt.to_string(),
                reasoning: None,
            }];

            return OpenAiProvider::new(base_url, &api_key_unmask, model)
                .chat_completion(&messages, &ChatCompletionOptions::default(), event_sender)
                .await
                .map(|message| message.content);
        } else {
            for k in ["base_url", "api_key", "model"] {
                if !options.contains_key(k) {
                    return Err(AiterError::Invalid(format!(
                        "Missing required option {}",
                        k
                    )));
                }
            }
        }
    } else {
        return Err(AiterError::Invalid(format!(
            "Unsupported protocol '{}'",
            protocol
        )));
    }

    Err(AiterError::Invalid("No Chat LLM".to_string()))
}

impl Default for ChatCompletionOptions {
    fn default() -> Self {
        Self {
            temperature: LLM_CHAT_TEMPERATURE_DEFAULT,
        }
    }
}

impl ChatCompletionOptions {
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}
