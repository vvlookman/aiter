pub mod ahp;
pub mod mcp;

#[derive(strum::Display, strum::EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum ToolType {
    Ahp, // Aiter HTTP Protocol
    Mcp, // Model Context Protocol
}
