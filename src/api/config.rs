use std::str::FromStr;

use crate::{AiterError, db, error::AiterResult};

pub async fn get(key: &str) -> AiterResult<Option<String>> {
    if let Ok(config_key) = db::core::config::ConfigKey::from_str(key) {
        db::core::config::get(&config_key).await
    } else {
        Err(AiterError::Invalid(format!("Key '{key}' is invalid")))
    }
}

pub async fn set(key: &str, value: &str) -> AiterResult<()> {
    if let Ok(config_key) = db::core::config::ConfigKey::from_str(key) {
        db::core::config::set(&config_key, value).await
    } else {
        Err(AiterError::Invalid(format!("Key '{key}' is invalid")))
    }
}
