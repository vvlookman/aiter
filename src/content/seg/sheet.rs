use serde::{Deserialize, Serialize};

use crate::{
    Tokenizer,
    content::{
        doc::sheet::SheetData,
        frag::FragContent,
        seg::{SegContent, SegContentType},
    },
    error::AiterResult,
    utils::compress,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SheetSegContent {
    pub data: SheetData,
}

impl SegContent for SheetSegContent {
    fn get_type(&self) -> SegContentType {
        SegContentType::Sheet
    }

    fn split(&self, _tokenizer: &Tokenizer) -> Vec<Box<dyn FragContent>> {
        vec![]
    }

    fn to_string(&self) -> String {
        self.data.to_string()
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
