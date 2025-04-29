use aiter::{api::learn::*, *};
use bytesize::ByteSize;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{sync::mpsc, time::Duration};

use crate::cli;

#[derive(clap::Args)]
pub struct DigestCommand {
    #[arg(
        long = "ai",
        value_name = "AI",
        help = "The character performing the operation, it is the alias of `@<AI>`"
    )]
    ai: Option<String>,

    #[arg(
        short = 'b',
        long = "batch",
        default_value = "2",
        help = "Documents processed simultaneously, default value is 2"
    )]
    batch: usize,

    #[arg(
        short = 'c',
        long = "concurrent",
        default_value = "8",
        help = "Concurrency of processing, default value is 8"
    )]
    concurrent: usize,

    #[arg(
        short = 'd',
        long = "deep",
        help = "Deeply digest and understand the content"
    )]
    deep: bool,

    #[arg(
        short = 'r',
        long = "retry",
        help = "Retry digesting documents if it has undigested content"
    )]
    retry: bool,
}

impl DigestCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        let bot_name = self.ai.clone().unwrap_or("~".to_string()).cyan();

        let (event_sender, mut event_receiver) =
            mpsc::channel::<DigestEvent>(CHANNEL_BUFFER_DEFAULT);

        let ai = self.ai.clone();
        let options = DigestOptions::default()
            .with_batch(self.batch)
            .with_concurrent(self.concurrent)
            .with_deep(self.deep)
            .with_retry(self.retry);

        let mem_write_event_sender = api::mem::spawn_mem_write(ai.as_deref())
            .await
            .expect("Spawn mem write error");

        let handle = tokio::spawn(async move {
            digest(
                ai.as_deref(),
                &options,
                mem_write_event_sender,
                Some(event_sender),
            )
            .await
        });

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.set_message(format!("[{}]", bot_name));
        spinner.enable_steady_tick(Duration::from_millis(100));

        while let Some(event) = event_receiver.recv().await {
            match event {
                DigestEvent::Progress(message) => {
                    spinner.set_message(format!("[{}] {}", bot_name, message));
                }
            }
        }

        match handle.await {
            Ok(Ok(result)) => {
                let (doc_todo, doc_done) = result.doc_count;
                let (seg_todo, seg_done) = result.seg_size;
                let (frag_todo, frag_done) = result.frag_size;
                if doc_todo > doc_done {
                    spinner.finish_with_message(format!(
                        "[{}] {}",
                        bot_name,
                        format!(
                            "A total of {} documents were not been processed correctly, run `aiter digest` command again",
                            doc_todo - doc_done
                        )
                        .yellow()
                    ));
                } else if seg_todo > seg_done || frag_todo > frag_done {
                    spinner.finish_with_message(format!(
                        "[{}] {}",
                        bot_name,
                        format!(
                            "About {} of content not digested, run `aiter digest -r` command to try to fix. If it can't be processed after repeated attempts, it is usually because it contains content that LLM refuses to process, so please ignore it.", ByteSize((seg_todo - seg_done + frag_todo - frag_done).try_into().unwrap_or(0))
                        )
                        .yellow()
                    ));
                } else {
                    spinner.finish_with_message(format!("[{}] {}", bot_name, "✔".green()));
                }
            }
            Ok(Err(err)) => {
                spinner.finish_with_message(format!("[{}] {}", bot_name, err.to_string().red()));
            }
            Err(err) => {
                spinner.finish_with_message(format!("[{}] {}", bot_name, err.to_string().red()));
            }
        }
    }
}
