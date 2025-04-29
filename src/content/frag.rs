use std::fmt::Debug;

use crate::{content::frag::text::TextFragContent, error::AiterResult};

pub mod text;

pub trait FragContent: Debug + Send + Sync {
    fn get_type(&self) -> FragContentType;
    fn to_string(&self) -> String;
    fn try_from_bytes(bytes: &[u8]) -> AiterResult<Self>
    where
        Self: Sized;
    fn try_into_bytes(&self) -> AiterResult<Vec<u8>>;
}

#[derive(strum::Display, strum::EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum FragContentType {
    Text,
}

pub fn decode_content(content: &[u8], content_type: &str) -> AiterResult<Box<dyn FragContent>> {
    match content_type.parse::<FragContentType>()? {
        FragContentType::Text => Ok(Box::new(TextFragContent::try_from_bytes(content)?)),
    }
}
