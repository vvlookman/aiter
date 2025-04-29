use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemDocDeleteCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    id: String,
}

impl MemDocDeleteCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if cli::confirm_action(&format!(
            "Are you sure you want to delete doc with ID '{}'?",
            self.id.yellow()
        )) {
            let mem_write_event_sender = api::mem::spawn_mem_write(self.ai.as_deref())
                .await
                .expect("Spawn mem write error");

            match api::mem::doc::delete(self.ai.as_deref(), &self.id, mem_write_event_sender).await
            {
                Ok(doc) => {
                    if let Some(doc) = doc {
                        println!("Doc '{}' has been deleted", doc.source);
                    } else {
                        println!("Doc does not exist");
                    }
                }
                Err(err) => {
                    println!("{}", err.to_string().red());
                }
            }
        }
    }
}
