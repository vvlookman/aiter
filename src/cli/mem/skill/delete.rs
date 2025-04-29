use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct MemSkillDeleteCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    id: String,
}

impl MemSkillDeleteCommand {
    pub async fn exec(&self) {
        if !cli::is_ai_valid(self.ai.as_deref()).await {
            return;
        }

        if cli::confirm_action(&format!(
            "Are you sure you want to delete skill with ID '{}'?",
            self.id.yellow()
        )) {
            let mem_write_event_sender = api::mem::spawn_mem_write(self.ai.as_deref())
                .await
                .expect("Spawn mem write error");

            match api::mem::skill::delete(self.ai.as_deref(), &self.id, mem_write_event_sender)
                .await
            {
                Ok(skill) => {
                    if let Some(skill) = skill {
                        println!("Skill '{}' has been deleted", skill.trigger);
                    } else {
                        println!("Skill does not exist");
                    }
                }
                Err(err) => {
                    println!("{}", err.to_string().red());
                }
            }
        }
    }
}
