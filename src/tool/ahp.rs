use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ulid::Ulid;

use crate::{
    AiterError,
    db::core::tool::{Tool, ToolEntity},
    error::AiterResult,
    llm::{ChatFunction, ChatFunctionParameter},
    tool::ToolType,
    utils::{
        json::{json_value_to_string, string_to_json_value},
        net::{http_get, http_post},
    },
};

pub fn chat_function_from_ahp(tool: &ToolEntity) -> AiterResult<ChatFunction> {
    let mut parameters: HashMap<String, ChatFunctionParameter> = HashMap::new();

    let tool_parameters: serde_json::Value = serde_json::from_str(&tool.parameters)?;
    if let Some(params) = tool_parameters["params"].as_object() {
        for (key, value) in params {
            if let (Some(r#type), Some(description)) =
                (value["type"].as_str(), value["description"].as_str())
            {
                parameters.insert(
                    key.to_string(),
                    ChatFunctionParameter {
                        r#type: r#type.to_string(),
                        description: description.to_string(),
                    },
                );
            }
        }
    }

    Ok(ChatFunction {
        name: tool.id.clone(),
        description: tool.description.clone(),
        parameters,
    })
}

pub async fn parse_ahp(
    title: Option<&str>,
    url: &str,
    headers: &HashMap<String, String>,
) -> AiterResult<Vec<Tool>> {
    let meta_resp = http_get(url, Some("/meta"), &HashMap::new(), &HashMap::new()).await?;
    let meta: AhpMeta = serde_json::from_slice(&meta_resp)?;

    let toolset_id = Ulid::new().to_string();
    let toolset_title = title.unwrap_or(&meta.title).to_string();
    let toolset_options = serde_json::json!({
        "url": url,
        "headers": headers
    });

    let mut tools: Vec<Tool> = vec![];

    for service in meta.services {
        let tool_id = Ulid::new().to_string();

        let tool = Tool {
            id: tool_id.to_string(),
            toolset_id: toolset_id.clone(),
            toolset_title: toolset_title.clone(),
            toolset_options: toolset_options.clone(),
            r#type: ToolType::Ahp.to_string(),
            name: service.path.to_string(),
            description: service.description.to_string(),
            parameters: json!({
                "path": service.path,
                "method": service.method,
                "params": service.params
            }),
        };

        tools.push(tool);
    }

    Ok(tools)
}

pub async fn run_ahp(tool: &ToolEntity, options: &HashMap<String, String>) -> AiterResult<String> {
    let toolset_options: serde_json::Value = serde_json::from_str(&tool.toolset_options)?;

    let url = toolset_options["url"]
        .as_str()
        .ok_or(AiterError::Invalid("Missing AHP option 'url'".to_string()))?;

    let mut headers: HashMap<String, String> = HashMap::new();
    if let Some(option_headers) = toolset_options["headers"].as_object() {
        for (k, v) in option_headers {
            if let Some(v) = v.as_str() {
                headers.insert(k.to_string(), v.to_string());
            }
        }
    }

    let tool_parameters: serde_json::Value = serde_json::from_str(&tool.parameters)?;
    let mut call_arguments: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (k, v) in options {
        if let Some(param_type) = tool_parameters["params"][k]["type"].as_str() {
            let param_val = string_to_json_value(v, param_type);
            call_arguments.insert(k.to_string(), param_val);
        }
    }

    let path = tool_parameters["path"].as_str();
    let method = tool_parameters["method"]
        .as_str()
        .unwrap_or("GET")
        .to_lowercase();

    let bytes = if method == "get" {
        let mut query: HashMap<String, String> = HashMap::new();
        for (key, value) in call_arguments {
            query.insert(key, json_value_to_string(&value));
        }

        http_get(url, path, &headers, &query).await?
    } else {
        http_post(
            url,
            path,
            &HashMap::new(),
            &Value::Object(call_arguments),
            &headers,
        )
        .await?
    };

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[derive(Serialize, Deserialize)]
struct AhpMeta {
    title: String,
    services: Vec<AhpMetaService>,
}

#[derive(Serialize, Deserialize)]
struct AhpMetaService {
    path: String,
    description: String,
    method: Option<String>,
    params: HashMap<String, AhpMetaParam>,
}

#[derive(Serialize, Deserialize)]
struct AhpMetaParam {
    r#type: String,
    description: String,
    required: Option<bool>,
    constraints: Option<HashMap<String, serde_json::Value>>,
}
