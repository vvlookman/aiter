use std::collections::HashMap;

use crate::chat::ChatCallToolTask;

pub mod prompt;
pub mod provider;

pub struct ChatCompletionOptions {
    pub temperature: f64,
}

#[derive(Debug)]
pub enum ChatEvent {
    StreamStart,
    CallToolStart(ChatCallToolTask),
    CallToolEnd(String, String, String),
    ReasoningStart,
    ReasoningContent(String),
    ReasoningEnd,
    StreamContent(String),
    StreamEnd,
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
