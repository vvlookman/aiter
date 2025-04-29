use std::path::Path;

use libsql::Rows;
use ulid::Ulid;

use crate::{
    content,
    content::seg::SegContent,
    db::{
        mem::{get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str, CURRENT_SIGNATURE_DIMS,
    },
    error::AiterResult,
    utils::{crypto::sha256, datetime::utc_to_iso_datetime_string, text::minhash},
    DB_VECTOR_NEIGHBORS, DIGEST_RETRY,
};

/// DocSeg(segment) is a continuous piece of content in a part of doc that has a certain degree of independence and integrity.
/// For example, a chapter of a book is a part, a segment is a part of the chapter.
#[derive(Clone, Debug)]
pub struct DocSeg {
    pub id: String,
    pub doc_id: String,
    pub part_id: String,
    pub index: u64,
    pub content: Vec<u8>,
    pub content_type: String,
    pub summary: Option<String>,
}

#[allow(dead_code)]
pub struct DocSegEntity {
    pub id: String,
    pub doc_id: String,
    pub part_id: String,
    pub index: u64,
    pub content: Vec<u8>,
    pub content_type: String,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        &format!(
            r#"
CREATE TABLE IF NOT EXISTS "doc_seg" (
    "id"            TEXT PRIMARY KEY,
    "doc_id"        TEXT NOT NULL,
    "part_id"       TEXT NOT NULL,
    "index"         INTEGER,
    "content"       BLOB NOT NULL,
    "content_type"  TEXT NOT NULL,
    "content_size"  INTEGER DEFAULT 0,
    "content_hash"  TEXT NOT NULL,
    "content_sig"   F16_BLOB({}),
    "summary"       TEXT,
    "digest_start"  TIMESTAMP,
    "digest_end"    TIMESTAMP,
    "digest_retry"  INTEGER DEFAULT 0,
    "digest_error"  TEXT,
    "created_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
            CURRENT_SIGNATURE_DIMS
        ),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_seg_doc_id" ON "doc_seg" ("doc_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_seg_content_sig" ON "doc_seg" (libsql_vector_idx(content_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get(db_path: &Path, id: &str) -> AiterResult<Option<DocSegEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "index", "content", "content_type", "summary", "created_at", "updated_at"
FROM "doc_seg"
WHERE "id" = ?
LIMIT 1
;"#,
            [id],
        )
        .await?;

    Ok(DocSegEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn get_not_digested(db_path: &Path, doc_id: &str) -> AiterResult<Option<DocSegEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "index", "content", "content_type", "summary", "created_at", "updated_at"
FROM "doc_seg"
WHERE "digest_start" IS NULL AND "digest_end" IS NULL AND "digest_retry" < ? AND "doc_id" = ?
ORDER BY "digest_retry"
LIMIT 1
;"#,
            (DIGEST_RETRY, doc_id),
        )
        .await?;

    Ok(DocSegEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list_by_part(
    db_path: &Path,
    doc_id: &str,
    part_id: &str,
) -> AiterResult<Vec<DocSegEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "index", "content", "content_type", "summary", "created_at", "updated_at"
FROM "doc_seg"
WHERE "doc_id" = ? AND "part_id" = ?
ORDER BY "index"
;"#,
            (doc_id, part_id),
        )
        .await?;

    DocSegEntity::collect_rows(&mut rows).await
}

pub async fn list_not_digested_doc_ids(db_path: &Path) -> AiterResult<Vec<String>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT DISTINCT "doc_id"
FROM "doc_seg"
WHERE "digest_end" IS NULL
;"#,
            (),
        )
        .await?;

    let mut vec = vec![];
    while let Some(row) = rows.next().await? {
        vec.push(row.get(0)?);
    }

    Ok(vec)
}

pub async fn list_summary_by_part(
    db_path: &Path,
    doc_id: &str,
    part_id: &str,
) -> AiterResult<Vec<(String, String)>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "summary"
FROM "doc_seg"
WHERE "doc_id" = ? AND "part_id" = ?
ORDER BY "index"
;"#,
            (doc_id, part_id),
        )
        .await?;

    let mut vec = vec![];
    while let Some(row) = rows.next().await? {
        vec.push((row.get(0)?, row.get(1)?));
    }

    Ok(vec)
}

pub async fn reset_not_digested(db_path: &Path) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .execute(
            r#"
UPDATE "doc_seg"
SET "digest_start" = NULL, "digest_retry" = 0
WHERE "digest_end" IS NULL
;"#,
            (),
        )
        .await?)
}

