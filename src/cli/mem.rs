use clap::Subcommand;

mod doc;
mod erase;
mod history;
mod merge;
mod skill;
mod stats;
mod vacuum;

#[derive(Subcommand)]
pub enum MemCommand {
    #[command(about = "Commands for documents in memories")]
    #[clap(subcommand)]
    Doc(Box<doc::MemDocCommand>),

    #[command(about = "Erase memories")]
    #[clap(visible_aliases = &["clear"])]
    Erase(Box<erase::MemEraseCommand>),

    #[command(about = "Commands for history in memories")]
    #[clap(subcommand)]
    History(Box<history::MemHistoryCommand>),

    #[command(about = "Merge memories from others")]
    Merge(Box<merge::MemMergeCommand>),

    #[command(about = "Commands for skills in memories")]
    #[clap(subcommand)]
    Skill(Box<skill::MemSkillCommand>),

    #[command(about = "Statistics of memories")]
    Stats(Box<stats::MemStatsCommand>),

    #[command(about = "Vacuum memories")]
    Vacuum(Box<vacuum::MemVacuumCommand>),
}

impl MemCommand {
    pub async fn exec(&self) {
        match self {
            MemCommand::Doc(cmd) => {
                cmd.exec().await;
            }
            MemCommand::Erase(cmd) => {
                cmd.exec().await;
            }
            MemCommand::History(cmd) => {
                cmd.exec().await;
            }
            MemCommand::Merge(cmd) => {
                cmd.exec().await;
            }
            MemCommand::Skill(cmd) => {
                cmd.exec().await;
            }
            MemCommand::Stats(cmd) => {
                cmd.exec().await;
            }
            MemCommand::Vacuum(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
