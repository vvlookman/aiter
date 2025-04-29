use aiter::{error::AiterResult, *};
use bytesize::ByteSize;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemVacuumCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,
}

impl MemVacuumCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        match self.exec_vacuum().await {
            Ok((size_before, size_after)) => {
                println!(
                    "Memories of {} have been vacuumed from {} to {}",
                    cli::display_ai(self.ai.as_deref(), false),
                    ByteSize(size_before),
                    ByteSize(size_after)
                );
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }

    async fn exec_vacuum(&self) -> AiterResult<(u64, u64)> {
        let size_before = api::mem::stats(self.ai.as_deref()).await?;
        api::mem::vacuum(self.ai.as_deref()).await?;
        let size_after = api::mem::stats(self.ai.as_deref()).await?;

        Ok((size_before, size_after))
    }
}
