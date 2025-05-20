use std::{collections::HashMap, path::Path};

use libsql::Rows;
use ulid::Ulid;

use crate::{
    DB_VECTOR_NEIGHBORS,
    db::{
        CURRENT_SIGNATURE_DIMS,
        mem::{doc, get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str,
    },
    error::AiterResult,
    utils::{
        datetime::utc_to_iso_datetime_string,
        text::{minhash, to_words},
    },
};

/// DocKnl(knowledge) is a Q&A pair.
/// The question if a "trigger", the answer is a piece of content or implicit information that comes from doc.
#[derive(Debug)]
pub struct DocKnl {
    pub id: String,
    pub doc_id: String,
    pub doc_ref: HashMap<String, String>,
    pub trigger: String,
}

#[allow(dead_code)]
pub struct DocKnlEntity {
    pub id: String,
    pub doc_id: String,
    pub doc_ref: HashMap<String, String>,
    pub trigger: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        &format!(
            r#"
CREATE TABLE IF NOT EXISTS "doc_knl" (
    "id"           TEXT PRIMARY KEY,
    "doc_id"       TEXT NOT NULL,
    "doc_ref"      TEXT NOT NULL,
    "trigger"      TEXT NOT NULL,
    "trigger_sig"  F16_BLOB({CURRENT_SIGNATURE_DIMS}),
    "created_at"   TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"   TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#
        ),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_knl_doc_id" ON "doc_knl" ("doc_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_knl_trigger" ON "doc_knl" ("trigger")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_doc_knl_trigger_sig" ON "doc_knl" (libsql_vector_idx(trigger_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "doc_knl_fts" USING fts5(
    "id"      UNINDEXED, 
    "doc_id"  UNINDEXED,
    "trigger")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn query_by_search(
    db_path: &Path,
    search: &str,
    limit: u64,
    match_keywords: bool,
) -> AiterResult<Vec<DocKnlEntity>> {
    let search_words = to_words(search, match_keywords);
    if search_words.is_empty() {
        return Ok(vec![]);
    }

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT t."id", t."doc_id", t."doc_ref", t."trigger", t."created_at", t."updated_at"
FROM "doc_knl" t JOIN "doc_knl_fts" ON t."id" = "doc_knl_fts"."id"
WHERE "doc_knl_fts" MATCH ?
ORDER BY "doc_knl_fts"."rank" DESC
LIMIT ?
;"#,
            (search_words.join(" "), limit.max(1)),
        )
        .await?;

    DocKnlEntity::collect_rows(&mut rows).await
}

pub async fn query_by_signature(
    db_path: &Path,
    signature: &[f32],
    limit: u64,
) -> AiterResult<Vec<DocKnlEntity>> {
    let sig = vec_f32_to_f16_str(signature);

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "doc_id", "doc_ref", "trigger", "created_at", "updated_at"
FROM "doc_knl"
WHERE rowid IN vector_top_k('idx_doc_knl_trigger_sig', vector16(?), ?)
;"#,
            (sig, limit.max(1)),
        )
        .await?;

    DocKnlEntity::collect_rows(&mut rows).await
}

pub async fn upsert_batch(db_path: &Path, doc_knls: &[DocKnl], context: &str) -> AiterResult<()> {
    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let mut doc_knl_map: HashMap<String, (String, Vec<String>, String)> = HashMap::new();
    for doc_knl in doc_knls {
        let trigger_str = doc_knl.trigger.trim();
        if trigger_str.is_empty() {
            continue;
        }

        let trigger_with_context = format!("**{}** {}", context, &trigger_str);
        let trigger_words = to_words(&trigger_with_context, false);
        let trigger_sig =
            vec_f32_to_f16_str(&minhash(&trigger_with_context, signature_dims, &tokenizer)?);

        let doc_ref = serde_json::to_string(&doc_knl.doc_ref)?;
        doc_knl_map.insert(doc_knl.id.clone(), (doc_ref, trigger_words, trigger_sig));
    }

    let conn = &mut open(db_path).await?;
    let tx = conn.transaction().await?;

    for doc_knl in doc_knls {
        if doc::get(db_path, &doc_knl.doc_id).await.ok().is_none() {
            continue;
        }

        if let Some((doc_ref, trigger_words, trigger_sig)) = doc_knl_map.get(&doc_knl.id) {
            let exists_id: Option<String> = {
                tx.query(
                    r#"
SELECT "id" 
FROM "doc_knl" 
WHERE "trigger" = ? AND "doc_id" = ?
LIMIT 1
;"#,
                    (doc_knl.trigger.as_str(), doc_knl.doc_id.as_str()),
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
UPDATE "doc_knl"
SET 
    "id" = ?, 
    "doc_ref" = ?, 
    "updated_at" = CURRENT_TIMESTAMP
WHERE "id" = ?
;"#,
                    (doc_knl.id.as_str(), doc_ref.clone(), id),
                )
                .await?;
            } else {
                tx.execute(
                    r#"
INSERT INTO "doc_knl" 
    ("id", "doc_id", "doc_ref", "trigger", "trigger_sig") 
VALUES 
    (?, ?, ?, ?, vector16(?))
;"#,
                    (
                        doc_knl.id.as_str(),
                        doc_knl.doc_id.as_str(),
                        doc_ref.clone(),
                        doc_knl.trigger.as_str(),
                        trigger_sig.as_str(),
                    ),
                )
                .await?;

                tx.execute(
                    r#"
INSERT INTO "doc_knl_fts" 
    ("id", "doc_id", "trigger") 
VALUES 
    (?, ?, ?)
;"#,
                    (
                        doc_knl.id.as_str(),
                        doc_knl.doc_id.as_str(),
                        trigger_words.join(" "),
                    ),
                )
                .await?;
            }
        }
    }

    tx.commit().await?;

    Ok(())
}

impl DocKnl {
    pub fn new(doc_id: &str, doc_ref: HashMap<String, String>, trigger: &str) -> Self {
        Self {
            id: Ulid::new().to_string(),
            doc_id: doc_id.to_string(),
            doc_ref,
            trigger: trigger.to_string(),
        }
    }
}

impl DocKnlEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                doc_id: row.get(1)?,
                doc_ref: serde_json::from_str(&row.get::<String>(2)?).unwrap_or_default(),
                trigger: row.get(3)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(4)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(5)?),
            });
        }

        Ok(vec)
    }
}
