use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct LlmRenameCommand {
    name: String,

    new_name: String,
}

impl LlmRenameCommand {
    pub async fn exec(&self) {
        match api::llm::get_by_name(&self.name).await {
            Ok(row) => {
                if row.is_none() {
                    println!("LLM '{}' not exists", self.name.red());
                    return;
                }
            }
            Err(err) => {
                println!("{}", err.to_string().red());
                return;
            }
        }

        let new_name = self.new_name.trim();

        if cli::confirm_action(&format!(
            "Are you sure you want to rename LLM '{}' to '{}'?",
            self.name.yellow(),
            new_name.green()
        )) {
            if let Err(err) = api::llm::rename(&self.name, new_name).await {
                println!("{}", err.to_string().red());
            } else {
                println!("LLM '{}' has been renamed to '{}'", self.name, new_name);
            }
        }
    }
}
