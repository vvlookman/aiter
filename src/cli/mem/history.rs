use clap::Subcommand;

mod clear;

#[derive(Subcommand)]
pub enum MemHistoryCommand {
    #[command(about = "Clear all history")]
    #[clap(visible_aliases = &["clean"])]
    Clear(Box<clear::MemHistoryClearCommand>),
}

impl MemHistoryCommand {
    pub async fn exec(&self) {
        match self {
            MemHistoryCommand::Clear(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
