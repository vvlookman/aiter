use serde::{Deserialize, Serialize};

use crate::{
    content::{
        doc::{DocContent, DocContentType},
        seg::{text::TextSegContent, SegContent},
    },
    error::AiterResult,
    utils::{
        compress,
        text::{split_by_max_tokens, truncate_format},
    },
    Tokenizer, SPLIT_TOKENS_OF_SEG, TRUNCATE_PREVIEW,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TextDoc {
    pub title: Option<String>,
    pub pages: Vec<String>,
    pub outlines: Vec<TextDocOutline>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TextDocOutline {
    pub title: String,
    pub page: usize,
    pub children: Vec<TextDocOutline>,
}

impl DocContent for TextDoc {
    fn get_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn get_preview(&self) -> String {
        self.pages.first().map_or(String::new(), |s| {
            truncate_format(s.trim(), TRUNCATE_PREVIEW, false)
        })
    }

    fn get_type(&self) -> DocContentType {
        DocContentType::Text
    }

    fn split(&self, tokenizer: &Tokenizer) -> Vec<Vec<Box<dyn SegContent>>> {
        self.pages
            .iter()
            .map(|page| {
                let text = page.to_string();
                split_by_max_tokens(&text, SPLIT_TOKENS_OF_SEG, tokenizer)
                    .iter()
                    .map(|s| {
                        Box::new(TextSegContent {
                            text: s.to_string(),
                        }) as Box<dyn SegContent>
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn to_string(&self) -> String {
        let mut s = String::new();

        if let Some(title) = &self.title {
            s.push_str(&format!("# {title}"));
            s.push_str("\n\n");
        }

        for page in &self.pages {
            s.push_str(page);
            s.push_str("\n\n");
        }

        s
    }

    fn try_from_bytes(bytes: &[u8]) -> AiterResult<Self> {
        let json = compress::decode(bytes)?;
        Ok(serde_json::from_slice(&json)?)
    }

    fn try_into_bytes(&self) -> AiterResult<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        Ok(compress::encode(&json)?)
    }
}
