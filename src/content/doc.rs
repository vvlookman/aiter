use std::fmt::Debug;

use crate::{
    content::{
        doc::{markdown::MarkdownDoc, sheet::SheetDoc, text::TextDoc},
        seg::SegContent,
    },
    error::AiterResult,
    Tokenizer,
};

pub mod markdown;
pub mod sheet;
pub mod text;

pub trait DocContent: Debug + Send + Sync {
    fn get_title(&self) -> Option<String>;
    fn get_preview(&self) -> String;
    fn get_type(&self) -> DocContentType;
    fn split(&self, tokenizer: &Tokenizer) -> Vec<Vec<Box<dyn SegContent>>>;
    fn to_string(&self) -> String;
    fn try_from_bytes(bytes: &[u8]) -> AiterResult<Self>
    where
        Self: Sized;
    fn try_into_bytes(&self) -> AiterResult<Vec<u8>>;
}

#[derive(strum::Display, strum::EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum DocContentType {
    Markdown,
    Sheet,
    Text,
}

pub fn decode_content(content: &[u8], content_type: &str) -> AiterResult<Box<dyn DocContent>> {
    match content_type.parse::<DocContentType>()? {
        DocContentType::Markdown => Ok(Box::new(MarkdownDoc::try_from_bytes(content)?)),
        DocContentType::Sheet => Ok(Box::new(SheetDoc::try_from_bytes(content)?)),
        DocContentType::Text => Ok(Box::new(TextDoc::try_from_bytes(content)?)),
    }
}
