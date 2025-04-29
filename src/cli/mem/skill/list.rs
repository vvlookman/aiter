use aiter::*;
use colored::Colorize;
use tabled::Table;

use crate::cli;

#[derive(clap::Args)]
pub struct MemSkillListCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    #[arg(
        short = 'n',
        value_name = "N",
        default_value = "20",
        help = "Max number of results, default value is 20"
    )]
    limit: u64,

    #[arg(
        short = 's',
        value_name = "S",
        default_value = "0",
        help = "Skip number of results, default value is 0"
    )]
    offset: u64,

    search: Option<String>,
}

impl MemSkillListCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        match api::mem::skill::list(
            self.ai.as_deref(),
            &self.search.clone().unwrap_or("".to_string()),
            self.limit,
            self.offset,
        )
        .await
        {
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
