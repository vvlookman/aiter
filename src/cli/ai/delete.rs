use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct AiDeleteCommand {
    name: String,
}

impl AiDeleteCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(Some(self.name.as_str())).await {
            return;
        }

        if cli::confirm_action(&format!(
            "Are you sure you want to delete AI '{}'?",
            self.name.yellow()
        )) {
            if let Err(err) = api::ai::delete(&self.name).await {
                println!("{}", err.to_string().red());
            } else {
                println!("AI '{}' has been deleted", self.name);
            }
        }
    }
}
