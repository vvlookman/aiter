//! Short-term memory

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use tokio::{
    sync::mpsc::Sender,
    task,
    task::JoinHandle,
    time::{sleep, Duration},
};

use crate::{
    api, content,
    content::{
        doc::{sheet::SheetDoc, DocContentType},
        seg::SegContentType,
    },
    db::mem::get_mem_tokenizer,
    error::AiterResult,
    learn::{DigestEvent, *},
    llm::{
        prompt::summarize::{make_summarize_sheet_prompt, make_summarize_text_prompt},
        ChatCompletionOptions,
    },
    utils::{
        markdown::extract_code_block,
        text::{to_tokens, truncate_format},
    },
    FILTER_INFORMATIVE_TOKENS, LLM_CHAT_TEMPERATURE_STABLE, SPLIT_TOKENS_OF_SEG,
    TRUNCATE_PROGRESS_MESSAGE,
};

pub struct DocDigestor {
    mem_path: PathBuf,
    doc_id: String,
    mem_write_event_sender: Sender<MemWriteEvent>,
    progress_sender: Option<Sender<DigestEvent>>,

    doc_meta: DashMap<String, String>,
    doc_refers: DashSet<String>,
    seg_summaries_cache: Arc<DashMap<String, Option<String>>>,
}

pub struct DocDigested {
    pub part_todo: usize,
    pub part_done: usize,
    pub seg_todo: usize,
    pub seg_done: usize,
    pub frag_todo: usize,
    pub frag_done: usize,
}

