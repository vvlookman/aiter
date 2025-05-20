use std::path::Path;

use libsql::Rows;
use ulid::Ulid;

use crate::{
    DB_VECTOR_NEIGHBORS, DIGEST_RETRY, content,
    content::frag::FragContent,
    db::{
        CURRENT_SIGNATURE_DIMS,
        mem::{get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str,
    },
    error::AiterResult,
    utils::{
        crypto::sha256,
        datetime::utc_to_iso_datetime_string,
        text::{minhash, to_words},
    },
};

/// DocFrag(fragment) is a tiny piece of content in a doc, such as several sentences.
#[derive(Clone, Debug)]
pub struct DocFrag {
    pub id: String,
    pub doc_id: String,
    pub part_id: String,
    pub seg_id: String,
    pub index: u64,
    pub content: Vec<u8>,
    pub content_type: String,
}

#[allow(dead_code)]
pub struct DocFragEntity {
    pub id: String,
    pub doc_id: String,
    pub part_id: String,
    pub seg_id: String,
    pub index: u64,
    pub content: Vec<u8>,
    pub content_type: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        &format!(
            r#"
CREATE TABLE IF NOT EXISTS "doc_frag" (
    "id"            TEXT PRIMARY KEY,
    "doc_id"        TEXT NOT NULL,
    "part_id"       TEXT NOT NULL,
    "seg_id"        TEXT NOT NULL,
    "index"         INTEGER,
    "content"       BLOB NOT NULL,
    "content_type"  TEXT NOT NULL,
    "content_size"  INTEGER DEFAULT 0,
    "content_hash"  TEXT NOT NULL,
    "content_sig"   F16_BLOB({CURRENT_SIGNATURE_DIMS}),
    "digest_start"  TIMESTAMP,
    "digest_end"    TIMESTAMP,
    "digest_retry"  INTEGER DEFAULT 0,
    "digest_error"  TEXT,
    "created_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#
        ),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_frag_doc_id" ON "doc_frag" ("doc_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_frag_content_sig" ON "doc_frag" (libsql_vector_idx(content_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "doc_frag_fts" USING fts5(
    "id"      UNINDEXED, 
    "doc_id"  UNINDEXED,
    "content")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get(db_path: &Path, id: &str) -> AiterResult<Option<DocFragEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "seg_id", "index", "content", "content_type", "created_at", "updated_at"
FROM "doc_frag"
WHERE "id" = ?
LIMIT 1
;"#,
            [id],
        )
        .await?;

    Ok(DocFragEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn get_by_index(
    db_path: &Path,
    doc_id: &str,
    seg_id: &str,
    index: u64,
) -> AiterResult<Option<DocFragEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "seg_id", "index", "content", "content_type", "created_at", "updated_at"
FROM "doc_frag"
WHERE "doc_id" = ? AND "seg_id" = ? AND "index" = ?
LIMIT 1
;"#,
             (doc_id, seg_id, index),
        )
        .await?;

    Ok(DocFragEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn get_not_digested(db_path: &Path, doc_id: &str) -> AiterResult<Option<DocFragEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "seg_id", "index", "content", "content_type", "created_at", "updated_at"
FROM "doc_frag"
WHERE "digest_start" IS NULL AND "digest_end" IS NULL AND "digest_retry" < ? AND "doc_id" = ?
ORDER BY "digest_retry"
LIMIT 1
;"#,
             (DIGEST_RETRY, doc_id),
        )
        .await?;

    Ok(DocFragEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list_not_digested_doc_ids(db_path: &Path) -> AiterResult<Vec<String>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT DISTINCT "doc_id"
FROM "doc_frag"
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

pub async fn query_by_search(
    db_path: &Path,
    search: &str,
    limit: u64,
    match_keywords: bool,
) -> AiterResult<Vec<DocFragEntity>> {
    let search_words = to_words(search, match_keywords);
    if search_words.is_empty() {
        return Ok(vec![]);
    }

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT t."id", t."doc_id", t."part_id", t."seg_id", t."index", t."content", t."content_type", t."created_at", t."updated_at"
FROM "doc_frag" t JOIN "doc_frag_fts" ON t."id" = "doc_frag_fts"."id"
WHERE "doc_frag_fts" MATCH ?
ORDER BY "doc_frag_fts"."rank" DESC
LIMIT ?
;"#,
            (search_words.join(" "), limit.max(1)),
        )
        .await?;

    DocFragEntity::collect_rows(&mut rows).await
}

pub async fn query_by_signature(
    db_path: &Path,
    signature: &[f32],
    limit: u64,
) -> AiterResult<Vec<DocFragEntity>> {
    let sig = vec_f32_to_f16_str(signature);

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "part_id", "seg_id", "index", "content", "content_type", "created_at", "updated_at"
FROM "doc_frag"
WHERE rowid IN vector_top_k('idx_doc_frag_content_sig', vector16(?), ?)
;"#,
            (sig, limit.max(1)),
        )
        .await?;

    DocFragEntity::collect_rows(&mut rows).await
}

