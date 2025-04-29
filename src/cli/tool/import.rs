use aiter::*;
use colored::Colorize;
use tabled::Table;

#[derive(clap::Args)]
pub struct ToolImportCommand {
    #[arg(
        short = 'O',
        long = "option",
        help = "Import option, related to type, e.g.\n  --type AHP -O url:http://xxx -O header:API_KEY:xxx\n  --type MCP -O cmd:npx -O arg:-y -O arg:@modelcontextprotocol/server-google-maps -O env:GOOGLE_MAPS_API_KEY:xxx"
    )]
    options: Vec<String>,

    #[arg(
        long = "title",
        help = "Title of import toolset, read from service if not specified"
    )]
    title: Option<String>,

    #[arg(
        short = 't',
        long = "type",
        help = "Toolset's type, currently supported types: AHP/MCP"
    )]
    r#type: String,
}

impl ToolImportCommand {
    pub async fn exec(&self) {
        match api::tool::import(&self.r#type, self.title.as_deref(), &self.options).await {
            Ok(tools) => {
                println!("{}", Table::new(tools));
            }
            Err(err) => {
                println!("{}", err.to_string().red());
            }
        }
    }
}
