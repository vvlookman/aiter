use std::{collections::HashMap, str::FromStr};

use tokio::sync::mpsc::Sender;

use crate::{
    db::mem::history_chat::HistoryChatEntity,
    error::AiterResult,
    llm::{ChatCompletionOptions, ChatEvent, ChatFunction, ChatFunctionCall, ChatMessage, Role},
};

pub mod open_ai;

pub trait ChatProvider {
    fn chat_completion(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
        chat_event_sender: Option<Sender<ChatEvent>>,
    ) -> impl std::future::Future<Output = AiterResult<ChatMessage>> + Send;

    fn chat_function_calls(
        &self,
        messages: &[ChatMessage],
        functions: &[ChatFunction],
    ) -> impl std::future::Future<Output = AiterResult<Vec<ChatFunctionCall>>> + Send;
}

impl From<HistoryChatEntity> for ChatMessage {
    fn from(historical_chat: HistoryChatEntity) -> Self {
        let (content, reasoning) = if let Ok(json) =
            serde_json::from_str::<HashMap<String, String>>(&historical_chat.content)
        {
            (
                json.get("content")
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                json.get("reasoning").map(|s| s.to_string()),
            )
        } else {
            (historical_chat.content, None)
        };

        Self {
            role: Role::from_str(&historical_chat.role).unwrap_or(Role::Bot),
            content,
            reasoning,
        }
    }
}
