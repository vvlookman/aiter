use aiter::*;
use colored::Colorize;
use tabled::Table;

#[derive(clap::Args)]
pub struct LlmListCommand;

impl LlmListCommand {
    pub async fn exec(&self) {
        match api::llm::list().await {
            Ok(mut rows) => {
                for row in &mut rows {
                    row.created_at = utils::datetime::iso_to_local_datetime_string(&row.created_at);
                    row.updated_at = utils::datetime::iso_to_local_datetime_string(&row.updated_at);
                }

                println!("{}", Table::new(rows));
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
