use std::collections::HashMap;

use futures::StreamExt;
use serde::Serialize;
use serde_json::{json, Map, Value};
use tokio::sync::mpsc;

use crate::{
    error::*,
    llm::{provider::*, ChatCompletionEvent, ChatCompletionStream},
    utils::{json::json_value_to_string, net::join_url},
    CHANNEL_BUFFER_DEFAULT,
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
    ) -> AiterResult<ChatMessage> {
        let mut content = String::new();
        let mut reasoning_content = String::new();

        let mut stream = self.stream_chat_completion(messages, options).await?;
        while let Some(event) = stream.next().await {
            match event {
                ChatCompletionEvent::CallToolStart(_task) => {}
                ChatCompletionEvent::CallToolEnd(_task_id, _result, _time) => {}
                ChatCompletionEvent::CallToolFail(_task_id, _error, _time) => {}
                ChatCompletionEvent::Content(delta) => {
                    content.push_str(&delta);
                }
                ChatCompletionEvent::ReasoningContent(delta) => {
                    reasoning_content.push_str(&delta);
                }
                ChatCompletionEvent::Error(err) => {
                    return Err(err);
                }
            }
        }

        Ok(ChatMessage {
            role: Role::Bot,
            content,
            reasoning: if reasoning_content.is_empty() {
                None
            } else {
                Some(reasoning_content)
            },
        })
    }

    async fn chat_function_calls(
        &self,
        messages: &[ChatMessage],
        functions: &[ChatFunction],
    ) -> AiterResult<Vec<ChatFunctionCall>> {
        let request_url = join_url(&self.base_url, "/chat/completions")?;

        let mut messages_json_value = messages
            .iter()
            .map(chat_message_to_json_value)
            .collect::<Vec<_>>();
        if self.model.starts_with("qwen3") && !messages.is_empty() {
            if let Some(content) = messages_json_value[messages.len() - 1].get_mut("content") {
                if let Some(content_str) = content.as_str() {
                    *content = format!("{content_str} /no_think").into();
                }
            }
        }

        let request_body = json!({
            "model": self.model,
            "messages": messages_json_value,
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

    async fn stream_chat_completion(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
    ) -> AiterResult<ChatCompletionStream> {
        let request_url = join_url(&self.base_url, "/chat/completions")?;

        let mut messages_json_value = messages
            .iter()
            .map(chat_message_to_json_value)
            .collect::<Vec<_>>();
        if self.model.starts_with("qwen3") && !messages.is_empty() {
            if let Some(content) = messages_json_value[messages.len() - 1].get_mut("content") {
                if let Some(content_str) = content.as_str() {
                    let instruction = if options.enable_think {
                        "/think"
                    } else {
                        "/no_think"
                    };
                    *content = format!("{content_str} {instruction}").into();
                }
            }
        }

        let request_body = json!({
            "model": self.model,
            "messages": messages_json_value,
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
            let (sender, receiver) = mpsc::channel(CHANNEL_BUFFER_DEFAULT);

            tokio::spawn(async move {
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(chunk) => {
                            let chunk_str = String::from_utf8_lossy(&chunk);

                            for line in chunk_str.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" {
                                        break;
                                    }

                                    match serde_json::from_str::<Value>(data) {
                                        Ok(json) => {
                                            if let Some(delta_content) =
                                                json["choices"][0]["delta"]["content"].as_str()
                                            {
                                                let _ = sender
                                                    .send(ChatCompletionEvent::Content(
                                                        delta_content.to_string(),
                                                    ))
                                                    .await;
                                            } else if let Some(delta_reasoning_content) = json
                                                ["choices"][0]["delta"]["reasoning_content"]
                                                .as_str()
                                            {
                                                let _ = sender
                                                    .send(ChatCompletionEvent::ReasoningContent(
                                                        delta_reasoning_content.to_string(),
                                                    ))
                                                    .await;
                                            }
                                        }
                                        Err(err) => {
                                            let _ = sender
                                                .send(ChatCompletionEvent::Error(err.into()))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            let _ = sender.send(ChatCompletionEvent::Error(err.into())).await;
                        }
                    }
                }
            });

            Ok(ChatCompletionStream { receiver })
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
