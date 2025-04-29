use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct MemDocShowCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    id: String,

    #[clap(default_value = "0", help = "Part index in the document")]
    index: u64,
}

impl MemDocShowCommand {
    pub async fn exec(&self) {
        match api::mem::doc::get_part_as_text(self.ai.as_deref(), &self.id, self.index).await {
            Ok(content) => {
                if let Some(content) = content {
                    println!("{}", content);
                } else {
                    println!("{}", "[Not found]".yellow());
                }
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
