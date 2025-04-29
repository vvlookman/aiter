use serde::{Deserialize, Serialize};

use crate::{
    content::frag::{FragContent, FragContentType},
    error::AiterResult,
    utils::compress,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TextFragContent {
    pub text: String,
}

impl FragContent for TextFragContent {
    fn get_type(&self) -> FragContentType {
        FragContentType::Text
    }

    fn to_string(&self) -> String {
        self.text.clone()
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
