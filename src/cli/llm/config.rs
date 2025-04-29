use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct LlmConfigCommand {
    #[arg(
        short = 'O',
        long = "option",
        help = "LLM provider's option, e.g. -O base_url:https://api.openai.com/v1 -O api_key:sk-xxx -O model:gpt-3.5-turbo"
    )]
    options: Vec<String>,

    #[arg(
        short = 'p',
        long = "protocol",
        help = "LLM provider's protocol, e.g. -p openai"
    )]
    protocol: Option<String>,

    #[arg(short = 't', long = "type", help = "LLM provider's type, e.g. -t chat")]
    r#type: Option<String>,

    name: String,
}

impl LlmConfigCommand {
    pub async fn exec(&self) {
        let name = &self.name.trim();
        if name.is_empty() {
            println!("Please specify the name of LLM provider");
            return;
        }

        if let Some(protocol) = &self.protocol {
            if !api::llm::SUPPORTED_PROTOCOLS.contains(&protocol.as_str()) {
                println!(
                    "Invalid protocol '{}', available values: {}",
                    protocol,
                    api::llm::SUPPORTED_PROTOCOLS.join("/")
                );
                return;
            }
        }

        if let Some(r#type) = &self.r#type {
            if !cli::llm::is_type_valid(r#type.as_str()) {
                return;
            }
        }

        let options_map = VecOptions(&self.options).into_map();

        if let Err(err) = api::llm::config(
            name,
            self.r#type.as_deref(),
            self.protocol.as_deref(),
            &options_map,
        )
        .await
        {
            println!("{}", err.to_string().red());
        } else {
            println!(
                "LLM '{}' has been configured successfully",
                self.name.green()
            );
        }
    }
}
