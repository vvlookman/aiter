use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct MemSkillTestCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    #[arg(
        short = 'O',
        long = "option",
        help = "Options passed to skill, e.g. -O ip:localhost"
    )]
    options: Vec<String>,

    id: String,
}

impl MemSkillTestCommand {
    pub async fn exec(&self) {
        let options = VecOptions(&self.options).into_map();

        match api::mem::skill::test(self.ai.as_deref(), &self.id, &options).await {
            Ok(result) => {
                println!("{result}");
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
