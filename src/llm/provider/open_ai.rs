use std::collections::HashMap;

use futures::StreamExt;
use serde::Serialize;
use serde_json::{json, Map, Value};
use tokio::sync::mpsc::Sender;

use crate::{
    error::*,
    llm::{provider::*, ChatEvent},
    utils::{json::json_value_to_string, net::join_url},
};

pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

impl ChatProvider for OpenAiProvider {
    async fn chat_completion(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
        chat_event_sender: Option<Sender<ChatEvent>>,
    ) -> AiterResult<ChatMessage> {
        let request_url = join_url(&self.base_url, "/chat/completions")?;

        let request_body = json!({
            "model": self.model,
            "messages": messages.iter().map(chat_message_to_json_value).collect::<Vec<_>>(),
            "temperature": options.temperature,
            "stream": true,
        });

        let client = reqwest::Client::builder().build()?;

        let response = client
            .post(request_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let mut content = String::new();
            let mut reasoning = String::new();

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                let chunk_str = String::from_utf8_lossy(&chunk);

                for line in chunk_str.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            break;
                        }

                        let no_content_received = content.is_empty();
                        let no_reasoning_received = reasoning.is_empty();

                        let json: Value = serde_json::from_str(data)?;
                        if let Some(delta_content) = json["choices"][0]["delta"]["content"].as_str()
                        {
                            content.push_str(delta_content);

                            if let Some(chat_event_sender) = &chat_event_sender {
                                if no_content_received {
                                    if no_reasoning_received {
                                        let _ =
                                            chat_event_sender.send(ChatEvent::StreamStart).await;
                                    } else {
                                        let _ =
                                            chat_event_sender.send(ChatEvent::ReasoningEnd).await;
                                    }
                                }

                                if chat_event_sender
                                    .send(ChatEvent::StreamContent(delta_content.to_string()))
                                    .await
                                    .is_err()
                                {
                                    return Err(AiterError::Interrupted(content));
                                }
                            }
                        } else if let Some(delta_reasoning) =
                            json["choices"][0]["delta"]["reasoning_content"].as_str()
                        {
                            reasoning.push_str(delta_reasoning);

                            if let Some(chat_event_sender) = &chat_event_sender {
                                if no_reasoning_received {
                                    if no_content_received {
                                        let _ =
                                            chat_event_sender.send(ChatEvent::StreamStart).await;
                                    }

                                    let _ = chat_event_sender.send(ChatEvent::ReasoningStart).await;
                                }

                                if chat_event_sender
                                    .send(ChatEvent::ReasoningContent(delta_reasoning.to_string()))
                                    .await
                                    .is_err()
                                {
                                    return Err(AiterError::Interrupted(content));
                                }
                            }
                        }
                    }
                }
            }

            if let Some(chat_event_sender) = &chat_event_sender {
                let _ = chat_event_sender.send(ChatEvent::StreamEnd).await;
            }

            Ok(ChatMessage {
                role: Role::Bot,
                content,
                reasoning: if reasoning.is_empty() {
                    None
                } else {
                    Some(reasoning)
                },
            })
        } else {
            Err(AiterError::HttpStatusError(format!(
                "{} {}",
                response.status(),
                response.text().await.ok().unwrap_or_default()
            )))
        }
    }

    async fn chat_function_calls(
        &self,
        messages: &[ChatMessage],
        functions: &[ChatFunction],
    ) -> AiterResult<Vec<ChatFunctionCall>> {
        let request_url = join_url(&self.base_url, "/chat/completions")?;

        let request_body = json!({
            "model": self.model,
            "messages": messages.iter().map(chat_message_to_json_value).collect::<Vec<_>>(),
            "tools": functions.iter().map(chat_function_to_json_value).collect::<Vec<_>>(),
            "parallel_tool_calls": true,
            "stream": true,
        });

        let client = reqwest::Client::builder().build()?;

        let response = client
            .post(request_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let mut tool_calls_name_str: HashMap<u64, String> = HashMap::new();
            let mut tool_calls_arguments_str: HashMap<u64, String> = HashMap::new();

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                let chunk_str = String::from_utf8_lossy(&chunk);

                for line in chunk_str.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            break;
                        }

                        let json: Value = serde_json::from_str(data)?;

                        if let Some(delta_tool_calls) =
                            json["choices"][0]["delta"]["tool_calls"].as_array()
                        {
                            for delta_tool_call in delta_tool_calls {
                                if let (Some(index), Some(r#type)) = (
                                    delta_tool_call["index"].as_u64(),
                                    delta_tool_call["type"].as_str(),
                                ) {
                                    if r#type == "function" {
                                        if let Some(function) =
                                            delta_tool_call["function"].as_object()
                                        {
                                            if let Some(delta_name_value) = function.get("name") {
                                                if let Some(delta_name) = delta_name_value.as_str()
                                                {
                                                    if let Some(name) =
                                                        tool_calls_name_str.get(&index)
                                                    {
                                                        tool_calls_name_str.insert(
                                                            index,
                                                            name.to_string() + delta_name,
                                                        );
                                                    } else {
                                                        tool_calls_name_str
                                                            .insert(index, delta_name.to_string());
                                                    }
                                                }
                                            }

                                            if let Some(delta_arguments_value) =
                                                function.get("arguments")
                                            {
                                                if let Some(delta_arguments) =
                                                    delta_arguments_value.as_str()
                                                {
                                                    if let Some(arguments) =
                                                        tool_calls_arguments_str.get(&index)
                                                    {
                                                        tool_calls_arguments_str.insert(
                                                            index,
                                                            arguments.to_string() + delta_arguments,
                                                        );
                                                    } else {
                                                        tool_calls_arguments_str.insert(
                                                            index,
                                                            delta_arguments.to_string(),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut chat_tool_calls: Vec<ChatFunctionCall> = vec![];

            for (index, name) in tool_calls_name_str {
                if let Some(arguments) = tool_calls_arguments_str.get(&index) {
                    if let Ok(function_arguments) =
                        serde_json::from_str::<HashMap<String, serde_json::Value>>(arguments)
                    {
                        let mut arguments = HashMap::new();
                        for (key, value) in function_arguments {
                            arguments.insert(key.to_string(), json_value_to_string(&value));
                        }

                        chat_tool_calls.push(ChatFunctionCall {
                            name: name.to_string(),
                            arguments,
                        });
                    }
                }
            }

            Ok(chat_tool_calls)
        } else {
            Err(AiterError::HttpStatusError(format!(
                "{} {}",
                response.status(),
                response.text().await.ok().unwrap_or_default()
            )))
        }
    }
}

#[derive(strum::Display)]
enum OpenAiRole {
    #[strum(serialize = "user")]
    User,

    #[strum(serialize = "assistant")]
    Assistant,

    #[strum(serialize = "system")]
    System,

    #[strum(serialize = "function")]
    Function,

    #[strum(serialize = "tool")]
    Tool,
}

impl From<Role> for OpenAiRole {
    fn from(val: Role) -> Self {
        match val {
            Role::User => OpenAiRole::User,
            Role::Bot => OpenAiRole::Assistant,
            Role::System => OpenAiRole::System,
            Role::Func => OpenAiRole::Function,
            Role::Tool => OpenAiRole::Tool,
        }
    }
}

impl Serialize for OpenAiRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn chat_function_to_json_value(chat_function: &ChatFunction) -> Value {
    let parameters_map: Map<String, Value> =
        Map::from_iter(chat_function.parameters.iter().map(|(name, parameter)| {
            (
                name.to_string(),
                json!({
                    "type": parameter.r#type.to_string(),
                    "description": parameter.description.to_string(),
                }),
            )
        }));

    json!({
        "type": "function",
        "function": {
            "name": chat_function.name,
            "description": chat_function.description,
            "parameters": {
                "type": "object",
                "properties": Value::Object(parameters_map),
            }
        }
    })
}

fn chat_message_to_json_value(chat_message: &ChatMessage) -> Value {
    json!({
        "role": Into::<OpenAiRole>::into(chat_message.role).to_string(),
        "content": chat_message.content
    })
}
