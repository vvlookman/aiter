use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use dashmap::DashMap;
use tokio::sync::{mpsc, mpsc::Sender, oneshot};

use crate::{
    error::AiterResult, Tokenizer, CHANNEL_BUFFER_LARGE, CURRENT_SIGNATURE_DIMS, CURRENT_TOKENIZER,
};

pub mod doc;
pub mod doc_frag;
pub mod doc_implicit;
pub mod doc_knl;
pub mod doc_part;
pub mod doc_seg;
pub mod history_chat;
pub mod meta;
pub mod skill;

pub static MEM_SIGNATURE_DIMS_MAP: LazyLock<DashMap<PathBuf, usize>> = LazyLock::new(DashMap::new);
pub static MEM_TOKENIZER_MAP: LazyLock<DashMap<PathBuf, Tokenizer>> = LazyLock::new(DashMap::new);

#[derive(strum::Display, Debug)]
pub enum MemWriteEvent {
    DeleteDoc {
        doc_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    DeleteHistoryChat {
        rowid: i64,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    DeleteHistoryChatAll {
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    DeleteHistoryChatBySession {
        session: Option<String>,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    DeleteHistoryChatBySessionExchange {
        session: Option<String>,
        exchange: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    DeleteSkill {
        skill_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    InsertDoc {
        doc: doc::Doc,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    InsertHistoryChat {
        role: String,
        content: String,
        exchange: Option<String>,
        session: Option<String>,
        resp_sender: oneshot::Sender<AiterResult<i64>>,
    },

    SetDocDigestEnd {
        doc_id: String,
        success: bool,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocDigestError {
        doc_id: String,
        error: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocDigestStart {
        doc_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocSummary {
        doc_id: String,
        summary: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocFragDigestEnd {
        frag_id: String,
        success: bool,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocFragDigestError {
        frag_id: String,
        error: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocFragDigestStart {
        frag_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocPartDigestEnd {
        part_id: String,
        success: bool,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocPartDigestError {
        part_id: String,
        error: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocPartDigestStart {
        part_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocPartSummary {
        part_id: String,
        summary: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocSegDigestEnd {
        seg_id: String,
        success: bool,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocSegDigestError {
        seg_id: String,
        error: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocSegDigestStart {
        seg_id: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetDocSegSummary {
        seg_id: String,
        summary: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    SetHistoryChatContent {
        rowid: i64,
        content: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    UpsertDocFrag {
        doc_frag: doc_frag::DocFrag,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    UpsertDocImplicit {
        doc_implicit: doc_implicit::DocImplicit,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<Option<String>>>,
    },

    UpsertDocKnls {
        doc_knls: Vec<doc_knl::DocKnl>,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<()>>,
    },

    UpsertDocPart {
        doc_part: doc_part::DocPart,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<Option<String>>>,
    },

    UpsertDocSeg {
        doc_seg: doc_seg::DocSeg,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<Option<String>>>,
    },

    UpsertSkill {
        skill: skill::Skill,
        context: String,
        resp_sender: oneshot::Sender<AiterResult<Option<String>>>,
    },
}

pub fn get_mem_signature_dims(path: &Path) -> usize {
    MEM_SIGNATURE_DIMS_MAP
        .get(path)
        .map_or(CURRENT_SIGNATURE_DIMS, |v| *v.value())
}

pub fn get_mem_tokenizer(path: &Path) -> Tokenizer {
    MEM_TOKENIZER_MAP
        .get(path)
        .map_or(CURRENT_TOKENIZER, |v| *v.value())
}

pub fn spawn_mem_write(mem_path: &Path) -> Sender<MemWriteEvent> {
    let (db_write_sender, mut db_write_receiver) =
        mpsc::channel::<MemWriteEvent>(CHANNEL_BUFFER_LARGE);

    let db_path = mem_path.to_path_buf();

    tokio::spawn(async move {
        while let Some(event) = db_write_receiver.recv().await {
            match event {
                MemWriteEvent::DeleteDoc {
                    doc_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc::delete(&db_path, &doc_id).await);
                }

                MemWriteEvent::DeleteHistoryChat { rowid, resp_sender } => {
                    let _ = resp_sender.send(history_chat::delete(&db_path, rowid).await);
                }

                MemWriteEvent::DeleteHistoryChatAll { resp_sender } => {
                    let _ = resp_sender.send(history_chat::delete_all(&db_path).await);
                }

                MemWriteEvent::DeleteHistoryChatBySession {
                    session,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(history_chat::delete_by_session(&db_path, session.as_deref()).await);
                }

                MemWriteEvent::DeleteHistoryChatBySessionExchange {
                    session,
                    exchange,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(
                        history_chat::delete_by_session_exchange(
                            &db_path,
                            session.as_deref(),
                            &exchange,
                        )
                        .await,
                    );
                }

                MemWriteEvent::DeleteSkill {
                    skill_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(skill::delete(&db_path, &skill_id).await);
                }

                MemWriteEvent::InsertDoc { doc, resp_sender } => {
                    let _ = resp_sender.send(doc::insert(&db_path, &doc).await);
                }

                MemWriteEvent::InsertHistoryChat {
                    role,
                    content,
                    exchange,
                    session,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(
                        history_chat::insert(
                            &db_path,
                            &role,
                            &content,
                            exchange.as_deref(),
                            session.as_deref(),
                        )
                        .await,
                    );
                }

                MemWriteEvent::SetDocDigestEnd {
                    doc_id,
                    success,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc::set_digest_end(&db_path, &doc_id, success).await);
                }

                MemWriteEvent::SetDocDigestError {
                    doc_id,
                    error,
                    resp_sender,
                } => {
                    let _ =
                        resp_sender.send(doc::set_digest_error(&db_path, &doc_id, &error).await);
                }

                MemWriteEvent::SetDocDigestStart {
                    doc_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc::set_digest_start(&db_path, &doc_id).await);
                }

                MemWriteEvent::SetDocSummary {
                    doc_id,
                    summary,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc::set_summary(&db_path, &doc_id, &summary).await);
                }

                MemWriteEvent::SetDocFragDigestEnd {
                    frag_id,
                    success,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_frag::set_digest_end(&db_path, &frag_id, success).await);
                }

                MemWriteEvent::SetDocFragDigestError {
                    frag_id,
                    error,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_frag::set_digest_error(&db_path, &frag_id, &error).await);
                }

                MemWriteEvent::SetDocFragDigestStart {
                    frag_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_frag::set_digest_start(&db_path, &frag_id).await);
                }

                MemWriteEvent::SetDocPartDigestEnd {
                    part_id,
                    success,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_part::set_digest_end(&db_path, &part_id, success).await);
                }

                MemWriteEvent::SetDocPartDigestError {
                    part_id,
                    error,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_part::set_digest_error(&db_path, &part_id, &error).await);
                }

                MemWriteEvent::SetDocPartDigestStart {
                    part_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_part::set_digest_start(&db_path, &part_id).await);
                }

                MemWriteEvent::SetDocPartSummary {
                    part_id,
                    summary,
                    resp_sender,
                } => {
                    let _ =
                        resp_sender.send(doc_part::set_summary(&db_path, &part_id, &summary).await);
                }

                MemWriteEvent::SetDocSegDigestEnd {
                    seg_id,
                    success,
                    resp_sender,
                } => {
                    let _ =
                        resp_sender.send(doc_seg::set_digest_end(&db_path, &seg_id, success).await);
                }

                MemWriteEvent::SetDocSegDigestError {
                    seg_id,
                    error,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_seg::set_digest_error(&db_path, &seg_id, &error).await);
                }

                MemWriteEvent::SetDocSegDigestStart {
                    seg_id,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_seg::set_digest_start(&db_path, &seg_id).await);
                }

                MemWriteEvent::SetDocSegSummary {
                    seg_id,
                    summary,
                    resp_sender,
                } => {
                    let _ =
                        resp_sender.send(doc_seg::set_summary(&db_path, &seg_id, &summary).await);
                }

                MemWriteEvent::SetHistoryChatContent {
                    rowid,
                    content,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(history_chat::set_content(&db_path, rowid, &content).await);
                }

                MemWriteEvent::UpsertDocFrag {
                    doc_frag,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_frag::upsert(&db_path, &doc_frag, &context).await);
                }

                MemWriteEvent::UpsertDocImplicit {
                    doc_implicit,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_implicit::upsert(&db_path, &doc_implicit, &context).await);
                }

                MemWriteEvent::UpsertDocKnls {
                    doc_knls,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender
                        .send(doc_knl::upsert_batch(&db_path, &doc_knls, &context).await);
                }

                MemWriteEvent::UpsertDocPart {
                    doc_part,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_part::upsert(&db_path, &doc_part, &context).await);
                }

                MemWriteEvent::UpsertDocSeg {
                    doc_seg,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(doc_seg::upsert(&db_path, &doc_seg, &context).await);
                }

                MemWriteEvent::UpsertSkill {
                    skill,
                    context,
                    resp_sender,
                } => {
                    let _ = resp_sender.send(skill::upsert(&db_path, &skill, &context).await);
                }
            }
        }
    });

    db_write_sender
}
