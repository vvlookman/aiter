use aiter::*;
use colored::Colorize;

use crate::cli;

#[derive(clap::Args)]
pub struct ToolDeleteCommand {
    #[arg(long = "toolset", help = "Delete tools in the toolset")]
    toolset: bool,

    id: String,
}

impl ToolDeleteCommand {
    pub async fn exec(&self) {
        if self.toolset {
            if cli::confirm_action(&format!(
                "Are you sure you want to delete all tools in toolset '{}'?",
                self.id.yellow()
            )) {
                match api::tool::delete_by_toolset(&self.id).await {
                    Ok(tools) => {
                        if !tools.is_empty() {
                            println!(
                                "Total {} tools in toolset '{}' has been deleted",
                                tools.len(),
                                self.id
                            );
                        } else {
                            println!("No tool exists in toolset '{}'", self.id);
                        }
                    }
                    Err(err) => {
                        println!("{}", err.to_string().red());
                    }
                }
            }
        } else {
            if cli::confirm_action(&format!(
                "Are you sure you want to delete tool with ID '{}'?",
                self.id.yellow()
            )) {
                match api::tool::delete(&self.id).await {
                    Ok(tool) => {
                        if let Some(tool) = tool {
                            println!("Tool '{}' has been deleted", tool.description);
                        } else {
                            println!("Tool does not exist");
                        }
                    }
                    Err(err) => {
                        println!("{}", err.to_string().red());
                    }
                }
            }
        }
    }
}
