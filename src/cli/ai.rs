use clap::Subcommand;

mod add;
mod clone;
mod delete;
mod list;
mod rename;

#[derive(Subcommand)]
pub enum AiCommand {
    #[command(about = "Add AI")]
    #[clap(visible_aliases = &["create", "new"])]
    Add(Box<add::AiAddCommand>),

    #[command(about = "Clone AI")]
    #[clap(visible_aliases = &["copy"])]
    Clone(Box<clone::AiCloneCommand>),

    #[command(about = "Delete AI")]
    #[clap(visible_aliases = &["del", "remove", "rm"])]
    Delete(Box<delete::AiDeleteCommand>),

    #[command(about = "List AIs")]
    #[clap(visible_aliases = &["ls"])]
    List(Box<list::AiListCommand>),

    #[command(about = "Rename AI")]
    Rename(Box<rename::AiRenameCommand>),
}

impl AiCommand {
    pub async fn exec(&self) {
        match self {
            AiCommand::Add(cmd) => {
                cmd.exec().await;
            }
            AiCommand::Clone(cmd) => {
                cmd.exec().await;
            }
            AiCommand::Delete(cmd) => {
                cmd.exec().await;
            }
            AiCommand::List(cmd) => {
                cmd.exec().await;
            }
            AiCommand::Rename(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
