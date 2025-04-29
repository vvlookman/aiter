use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};

use tokio::sync::mpsc::Sender;

use crate::{
    api::{get_docs_dir_path, get_mem_path},
    content::{
        doc::DocContent,
        parsers::{csv, docx, epub, md, pdf, txt, xlsx},
    },
    db::mem::MemWriteEvent,
    error::{AiterError, AiterResult},
    learn,
    utils::fs,
};

pub enum DigestEvent {
    Progress(String),
}

pub struct DigestOptions {
    pub batch: usize,
    pub concurrent: usize,
    pub deep: bool,
    pub retry: bool,
}

pub struct DigestResult {
    pub doc_count: (usize, usize),
    pub part_count: (usize, usize),
    pub seg_size: (usize, usize),
    pub frag_size: (usize, usize),
}

pub enum ReadEvent {
    Progress(String),
}

pub struct ReadResult {
    pub doc_id: String,
    pub doc_exists: bool,
}

pub async fn digest(
    ai_name: Option<&str>,
    options: &DigestOptions,
    mem_write_event_sender: Sender<MemWriteEvent>,
    digest_event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<DigestResult> {
    let mem_path = get_mem_path(ai_name).await?;
    learn::digest(
        &mem_path,
        options,
        mem_write_event_sender,
        digest_event_sender,
    )
    .await
}

pub async fn digest_doc(
    ai_name: Option<&str>,
    doc_id: &str,
    options: &DigestOptions,
    mem_write_event_sender: Sender<MemWriteEvent>,
    digest_event_sender: Option<Sender<DigestEvent>>,
) -> AiterResult<DigestResult> {
    let mem_path = get_mem_path(ai_name).await?;
    learn::digest_doc(
        &mem_path,
        doc_id,
        options,
        mem_write_event_sender,
        digest_event_sender,
    )
    .await
}

pub async fn read_doc(
    ai_name: Option<&str>,
    path: &Path,
    filename: Option<&str>,
    format: Option<&str>,
    keep: bool,
    mem_write_event_sender: Sender<MemWriteEvent>,
    read_event_sender: Option<Sender<ReadEvent>>,
) -> AiterResult<ReadResult> {
    let source = filename
        .map(|s| s.to_string())
        .unwrap_or(fs::extract_filename_from_path(path));

    let suffix = if format.is_some() {
        format.map(|format| format.to_lowercase())
    } else {
        PathBuf::from(&source)
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
    };

    if let Some(suffix) = suffix {
        let mem_path = get_mem_path(ai_name).await?;

        let doc: Box<dyn DocContent> = match suffix.as_str() {
            "csv" => Box::new(csv::to_sheet_doc(path, &source)?),
            "docx" => Box::new(docx::to_text_doc(path, &source)?),
            "epub" => Box::new(epub::to_text_doc(path, &source)?),
            "md" => Box::new(md::to_markdown_doc(path, &source)?),
            "pdf" => Box::new(pdf::to_text_doc(path, &source)?),
            "txt" => Box::new(txt::to_text_doc(path, &source)?),
            "xlsx" | "xls" | "xlsm" | "xlsb" | "xla" | "xlam" | "ods" => {
                Box::new(xlsx::to_sheet_doc(path, &source)?)
            }
            _ => {
                return Err(AiterError::Unsupported(format!(
                    "Format '{}' is not currently supported",
                    suffix
                )))
            }
        };

        let read_result = learn::read_doc(
            &mem_path,
            &source,
            &*doc,
            mem_write_event_sender,
            read_event_sender,
        )
        .await?;

        if keep && !read_result.doc_exists {
            let docs_path = get_docs_dir_path(ai_name).await?;
            create_dir_all(&docs_path)?;

            let keep_path = docs_path.join(&read_result.doc_id);
            copy(path, &keep_path)?;
        }

        Ok(read_result)
    } else {
        Err(AiterError::Unsupported(format!(
            "Unknown format and not specified: {}",
            path.display()
        )))
    }
}

impl Default for DigestOptions {
    fn default() -> Self {
        Self {
            batch: 1,
            concurrent: 1,
            deep: false,
            retry: false,
        }
    }
}

impl DigestOptions {
    pub fn with_batch(mut self, batch: usize) -> Self {
        self.batch = batch.max(1);
        self
    }

    pub fn with_concurrent(mut self, concurrent: usize) -> Self {
        self.concurrent = concurrent.max(1);
        self
    }

    pub fn with_deep(mut self, deep: bool) -> Self {
        self.deep = deep;
        self
    }

    pub fn with_retry(mut self, retry: bool) -> Self {
        self.retry = retry;
        self
    }
}
