use aiter::*;
use clap::Subcommand;

mod active;
mod config;
mod delete;
mod list;
mod rename;
mod test;

#[derive(Subcommand)]
pub enum LlmCommand {
    #[command(about = "Active LLM as the default provider")]
    Active(Box<active::LlmActiveCommand>),

    #[command(about = "Configure LLM provider")]
    Config(Box<config::LlmConfigCommand>),

    #[command(about = "Delete LLM provider")]
    #[clap(visible_aliases = &["del", "remove", "rm"])]
    Delete(Box<delete::LlmDeleteCommand>),

    #[command(about = "List LLM providers")]
    #[clap(visible_aliases = &["ls"])]
    List(Box<list::LlmListCommand>),

    #[command(about = "Rename LLM provider")]
    Rename(Box<rename::LlmRenameCommand>),

    #[command(about = "Test the default LLM provider")]
    Test(Box<test::LlmTestCommand>),
}

impl LlmCommand {
    pub async fn exec(&self) {
        match self {
            LlmCommand::Active(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::Config(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::Delete(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::List(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::Rename(cmd) => {
                cmd.exec().await;
            }
            LlmCommand::Test(cmd) => {
                cmd.exec().await;
            }
        }
    }
}

fn is_type_valid(r#type: &str) -> bool {
    if api::llm::SUPPORTED_TYPES.contains(&r#type) {
        return true;
    }

    println!(
        "Invalid type '{}', available values: {}",
        r#type,
        api::llm::SUPPORTED_TYPES.join("/")
    );

    false
}
