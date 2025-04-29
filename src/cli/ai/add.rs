use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct AiAddCommand {
    name: String,
}

impl AiAddCommand {
    pub async fn exec(&self) {
        let name = self.name.trim();

        if let Err(err) = api::ai::add(name).await {
            println!("{}", err.to_string().red());
        } else {
            println!("AI '{}' has been created", name.green());
        }
    }
}
