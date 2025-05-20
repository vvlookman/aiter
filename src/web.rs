use std::sync::LazyLock;

use actix_web::*;
use dashmap::DashMap;
use rust_embed::RustEmbed;
use tokio::sync::{RwLock, mpsc};

use crate::{
    api::mem::{MemWriteEvent, spawn_mem_write},
    db,
    error::AiterResult,
};

pub mod api;
pub mod utils;

pub static WEB_BASE: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));

pub struct AppState {
    pub notify_digest_event_sender: mpsc::Sender<NotifyDigestEvent>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub digest_batch: usize,
    pub digest_concurrent: usize,
    pub digest_deep: bool,
    pub skip_digest: bool,
}

#[derive(Debug)]
pub struct NotifyDigestEvent {
    pub ai: Option<String>,
    pub doc_id: String,
}

#[derive(RustEmbed)]
#[folder = "webui/dist/"]
pub struct WebUi;

pub async fn serve_webui(path: web::Path<String>) -> Result<impl Responder> {
    let path = if path.is_empty() {
        "index.html"
    } else {
        &path.to_string()
    };

    if let Some(asset) = WebUi::get(path) {
        let content = asset.data;
        let content_type = mime_guess::from_path(path).first_or_octet_stream();

        if path == "index.html" {
            let base: String = {
                let guard = WEB_BASE.read().await;
                guard.to_string()
            }
            .trim()
            .to_string();

            let replacer: &str = if base.is_empty() || base == "/" {
                ""
            } else {
                if base.starts_with("/") {
                    &base
                } else {
                    &format!("/{base}")
                }
            };

            let original = String::from_utf8_lossy(&content);
            let replaced = original.replace("/AITER_BASE", replacer);
            Ok(HttpResponse::Ok().content_type(content_type).body(replaced))
        } else {
            Ok(HttpResponse::Ok().content_type(content_type).body(content))
        }
    } else {
        Ok(HttpResponse::NotFound().body("404 Not found"))
    }
}

pub async fn get_mem_write_event_sender(
    ai: Option<&str>,
) -> AiterResult<mpsc::Sender<MemWriteEvent>> {
    let ai_key = ai.unwrap_or_default();
    if let Some(sender) = MEM_WRITE_EVENT_SENDERS.get(ai_key) {
        Ok(sender.clone())
    } else {
        let sender = spawn_mem_write(ai).await?;
        MEM_WRITE_EVENT_SENDERS.insert(ai_key.to_string(), sender.clone());

        Ok(sender)
    }
}

static MEM_WRITE_EVENT_SENDERS: LazyLock<DashMap<String, mpsc::Sender<db::mem::MemWriteEvent>>> =
    LazyLock::new(DashMap::new);
