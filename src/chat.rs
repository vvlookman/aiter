use std::{
    collections::{HashMap, HashSet},
    path::Path,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{
    sync::{mpsc::Sender, oneshot},
    task::JoinHandle,
};
use ulid::Ulid;

use crate::{
    api, db,
    db::mem::MemWriteEvent,
    error::AiterResult,
    llm::{
        prompt::{
            generate::{make_answer_by_candidates_prompt, make_no_answer_prompt},
            intent::{make_break_question_prompt, make_simplify_questions_prompt},
        },
        ChatCompletionOptions, ChatEvent, ChatFunction, ChatMessage, Role,
    },
    retrieve::{
        doc::{retrieve_doc_frag, retrieve_doc_implicit, retrieve_doc_knl},
        skill::retrieve_skill,
        RetrieveMethod,
    },
    tool::{ahp::chat_function_from_ahp, mcp::chat_function_from_mcp, ToolType},
    utils::{datetime::now_iso_datetime_string, markdown::extract_code_block},
    AiterError, VecOptions, LLM_CHAT_TEMPERATURE_STABLE,
};

#[derive(Default)]
pub struct ChatOptions {
    pub deep: bool,
    pub exchange: Option<String>,
    pub llm_for_chat: Option<String>,
    pub llm_for_reasoning: Option<String>,
    pub llm_options: Vec<String>,
    pub retrace: u64,
    pub session: Option<String>,
    pub strict: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCallToolTask {
    pub id: String,
    pub tool_id: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ChatResponseSource {
    Llm,
    Retrieve,
}

pub async fn chat_step(
    mem_path: &Path,
    answer_rowid: i64,
    question: &str,
    chat_options: &ChatOptions,
    chat_history: &[ChatMessage],
    mem_write_event_sender: Sender<MemWriteEvent>,
    chat_event_sender: Option<Sender<ChatEvent>>,
) -> AiterResult<(ChatMessage, ChatResponseSource)> {
    let history_questions = chat_history
        .iter()
        .filter(|m| m.role == Role::User)
        .map(|m| m.content.clone())
        .collect::<Vec<_>>();

    let mut related_questions: HashSet<String> = HashSet::new();
    let mut candidates: HashSet<String> = HashSet::new();

    // Break down the question into sub-questions
    {
        let prompt = make_break_question_prompt(question, &history_questions);
        let json_text = extract_code_block(
            &api::llm::chat_completion(
                &prompt,
                &[],
                &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
                chat_options.llm_for_chat.as_deref(),
                None,
            )
            .await?
            .content,
        );
        if let Ok(sub_questions) = serde_json::from_str::<Vec<String>>(&json_text) {
            if !sub_questions.is_empty() {
                log::debug!(
                    "Question [{}] is broken down into: {:?}",
                    &question,
                    &sub_questions
                );
                related_questions.extend(sub_questions);
            }
        }
    }

    // Try retrieve contents by full text search
    {
        let contents = retrieve_contents(
            &RetrieveMethod::Fts,
            mem_path,
            question,
            &related_questions
                .clone()
                .into_iter()
                .collect::<Vec<String>>(),
            chat_options.deep,
        )
        .await?;
        candidates.extend(contents);
    }

    // Try retrieve contents by full text search again if no content retrieved, simplify all questions before that
    if candidates.is_empty() {
        let not_simplify_questions: Vec<String> =
            HashSet::<String>::from_iter(related_questions.clone())
                .into_iter()
                .chain(std::iter::once(question.to_string()))
                .collect();

        let prompt = make_simplify_questions_prompt(&not_simplify_questions);
        let json_text = extract_code_block(
            &api::llm::chat_completion(
                &prompt,
                &[],
                &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
                chat_options.llm_for_chat.as_deref(),
                None,
            )
            .await?
            .content,
        );
        if let Ok(simple_questions) = serde_json::from_str::<Vec<String>>(&json_text) {
            if !simple_questions.is_empty() {
                log::debug!(
                    "Questions {:?} are simplified to: {:?}",
                    &not_simplify_questions,
                    &simple_questions
                );
                related_questions.extend(simple_questions);

                // Retrieve contents by full text search again
                {
                    let contents = retrieve_contents(
                        &RetrieveMethod::Fts,
                        mem_path,
                        question,
                        &related_questions
                            .clone()
                            .into_iter()
                            .collect::<Vec<String>>(),
                        chat_options.deep,
                    )
                    .await?;
                    candidates.extend(contents);
                }
            }
        }
    }

    let related_questions_vec: Vec<String> = related_questions.into_iter().collect();
    log::debug!("All related questions: {:?}", &related_questions_vec);

    // Retrieve contents by vector match
    {
        let contents = retrieve_contents(
            &RetrieveMethod::Vec,
            mem_path,
            question,
            &related_questions_vec,
            chat_options.deep,
        )
        .await?;
        candidates.extend(contents);
    }

    // Retrieve skills
    let mut skill_retrievers: Vec<JoinHandle<AiterResult<Vec<db::mem::skill::SkillEntity>>>> =
        vec![];

    {
        let mem_path = mem_path.to_path_buf();
        let question = question.to_string();
        let related_questions_vec = related_questions_vec.clone();
        let deep = chat_options.deep;
        skill_retrievers.push(tokio::spawn(async move {
            retrieve_skill(
                &RetrieveMethod::Vec,
                &mem_path,
                &question,
                &related_questions_vec,
                deep,
            )
            .await
        }));
    }

    let mut skills_map: HashMap<String, db::mem::skill::SkillEntity> = HashMap::new();
    for handle in skill_retrievers {
        for skill in handle.await?? {
            skills_map.insert(skill.id.clone(), skill);
        }
    }

    let mut call_tool_tasks: Vec<(ChatCallToolTask, String, String)> = vec![];

    let skills = skills_map.values().cloned().collect::<Vec<_>>();
    if !skills.is_empty() {
        log::debug!("Skills: {:?}", skills);

        call_tool_tasks = invoke_skills(
            &skills,
            question,
            chat_history,
            chat_options.llm_for_chat.as_deref(),
            chat_event_sender.clone(),
        )
        .await?;

        let skill_candidates: Vec<String> = call_tool_tasks
            .iter()
            .map(|(task, result, _time)| {
                json!({
                    "description": task.description,
                    "parameters": task.parameters,
                    "result": result,
                })
                .to_string()
            })
            .collect();
        candidates.extend(skill_candidates);
    }

    log::debug!("Candidates: {:?}", candidates);

    // Generate answer
    let mut chat_completion_options = ChatCompletionOptions::default();
    if chat_options.deep {
        chat_completion_options = chat_completion_options.with_enable_think(true);
    }

    let llm_options = VecOptions(&chat_options.llm_options);
    if let Some(temperature_str) = llm_options.get("temperature") {
        if let Ok(temperature) = temperature_str.parse() {
            chat_completion_options = chat_completion_options.with_temperature(temperature);
        }
    }

    let llm_for_chat: Option<String> = if chat_options.deep {
        if chat_options.llm_for_reasoning.is_some() {
            chat_options.llm_for_reasoning.clone()
        } else {
            db::core::config::get(&db::core::config::ConfigKey::ActiveReasoningLlm).await?
        }
    } else {
        chat_options.llm_for_chat.clone()
    };

    let (result, source) = if !candidates.is_empty() {
        let prompt = make_answer_by_candidates_prompt(
            question,
            &history_questions,
            &candidates.into_iter().collect::<Vec<_>>(),
            chat_options.strict,
        );

        let result = api::llm::chat_completion(
            &prompt,
            chat_history,
            &chat_completion_options,
            llm_for_chat.as_deref(),
            chat_event_sender,
        )
        .await;

        (result, ChatResponseSource::Retrieve)
    } else {
        if chat_options.strict {
            let prompt = make_no_answer_prompt(question);

            let result = api::llm::chat_completion(
                &prompt,
                &[],
                &chat_completion_options,
                llm_for_chat.as_deref(),
                chat_event_sender,
            )
            .await;

            (result, ChatResponseSource::Llm)
        } else {
            let result = api::llm::chat_completion(
                question,
                chat_history,
                &chat_completion_options,
                llm_for_chat.as_deref(),
                chat_event_sender,
            )
            .await;

            (result, ChatResponseSource::Llm)
        }
    };

    match result {
        Ok(message) => {
            let call_tools = call_tool_tasks
                .iter()
                .map(|(task, _result, time)| {
                    json!({
                        "task": task,
                        "time": time
                    })
                })
                .collect::<Vec<_>>();

            let json_str = json!({
                "content": message.content,
                "reasoning": message.reasoning,
                "call_tools": call_tools,
            })
            .to_string();

            {
                let (resp_sender, resp_receiver) = oneshot::channel();
                mem_write_event_sender
                    .send(MemWriteEvent::SetHistoryChatContent {
                        rowid: answer_rowid,
                        content: json_str,
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;
            }

            Ok((message, source))
        }
        Err(err) => {
            match err {
                AiterError::Interrupted(ref content) => {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    mem_write_event_sender
                        .send(MemWriteEvent::SetHistoryChatContent {
                            rowid: answer_rowid,
                            content: content.clone(),
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }
                _ => {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    mem_write_event_sender
                        .send(MemWriteEvent::DeleteHistoryChat {
                            rowid: answer_rowid,
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }
            }

            Err(err)
        }
    }
}

async fn retrieve_contents(
    method: &RetrieveMethod,
    mem_path: &Path,
    question: &str,
    related_questions: &[String],
    deep: bool,
) -> AiterResult<Vec<String>> {
    let mut content_retrievers: Vec<JoinHandle<AiterResult<Vec<String>>>> = vec![];

    {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let question = question.to_string();
        let related_questions = related_questions.to_vec();
        content_retrievers.push(tokio::spawn(async move {
            retrieve_doc_implicit(&method, &mem_path, &question, &related_questions, deep).await
        }));
    }

    {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let question = question.to_string();
        let related_questions = related_questions.to_vec();
        content_retrievers.push(tokio::spawn(async move {
            retrieve_doc_frag(&method, &mem_path, &question, &related_questions, deep).await
        }));
    }

    {
        let method = method.clone();
        let mem_path = mem_path.to_path_buf();
        let question = question.to_string();
        let related_questions = related_questions.to_vec();
        content_retrievers.push(tokio::spawn(async move {
            retrieve_doc_knl(&method, &mem_path, &question, &related_questions, deep).await
        }));
    }

    let mut candidates: HashSet<String> = HashSet::new();
    for handle in content_retrievers {
        candidates.extend(handle.await??);
    }

    Ok(candidates.into_iter().collect())
}

async fn invoke_skills(
    skills: &[db::mem::skill::SkillEntity],
    question: &str,
    chat_history: &[ChatMessage],
    chat_llm_name: Option<&str>,
    chat_event_sender: Option<Sender<ChatEvent>>,
) -> AiterResult<Vec<(ChatCallToolTask, String, String)>> {
    let mut functions: Vec<ChatFunction> = vec![];
    for skill in skills {
        if let Some(tool) = api::tool::get(&skill.tool_id).await? {
            let chat_function = match ToolType::from_str(&tool.r#type)? {
                ToolType::Ahp => chat_function_from_ahp(&tool),
                ToolType::Mcp => chat_function_from_mcp(&tool),
            };

            if let Ok(chat_function) = chat_function {
                functions.push(chat_function);
            }
        }
    }
    log::debug!("Functions: {:?}", functions);

    let function_calls =
        api::llm::chat_function_calls(&functions, question, chat_history, chat_llm_name).await?;
    log::debug!("Function calls: {:?}", function_calls);

    let mut calls: Vec<JoinHandle<AiterResult<(ChatCallToolTask, String, String)>>> = vec![];
    for function_call in function_calls {
        let tool_id = function_call.name;
        let options = function_call.arguments;

        if let Some(tool) = api::tool::get(&tool_id).await? {
            let description = tool.description;

            let tool_parameters: serde_json::Value = serde_json::from_str(&tool.parameters)?;
            let mut parameters: HashMap<String, String> = HashMap::new();
            for (k, v) in &options {
                if let Some(param_description) =
                    tool_parameters["params"][k]["description"].as_str()
                {
                    parameters.insert(param_description.to_string(), v.to_string());
                }
            }

            let task = ChatCallToolTask {
                id: Ulid::new().to_string(),
                tool_id: tool_id.clone(),
                description,
                parameters,
            };

            if let Some(chat_event_sender) = &chat_event_sender {
                chat_event_sender
                    .send(ChatEvent::CallToolStart(task.clone()))
                    .await?;
            }

            calls.push(tokio::spawn(async move {
                let result = api::tool::run(&tool_id, &options).await?;
                let time = now_iso_datetime_string();

                Ok((task, result, time))
            }));
        }
    }

    let mut call_results = vec![];
    for handle in calls {
        // Allow call function tool failed
        if let Ok(handle_result) = handle.await {
            match handle_result {
                Ok((task, result, time)) => {
                    call_results.push((task.clone(), result.clone(), time.clone()));

                    if let Some(chat_event_sender) = &chat_event_sender {
                        let _ = chat_event_sender
                            .send(ChatEvent::CallToolEnd(task.id.to_string(), result, time))
                            .await;
                    }
                }
                Err(err) => {
                    log::error!("Call function failed: {:?}", err);
                }
            }
        }
    }

    Ok(call_results)
}

impl ChatOptions {
    pub fn with_deep(mut self, deep: bool) -> Self {
        self.deep = deep;
        self
    }

    pub fn with_exchange(mut self, exchange: Option<String>) -> Self {
        self.exchange = exchange;
        self
    }

    pub fn with_llm_for_chat(mut self, llm_for_chat: Option<String>) -> Self {
        self.llm_for_chat = llm_for_chat;
        self
    }

    pub fn with_llm_for_reasoning(mut self, llm_for_reasoning: Option<String>) -> Self {
        self.llm_for_reasoning = llm_for_reasoning;
        self
    }

    pub fn with_llm_options(mut self, llm_options: Vec<String>) -> Self {
        self.llm_options = llm_options;
        self
    }

    pub fn with_retrace(mut self, retrace: u64) -> Self {
        self.retrace = retrace;
        self
    }

    pub fn with_session(mut self, session: Option<String>) -> Self {
        self.session = session;
        self
    }

    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}
