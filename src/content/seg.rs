use std::fmt::Debug;

use crate::{
    Tokenizer,
    content::{
        frag::FragContent,
        seg::{sheet::SheetSegContent, text::TextSegContent},
    },
    error::AiterResult,
};

pub mod sheet;
pub mod text;

pub trait SegContent: Debug + Send + Sync {
    fn get_type(&self) -> SegContentType;
    fn split(&self, tokenizer: &Tokenizer) -> Vec<Box<dyn FragContent>>;
    fn to_string(&self) -> String;
    fn try_from_bytes(bytes: &[u8]) -> AiterResult<Self>
    where
        Self: Sized;
    fn try_into_bytes(&self) -> AiterResult<Vec<u8>>;
}

#[derive(strum::Display, strum::EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum SegContentType {
    Sheet,
    Text,
}

pub fn decode_content(content: &[u8], content_type: &str) -> AiterResult<Box<dyn SegContent>> {
    match content_type.parse::<SegContentType>()? {
        SegContentType::Sheet => Ok(Box::new(SheetSegContent::try_from_bytes(content)?)),
        SegContentType::Text => Ok(Box::new(TextSegContent::try_from_bytes(content)?)),
    }
}
