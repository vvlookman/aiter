use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct LlmActiveCommand {
    #[arg(
        short = 't',
        long = "type",
        default_value = "chat",
        help = "LLM provider's type, the default value is chat"
    )]
    r#type: String,

    name: String,
}

impl LlmActiveCommand {
    pub async fn exec(&self) {
        if !cli::llm::is_type_valid(&self.r#type) {
            return;
        }

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

        if let Err(err) = api::llm::active(&self.r#type, &self.name).await {
            println!("{}", err.to_string().red());
        } else {
            println!("LLM '{}' has been activated", self.name.green());
        }
    }
}
