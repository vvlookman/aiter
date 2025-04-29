use clap::Subcommand;

mod delete;
mod list;
mod pull;
mod show;

#[derive(Subcommand)]
pub enum MemDocCommand {
    #[command(about = "Delete a document")]
    #[clap(visible_aliases = &["del", "remove", "rm"])]
    Delete(Box<delete::MemDocDeleteCommand>),

    #[command(about = "List documents")]
    #[clap(visible_aliases = &["ls"])]
    List(Box<list::MemDocListCommand>),

    #[command(
        about = "Get the original document if it exists, be sure to specify the --keep option when executing the read command"
    )]
    Pull(Box<pull::MemDocPullCommand>),

    #[command(about = "Show document's content")]
    Show(Box<show::MemDocShowCommand>),
}

impl MemDocCommand {
    pub async fn exec(&self) {
        match self {
            MemDocCommand::Delete(cmd) => {
                cmd.exec().await;
            }
            MemDocCommand::List(cmd) => {
                cmd.exec().await;
            }
            MemDocCommand::Pull(cmd) => {
                cmd.exec().await;
            }
            MemDocCommand::Show(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
