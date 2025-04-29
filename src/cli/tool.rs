use clap::Subcommand;

mod delete;
mod import;
mod list;

#[derive(Subcommand)]
pub enum ToolCommand {
    #[command(about = "Delete Tool or Toolset")]
    #[clap(visible_aliases = &["del", "remove", "rm"])]
    Delete(Box<delete::ToolDeleteCommand>),

    #[command(about = "Import Tools")]
    Import(Box<import::ToolImportCommand>),

    #[command(about = "List Tools")]
    #[clap(visible_aliases = &["ls"])]
    List(Box<list::ToolListCommand>),
}

impl ToolCommand {
    pub async fn exec(&self) {
        match self {
            ToolCommand::Delete(cmd) => {
                cmd.exec().await;
            }
            ToolCommand::Import(cmd) => {
                cmd.exec().await;
            }
            ToolCommand::List(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