impl DocDigestor {
    pub fn new(
        mem_path: &Path,
        doc_id: &str,
        mem_write_event_sender: Sender<MemWriteEvent>,
        progress_sender: Option<Sender<DigestEvent>>,
    ) -> Self {
        Self {
            mem_path: mem_path.to_path_buf(),
            doc_id: doc_id.to_string(),
            mem_write_event_sender: mem_write_event_sender.clone(),
            progress_sender: progress_sender.clone(),

            doc_meta: DashMap::new(),
            doc_refers: DashSet::new(),
            seg_summaries_cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn digest_frags(&self, concurrent: usize) -> AiterResult<(usize, usize)> {
        let frags_status: Arc<DashMap<String, (usize, bool)>> = Arc::new(DashMap::new());

        let mut handles: Vec<JoinHandle<AiterResult<()>>> = vec![];
        for i in 0..concurrent.max(1) {
            let mem_path = self.mem_path.to_path_buf();
            let doc_id = self.doc_id.clone();
            let mem_write_event_sender = self.mem_write_event_sender.clone();
            let progress_sender = self.progress_sender.clone();

            let doc_source = self
                .doc_meta
                .get("source")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let doc_refers: Vec<String> = self
                .doc_refers
                .iter()
                .map(|entry| entry.key().clone())
                .collect();
            let doc_context = self
                .doc_meta
                .get("context")
                .map(|v| v.to_string())
                .unwrap_or_default();

            let frags_status = Arc::clone(&frags_status);
            let seg_summaries_cache = Arc::clone(&self.seg_summaries_cache);

            let handle: JoinHandle<AiterResult<()>> = task::spawn(async move {
                sleep(Duration::from_secs(i.try_into().unwrap_or(0))).await;

                while let Some(frag) = doc_frag::get_not_digested(&mem_path, &doc_id).await? {
                    let frag_id = frag.id.to_string();
                    let frag_content =
                        content::frag::decode_content(&frag.content, &frag.content_type)?;
                    let frag_text = frag_content.to_string();
                    let frag_size = frag_text.len();
                    let seg_id = frag.seg_id.to_string();

                    if let Some(progress_sender) = &progress_sender {
                        let _ = progress_sender
                            .send(DigestEvent::Progress(truncate_format(
                                &format!("[{}] {}", &doc_source, &frag_text),
                                TRUNCATE_PROGRESS_MESSAGE,
                                true,
                            )))
                            .await;
                    }

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocFragDigestStart {
                                frag_id: frag_id.clone(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    let process = async || {
                        let mut refers = doc_refers.clone();

                        if let Some(seg_summary) = seg_summaries_cache.get(&seg_id) {
                            if let Some(seg_summary) = &*seg_summary {
                                refers.push(seg_summary.clone());
                            }
                        } else {
                            if let Some(seg) = doc_seg::get(&mem_path, &seg_id).await? {
                                seg_summaries_cache.insert(seg_id.to_string(), seg.summary.clone());

                                if let Some(seg_summary) = seg.summary {
                                    refers.push(seg_summary);
                                }
                            }
                        };

                        let mut doc_ref = HashMap::new();
                        doc_ref.insert("seg_id".to_string(), frag.seg_id.to_string());
                        doc_ref.insert("frag_id".to_string(), frag.id.to_string());

                        let questions = utils::extract_questions(&frag_text, &refers).await?;
                        let doc_knls = questions
                            .iter()
                            .map(|question| {
                                doc_knl::DocKnl::new(&doc_id, doc_ref.clone(), question)
                            })
                            .collect::<Vec<_>>();

                        {
                            let (resp_sender, resp_receiver) = oneshot::channel();
                            mem_write_event_sender
                                .send(MemWriteEvent::UpsertDocKnls {
                                    doc_knls,
                                    context: doc_context.clone(),
                                    resp_sender,
                                })
                                .await?;
                            let _ = resp_receiver.await?;
                        }

                        AiterResult::Ok(())
                    };

                    frags_status.insert(frag_id.clone(), (frag_size, false));

                    let process_result = process().await;
                    let success = process_result.is_ok();

                    frags_status.insert(frag_id.clone(), (frag_size, success));

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocFragDigestEnd {
                                frag_id: frag_id.clone(),
                                success,
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    if let Err(err) = &process_result {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocFragDigestError {
                                frag_id: frag_id.clone(),
                                error: err.to_string(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }
                }

                Ok(())
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(err) = handle.await {
                return Err(err.into());
            }
        }

        let size_todo = frags_status.par_iter().map(|v| v.0).sum();
        let size_done = frags_status.par_iter().filter(|v| v.1).map(|v| v.0).sum();

        Ok((size_todo, size_done))
    }

    pub async fn digest_parts(&self, concurrent: usize) -> AiterResult<(usize, usize)> {
        let parts_status: Arc<DashMap<String, bool>> = Arc::new(DashMap::new());

        let mut handles: Vec<JoinHandle<AiterResult<()>>> = vec![];
        for i in 0..concurrent.max(1) {
            let mem_path = self.mem_path.to_path_buf();
            let doc_id = self.doc_id.clone();
            let mem_write_event_sender = self.mem_write_event_sender.clone();
            let progress_sender = self.progress_sender.clone();

            let doc_content_type = self.doc_meta.get("content_type").map(|v| v.to_string());
            let doc_source = self
                .doc_meta
                .get("source")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let doc_refers: Vec<String> = self
                .doc_refers
                .iter()
                .map(|entry| entry.key().clone())
                .collect();
            let doc_context = self
                .doc_meta
                .get("context")
                .map(|v| v.to_string())
                .unwrap_or_default();

            let parts_status = Arc::clone(&parts_status);

            let handle: JoinHandle<AiterResult<()>> = task::spawn(async move {
                sleep(Duration::from_secs(i.try_into().unwrap_or(0))).await;

                while let Some(part) = doc_part::get_not_digested(&mem_path, &doc_id).await? {
                    let part_id = part.id.to_string();
                    let part_index = part.index;

                    let seg_summaries = doc_seg::list_summary_by_part(&mem_path, &doc_id, &part.id)
                        .await?
                        .into_iter()
                        .map(|(_, summary)| summary)
                        .collect::<Vec<_>>();
                    if seg_summaries.is_empty() {
                        {
                            let (resp_sender, resp_receiver) = oneshot::channel();
                            mem_write_event_sender
                                .send(MemWriteEvent::SetDocPartDigestEnd {
                                    part_id: part.id.clone(),
                                    success: true,
                                    resp_sender,
                                })
                                .await?;
                            let _ = resp_receiver.await?;
                        }

                        continue;
                    }

                    let mut refers: Vec<String> = doc_refers.clone();
                    if let Some(title) = &part.title {
                        refers.push(format!(
                            "这部分内容的标题为`{}`，其中可能包含概括这部分内容的关键信息",
                            title
                        ));
                    }

                    if let Some(progress_sender) = &progress_sender {
                        let _ = progress_sender
                            .send(DigestEvent::Progress(truncate_format(
                                &format!("[{}] {}", &doc_source, &part.title.unwrap_or_default()),
                                TRUNCATE_PROGRESS_MESSAGE,
                                true,
                            )))
                            .await;
                    }

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocPartDigestStart {
                                part_id: part_id.clone(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    let process = async || {
                        let tokenizer = get_mem_tokenizer(&mem_path);

                        let mut part_summary = String::new();

                        if seg_summaries.len() > 1 {
                            let summaries = utils::summarize_across_texts(
                                &doc_source,
                                &seg_summaries,
                                &refers,
                                SPLIT_TOKENS_OF_SEG,
                                &tokenizer,
                                1,
                                progress_sender.clone(),
                            )
                            .await?;

                            part_summary.push_str(&summaries.join("\n\n"));

                            let questions_map = utils::extract_implicit_knowledges_across_texts(
                                &doc_source,
                                &summaries,
                                &refers,
                                SPLIT_TOKENS_OF_SEG,
                                &tokenizer,
                                1,
                                progress_sender.clone(),
                            )
                            .await?;
                            for (text, questions) in questions_map {
                                let implicit = doc_implicit::DocImplicit::new(&doc_id, &text);

                                let mut doc_ref = HashMap::new();
                                doc_ref.insert("implicit_id".to_string(), implicit.id.to_string());

                                let doc_knls = questions
                                    .iter()
                                    .map(|question| {
                                        doc_knl::DocKnl::new(&doc_id, doc_ref.clone(), question)
                                    })
                                    .collect::<Vec<_>>();

                                {
                                    let (resp_sender, resp_receiver) = oneshot::channel();
                                    mem_write_event_sender
                                        .send(MemWriteEvent::UpsertDocImplicit {
                                            doc_implicit: implicit,
                                            context: doc_context.clone(),
                                            resp_sender,
                                        })
                                        .await?;
                                    let _ = resp_receiver.await?;
                                }

                                {
                                    let (resp_sender, resp_receiver) = oneshot::channel();
                                    mem_write_event_sender
                                        .send(MemWriteEvent::UpsertDocKnls {
                                            doc_knls,
                                            context: doc_context.clone(),
                                            resp_sender,
                                        })
                                        .await?;
                                    let _ = resp_receiver.await?;
                                }
                            }
                        } else {
                            if let Some(summary) = seg_summaries.first() {
                                part_summary.push_str(summary);
                            }
                        };

                        if !part_summary.is_empty()
                            && to_tokens(&part_summary, &tokenizer).len()
                                > FILTER_INFORMATIVE_TOKENS
                        {
                            let (resp_sender, resp_receiver) = oneshot::channel();
                            mem_write_event_sender
                                .send(MemWriteEvent::SetDocPartSummary {
                                    part_id: part_id.clone(),
                                    summary: part_summary,
                                    resp_sender,
                                })
                                .await?;
                            let _ = resp_receiver.await?;
                        }

                        // Try to summarize the whole sheets if the content type is Sheet
                        // Ignore any error because the content may exceed the tokens window size
                        if let Some(content_type) = &doc_content_type {
                            if let Ok(DocContentType::Sheet) =
                                content_type.parse::<DocContentType>()
                            {
                                let process_sheet = async || {
                                    if let Some(doc_content) =
                                        doc::get_content(&mem_path, &doc_id).await?
                                    {
                                        let doc = SheetDoc::try_from_bytes(&doc_content)?;
                                        if let Some((_, sheet_data)) =
                                            doc.pages.get(part_index as usize)
                                        {
                                            let sheet_text = sheet_data.to_string();

                                            let prompt =
                                                make_summarize_sheet_prompt(&sheet_text, &refers);
                                            let part_summary = extract_code_block(
                                                &api::llm::chat_completion(
                                                    &prompt,
                                                    &[],
                                                    &ChatCompletionOptions::default()
                                                        .with_temperature(
                                                            LLM_CHAT_TEMPERATURE_STABLE,
                                                        ),
                                                    None,
                                                    None,
                                                )
                                                .await?
                                                .content,
                                            );

                                            if !part_summary.is_empty()
                                                && to_tokens(&part_summary, &tokenizer).len()
                                                    > FILTER_INFORMATIVE_TOKENS
                                            {
                                                {
                                                    let (resp_sender, resp_receiver) =
                                                        oneshot::channel();
                                                    mem_write_event_sender
                                                        .send(MemWriteEvent::SetDocPartSummary {
                                                            part_id: part_id.clone(),
                                                            summary: part_summary.clone(),
                                                            resp_sender,
                                                        })
                                                        .await?;
                                                    let _ = resp_receiver.await?;
                                                }

                                                let questions_map =
                                                    utils::extract_implicit_knowledges(
                                                        &part_summary,
                                                        &refers,
                                                    )
                                                    .await?;
                                                for (text, questions) in questions_map {
                                                    let implicit = doc_implicit::DocImplicit::new(
                                                        &doc_id, &text,
                                                    );

                                                    let mut doc_ref = HashMap::new();
                                                    doc_ref.insert(
                                                        "implicit_id".to_string(),
                                                        implicit.id.to_string(),
                                                    );

                                                    let doc_knls = questions
                                                        .iter()
                                                        .map(|question| {
                                                            doc_knl::DocKnl::new(
                                                                &doc_id,
                                                                doc_ref.clone(),
                                                                question,
                                                            )
                                                        })
                                                        .collect::<Vec<_>>();

                                                    {
                                                        let (resp_sender, resp_receiver) =
                                                            oneshot::channel();
                                                        mem_write_event_sender
                                                            .send(
                                                                MemWriteEvent::UpsertDocImplicit {
                                                                    doc_implicit: implicit,
                                                                    context: doc_context.clone(),
                                                                    resp_sender,
                                                                },
                                                            )
                                                            .await?;
                                                        let _ = resp_receiver.await?;
                                                    }

                                                    {
                                                        let (resp_sender, resp_receiver) =
                                                            oneshot::channel();
                                                        mem_write_event_sender
                                                            .send(MemWriteEvent::UpsertDocKnls {
                                                                doc_knls,
                                                                context: doc_context.clone(),
                                                                resp_sender,
                                                            })
                                                            .await?;
                                                        let _ = resp_receiver.await?;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    AiterResult::Ok(())
                                };

                                let _ = process_sheet().await;
                            }
                        }

                        AiterResult::Ok(())
                    };

                    parts_status.insert(part_id.clone(), false);

                    let process_result = process().await;
                    let success = process_result.is_ok();

                    parts_status.insert(part_id.clone(), success);

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocPartDigestEnd {
                                part_id: part_id.clone(),
                                success,
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    if let Err(err) = &process_result {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocPartDigestError {
                                part_id: part_id.clone(),
                                error: err.to_string(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }
                }

                Ok(())
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(err) = handle.await {
                return Err(err.into());
            }
        }

        let count_todo = parts_status.par_iter().count();
        let count_done = parts_status.par_iter().filter(|v| **v).count();

        Ok((count_todo, count_done))
    }

    pub async fn digest_segs(&self, concurrent: usize) -> AiterResult<(usize, usize)> {
        let segs_status: Arc<DashMap<String, (usize, bool)>> = Arc::new(DashMap::new());

        let mut handles: Vec<JoinHandle<AiterResult<()>>> = vec![];
        for i in 0..concurrent.max(1) {
            let mem_path = self.mem_path.to_path_buf();
            let doc_id = self.doc_id.clone();
            let mem_write_event_sender = self.mem_write_event_sender.clone();
            let progress_sender = self.progress_sender.clone();

            let doc_source = self
                .doc_meta
                .get("source")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let doc_refers: Vec<String> = self
                .doc_refers
                .iter()
                .map(|entry| entry.key().clone())
                .collect();
            let doc_context = self
                .doc_meta
                .get("context")
                .map(|v| v.to_string())
                .unwrap_or_default();

            let segs_status = Arc::clone(&segs_status);

            let handle: JoinHandle<AiterResult<()>> = task::spawn(async move {
                sleep(Duration::from_secs(i.try_into().unwrap_or(0))).await;

                while let Some(seg) = doc_seg::get_not_digested(&mem_path, &doc_id).await? {
                    let seg_id = seg.id.to_string();
                    let seg_content =
                        content::seg::decode_content(&seg.content, &seg.content_type)?;
                    let seg_content_type = seg.content_type.parse::<SegContentType>()?;
                    let seg_text = seg_content.to_string();
                    let seg_size = seg_text.len();

                    if let Some(progress_sender) = &progress_sender {
                        let _ = progress_sender
                            .send(DigestEvent::Progress(truncate_format(
                                &format!("[{}] {}", &doc_source, &seg_text),
                                TRUNCATE_PROGRESS_MESSAGE,
                                true,
                            )))
                            .await;
                    }

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocSegDigestStart {
                                seg_id: seg_id.clone(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    let process = async || {
                        let tokenizer = get_mem_tokenizer(&mem_path);

                        // Summarize
                        {
                            let prompt = match seg_content_type {
                                SegContentType::Sheet => {
                                    make_summarize_sheet_prompt(&seg_text, &doc_refers)
                                }
                                SegContentType::Text => {
                                    make_summarize_text_prompt(&seg_text, &doc_refers)
                                }
                            };

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

                            if !summary.is_empty()
                                && to_tokens(&summary, &tokenizer).len() > FILTER_INFORMATIVE_TOKENS
                            {
                                let (resp_sender, resp_receiver) = oneshot::channel();
                                mem_write_event_sender
                                    .send(MemWriteEvent::SetDocSegSummary {
                                        seg_id: seg_id.clone(),
                                        summary,
                                        resp_sender,
                                    })
                                    .await?;
                                let _ = resp_receiver.await?;
                            }
                        }

                        if let SegContentType::Text = seg_content_type {
                            // Extract implicit knowledges
                            {
                                let questions_map =
                                    utils::extract_implicit_knowledges(&seg_text, &doc_refers)
                                        .await?;
                                for (text, questions) in questions_map {
                                    let implicit = doc_implicit::DocImplicit::new(&doc_id, &text);

                                    let mut doc_ref = HashMap::new();
                                    doc_ref.insert("seg_id".to_string(), seg_id.to_string());
                                    doc_ref
                                        .insert("implicit_id".to_string(), implicit.id.to_string());

                                    let doc_knls = questions
                                        .iter()
                                        .map(|question| {
                                            doc_knl::DocKnl::new(&doc_id, doc_ref.clone(), question)
                                        })
                                        .collect::<Vec<_>>();

                                    {
                                        let (resp_sender, resp_receiver) = oneshot::channel();
                                        mem_write_event_sender
                                            .send(MemWriteEvent::UpsertDocImplicit {
                                                doc_implicit: implicit,
                                                context: doc_context.clone(),
                                                resp_sender,
                                            })
                                            .await?;
                                        let _ = resp_receiver.await?;
                                    }

                                    {
                                        let (resp_sender, resp_receiver) = oneshot::channel();
                                        mem_write_event_sender
                                            .send(MemWriteEvent::UpsertDocKnls {
                                                doc_knls,
                                                context: doc_context.clone(),
                                                resp_sender,
                                            })
                                            .await?;
                                        let _ = resp_receiver.await?;
                                    }
                                }
                            }
                        }

                        AiterResult::Ok(())
                    };

                    segs_status.insert(seg_id.clone(), (seg_size, false));

                    let process_result = process().await;
                    let success = process_result.is_ok();

                    segs_status.insert(seg_id.clone(), (seg_size, success));

                    {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocSegDigestEnd {
                                seg_id: seg_id.clone(),
                                success,
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }

                    if let Err(err) = &process_result {
                        let (resp_sender, resp_receiver) = oneshot::channel();
                        mem_write_event_sender
                            .send(MemWriteEvent::SetDocSegDigestError {
                                seg_id: seg_id.clone(),
                                error: err.to_string(),
                                resp_sender,
                            })
                            .await?;
                        let _ = resp_receiver.await?;
                    }
                }

                Ok(())
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(err) = handle.await {
                return Err(err.into());
            }
        }

        let size_todo = segs_status.par_iter().map(|v| v.0).sum();
        let size_done = segs_status.par_iter().filter(|v| v.1).map(|v| v.0).sum();

        Ok((size_todo, size_done))
    }

    pub async fn load_meta(&self) -> AiterResult<()> {
        if let Some(doc) = doc::get(&self.mem_path, &self.doc_id).await? {
            let doc_context = doc.get_context();

            self.doc_meta
                .insert("source".to_string(), doc.source.to_string());
            self.doc_meta
                .insert("content_type".to_string(), doc.content_type.to_string());
            self.doc_meta
                .insert("context".to_string(), doc_context.to_string());

            self.doc_refers.insert(format!(
                "内容标题为`{}`，其中可能包含概括这部分内容的关键信息",
                &doc_context
            ));
        }

        Ok(())
    }

    pub async fn summarize_doc(&self, concurrent: usize) -> AiterResult<()> {
        let part_summaries = doc_part::list_summary_by_doc(&self.mem_path, &self.doc_id)
            .await?
            .into_iter()
            .map(|(_, summary)| summary)
            .collect::<Vec<_>>();
        if part_summaries.is_empty() {
            {
                let (resp_sender, resp_receiver) = oneshot::channel();
                self.mem_write_event_sender
                    .send(MemWriteEvent::SetDocDigestEnd {
                        doc_id: self.doc_id.clone(),
                        success: true,
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;
            }

            return Ok(());
        }

        let doc_source = self
            .doc_meta
            .get("source")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let doc_refers: Vec<String> = self
            .doc_refers
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        let doc_context = self
            .doc_meta
            .get("context")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let tokenizer = get_mem_tokenizer(&self.mem_path);

        if part_summaries.len() > 1 {
            let summaries = utils::summarize_across_texts(
                &doc_source,
                &part_summaries,
                &doc_refers,
                SPLIT_TOKENS_OF_SEG,
                &tokenizer,
                concurrent,
                self.progress_sender.clone(),
            )
            .await?;

            let doc_summary = summaries.join("\n\n");
            if !doc_summary.is_empty()
                && to_tokens(&doc_summary, &tokenizer).len() > FILTER_INFORMATIVE_TOKENS
            {
                let (resp_sender, resp_receiver) = oneshot::channel();
                self.mem_write_event_sender
                    .send(MemWriteEvent::SetDocSummary {
                        doc_id: self.doc_id.clone(),
                        summary: doc_summary,
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;
            }

            let questions_map = utils::extract_implicit_knowledges_across_texts(
                &doc_source,
                &summaries,
                &doc_refers,
                SPLIT_TOKENS_OF_SEG,
                &tokenizer,
                concurrent,
                self.progress_sender.clone(),
            )
            .await?;
            for (text, questions) in questions_map {
                let implicit = doc_implicit::DocImplicit::new(&self.doc_id, &text);

                let mut doc_ref = HashMap::new();
                doc_ref.insert("implicit_id".to_string(), implicit.id.to_string());

                let doc_knls = questions
                    .iter()
                    .map(|question| doc_knl::DocKnl::new(&self.doc_id, doc_ref.clone(), question))
                    .collect::<Vec<_>>();

                {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    self.mem_write_event_sender
                        .send(MemWriteEvent::UpsertDocImplicit {
                            doc_implicit: implicit,
                            context: doc_context.clone(),
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }

                {
                    let (resp_sender, resp_receiver) = oneshot::channel();
                    self.mem_write_event_sender
                        .send(MemWriteEvent::UpsertDocKnls {
                            doc_knls,
                            context: doc_context.clone(),
                            resp_sender,
                        })
                        .await?;
                    let _ = resp_receiver.await?;
                }
            }
        } else {
            if let Some(summary) = part_summaries.first() {
                let (resp_sender, resp_receiver) = oneshot::channel();
                self.mem_write_event_sender
                    .send(MemWriteEvent::SetDocSummary {
                        doc_id: self.doc_id.clone(),
                        summary: summary.to_string(),
                        resp_sender,
                    })
                    .await?;
                let _ = resp_receiver.await?;
            }
        }

        Ok(())
    }
}
