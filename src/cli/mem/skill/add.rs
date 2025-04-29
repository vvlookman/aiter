use aiter::*;
use colored::Colorize;

#[derive(clap::Args)]
pub struct MemSkillAddCommand {
    #[arg(long = "ai", value_name = "AI", help = "Alias of `@<AI>`")]
    ai: Option<String>,

    #[arg(
        long = "trigger",
        help = "Trigger of skill, use the tool's prompt if not specified"
    )]
    trigger: Option<String>,

    tool_id: String,
}

impl MemSkillAddCommand {
    pub async fn exec(&self) {
        let mem_write_event_sender = api::mem::spawn_mem_write(self.ai.as_deref())
            .await
            .expect("Spawn mem write error");

        match api::mem::skill::add(
            self.ai.as_deref(),
            &self.tool_id,
            self.trigger.as_deref(),
            mem_write_event_sender,
        )
        .await
        {
            Ok(skill) => {
                if let Some(skill) = skill {
                    println!("Skill '{}' has been added", &skill.trigger.green());
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
