use std::io::{stdin, stdout, Write};

use aiter::api;
use clap::Subcommand;
use colored::Colorize;

mod ai;
mod chat;
mod dev;
mod digest;
mod info;
mod learn;
mod llm;
mod mem;
mod read;
mod serve;
mod tool;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage AI characters")]
    #[clap(subcommand)]
    Ai(Box<ai::AiCommand>),

    #[command(about = "Chat with AI, the default AI is used if AI is not specified by `@<AI>`")]
    Chat(Box<chat::ChatCommand>),

    #[command(about = "For development")]
    #[clap(subcommand)]
    Dev(Box<dev::DevCommand>),

    #[command(about = "Digest read documents")]
    Digest(Box<digest::DigestCommand>),

    #[command(about = "Display system-wide information")]
    Info(Box<info::InfoCommand>),

    #[command(about = "Read & Digest a single document")]
    Learn(Box<learn::LearnCommand>),

    #[command(about = "LLM configuration and testing")]
    #[clap(subcommand)]
    Llm(Box<llm::LlmCommand>),

    #[command(about = "Manipulation of memories")]
    #[clap(subcommand)]
    Mem(Box<mem::MemCommand>),

    #[command(about = "Read documents")]
    Read(Box<read::ReadCommand>),

    #[command(about = "Start web server")]
    Serve(Box<serve::ServeCommand>),

    #[command(about = "Tools")]
    #[clap(subcommand)]
    Tool(Box<tool::ToolCommand>),
}

fn confirm_action(prompt: &str) -> bool {
    print!("{prompt} (y/N) ");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase() == "y"
}

fn display_ai(name: Option<&str>, capitalize_first: bool) -> String {
    if let Some(name) = name {
        format!("AI '{name}'")
    } else {
        format!("{}he default AI", if capitalize_first { "T" } else { "t" })
    }
}

async fn is_ai_valid(name: Option<&str>) -> bool {
    if let Some(name) = name {
        match api::ai::get(name).await {
            Ok(ai) => {
                if ai.is_none() {
                    println!("AI '{}' does not exist, create AI with the `aiter ai new <NAME>` command first", name.yellow());
                    return false;
                }
            }
            Err(err) => {
                println!("{}", err.to_string().red());
                return false;
            }
        }
    }

    true
}
