use std::path::{Path, PathBuf};

use libsql::Rows;
use serde::Serialize;
use tabled::Tabled;
use ulid::Ulid;

use crate::{
    content,
    content::doc::DocContent,
    db::{
        mem::{get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str, CURRENT_SIGNATURE_DIMS,
    },
    error::AiterResult,
    utils::{
        crypto::sha256,
        datetime::utc_to_iso_datetime_string,
        fs::extract_filestem_from_path,
        text::{minhash, to_words},
    },
    DB_VECTOR_NEIGHBORS, DIGEST_RETRY,
};

/// Doc represents a piece of self-relevant content, usually from a book, a movie, etc.
#[derive(Clone, Debug)]
pub struct Doc {
    pub id: String,
    pub source: String,
    pub content: Vec<u8>,
    pub content_type: String,
}

#[derive(Clone, Tabled, Serialize)]
pub struct DocEntity {
    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Source")]
    pub source: String,

    #[tabled(rename = "Type")]
    pub content_type: String,

    #[tabled(rename = "Title")]
    pub title: String,

    #[tabled(rename = "Preview")]
    pub preview: String,

    #[tabled(skip)]
    pub digest_start: String,

    #[tabled(rename = "Digested Time")]
    pub digest_end: String,

    #[tabled(rename = "Digest Retried")]
    pub digest_retry: u64,

    #[tabled(rename = "Digest Error")]
    pub digest_error: String,

    #[tabled(rename = "Created Time")]
    pub created_at: String,

    #[tabled(rename = "Updated Time")]
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        &format!(
            r#"
CREATE TABLE IF NOT EXISTS "doc" (
    "id"            TEXT PRIMARY KEY,
    "source"        TEXT NOT NULL,
    "content"       BLOB NOT NULL,
    "content_type"  TEXT NOT NULL,
    "content_size"  INTEGER DEFAULT 0,
    "content_hash"  TEXT NOT NULL,
    "content_sig"   F16_BLOB({}),
    "title"         TEXT,
    "preview"       TEXT NOT NULL,
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
CREATE INDEX IF NOT EXISTS "idx_doc_updated_at" ON "doc" ("updated_at")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_content_sig" ON "doc" (libsql_vector_idx(content_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "doc_fts" USING fts5(
    "id"  UNINDEXED, 
    "source")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn count_not_digested(db_path: &Path) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .query(
            r#"
SELECT count("id")
FROM "doc"
WHERE "digest_start" IS NULL AND "digest_end" IS NULL AND "digest_retry" < ?
;"#,
            [DIGEST_RETRY],
        )
        .await?
        .next()
        .await?
        .and_then(|row| row.get::<u64>(0).ok())
        .unwrap_or(0))
}

pub async fn delete(db_path: &Path, doc_id: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
DELETE FROM "doc" 
WHERE "id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_fts" 
WHERE "id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_part" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_seg" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_frag" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_frag_fts" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_implicit" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_implicit_fts" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_knl" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "doc_knl_fts" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "skill" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "skill_fts" 
WHERE "doc_id" = ?
;"#,
        [doc_id],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get(db_path: &Path, id: &str) -> AiterResult<Option<DocEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "id" = ?;
LIMIT 1
;"#,
            [id],
        )
    .await?;

    Ok(DocEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn get_content(db_path: &Path, id: &str) -> AiterResult<Option<Vec<u8>>> {
    let conn = open(db_path).await?;
    conn.query(
        r#"
SELECT "content"
FROM "doc"
WHERE "id" = ?;
LIMIT 1
;"#,
        [id],
    )
    .await?
    .next()
    .await?
    .map(|row| Ok(row.get::<Vec<u8>>(0)?))
    .transpose()
}

pub async fn get_not_digested(db_path: &Path) -> AiterResult<Option<DocEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "digest_start" IS NULL AND "digest_end" IS NULL AND "digest_retry" < ?
ORDER BY "digest_retry"
LIMIT 1
;"#,
            [DIGEST_RETRY],
        )
    .await?;

    Ok(DocEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn get_same(db_path: &Path, doc: &Doc) -> AiterResult<Option<DocEntity>> {
    let doc_content = content::doc::decode_content(&doc.content, &doc.content_type)?;
    let content_str = doc_content.to_string();
    let content_hash = sha256(content_str.as_bytes());

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "content_hash" = ?
LIMIT 1
;"#,
            [content_hash],
        )
    .await?;

    Ok(DocEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn insert(db_path: &Path, doc: &Doc) -> AiterResult<()> {
    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let doc_content = content::doc::decode_content(&doc.content, &doc.content_type)?;
    let content_str = doc_content.to_string();
    let content_hash = sha256(content_str.as_bytes());
    let content_sig = vec_f32_to_f16_str(&minhash(&content_str, signature_dims, &tokenizer)?);
    let title = doc_content.get_title();
    let preview = doc_content.get_preview();
    let source_words = to_words(&doc.source, false);

    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
INSERT INTO "doc" 
    ("id", "source", "content", "content_type", "content_size", "content_hash", "content_sig", "title", "preview") 
VALUES 
    (?, ?, ?, ?, ?, ?, vector16(?), ?, ?)
;"#,
        (
            doc.id.as_str(),
            doc.source.as_str(),
            doc.content.clone(),
            doc.content_type.as_str(),
            doc.content.len() as u64,
            content_hash.as_str(),
            content_sig,
            title,
            preview.as_str(),
        ),
    )
    .await?;

    if !source_words.is_empty() {
        tx.execute(
            r#"
INSERT INTO "doc_fts" 
    ("id", "source") 
VALUES 
    (?, ?)
;"#,
            (doc.id.as_str(), source_words.join(" ")),
        )
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn list(
    db_path: &Path,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<DocEntity>> {
    let search_words = to_words(search, false);

    let conn = open(db_path).await?;
    let mut rows = if search_words.is_empty() {
        conn.query(
            r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
ORDER BY "updated_at" DESC
LIMIT ? 
OFFSET ?
;"#, (limit.max(1), offset)
        ).await?
    } else {
        conn.query(
            r#"
SELECT t."id", t."source", t."content_type", t."title", t."preview", t."digest_start", t."digest_end", t."digest_retry", t."digest_error", t."created_at", t."updated_at"
FROM "doc" t JOIN "doc_fts" ON t."id" = "doc_fts"."id"
WHERE "doc_fts" MATCH ?
ORDER BY "updated_at" DESC
LIMIT ?
OFFSET ?
;"#, (search_words.join(" "), limit.max(1), offset)
        ).await?
    };

    DocEntity::collect_rows(&mut rows).await
}

pub async fn list_by_ids(db_path: &Path, ids: &[String]) -> AiterResult<Vec<DocEntity>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = vec!["?"; ids.len()].join(",");

    let conn = open(db_path).await?;
    let mut rows = conn.query(&format!(r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "id" IN ({})
ORDER BY "updated_at" DESC
;"#
, placeholders), ids.to_vec()).await?;

    DocEntity::collect_rows(&mut rows).await
}

pub async fn list_digesting(db_path: &Path, limit: u64) -> AiterResult<Vec<DocEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn.query(r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "digest_start" IS NOT NULL AND "digest_end" IS NULL
ORDER BY "updated_at" DESC
LIMIT ? 
;"#, [limit.max(1)]).await?;

    DocEntity::collect_rows(&mut rows).await
}

pub async fn list_not_digested(db_path: &Path) -> AiterResult<Vec<DocEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn.query(r#"
SELECT "id", "source", "content_type", "title", "preview", "digest_start", "digest_end", "digest_retry", "digest_error", "created_at", "updated_at"
FROM "doc"
WHERE "digest_end" IS NULL
;"#, ()).await?;

    DocEntity::collect_rows(&mut rows).await
}

pub async fn reset_not_digested(db_path: &Path) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .execute(
            r#"
UPDATE "doc"
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
UPDATE "doc"
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
UPDATE "doc" 
SET 
    "digest_end" = CURRENT_TIMESTAMP
WHERE 
    "id" = ?
;"#
        } else {
            r#"
UPDATE "doc" 
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
UPDATE "doc" 
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
UPDATE "doc" 
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

pub async fn set_not_digested(db_path: &Path, doc_id: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "doc"
SET "digest_start" = NULL, "digest_end" = NULL, "digest_retry" = 0
WHERE "id" = ?
;"#,
        [doc_id],
    )
    .await?;

    Ok(())
}

pub async fn set_summary(db_path: &Path, id: &str, summary: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "doc" 
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

impl Doc {
    pub fn new(source: &str, doc_content: &dyn DocContent) -> AiterResult<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            source: source.to_string(),
            content: doc_content.try_into_bytes()?,
            content_type: doc_content.get_type().to_string(),
        })
    }
}

impl DocEntity {
    pub fn get_context(&self) -> String {
        let filestem = extract_filestem_from_path(&PathBuf::from(self.source.to_string()));

        let trimmed_title = self.title.trim();
        if trimmed_title.is_empty() {
            filestem
        } else {
            if filestem.contains(trimmed_title) {
                filestem
            } else {
                trimmed_title.to_string()
            }
        }
    }

    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                source: row.get(1)?,
                content_type: row.get(2)?,
                title: row.get::<Option<String>>(3)?.unwrap_or_default(),
                preview: row.get(4)?,
                digest_start: row
                    .get::<Option<String>>(5)?
                    .as_deref()
                    .map(utc_to_iso_datetime_string)
                    .unwrap_or_default(),
                digest_end: row
                    .get::<Option<String>>(6)?
                    .as_deref()
                    .map(utc_to_iso_datetime_string)
                    .unwrap_or_default(),
                digest_retry: row.get(7)?,
                digest_error: row.get::<Option<String>>(8)?.unwrap_or_default(),
                created_at: utc_to_iso_datetime_string(&row.get::<String>(9)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(10)?),
            });
        }

        Ok(vec)
    }
}