pub async fn reset_not_digested_but_started(db_path: &Path) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .execute(
            r#"
UPDATE "doc_seg"
SET "digest_start" = NULL, "digest_retry" = 0
WHERE "digest_start" IS NOT NULL AND "digest_end" IS NULL
;"#,
            (),
        )
        .await?)
}

pub async fn set_digest_end(db_path: &Path, id: &str, success: bool) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        if success {
            r#"
UPDATE "doc_seg" 
SET 
    "digest_end" = CURRENT_TIMESTAMP
WHERE 
    "id" = ?
;"#
        } else {
            r#"
UPDATE "doc_seg" 
SET 
    "digest_start" = NULL, "digest_end" = NULL, "digest_retry" = "digest_retry" + 1
WHERE 
    "id" = ?
;"#
        },
        [id],
    )
    .await?;

    Ok(())
}

pub async fn set_digest_error(db_path: &Path, id: &str, error: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "doc_seg" 
SET 
    "digest_error" = ? 
WHERE 
    "id" = ?
;"#,
        (error, id),
    )
    .await?;

    Ok(())
}

pub async fn set_digest_start(db_path: &Path, id: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "doc_seg" 
SET 
    "digest_start" = CURRENT_TIMESTAMP, "digest_error" = NULL 
WHERE 
    "id" = ?
;"#,
        [id],
    )
    .await?;

    Ok(())
}

pub async fn set_summary(db_path: &Path, id: &str, summary: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "doc_seg" 
SET 
    "summary" = ? 
WHERE 
    "id" = ?
;"#,
        (summary, id),
    )
    .await?;

    Ok(())
}

pub async fn upsert(
    db_path: &Path,
    doc_seg: &DocSeg,
    context: &str,
) -> AiterResult<Option<String>> {
    let content_str =
        content::seg::decode_content(&doc_seg.content, &doc_seg.content_type)?.to_string();
    if content_str.trim().is_empty() {
        return Ok(None);
    }

    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let content_hash = sha256(content_str.as_bytes());
    let content_with_context = format!("**{}** {}", context, content_str);
    let content_sig =
        vec_f32_to_f16_str(&minhash(&content_with_context, signature_dims, &tokenizer)?);

    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    let return_id = {
        let exists_id: Option<String> = {
            tx.query(
                r#"
SELECT "id" 
FROM "doc_seg" 
WHERE "content_hash" = ? AND "doc_id" = ?
;"#,
                (content_hash.as_str(), doc_seg.doc_id.as_str()),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<String>(0))
            .transpose()?
        };

        if let Some(id) = exists_id {
            tx.execute(
                r#"
UPDATE "doc_seg"
SET 
    "id" = ?, 
    "part_id" = ?,
    "index" = ?, 
    "updated_at" = CURRENT_TIMESTAMP,
    "digest_end" = NULL
WHERE "id" = ?
;"#,
                (
                    doc_seg.id.as_str(),
                    doc_seg.part_id.as_str(),
                    doc_seg.index,
                    id.as_str(),
                ),
            )
            .await?;

            id
        } else {
            tx.execute(
            r#"
INSERT INTO "doc_seg" 
    ("id", "doc_id", "part_id", "index", "content", "content_type", "content_size", "content_hash", "content_sig", "summary") 
VALUES 
    (?, ?, ?, ?, ?, ?, ?, ?, vector16(?), ?)
;"#,
            (
                doc_seg.id.as_str(),
                doc_seg.doc_id.as_str(),
                doc_seg.part_id.as_str(),
                doc_seg.index,
                doc_seg.content.clone(),
                doc_seg.content_type.as_str(),
                doc_seg.content.len() as u64,
                content_hash.as_str(),
                content_sig.as_str(),
                doc_seg.summary.clone(),
            ),
        ).await?;

            doc_seg.id.clone()
        }
    };

    tx.commit().await?;

    Ok(Some(return_id))
}

impl DocSeg {
    pub fn new<T: SegContent + ?Sized>(
        doc_id: &str,
        part_id: &str,
        index: u64,
        seg_content: &T,
    ) -> AiterResult<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            doc_id: doc_id.to_string(),
            part_id: part_id.to_string(),
            index,
            content: seg_content.try_into_bytes()?,
            content_type: seg_content.get_type().to_string(),
            summary: None,
        })
    }
}

impl DocSegEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                doc_id: row.get(1)?,
                part_id: row.get(2)?,
                index: row.get(3)?,
                content: row.get(4)?,
                content_type: row.get(5)?,
                summary: row.get(6)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(7)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(8)?),
            });
        }

        Ok(vec)
    }
}
