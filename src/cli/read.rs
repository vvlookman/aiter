use std::{fs::read_dir, path::Path};

use aiter::{api::learn::*, *};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use tokio::{sync::mpsc, time::Duration};

use crate::cli;

#[derive(clap::Args)]
pub struct ReadCommand {
    #[arg(
        long = "ai",
        value_name = "AI",
        help = "The character performing the operation, it is the alias of `@<AI>`"
    )]
    ai: Option<String>,

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

    #[clap(required = true, help = "Source file or directory")]
    sources: Vec<String>,
}

impl ReadCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        for source in &self.sources {
            let source_path = Path::new(source);
            if source_path.is_dir() {
                if let Ok(entries) = read_dir(source_path) {
                    let mut entries: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
                    entries.par_sort_by(|a, b| {
                        utils::text::compare_phonetic(
                            &a.file_name().to_string_lossy(),
                            &b.file_name().to_string_lossy(),
                        )
                    });

                    for entry in entries {
                        let entry_path = entry.path();
                        self.exec_read_from_file(&entry_path).await;
                    }
                }
            } else {
                self.exec_read_from_file(source_path).await;
            }
        }
    }

    async fn exec_read_from_file(&self, path: &Path) {
        let filename = utils::fs::extract_filename_from_path(path);
        if filename.starts_with('.') {
            return;
        }

        let bot_name = self.ai.clone().unwrap_or("~".to_string()).cyan();

        let (event_sender, mut event_receiver) = mpsc::channel::<ReadEvent>(CHANNEL_BUFFER_DEFAULT);

        let ai = self.ai.clone();
        let path_buf = path.to_path_buf();
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
                        "✔".green()
                    ));
                } else {
                    spinner.finish_with_message(format!(
                        "[{}] [{}] {}",
                        bot_name,
                        filename,
                        "✔".green()
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
