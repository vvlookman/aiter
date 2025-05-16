use std::io::{stdout, Write};

use aiter::{
    api::llm::{ChatCompletionEvent, ChatCompletionOptions, ChatCompletionStream},
    error::AiterResult,
    *,
};
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct LlmTestCommand {
    #[arg(
        short = 'L',
        long = "llm-option",
        help = "Additional option passed to LLM, e.g. -L temperature:0.6"
    )]
    llm_options: Vec<String>,

    #[arg(
        short = 't',
        long = "type",
        default_value = "chat",
        help = "LLM provider's type, the default value is chat, currently supported types: chat/reasoning"
    )]
    r#type: String,

    prompt: String,
}

impl LlmTestCommand {
    pub async fn exec(&self) {
        if !cli::llm::is_type_valid(&self.r#type) {
            return;
        }

        let mut chat_completion_options = ChatCompletionOptions::default();
        let llm_options = VecOptions(&self.llm_options);
        if let Some(temperature_str) = llm_options.get("temperature") {
            if let Ok(temperature) = temperature_str.parse() {
                chat_completion_options = chat_completion_options.with_temperature(temperature);
            }
        }

        let r#type = self.r#type.clone();
        let prompt = self.prompt.clone();

        let result: AiterResult<ChatCompletionStream> = match r#type.as_str() {
            "chat" => {
                api::llm::stream_chat_completion(&prompt, &[], &chat_completion_options, None).await
            }
            "reasoning" => match api::llm::get_actived_name("reasoning").await {
                Ok(llm_name) => {
                    if llm_name.is_some() {
                        api::llm::stream_chat_completion(
                            &prompt,
                            &[],
                            &chat_completion_options,
                            llm_name.as_deref(),
                        )
                        .await
                    } else {
                        Err(error::AiterError::NotExists(
                            "Reasoning LLM not exists".to_string(),
                        ))
                    }
                }
                Err(err) => Err(err),
            },
            _ => Err(error::AiterError::Invalid(format!(
                "Invalid LLM type: {}",
                r#type
            ))),
        };

        match result {
            Ok(mut stream) => {
                let mut has_content = false;
                let mut has_reasoning_content = false;

                while let Some(event) = stream.next().await {
                    match event {
                        ChatCompletionEvent::CallToolStart(_task) => {}
                        ChatCompletionEvent::CallToolEnd(_task_id, _result, _time) => {}
                        ChatCompletionEvent::CallToolFail(_task_id, _error, _time) => {}
                        ChatCompletionEvent::Content(delta) => {
                            if !has_content && has_reasoning_content {
                                print!("\n\n");
                                stdout().flush().unwrap();
                            }

                            has_content = true;
                            print!("{}", delta);
                            stdout().flush().unwrap();
                        }
                        ChatCompletionEvent::ReasoningContent(delta) => {
                            has_reasoning_content = true;
                            print!("{}", delta.bright_black());
                            stdout().flush().unwrap();
                        }
                        ChatCompletionEvent::Error(err) => {
                            println!("{}", err.to_string().red());
                            break;
                        }
                    }
                }

                println!();
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
