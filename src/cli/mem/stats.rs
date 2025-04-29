use aiter::*;
use bytesize::ByteSize;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemStatsCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,
}

impl MemStatsCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        match api::mem::stats(self.ai.as_deref()).await {
            Ok(size) => {
                println!(
                    "{} has a total of {} memories",
                    cli::display_ai(self.ai.as_deref(), true),
                    ByteSize(size)
                );
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
