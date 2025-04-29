use std::path::Path;

use libsql::Rows;
use ulid::Ulid;

use crate::{
    db::open, error::AiterResult, utils::datetime::utc_to_iso_datetime_string, DIGEST_RETRY,
};

/// DocPart is a complete and separate piece of content in a doc.
/// For example, a book is a doc, a chapter is a part.
#[derive(Clone, Debug)]
pub struct DocPart {
    pub id: String,
    pub doc_id: String,
    pub index: u64,
    pub title: Option<String>,
    pub summary: Option<String>,
}

#[allow(dead_code)]
pub struct DocPartEntity {
    pub id: String,
    pub doc_id: String,
    pub index: u64,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
CREATE TABLE IF NOT EXISTS "doc_part" (
    "id"            TEXT PRIMARY KEY,
    "doc_id"        TEXT NOT NULL,
    "index"         INTEGER,
    "title"         TEXT,
    "summary"       TEXT,
    "digest_start"  TIMESTAMP,
    "digest_end"    TIMESTAMP,
    "digest_retry"  INTEGER DEFAULT 0,
    "digest_error"  TEXT,
    "created_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"    TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_part_doc_id" ON "doc_part" ("doc_id")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn count(db_path: &Path, doc_id: &str) -> AiterResult<u64> {
    let conn = open(db_path).await?;
    Ok(conn
        .query(
            r#"
SELECT count(rowid)
FROM "doc_part"
WHERE "doc_id"= ?
;"#,
            [doc_id],
        )
        .await?
        .next()
        .await?
        .map(|row| row.get::<u64>(0).unwrap_or(0))
        .unwrap_or(0))
}

pub async fn get_not_digested(db_path: &Path, doc_id: &str) -> AiterResult<Option<DocPartEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "index", "title", "summary", "created_at", "updated_at"
FROM "doc_part"
WHERE "digest_start" IS NULL AND "digest_end" IS NULL AND "digest_retry" < ? AND "doc_id" = ?
ORDER BY "digest_retry"
LIMIT 1
;"#,
            (DIGEST_RETRY, doc_id),
        )
        .await?;

    Ok(DocPartEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list_not_digested_doc_ids(db_path: &Path) -> AiterResult<Vec<String>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT DISTINCT "doc_id"
FROM "doc_part"
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

pub async fn get_by_index(
    db_path: &Path,
    doc_id: &str,
    part_index: u64,
) -> AiterResult<Option<DocPartEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "index", "title", "summary", "created_at", "updated_at"
FROM "doc_part"
WHERE "doc_id" = ? AND "index" = ?
LIMIT 1
;"#,
            (doc_id, part_index),
        )
        .await?;

    Ok(DocPartEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list_summary_by_doc(
    db_path: &Path,
    doc_id: &str,
) -> AiterResult<Vec<(String, String)>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "summary"
FROM "doc_part"
WHERE "doc_id" = ?
ORDER BY "index"
;"#,
            [doc_id],
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
UPDATE "doc_part"
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
UPDATE "doc_part"
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
UPDATE "doc_part" 
SET 
    "digest_end" = CURRENT_TIMESTAMP
WHERE 
    "id" = ?
;"#
        } else {
            r#"
UPDATE "doc_part" 
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
UPDATE "doc_part" 
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
UPDATE "doc_part" 
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
UPDATE "doc_part" 
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
    doc_part: &DocPart,
    _context: &str,
) -> AiterResult<Option<String>> {
    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    let return_id = {
        let exists_id: Option<String> = {
            tx.query(
                r#"
SELECT "id" 
FROM "doc_part" 
WHERE "doc_id" = ? AND "index" = ?
;"#,
                (doc_part.doc_id.as_str(), doc_part.index),
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
UPDATE "doc_part"
SET 
    "id" = ?, 
    "title" = ?,
    "summary" = ?, 
    "updated_at" = CURRENT_TIMESTAMP,
    "digest_end" = NULL
WHERE "id" = ?
;"#,
                (
                    doc_part.id.as_str(),
                    doc_part.title.clone(),
                    doc_part.summary.clone(),
                    id.as_str(),
                ),
            )
            .await?;

            id
        } else {
            tx.execute(
                r#"
INSERT INTO "doc_part" 
    ("id", "doc_id", "index", "title","summary") 
VALUES 
    (?, ?, ?, ?, ?)
;"#,
                (
                    doc_part.id.as_str(),
                    doc_part.doc_id.as_str(),
                    doc_part.index,
                    doc_part.title.clone(),
                    doc_part.summary.clone(),
                ),
            )
            .await?;

            doc_part.id.clone()
        }
    };

    tx.commit().await?;

    Ok(Some(return_id))
}

impl DocPart {
    pub fn new(doc_id: &str, index: u64, title: Option<String>) -> Self {
        Self {
            id: Ulid::new().to_string(),
            doc_id: doc_id.to_string(),
            index,
            title,
            summary: None,
        }
    }
}

impl DocPartEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                doc_id: row.get(1)?,
                index: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(5)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(6)?),
            });
        }

        Ok(vec)
    }
}
