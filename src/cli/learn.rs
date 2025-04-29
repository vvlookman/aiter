use std::path::PathBuf;

use aiter::{api::learn::*, *};
use bytesize::ByteSize;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{sync::mpsc, time::Duration};

use crate::cli;

#[derive(clap::Args)]
pub struct LearnCommand {
    #[arg(
        long = "ai",
        value_name = "AI",
        help = "The character performing the operation, it is the alias of `@<AI>`"
    )]
    ai: Option<String>,

    #[arg(
        long = "digest-concurrent",
        default_value = "8",
        help = "Concurrency of digesting, default value is 8"
    )]
    digest_concurrent: usize,

    #[arg(
        long = "digest-deep",
        help = "Deeply digest and understand the content"
    )]
    digest_deep: bool,

    #[arg(
        short = 'f',
        long = "format",
        help = "Specify the data source format rather than judging by suffix, currently supported formats: csv/docx/epub/md/pdf/txt/xlsx"
    )]
    format: Option<String>,

    #[arg(
        short = 'k',
        long = "keep",
        help = "Keep the original document to pull later"
    )]
    keep: bool,

    #[clap(required = true, help = "Source file")]
    source: PathBuf,
}

impl LearnCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if let Some((doc_id, need_digest)) = self.exec_read().await {
            if need_digest {
                self.exec_digest(&doc_id).await;
            }
        }
    }

    async fn exec_read(&self) -> Option<(String, bool)> {
        let filename = utils::fs::extract_filename_from_path(&self.source);
        let bot_name = self.ai.clone().unwrap_or("~".to_string()).cyan();

        let (event_sender, mut event_receiver) = mpsc::channel::<ReadEvent>(CHANNEL_BUFFER_DEFAULT);

        let ai = self.ai.clone();
        let path_buf = self.source.to_path_buf();
        let format = self.format.clone();
        let keep = self.keep;

        let mem_write_event_sender = api::mem::spawn_mem_write(ai.as_deref())
            .await
            .expect("Spawn mem write error");

        let handle = tokio::spawn(async move {
            read_doc(
                ai.as_deref(),
                &path_buf,
                None,
                format.as_deref(),
                keep,
                mem_write_event_sender,
                Some(event_sender),
            )
            .await
        });

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.set_message(format!("[{}] [{}]", bot_name, filename));
        spinner.enable_steady_tick(Duration::from_millis(100));

        while let Some(event) = event_receiver.recv().await {
            match event {
                ReadEvent::Progress(message) => {
                    spinner.set_message(format!("[{}] [{}] {}", bot_name, filename, message));
                }
            }
        }

        match handle.await {
            Ok(Ok(result)) => {
                if result.doc_exists {
                    spinner.finish_with_message(format!(
                        "[{}] [{}] Duplicate content exists in \"{}\" {}",
                        bot_name,
                        filename,
                        result.doc_id.yellow(),
                        "[R] ✔".green()
                    ));

                    Some((result.doc_id, false))
                } else {
                    spinner.finish_with_message(format!(
                        "[{}] [{}] {}",
                        bot_name,
                        filename,
                        "[R] ✔".green()
                    ));

                    Some((result.doc_id, true))
                }
            }
            Ok(Err(err)) => {
                spinner.finish_with_message(format!(
                    "[{}] [{}] {}",
                    bot_name,
                    filename,
                    err.to_string().red()
                ));

                None
            }
            Err(err) => {
                spinner.finish_with_message(format!(
                    "[{}] [{}] {}",
                    bot_name,
                    filename,
                    err.to_string().red()
                ));

                None
            }
        }
    }

    async fn exec_digest(&self, doc_id: &str) {
        let filename = utils::fs::extract_filename_from_path(&self.source);
        let bot_name = self.ai.clone().unwrap_or("~".to_string()).cyan();

        let (event_sender, mut event_receiver) =
            mpsc::channel::<DigestEvent>(CHANNEL_BUFFER_DEFAULT);

        let ai = self.ai.clone();
        let doc_id = doc_id.to_string();
        let options = DigestOptions::default()
            .with_concurrent(self.digest_concurrent)
            .with_deep(self.digest_deep);

        let mem_write_event_sender = api::mem::spawn_mem_write(ai.as_deref())
            .await
            .expect("Spawn mem write error");

        let handle = tokio::spawn(async move {
            digest_doc(
                ai.as_deref(),
                &doc_id,
                &options,
                mem_write_event_sender,
                Some(event_sender),
            )
            .await
        });

        let spinner = ProgressBar::new_spinner();
        spinner
            .set_style(ProgressStyle::with_template("{msg} {spinner:.cyan} [{elapsed}]").unwrap());
        spinner.set_message(format!("[{}] [{}]", bot_name, filename));
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
                let (seg_todo, seg_done) = result.seg_size;
                let (frag_todo, frag_done) = result.frag_size;
                if seg_todo > seg_done || frag_todo > frag_done {
                    spinner.finish_with_message(format!(
                        "[{}] [{}] {}",
                        bot_name,
                        filename,
                        format!(
                            "About {} of content not digested, run `aiter digest -r` command to try to fix. If it can't be processed after repeated attempts, it is usually because it contains content that LLM refuses to process, so please ignore it.", ByteSize((seg_todo - seg_done + frag_todo - frag_done).try_into().unwrap_or(0))
                        )
                        .yellow()
                    ));
                } else {
                    spinner.finish_with_message(format!(
                        "[{}] [{}] {}",
                        bot_name,
                        filename,
                        "[D] ✔".green()
                    ));
                }
            }
            Ok(Err(err)) => {
                spinner.finish_with_message(format!(
                    "[{}] [{}] {}",
                    bot_name,
                    filename,
                    err.to_string().red()
                ));
            }
            Err(err) => {
                spinner.finish_with_message(format!(
                    "[{}] [{}] {}",
                    bot_name,
                    filename,
                    err.to_string().red()
                ));
            }
        }
    }
}
