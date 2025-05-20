use std::io::{stdout, Write};

use aiter::{
    api::{chat::ChatOptions, llm::ChatCompletionEvent},
    *,
};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::Duration;

use crate::cli;

#[derive(clap::Args)]
pub struct ChatCommand {
    #[arg(
        long = "ai",
        value_name = "AI",
        help = "The character performing the operation, it is the alias of `@<AI>`"
    )]
    ai: Option<String>,

    #[arg(
        short = 'd',
        long = "deep",
        help = "Deep think, LLM will try to use reasoning LLM and understand the user's intent"
    )]
    deep: bool,

    #[arg(
        short = 'C',
        long = "llm-for-chat",
        help = "LLM for chat, e.g. -C deepseek-chat"
    )]
    llm_for_chat: Option<String>,

    #[arg(
        short = 'R',
        long = "llm-for-reasoning",
        help = "LLM for reasoning, e.g. -R deepseek-reasoner"
    )]
    llm_for_reasoning: Option<String>,

    #[arg(
        short = 'O',
        long = "llm-option",
        help = "Additional option passed to LLM, e.g. -O temperature:0.6"
    )]
    llm_options: Vec<String>,

    #[arg(
        short = 'r',
        long = "retrace",
        default_value = "0",
        help = "Look back for previous chat history, default value is 0"
    )]
    retrace: u64,

    #[arg(
        short = 'S',
        long = "session",
        help = "Specify the session identifier of the conversation, e.g. -S session_id"
    )]
    session: Option<String>,

    #[arg(
        short = 's',
        long = "strict",
        help = "Strict mode, LLM will not attempt to answer on its own"
    )]
    strict: bool,

    message: String,
}

impl ChatCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        let ai = self.ai.clone();
        let message = self.message.clone();
        let chat_options = ChatOptions::default()
            .with_deep(self.deep)
            .with_exchange(None)
            .with_llm_for_chat(self.llm_for_chat.clone())
            .with_llm_for_reasoning(self.llm_for_reasoning.clone())
            .with_llm_options(self.llm_options.clone())
            .with_retrace(self.retrace)
            .with_session(self.session.clone())
            .with_strict(self.strict);

        let bot_name = self.ai.clone().unwrap_or("~".to_string()).cyan();

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{msg} {spinner:.cyan}").unwrap());
        spinner.set_message(format!("[{bot_name}]"));
        spinner.enable_steady_tick(Duration::from_millis(100));

        let mem_write_event_sender = api::mem::spawn_mem_write(ai.as_deref())
            .await
            .expect("Spawn mem write error");

        match api::chat::chat(
            ai.as_deref(),
            &message,
            &chat_options,
            mem_write_event_sender,
        )
        .await
        {
            Ok(mut stream) => {
                spinner.finish_with_message(format!(
                    "[{}] {}",
                    bot_name,
                    utils::datetime::now_local_datetime_string()
                ));

                let mut has_content = false;
                let mut has_reasoning_content = false;

                while let Some(event) = stream.next().await {
                    match event {
                        ChatCompletionEvent::CallToolStart(task) => {
                            let task_name = format!("[{}]", task.description);
                            let task_params = if task.parameters.is_empty() {
                                "".to_string()
                            } else {
                                format!(
                                    " ( {} )",
                                    task.parameters
                                        .iter()
                                        .map(|(k, v)| format!("{k}={v}"))
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                )
                            };
                            println!("{task_name} {task_params}");
                            stdout().flush().unwrap();
                        }
                        ChatCompletionEvent::CallToolEnd(_task_id, _result, _time) => {}
                        ChatCompletionEvent::CallToolFail(_task_id, _error, _time) => {}
                        ChatCompletionEvent::Content(delta) => {
                            if !has_content && has_reasoning_content {
                                print!("\n\n");
                                stdout().flush().unwrap();
                            }

                            has_content = true;
                            print!("{delta}");
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
                spinner.finish_with_message(format!("[{}] {}", bot_name, err.to_string().red()));
            }
        }
    }
}
