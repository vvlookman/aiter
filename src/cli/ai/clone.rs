use aiter::*;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::Duration;

use crate::cli;

#[derive(clap::Args)]
pub struct AiCloneCommand {
    #[arg(
        short = 'f',
        long = "from",
        value_name = "FROM_NAME",
        help = "The name being cloned, if not set means that it is being cloned from the default TA"
    )]
    from_name: Option<String>,

    name: String,
}

impl AiCloneCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.from_name.as_deref()).await {
            return;
        }

        let from_name = self.from_name.clone();
        let name = self.name.trim();

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{msg} {spinner:.cyan}").unwrap());
        spinner.set_message(format!(
            "[{}] Cloning...",
            from_name.as_deref().unwrap_or("~")
        ));
        spinner.enable_steady_tick(Duration::from_millis(100));

        if let Err(err) = api::ai::clone(from_name.as_deref(), name).await {
            spinner.finish_with_message(format!("{}", err.to_string().red()));
        } else {
            spinner.finish_with_message(format!(
                "AI '{}' has been cloned as '{}'",
                from_name.as_deref().unwrap_or("~"),
                name
            ));
        }
    }
}
