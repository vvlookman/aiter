use std::str::FromStr;

use crate::{db, error::AiterResult};

pub async fn get(key: &str) -> AiterResult<Option<String>> {
    let config_key = db::core::config::ConfigKey::from_str(key)?;
    db::core::config::get(&config_key).await
}

pub async fn set(key: &str, value: &str) -> AiterResult<()> {
    let config_key = db::core::config::ConfigKey::from_str(key)?;
    db::core::config::set(&config_key, value).await
}
