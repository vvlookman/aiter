use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct InfoCommand;

impl InfoCommand {
    pub async fn exec(&self) {
        let version = api::sys::get_version().await;
        println!("Version: {}", version.cyan().bold());

        let data_path = api::sys::get_data_path().await;
        println!("Data: {}", data_path.cyan().bold());

        let llm_for_chat = api::llm::get_actived_name("chat")
            .await
            .ok()
            .flatten()
            .unwrap_or("".to_string());
        println!("Default Chat LLM: {}", llm_for_chat.cyan().bold());

        let llm_for_reasoning = api::llm::get_actived_name("reasoning")
            .await
            .ok()
            .flatten()
            .unwrap_or("".to_string());
        println!("Default Reasoning LLM: {}", llm_for_reasoning.cyan().bold());
    }
}
