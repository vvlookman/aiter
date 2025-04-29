use std::io::{stdout, Write};

use aiter::{
    api::llm::{ChatCompletionOptions, ChatEvent},
    error::AiterResult,
    *,
};
use colored::Colorize;
use tokio::sync::mpsc;

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

        let (event_sender, mut event_receiver) = mpsc::channel::<ChatEvent>(CHANNEL_BUFFER_DEFAULT);

        let mut chat_completion_options = ChatCompletionOptions::default();
        let llm_options = VecOptions(&self.llm_options);
        if let Some(temperature_str) = llm_options.get("temperature") {
            if let Ok(temperature) = temperature_str.parse() {
                chat_completion_options = chat_completion_options.with_temperature(temperature);
            }
        }

        let r#type = self.r#type.clone();
        let prompt = self.prompt.clone();

        let handle = tokio::spawn(async move {
            let result: AiterResult<()> = match r#type.as_str() {
                "chat" => {
                    match api::llm::chat_completion(
                        &prompt,
                        &[],
                        &chat_completion_options,
                        None,
                        Some(event_sender),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(err) => Err(err),
                    }
                }
                "reasoning" => match api::llm::get_actived_name("reasoning").await {
                    Ok(llm_name) => {
                        if llm_name.is_some() {
                            match api::llm::chat_completion(
                                &prompt,
                                &[],
                                &chat_completion_options,
                                llm_name.as_deref(),
                                Some(event_sender),
                            )
                            .await
                            {
                                Ok(_) => Ok(()),
                                Err(err) => Err(err),
                            }
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

            if let Err(err) = result {
                println!("{}", err.to_string().red());
            }
        });

        while let Some(event) = event_receiver.recv().await {
            match event {
                ChatEvent::StreamStart => {}
                ChatEvent::CallToolStart(_task) => {}
                ChatEvent::CallToolEnd(_task_id, _result, _time) => {}
                ChatEvent::ReasoningStart => {}
                ChatEvent::ReasoningContent(delta) => {
                    print!("{}", delta.bright_black());
                    stdout().flush().unwrap();
                }
                ChatEvent::ReasoningEnd => {
                    print!("\n\n");
                    stdout().flush().unwrap();
                }
                ChatEvent::StreamContent(delta) => {
                    print!("{}", delta);
                    stdout().flush().unwrap();
                }
                ChatEvent::StreamEnd => {
                    println!();
                }
            }
        }

        let _ = handle.await;
    }
}
