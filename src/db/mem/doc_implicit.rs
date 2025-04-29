use std::path::Path;

use libsql::Rows;
use ulid::Ulid;

use crate::{
    db::{
        mem::{get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str, CURRENT_SIGNATURE_DIMS,
    },
    error::AiterResult,
    utils::{
        datetime::utc_to_iso_datetime_string,
        text::{minhash, to_words},
    },
    DB_VECTOR_NEIGHBORS,
};

/// DocImplicit is a piece of implicit information extracted from a doc.
#[derive(Debug)]
pub struct DocImplicit {
    pub id: String,
    pub doc_id: String,
    pub content: String,
}

#[allow(dead_code)]
pub struct DocImplicitEntity {
    pub id: String,
    pub doc_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        &format!(
            r#"
CREATE TABLE IF NOT EXISTS "doc_implicit" (
    "id"            TEXT PRIMARY KEY,
    "doc_id"        TEXT NOT NULL,
    "content"       TEXT NOT NULL,
    "content_size"  INTEGER DEFAULT 0,
    "content_sig"   F16_BLOB({}),
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
CREATE INDEX IF NOT EXISTS "idx_doc_implicit_doc_id" ON "doc_implicit" ("doc_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_implicit_content" ON "doc_implicit" ("content")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_implicit_content_sig" ON "doc_implicit" (libsql_vector_idx(content_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "doc_implicit_fts" USING fts5(
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

pub async fn get(db_path: &Path, id: &str) -> AiterResult<Option<DocImplicitEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "content", "created_at", "updated_at"
FROM "doc_implicit"
WHERE "id" = ?;
LIMIT 1
;"#,
            [id],
        )
        .await?;

    Ok(DocImplicitEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn query_by_search(
    db_path: &Path,
    search: &str,
    limit: u64,
    match_keywords: bool,
) -> AiterResult<Vec<DocImplicitEntity>> {
    let search_words = to_words(search, match_keywords);
    if search_words.is_empty() {
        return Ok(vec![]);
    }

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT t."id", t."doc_id", t."content", t."created_at", t."updated_at"
FROM "doc_implicit" t JOIN "doc_implicit_fts" ON t."id" = "doc_implicit_fts"."id"
WHERE "doc_implicit_fts" MATCH ?
ORDER BY "doc_implicit_fts"."rank" DESC
LIMIT ?
;"#,
            (search_words.join(" "), limit.max(1)),
        )
        .await?;

    DocImplicitEntity::collect_rows(&mut rows).await
}

pub async fn query_by_signature(
    db_path: &Path,
    signature: &[f32],
    limit: u64,
) -> AiterResult<Vec<DocImplicitEntity>> {
    let sig = vec_f32_to_f16_str(signature);

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "content", "created_at", "updated_at"
FROM "doc_implicit"
WHERE rowid IN vector_top_k('idx_doc_implicit_content_sig', vector16(?), ?)
;"#,
            (sig, limit.max(1)),
        )
        .await?;

    DocImplicitEntity::collect_rows(&mut rows).await
}

pub async fn upsert(
    db_path: &Path,
    doc_implicit: &DocImplicit,
    context: &str,
) -> AiterResult<Option<String>> {
    let content_str = doc_implicit.content.trim();
    if content_str.is_empty() {
        return Ok(None);
    }

    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let content_with_context = format!("**{}** {}", context, content_str);
    let content_words = to_words(&content_with_context, false);
    let content_sig =
        vec_f32_to_f16_str(&minhash(&content_with_context, signature_dims, &tokenizer)?);

    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    let return_id = {
        let exists_id: Option<String> = {
            tx.query(
                r#"
SELECT "id" 
FROM "doc_implicit" 
WHERE "content" = ? AND "doc_id" = ?
LIMIT 1
;"#,
                (content_str, doc_implicit.doc_id.as_str()),
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
UPDATE "doc_implicit"
SET "updated_at" = CURRENT_TIMESTAMP
WHERE "id" = ?
;"#,
                [id.as_str()],
            )
            .await?;

            id
        } else {
            tx.execute(
                r#"
INSERT INTO "doc_implicit" 
    ("id", "doc_id", "content", "content_size", "content_sig") 
VALUES 
    (?, ?, ?, ?, vector16(?))
;"#,
                (
                    doc_implicit.id.as_str(),
                    doc_implicit.doc_id.as_str(),
                    content_str,
                    content_str.len() as u64,
                    content_sig,
                ),
            )
            .await?;

            tx.execute(
                r#"
INSERT INTO "doc_implicit_fts" 
    ("id", "doc_id", "content") 
VALUES 
    (?, ?, ?)
;"#,
                (
                    doc_implicit.id.as_str(),
                    doc_implicit.doc_id.as_str(),
                    content_words.join(" "),
                ),
            )
            .await?;

            doc_implicit.id.clone()
        }
    };

    tx.commit().await?;

    Ok(Some(return_id))
}

impl DocImplicit {
    pub fn new(doc_id: &str, content: &str) -> Self {
        Self {
            id: Ulid::new().to_string(),
            doc_id: doc_id.to_string(),
            content: content.to_string(),
        }
    }
}

impl DocImplicitEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                doc_id: row.get(1)?,
                content: row.get(2)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(3)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(4)?),
            });
        }

        Ok(vec)
    }
}
