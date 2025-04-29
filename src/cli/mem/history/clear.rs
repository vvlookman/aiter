use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemHistoryClearCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,
}

impl MemHistoryClearCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if cli::confirm_action("Are you sure you want to clear all history?") {
            let mem_write_event_sender = api::mem::spawn_mem_write(self.ai.as_deref())
                .await
                .expect("Spawn mem write error");

            match api::mem::history::clear(self.ai.as_deref(), mem_write_event_sender).await {
                Ok(_) => {
                    println!("All history has been cleared {}", "✔".green());
                }
                Err(err) => {
                    println!("{}", err.to_string().red());
                }
            }
        }
    }
}
