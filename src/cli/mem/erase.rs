use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemEraseCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,
}

impl MemEraseCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if cli::confirm_action(&format!(
            "Are you sure you want to erase all memories of {}?",
            cli::display_ai(self.ai.as_deref(), false)
        )) {
            if let Err(err) = api::mem::erase(self.ai.as_deref()).await {
                println!("{}", err.to_string().red());
            } else {
                println!(
                    "All memories of {} have been erased",
                    cli::display_ai(self.ai.as_deref(), false)
                );
            }
        }
    }
}
