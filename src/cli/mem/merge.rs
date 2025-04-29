use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemMergeCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    #[arg(
        short = 'f',
        long = "from",
        value_name = "FROM_NAME",
        help = "The name being merged, if not set means that it is being merged from the default TA"
    )]
    from_name: Option<String>,
}

impl MemMergeCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if self.ai == self.from_name {
            println!("{}", "Can not merge self".yellow());
            return;
        }

        if cli::confirm_action(&format!(
            "Are you sure you want to merge all memories from {}?",
            cli::display_ai(self.from_name.as_deref(), false)
        )) {
            if let Err(err) = api::mem::merge(self.ai.as_deref(), self.from_name.as_deref()).await {
                println!("{}", err.to_string().red());
            } else {
                println!(
                    "All memories from {} have been merged successfully",
                    cli::display_ai(self.from_name.as_deref(), false)
                );
            }
        }
    }
}
