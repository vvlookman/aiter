//! # aiter lib

use std::{collections::HashMap, env, fs::create_dir_all, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use rayon::prelude::*;

use crate::{error::AiterError, utils::text::Tokenizer};

pub mod api;
pub mod error;
pub mod utils;
pub mod web;

pub static CHANNEL_BUFFER_DEFAULT: usize = 64;
pub static CHANNEL_BUFFER_LARGE: usize = 256;

/// Options that each item is String in <key>:<value> format
pub struct VecOptions<'a>(pub &'a [String]);

pub async fn init() {
    env_logger::Builder::new()
        .parse_filters(env::var("LOG").as_deref().unwrap_or("off"))
        .init();

    if !DATA_DIR.exists() {
        create_dir_all(&*DATA_DIR).expect("Unable to create data directory!");
    }

    let ensure_db = async || {
        db::ensure_core_tables().await?;
        db::ensure_mem_tables(&DB_DEFAULT_MEM_PATH).await?;

        Ok::<_, AiterError>(())
    };

    if let Err(err) = ensure_db().await {
        panic!("Check database error: {}", err);
    }

    // Update database if needed
    if let Ok(Some(db_version_str)) = db::core::meta::get_db_version().await {
        let db_version = db_version_str.parse::<u64>().unwrap_or(CURRENT_DB_VERSION);
        if db_version < CURRENT_DB_VERSION {
            if let Err(err) = api::sys::update().await {
                panic!("Update database error: {}", err);
            }
        }
    }
}

mod chat;
mod content;
mod db;
mod learn;
mod llm;
mod retrieve;
mod tool;

static CURRENT_DB_VERSION: u64 = 1; // Update when the db schema has been changed, schema patches are updated in the db.updates module
static CURRENT_SIGNATURE_DIMS: usize = 256;
static CURRENT_TOKENIZER: Tokenizer = Tokenizer::O200kBase;

static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        Some(proj_dirs) => proj_dirs.data_dir().to_path_buf(),
        None => env::current_dir()
            .expect("Unable to get current directory!")
            .join("data"),
    });

static DB_BUSY_SECS: u64 = 5;
static DB_CORE_PATH: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("core.db"));
static DB_DEFAULT_MEM_PATH: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("mem.db"));
static DB_VECTOR_NEIGHBORS: LazyLock<usize> =
    LazyLock::new(|| 3 * (((CURRENT_SIGNATURE_DIMS / 256) as f64).sqrt() as usize).max(1));

static CHAT_HISTORY_LIMIT: u64 = 100;
static DIGEST_RETRY: u64 = 3;
static FILTER_INFORMATIVE_TOKENS: usize = 5;
static LLM_CHAT_TEMPERATURE_DEFAULT: f64 = 0.6;
static LLM_CHAT_TEMPERATURE_STABLE: f64 = 0.0;
static RETRIEVE_FRAG_SURROUND: usize = 1;
static RETRIEVE_FTS_LIMIT: usize = 10;
static RETRIEVE_VEC_LIMIT: usize = 10;
static SPLIT_TOKENS_OF_FRAG: usize = 160;
static SPLIT_TOKENS_OF_SEG: usize = 1600;
static TRUNCATE_LOG_MESSAGE: usize = 100;
static TRUNCATE_PREVIEW: usize = 100;
static TRUNCATE_PROGRESS_MESSAGE: usize = 50;

impl VecOptions<'_> {
    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(option_text) = self.0.par_iter().find_any(|s| {
            s.to_lowercase()
                .starts_with(&format!("{}:", name.to_lowercase()))
        }) {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            parts.get(1).map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    pub fn into_map(self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();

        for option_text in self.0 {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            if parts.len() == 2 {
                map.insert(parts[0].to_string(), parts[1].trim().to_string());
            }
        }

        map
    }

    pub fn into_tuples(self) -> Vec<(String, String)> {
        let mut tuples: Vec<(String, String)> = vec![];

        for option_text in self.0 {
            let parts: Vec<_> = option_text.splitn(2, ':').collect();
            if parts.len() == 2 {
                tuples.push((parts[0].to_string(), parts[1].trim().to_string()));
            }
        }

        tuples
    }
}
