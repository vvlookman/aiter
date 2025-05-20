use std::collections::HashMap;

use tokio::sync::{mpsc::Sender, oneshot};

use crate::{api, api::get_mem_path, db, db::mem::MemWriteEvent, error::AiterResult, AiterError};

pub type SkillEntity = db::mem::skill::SkillEntity;

pub async fn add(
    ai_name: Option<&str>,
    tool_id: &str,
    trigger: Option<&str>,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<Option<SkillEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    if let Some(tool) = db::core::tool::get(tool_id).await? {
        let trigger = trigger.unwrap_or(tool.description.as_str());

        let skill = db::mem::skill::Skill::new(tool_id, trigger);
        {
            let (resp_sender, resp_receiver) = oneshot::channel();
            mem_write_event_sender
                .send(MemWriteEvent::UpsertSkill {
                    skill: skill.clone(),
                    context: tool.toolset_title.clone(),
                    resp_sender,
                })
                .await?;
            let _ = resp_receiver.await?;
        }

        db::mem::skill::get(&mem_path, &skill.id).await
    } else {
        Err(AiterError::NotExists(format!(
            "Tool '{tool_id}' not exists"
        )))
    }
}

pub async fn adds(
    ai_name: Option<&str>,
    toolset_id: &str,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<Vec<SkillEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    let mut skills = vec![];

    let tools = db::core::tool::query_by_toolset(toolset_id).await?;
    for tool in tools {
        let skill = db::mem::skill::Skill::new(&tool.id, &tool.description);
        {
            let (resp_sender, resp_receiver) = oneshot::channel();
            mem_write_event_sender
                .send(MemWriteEvent::UpsertSkill {
                    skill: skill.clone(),
                    context: tool.toolset_title.clone(),
                    resp_sender,
                })
                .await?;
            let _ = resp_receiver.await?;
        }

        if let Some(skill) = db::mem::skill::get(&mem_path, &skill.id).await? {
            skills.push(skill);
        }
    }

    Ok(skills)
}

pub async fn delete(
    ai_name: Option<&str>,
    skill_id: &str,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<Option<SkillEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    let skill = db::mem::skill::get(&mem_path, skill_id).await?;
    {
        let (resp_sender, resp_receiver) = oneshot::channel();
        mem_write_event_sender
            .send(MemWriteEvent::DeleteSkill {
                skill_id: skill_id.to_string(),
                resp_sender,
            })
            .await?;
        let _ = resp_receiver.await?;
    }

    Ok(skill)
}

pub async fn list(
    ai_name: Option<&str>,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<SkillEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::skill::list(&mem_path, search, limit, offset).await
}

pub async fn test(
    ai_name: Option<&str>,
    skill_id: &str,
    options: &HashMap<String, String>,
) -> AiterResult<String> {
    let mem_path = get_mem_path(ai_name).await?;

    let skill = db::mem::skill::get(&mem_path, skill_id)
        .await?
        .ok_or(AiterError::NotExists(format!(
            "Skill '{skill_id}' not exists"
        )))?;

    api::tool::run(&skill.tool_id, options).await
}
