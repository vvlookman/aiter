use clap::Subcommand;

mod add;
mod adds;
mod delete;
mod list;
mod test;

#[derive(Subcommand)]
pub enum MemSkillCommand {
    #[command(about = "Add skill")]
    #[clap(visible_aliases = &["create", "new"])]
    Add(Box<add::MemSkillAddCommand>),

    #[command(about = "Add skills")]
    Adds(Box<adds::MemSkillAddsCommand>),

    #[command(about = "Delete skill")]
    #[clap(visible_aliases = &["del", "remove", "rm"])]
    Delete(Box<delete::MemSkillDeleteCommand>),

    #[command(about = "List skills")]
    #[clap(visible_aliases = &["ls"])]
    List(Box<list::MemSkillListCommand>),

    #[command(about = "Test skill")]
    Test(Box<test::MemSkillTestCommand>),
}

impl MemSkillCommand {
    pub async fn exec(&self) {
        match self {
            MemSkillCommand::Add(cmd) => {
                cmd.exec().await;
            }
            MemSkillCommand::Adds(cmd) => {
                cmd.exec().await;
            }
            MemSkillCommand::Delete(cmd) => {
                cmd.exec().await;
            }
            MemSkillCommand::List(cmd) => {
                cmd.exec().await;
            }
            MemSkillCommand::Test(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
