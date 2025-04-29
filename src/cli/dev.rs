use clap::Subcommand;

mod sql;

#[derive(Subcommand)]
pub enum DevCommand {
    #[command(about = "Execute sql")]
    Sql(Box<sql::DevSqlCommand>),
}

impl DevCommand {
    pub async fn exec(&self) {
        match self {
            DevCommand::Sql(cmd) => {
                cmd.exec().await;
            }
        }
    }
}
