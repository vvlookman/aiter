use std::collections::HashMap;

use reqwest::Method;
use url::Url;

use crate::{error::AiterResult, AiterError};

pub async fn http_get(
    url: &str,
    path: Option<&str>,
    query: &HashMap<String, String>,
    headers: &HashMap<String, String>,
) -> AiterResult<Vec<u8>> {
    let request_url = if let Some(path) = path {
        &join_url(url, path)?
    } else {
        url
    };

    let client = reqwest::Client::builder().build()?;

    let mut request_builder = client.request(Method::GET, request_url);
    request_builder = request_builder.query(query);

    for (k, v) in headers {
        request_builder = request_builder.header(k, v);
    }

    let response = request_builder.send().await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        Err(AiterError::HttpStatusError(format!(
            "{} {}",
            response.status(),
            response.text().await.ok().unwrap_or_default()
        )))
    }
}

pub async fn http_post(
    url: &str,
    path: Option<&str>,
    query: &HashMap<String, String>,
    body: &serde_json::Value,
    headers: &HashMap<String, String>,
) -> AiterResult<Vec<u8>> {
    let request_url = if let Some(path) = path {
        &join_url(url, path)?
    } else {
        url
    };

    let client = reqwest::Client::builder().build()?;

    let mut request_builder = client.request(Method::POST, request_url);
    request_builder = request_builder.query(query);
    request_builder = request_builder.json(body);

    for (k, v) in headers {
        request_builder = request_builder.header(k, v);
    }

    let response = request_builder.send().await?;

    if response.status().is_success() {
        Ok(response.bytes().await?.to_vec())
    } else {
        Err(AiterError::HttpStatusError(format!(
            "{} {}",
            response.status(),
            response.text().await.ok().unwrap_or_default()
        )))
    }
}

pub fn join_url(base_url: &str, extend_url: &str) -> Result<String, url::ParseError> {
    let mut url = Url::parse(base_url)?;

    url.path_segments_mut()
        .map_err(|_| url::ParseError::RelativeUrlWithCannotBeABaseBase)?
        .pop_if_empty()
        .extend(extend_url.split('/').filter(|s| !s.is_empty()));

    Ok(url.to_string())
}
