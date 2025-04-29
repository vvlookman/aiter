use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::{
    sync::{mpsc::Sender, Semaphore},
    task,
    task::JoinHandle,
};

use crate::{
    api,
    api::learn::DigestEvent,
    error::AiterResult,
    llm::{
        prompt::{
            extract::{make_extract_implicit_knowledges_prompt, make_extract_questions_prompt},
            generate::make_fix_json_prompt,
            summarize::make_summarize_text_prompt,
        },
        ChatCompletionOptions,
    },
    utils::{
        markdown::extract_code_block,
        text::{split_by_max_tokens, truncate_format},
    },
    Tokenizer, LLM_CHAT_TEMPERATURE_STABLE, TRUNCATE_LOG_MESSAGE, TRUNCATE_PROGRESS_MESSAGE,
};

pub async fn extract_implicit_knowledges(
    text: &str,
    refers: &[String],
) -> AiterResult<QuestionsMap> {
    let mut questions_map: QuestionsMap = HashMap::new();

    let prompt = make_extract_implicit_knowledges_prompt(text, refers);
    let json_text = extract_code_block(
        &api::llm::chat_completion(
            &prompt,
            &[],
            &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
            None,
            None,
        )
        .await?
        .content,
    );
    if let Ok(map) = serde_json::from_str::<QuestionsMap>(&json_text) {
        questions_map.extend(map);
    } else {
        let prompt = make_fix_json_prompt(&json_text);
        let fixed_json_text = extract_code_block(
            &api::llm::chat_completion(
                &prompt,
                &[],
                &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
                None,
                None,
            )
            .await?
            .content,
        );
        if let Ok(map) = serde_json::from_str::<QuestionsMap>(&fixed_json_text) {
            questions_map.extend(map);
        } else {
            log::debug!(
                "LLM extract implicit knowledges failed: {}\nParse JSON error: {}",
                truncate_format(text, TRUNCATE_LOG_MESSAGE, false),
                json_text
            );
        }
    }

    Ok(questions_map)
}

pub async fn extract_implicit_knowledges_across_texts(
    source: &str,
    texts: &[String],
    refers: &[String],
    max_tokens: usize,
    tokenizer: &Tokenizer,
    concurrent: usize,
    event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<QuestionsMap> {
    let semaphore = Arc::new(Semaphore::new(concurrent.max(1)));
    let mut handles: Vec<JoinHandle<AiterResult<QuestionsMap>>> = vec![];

    let window_texts = split_by_max_tokens(&texts.join("\n"), max_tokens, tokenizer);
    for window_text in window_texts {
        if let Ok(permit) = semaphore.clone().acquire_owned().await {
            if let Some(event_sender) = &event_sender {
                let _ = event_sender
                    .send(DigestEvent::Progress(truncate_format(
                        &format!("[{}] {}", &source, &window_text),
                        TRUNCATE_PROGRESS_MESSAGE,
                        true,
                    )))
                    .await;
            }

            let refers = refers.to_vec();

            let handle = task::spawn(async move {
                let result = extract_implicit_knowledges(&window_text, &refers).await?;

                drop(permit);
                Ok(result)
            });

            handles.push(handle);
        }
    }

    let mut questions_map = QuestionsMap::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => {
                questions_map.extend(result);
            }
            Ok(Err(err)) => return Err(err),
            Err(err) => return Err(err.into()),
        }
    }

    Ok(questions_map)
}

pub async fn extract_questions(text: &str, refers: &[String]) -> AiterResult<Vec<String>> {
    let prompt = make_extract_questions_prompt(text, refers);
    let json_text = extract_code_block(
        &api::llm::chat_completion(
            &prompt,
            &[],
            &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
            None,
            None,
        )
        .await?
        .content,
    );
    if let Ok(vec) = serde_json::from_str::<Vec<String>>(&json_text) {
        let questions: HashSet<String> = vec.into_iter().collect();
        return Ok(questions.into_iter().collect());
    } else {
        let prompt = make_fix_json_prompt(&json_text);
        let fixed_json_text = extract_code_block(
            &api::llm::chat_completion(
                &prompt,
                &[],
                &ChatCompletionOptions::default().with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
                None,
                None,
            )
            .await?
            .content,
        );
        if let Ok(vec) = serde_json::from_str::<Vec<String>>(&fixed_json_text) {
            let questions: HashSet<String> = vec.into_iter().collect();
            return Ok(questions.into_iter().collect());
        } else {
            log::debug!(
                "LLM extract questions failed: {}\nParse JSON error: {}",
                truncate_format(text, TRUNCATE_LOG_MESSAGE, false),
                json_text
            );
        }
    }

    Ok(vec![])
}

pub async fn summarize_across_texts(
    source: &str,
    texts: &[String],
    refers: &[String],
    max_tokens: usize,
    tokenizer: &Tokenizer,
    concurrent: usize,
    event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<Vec<String>> {
    let semaphore = Arc::new(Semaphore::new(concurrent.max(1)));
    let mut handles: Vec<JoinHandle<AiterResult<(usize, String)>>> = vec![];

    let window_texts = split_by_max_tokens(&texts.join("\n"), max_tokens, tokenizer);
    for (i, window_text) in window_texts.into_iter().enumerate() {
        if let Ok(permit) = semaphore.clone().acquire_owned().await {
            if let Some(event_sender) = &event_sender {
                let _ = event_sender
                    .send(DigestEvent::Progress(truncate_format(
                        &format!("[{}] {}", &source, &window_text),
                        TRUNCATE_PROGRESS_MESSAGE,
                        true,
                    )))
                    .await;
            }

            let refers = refers.to_vec();

            let handle = task::spawn(async move {
                let prompt = make_summarize_text_prompt(&window_text, &refers);
                let summary = extract_code_block(
                    &api::llm::chat_completion(
                        &prompt,
                        &[],
                        &ChatCompletionOptions::default()
                            .with_temperature(LLM_CHAT_TEMPERATURE_STABLE),
                        None,
                        None,
                    )
                    .await?
                    .content,
                );

                drop(permit);
                Ok((i, summary))
            });

            handles.push(handle);
        }
    }

    let mut summaries: Vec<(usize, String)> = vec![];

    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => {
                summaries.push(result);
            }
            Ok(Err(err)) => return Err(err),
            Err(err) => return Err(err.into()),
        }
    }

    summaries.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(summaries.into_iter().map(|(_, summary)| summary).collect())
}

type QuestionsMap = HashMap<String, Vec<String>>;
