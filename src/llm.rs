use std::collections::HashMap;

use tokio::sync::mpsc::Receiver;

use crate::{chat::ChatCallToolTask, AiterError, LLM_CHAT_TEMPERATURE_DEFAULT};

pub mod prompt;
pub mod provider;

#[derive(Debug)]
pub enum ChatCompletionEvent {
    CallToolStart(ChatCallToolTask),
    CallToolEnd(String, String, String),
    CallToolFail(String, String, String),
    Content(String),
    ReasoningContent(String),
    Error(AiterError),
}

pub struct ChatCompletionOptions {
    pub enable_think: bool, // Some multi-mode-models can switch between think/nothink mode, such as qwen3
    pub temperature: f64,
}

pub struct ChatCompletionStream {
    receiver: Receiver<ChatCompletionEvent>,
}

#[derive(Debug)]
pub struct ChatFunction {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, ChatFunctionParameter>,
}

#[derive(Debug)]
pub struct ChatFunctionParameter {
    pub r#type: String,
    pub description: String,
}

#[derive(Debug)]
pub struct ChatFunctionCall {
    pub name: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub reasoning: Option<String>,
}

#[allow(dead_code)]
#[derive(strum::Display, strum::EnumString, Copy, Clone, Debug, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum Role {
    Bot,
    User,
    System,
    Func,
    Tool,
}

impl Default for ChatCompletionOptions {
    fn default() -> Self {
        Self {
            enable_think: false,
            temperature: LLM_CHAT_TEMPERATURE_DEFAULT,
        }
    }
}

impl ChatCompletionOptions {
    pub fn with_enable_think(mut self, enable_think: bool) -> Self {
        self.enable_think = enable_think;
        self
    }

    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

impl ChatCompletionStream {
    pub fn new(receiver: Receiver<ChatCompletionEvent>) -> Self {
        Self { receiver }
    }

    pub fn close(&mut self) {
        self.receiver.close()
    }

    pub async fn next(&mut self) -> Option<ChatCompletionEvent> {
        self.receiver.recv().await
    }
}
