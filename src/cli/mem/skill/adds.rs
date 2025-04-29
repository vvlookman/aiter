use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct MemSkillAddsCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    toolset_id: String,
}

impl MemSkillAddsCommand {
    pub async fn exec(&self) {
        let mem_write_event_sender = api::mem::spawn_mem_write(self.ai.as_deref())
            .await
            .expect("Spawn mem write error");

        match api::mem::skill::adds(self.ai.as_deref(), &self.toolset_id, mem_write_event_sender)
            .await
        {
            Ok(skills) => {
                if !skills.is_empty() {
                    for skill in skills {
                        println!("Skill '{}' has been added", skill.trigger.green());
                    }
                } else {
                    println!("No skill added");
                }
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
