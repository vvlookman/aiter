use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct LlmDeleteCommand {
    name: String,
}

impl LlmDeleteCommand {
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

        if cli::confirm_action(&format!(
            "Are you sure you want to delete LLM '{}'?",
            self.name.yellow()
        )) {
            if let Err(err) = api::llm::delete(&self.name).await {
                println!("{}", err.to_string().red());
            } else {
                println!("LLM '{}' has been deleted", self.name);
            }
        }
    }
}
