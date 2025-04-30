use std::{
    fs::{copy, remove_file},
    path::{Path, PathBuf},
};

use tokio::sync::{mpsc::Sender, oneshot};

use crate::{
    api::{get_docs_dir_path, get_mem_path},
    content,
    content::doc::{sheet::SheetDoc, DocContent, DocContentType},
    db,
    db::mem::MemWriteEvent,
    error::AiterResult,
};

pub type DocEntity = db::mem::doc::DocEntity;

pub async fn count_part(ai_name: Option<&str>, doc_id: &str) -> AiterResult<u64> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc_part::count(&mem_path, doc_id).await
}

pub async fn delete(
    ai_name: Option<&str>,
    doc_id: &str,
    mem_write_event_sender: Sender<MemWriteEvent>,
) -> AiterResult<Option<DocEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    let doc = db::mem::doc::get(&mem_path, doc_id).await?;
    {
        let (resp_sender, resp_receiver) = oneshot::channel();
        mem_write_event_sender
            .send(MemWriteEvent::DeleteDoc {
                doc_id: doc_id.to_string(),
                resp_sender,
            })
            .await?;
        let _ = resp_receiver.await?;
    }

    if let Some(doc) = &doc {
        let keep_path = get_docs_dir_path(ai_name).await?.join(&doc.id);
        if keep_path.exists() {
            let _ = remove_file(keep_path);
        }
    }

    Ok(doc)
}

pub async fn get(ai_name: Option<&str>, doc_id: &str) -> AiterResult<Option<DocEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc::get(&mem_path, doc_id).await
}

pub async fn get_part_as_text(
    ai_name: Option<&str>,
    doc_id: &str,
    part_index: u64,
) -> AiterResult<Option<String>> {
    let mem_path = get_mem_path(ai_name).await?;

    if let Some(doc) = db::mem::doc::get(&mem_path, doc_id).await? {
        if doc.content_type == DocContentType::Sheet.to_string() {
            if let Some(doc_content) = db::mem::doc::get_content(&mem_path, doc_id).await? {
                let doc = SheetDoc::try_from_bytes(&doc_content)?;
                if let Some((_, sheet_data)) = doc.pages.get(part_index as usize) {
                    Ok(Some(sheet_data.to_string()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            if let Some(part) =
                db::mem::doc_part::get_by_index(&mem_path, doc_id, part_index).await?
            {
                let mut content: String = String::new();

                let segs = db::mem::doc_seg::list_by_part(&mem_path, doc_id, &part.id).await?;
                for seg in segs {
                    let seg_content =
                        content::seg::decode_content(&seg.content, &seg.content_type)?;
                    content.push_str(&seg_content.to_string());
                }

                Ok(Some(content))
            } else {
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

pub async fn list(
    ai_name: Option<&str>,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<DocEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc::list(&mem_path, search, limit, offset).await
}

pub async fn list_by_ids(ai_name: Option<&str>, ids: &[String]) -> AiterResult<Vec<DocEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc::list_by_ids(&mem_path, ids).await
}

pub async fn list_digesting(ai_name: Option<&str>, limit: u64) -> AiterResult<Vec<DocEntity>> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc::list_digesting(&mem_path, limit).await
}

pub async fn pull(
    ai_name: Option<&str>,
    doc_id: &str,
    dest_dir_path: &Path,
) -> AiterResult<Option<PathBuf>> {
    let mem_path = get_mem_path(ai_name).await?;

    let doc = db::mem::doc::get(&mem_path, doc_id).await?;

    if let Some(doc) = &doc {
        let keep_path = get_docs_dir_path(ai_name).await?.join(doc_id);
        if keep_path.exists() {
            let dest_path = dest_dir_path.join(&doc.source);
            copy(&keep_path, &dest_path)?;

            Ok(Some(dest_path))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn reset_not_digested_but_started(ai_name: Option<&str>) -> AiterResult<()> {
    let mem_path = get_mem_path(ai_name).await?;

    db::mem::doc::reset_not_digested_but_started(&mem_path).await?;
    db::mem::doc_frag::reset_not_digested_but_started(&mem_path).await?;
    db::mem::doc_part::reset_not_digested_but_started(&mem_path).await?;
    db::mem::doc_seg::reset_not_digested_but_started(&mem_path).await?;

    Ok(())
}
