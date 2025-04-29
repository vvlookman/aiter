use std::env;

use clap::Parser;

use crate::cli::Commands;

mod cli;

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    aiter::init().await;

    let mut args: Vec<String> = vec![];
    env::args().for_each(|arg| {
        if let Some(ai_stripped) = arg.strip_prefix('@') {
            args.push("--ai".to_string());
            args.push(ai_stripped.to_string());
        } else {
            args.push(arg);
        }
    });

    let cli = Cli::parse_from(args);
    match &cli.command {
        Commands::Ai(cmd) => {
            cmd.exec().await;
        }
        Commands::Chat(cmd) => {
            cmd.exec().await;
        }
        Commands::Dev(cmd) => {
            cmd.exec().await;
        }
        Commands::Digest(cmd) => {
            cmd.exec().await;
        }
        Commands::Info(cmd) => {
            cmd.exec().await;
        }
        Commands::Learn(cmd) => {
            cmd.exec().await;
        }
        Commands::Llm(cmd) => {
            cmd.exec().await;
        }
        Commands::Mem(cmd) => {
            cmd.exec().await;
        }
        Commands::Read(cmd) => {
            cmd.exec().await;
        }
        Commands::Serve(cmd) => {
            cmd.exec().await;
        }
        Commands::Tool(cmd) => {
            cmd.exec().await;
        }
    }
}
