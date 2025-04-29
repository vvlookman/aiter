use tokio::sync::{mpsc::Sender, oneshot};

use crate::{db::mem::MemWriteEvent, error::AiterResult};

pub async fn clear(
    _ai_name: Option<&str>,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<()> {
    let (resp_sender, resp_receiver) = oneshot::channel();
    mem_write_event_sender
        .send(MemWriteEvent::DeleteHistoryChatAll { resp_sender })
        .await?;
    let _ = resp_receiver.await?;

    Ok(())
}
