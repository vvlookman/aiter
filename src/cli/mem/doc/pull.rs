use std::path::PathBuf;

use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemDocPullCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    id: String,

    dest_dir: PathBuf,
}

impl MemDocPullCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        match api::mem::doc::pull(self.ai.as_deref(), &self.id, &self.dest_dir).await {
            Ok(dest) => {
                if let Some(dest) = dest {
                    println!(
                        "Doc '{}' has been pulled to {}",
                        &self.id.green(),
                        &dest.to_string_lossy().green()
                    );
                } else {
                    println!("Original document does not exist");
                }
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
