use std::collections::HashMap;

use rmcp::{model::CallToolRequestParam, transport::TokioChildProcess, ServiceExt};
use tokio::process::Command;
use ulid::Ulid;

use crate::{
    db::core::tool::{Tool, ToolEntity},
    error::AiterResult,
    llm::{ChatFunction, ChatFunctionParameter},
    tool::ToolType,
    utils::json::string_to_json_value,
    AiterError,
};

pub fn chat_function_from_mcp(tool: &ToolEntity) -> AiterResult<ChatFunction> {
    let mut parameters: HashMap<String, ChatFunctionParameter> = HashMap::new();

    let tool_parameters: serde_json::Value = serde_json::from_str(&tool.parameters)?;
    if let Some(properties) = tool_parameters["properties"].as_object() {
        for (key, value) in properties {
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

pub async fn parse_mcp(
    title: Option<&str>,
    cmd: &str,
    args: &[String],
    envs: &HashMap<String, String>,
) -> AiterResult<Vec<Tool>> {
    let service = ()
        .serve(TokioChildProcess::new(
            Command::new(cmd).args(args).envs(envs),
        )?)
        .await?;

    let toolset_id = Ulid::new().to_string();
    let toolset_title = title
        .unwrap_or(&service.peer_info().server_info.name)
        .to_string();
    let toolset_options = serde_json::json!({
        "cmd": cmd,
        "args": args,
        "envs": envs
    });

    let mcp_tools = service.list_all_tools().await?;
    service.cancel().await?;

    let mut tools: Vec<Tool> = vec![];

    for mcp_tool in mcp_tools {
        let tool_id = Ulid::new().to_string();

        let tool = Tool {
            id: tool_id.to_string(),
            toolset_id: toolset_id.clone(),
            toolset_title: toolset_title.clone(),
            toolset_options: toolset_options.clone(),
            r#type: ToolType::Mcp.to_string(),
            name: mcp_tool.name.to_string(),
            description: mcp_tool.description.to_string(),
            parameters: mcp_tool.schema_as_json_value(),
        };

        tools.push(tool);
    }

    Ok(tools)
}

pub async fn run_mcp(tool: &ToolEntity, options: &HashMap<String, String>) -> AiterResult<String> {
    let toolset_options: serde_json::Value = serde_json::from_str(&tool.toolset_options)?;

    let cmd = toolset_options["cmd"]
        .as_str()
        .ok_or(AiterError::Invalid("Missing MCP option 'cmd'".to_string()))?;

    let mut args: Vec<String> = vec![];
    if let Some(option_args) = toolset_options["args"].as_array() {
        for arg in option_args {
            if let Some(arg) = arg.as_str() {
                args.push(arg.to_string());
            }
        }
    }

    let mut envs: HashMap<String, String> = HashMap::new();
    if let Some(option_envs) = toolset_options["envs"].as_object() {
        for (k, v) in option_envs {
            if let Some(v) = v.as_str() {
                envs.insert(k.to_string(), v.to_string());
            }
        }
    }

    let service = ()
        .serve(TokioChildProcess::new(
            Command::new(cmd).args(&args).envs(&envs),
        )?)
        .await?;

    let tool_parameters: serde_json::Value = serde_json::from_str(&tool.parameters)?;
    let mut call_arguments: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (k, v) in options {
        if let Some(param_type) = tool_parameters["properties"][k]["type"].as_str() {
            let param_val = string_to_json_value(v, param_type);
            call_arguments.insert(k.to_string(), param_val);
        }
    }

    let tool_result = service
        .call_tool(CallToolRequestParam {
            name: tool.name.clone().into(),
            arguments: Some(call_arguments),
        })
        .await?;

    service.cancel().await?;

    let texts: Vec<String> = tool_result
        .content
        .iter()
        .filter_map(|content| match &content.raw {
            rmcp::model::RawContent::Text(r) => Some(r.text.clone()),
            _ => None,
        })
        .collect();

    Ok(texts.join("\n"))
}