pub async fn reset_not_digested(db_path: &Path) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .execute(
            r#"
UPDATE "doc_frag"
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
UPDATE "doc_frag"
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
UPDATE "doc_frag" 
SET 
    "digest_end" = CURRENT_TIMESTAMP
WHERE 
    "id" = ?
;"#
        } else {
            r#"
UPDATE "doc_frag" 
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
UPDATE "doc_frag" 
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
UPDATE "doc_frag" 
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

pub async fn upsert(db_path: &Path, doc_frag: &DocFrag, context: &str) -> AiterResult<()> {
    let content_str =
        content::frag::decode_content(&doc_frag.content, &doc_frag.content_type)?.to_string();
    if content_str.trim().is_empty() {
        return Ok(());
    }

    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let content_hash = sha256(content_str.as_bytes());
    let content_with_context = format!("**{context}** {content_str}");
    let content_words = to_words(&content_with_context, false);
    let content_sig =
        vec_f32_to_f16_str(&minhash(&content_with_context, signature_dims, &tokenizer)?);

    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    let exists_id: Option<String> = {
        tx.query(
            r#"
SELECT "id" 
FROM "doc_frag" 
WHERE "content_hash" = ? AND "seg_id" = ?
;"#,
            (content_hash.as_str(), doc_frag.seg_id.as_str()),
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
UPDATE "doc_frag"
SET 
    "id" = ?, 
    "index" = ?,
    "updated_at" = CURRENT_TIMESTAMP,
    "digest_end" = NULL
WHERE "id" = ?
;"#,
            (doc_frag.id.as_str(), doc_frag.index, id.as_str()),
        )
        .await?;
    } else {
        tx.execute(
            r#"
INSERT INTO "doc_frag" 
    ("id", "doc_id", "part_id", "seg_id", "index", "content", "content_type", "content_size", "content_hash", "content_sig") 
VALUES 
    (?, ?, ?, ?, ?, ?, ?, ?, ?, vector16(?))
;"#,
            (
                doc_frag.id.as_str(),
                doc_frag.doc_id.as_str(),
                doc_frag.part_id.as_str(),
                doc_frag.seg_id.as_str(),
                doc_frag.index,
                doc_frag.content.clone(),
                doc_frag.content_type.as_str(),
                doc_frag.content.len() as u64,
                content_hash.as_str(),
                content_sig.as_str(),
            ),
        )
        .await?;

        tx.execute(
            r#"
INSERT INTO "doc_frag_fts" 
    ("id", "doc_id", "content") 
VALUES 
    (?, ?, ?)
;"#,
            (
                doc_frag.id.as_str(),
                doc_frag.doc_id.as_str(),
                content_words.join(" "),
            ),
        )
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

impl DocFrag {
    pub fn new<T: FragContent + ?Sized>(
        doc_id: &str,
        part_id: &str,
        seg_id: &str,
        index: u64,
        frag_content: &T,
    ) -> AiterResult<Self> {
        Ok(Self {
            id: Ulid::new().to_string(),
            doc_id: doc_id.to_string(),
            part_id: part_id.to_string(),
            seg_id: seg_id.to_string(),
            index,
            content: frag_content.try_into_bytes()?,
            content_type: frag_content.get_type().to_string(),
        })
    }
}

impl DocFragEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                doc_id: row.get(1)?,
                part_id: row.get(2)?,
                seg_id: row.get(3)?,
                index: row.get(4)?,
                content: row.get(5)?,
                content_type: row.get(6)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(7)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(8)?),
            });
        }

        Ok(vec)
    }
}
