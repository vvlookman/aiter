use std::{collections::HashMap, str::FromStr};

use crate::{
    AiterError, VecOptions,
    api::get_mem_path,
    db,
    error::AiterResult,
    tool::{
        ToolType,
        ahp::{parse_ahp, run_ahp},
        mcp::{parse_mcp, run_mcp},
    },
};

pub type Tool = db::core::tool::Tool;
pub type ToolEntity = db::core::tool::ToolEntity;
pub type ToolsetEntity = db::core::tool::ToolsetEntity;

pub async fn delete(tool_id: &str) -> AiterResult<Option<ToolEntity>> {
    let tool = db::core::tool::get(tool_id).await?;

    db::core::tool::delete(tool_id).await?;

    let mut mem_paths = vec![get_mem_path(None).await?];
    for ai in db::core::ai::list().await? {
        mem_paths.push(get_mem_path(Some(&ai.name)).await?);
    }

    for mem_path in mem_paths {
        // Allow deleting skills by tool failed.
        let _ = db::mem::skill::delete_by_tool(&mem_path, tool_id).await;
    }

    Ok(tool)
}

pub async fn delete_by_toolset(toolset_id: &str) -> AiterResult<Vec<ToolEntity>> {
    let tools = db::core::tool::query_by_toolset(toolset_id).await?;

    for tool in &tools {
        delete(&tool.id).await?;
    }

    Ok(tools)
}

pub async fn get(tool_id: &str) -> AiterResult<Option<ToolEntity>> {
    db::core::tool::get(tool_id).await
}

pub async fn import(
    r#type: &str,
    title: Option<&str>,
    options: &[String],
) -> AiterResult<Vec<ToolEntity>> {
    let mut imported_tools: Vec<ToolEntity> = vec![];

    let tools = parse(r#type, title, options).await?;
    for tool in tools {
        db::core::tool::insert(&tool).await?;

        if let Some(tool_entity) = db::core::tool::get(&tool.id).await? {
            imported_tools.push(tool_entity);
        }
    }

    Ok(imported_tools)
}

pub async fn list(search: &str, limit: u64, offset: u64) -> AiterResult<Vec<ToolEntity>> {
    db::core::tool::list(search, limit, offset).await
}

pub async fn list_by_ids(ids: &[String]) -> AiterResult<Vec<ToolEntity>> {
    db::core::tool::list_by_ids(ids).await
}

pub async fn list_toolsets() -> AiterResult<Vec<ToolsetEntity>> {
    db::core::tool::list_toolsets().await
}

pub async fn query_by_toolset(toolset_id: &str) -> AiterResult<Vec<ToolEntity>> {
    db::core::tool::query_by_toolset(toolset_id).await
}

pub async fn parse(
    r#type: &str,
    title: Option<&str>,
    options: &[String],
) -> AiterResult<Vec<Tool>> {
    if let Ok(tool_type) = ToolType::from_str(r#type) {
        let vec_options = VecOptions(options);
        match tool_type {
            ToolType::Ahp => {
                if let Some(url) = vec_options.get("url") {
                    let mut headers: HashMap<String, String> = HashMap::new();

                    for (k, v) in vec_options.into_tuples() {
                        if k.to_lowercase() == "header" {
                            let parts: Vec<_> = v.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                headers.insert(parts[0].to_string(), parts[1].trim().to_string());
                            }
                        }
                    }

                    parse_ahp(title, &url, &headers).await
                } else {
                    Err(AiterError::Invalid("Missing AHP option 'url'".to_string()))
                }
            }
            ToolType::Mcp => {
                if let Some(cmd) = vec_options.get("cmd") {
                    let mut args: Vec<String> = vec![];
                    let mut envs: HashMap<String, String> = HashMap::new();

                    for (k, v) in vec_options.into_tuples() {
                        if k.to_lowercase() == "arg" {
                            args.push(v);
                        } else if k.to_lowercase() == "env" {
                            let parts: Vec<_> = v.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                envs.insert(parts[0].to_string(), parts[1].trim().to_string());
                            }
                        }
                    }

                    parse_mcp(title, &cmd, &args, &envs).await
                } else {
                    Err(AiterError::Invalid("Missing MCP option 'cmd'".to_string()))
                }
            }
        }
    } else {
        Err(AiterError::Invalid(format!("Invalid tool type: {type}")))
    }
}

pub async fn run(tool_id: &str, options: &HashMap<String, String>) -> AiterResult<String> {
    let tool = db::core::tool::get(tool_id)
        .await?
        .ok_or(AiterError::NotExists(format!(
            "Tool '{tool_id}' not exists"
        )))?;

    match ToolType::from_str(&tool.r#type)? {
        ToolType::Ahp => run_ahp(&tool, options).await,
        ToolType::Mcp => run_mcp(&tool, options).await,
    }
}
