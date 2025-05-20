use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    CHAT_HISTORY_LIMIT,
    api::get_mem_path,
    chat,
    chat::stream_chat,
    db,
    db::mem::MemWriteEvent,
    error::AiterResult,
    llm,
    llm::{ChatMessage, Role},
};

pub type ChatCompletionStream = llm::ChatCompletionStream;
pub type ChatOptions = chat::ChatOptions;
pub type HistoryChatEntity = db::mem::history_chat::HistoryChatEntity;

pub async fn chat(
    ai_name: Option<&str>,
    question: &str,
    chat_options: &ChatOptions,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<ChatCompletionStream> {
    let mem_path = get_mem_path(ai_name).await?;

    let max_history = chat_options.retrace.min(CHAT_HISTORY_LIMIT);
    let chat_history: Vec<ChatMessage> =
        db::mem::history_chat::retrace(&mem_path, max_history, chat_options.session.as_deref())
            .await?
            .into_iter()
            .map(|m| m.into())
            .collect();
    if !chat_history.is_empty() {
        log::debug!("Retrace history: {chat_history:?}");
    }

    {
        let (resp_sender, resp_receiver) = oneshot::channel();
        mem_write_event_sender
            .send(MemWriteEvent::InsertHistoryChat {
                role: Role::User.to_string(),
                content: question.to_string(),
                exchange: chat_options.exchange.clone(),
                session: chat_options.session.clone(),
                resp_sender,
            })
            .await?;
        let _ = resp_receiver.await?;
    }

    let answer_rowid = {
        let (resp_sender, resp_receiver) = oneshot::channel();
        mem_write_event_sender
            .send(MemWriteEvent::InsertHistoryChat {
                role: Role::Bot.to_string(),
                content: "".to_string(),
                exchange: chat_options.exchange.clone(),
                session: chat_options.session.clone(),
                resp_sender,
            })
            .await?;
        resp_receiver.await??
    };

    stream_chat(
        &mem_path,
        answer_rowid,
        question,
        chat_options,
        &chat_history,
        mem_write_event_sender,
    )
    .await
}

pub async fn clear(
    _ai_name: Option<&str>,
    session: Option<&str>,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<()> {
    let (resp_sender, resp_receiver) = oneshot::channel();
    mem_write_event_sender
        .send(MemWriteEvent::DeleteHistoryChatBySession {
            session: session.map(|s| s.to_string()),
            resp_sender,
        })
        .await?;
    let _ = resp_receiver.await?;

    Ok(())
}

pub async fn delete(
    _ai_name: Option<&str>,
    session: Option<&str>,
    exchange: &str,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<()> {
    let (resp_sender, resp_receiver) = oneshot::channel();
    mem_write_event_sender
        .send(MemWriteEvent::DeleteHistoryChatBySessionExchange {
            session: session.map(|s| s.to_string()),
            exchange: exchange.to_string(),
            resp_sender,
        })
        .await?;
    let _ = resp_receiver.await?;

    Ok(())
}

pub async fn history(
    ai_name: Option<&str>,
    session: Option<&str>,
) -> AiterResult<Vec<HistoryChatEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::history_chat::retrace(&mem_path, CHAT_HISTORY_LIMIT, session).await
}
