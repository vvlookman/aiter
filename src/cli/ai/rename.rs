use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct AiRenameCommand {
    name: String,

    new_name: String,
}

impl AiRenameCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(Some(self.name.as_str())).await {
            return;
        }

        let new_name = self.new_name.trim();

        if cli::confirm_action(&format!(
            "Are you sure you want to rename AI '{}' to '{}'?",
            self.name.yellow(),
            new_name.green()
        )) {
            if let Err(err) = api::ai::rename(&self.name, new_name).await {
                println!("{}", err.to_string().red());
            } else {
                println!("AI '{}' has been renamed to '{}'", self.name, new_name);
            }
        }
    }
}
