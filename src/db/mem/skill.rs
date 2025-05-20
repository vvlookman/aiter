use std::path::Path;

use libsql::Rows;
use serde::Serialize;
use tabled::Tabled;
use ulid::Ulid;

use crate::{
    DB_VECTOR_NEIGHBORS,
    db::{
        CURRENT_SIGNATURE_DIMS,
        mem::{get_mem_signature_dims, get_mem_tokenizer},
        open, vec_f32_to_f16_str,
    },
    error::AiterResult,
    utils::{
        datetime::utc_to_iso_datetime_string,
        text::{minhash, to_words},
    },
};

/// Skill is the ability to use a tool.
#[derive(Clone, Debug)]
pub struct Skill {
    pub id: String,
    pub tool_id: String,
    pub trigger: String,
}

#[derive(Clone, Debug, Tabled, Serialize)]
pub struct SkillEntity {
    #[tabled(skip)]
    pub id: String,

    #[tabled(rename = "Tool ID")]
    pub tool_id: String,

    #[tabled(rename = "Trigger")]
    pub trigger: String,

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
CREATE TABLE IF NOT EXISTS "skill" (
    "id"           TEXT PRIMARY KEY,
    "tool_id"      TEXT NOT NULL UNIQUE,
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
CREATE INDEX IF NOT EXISTS "idx_skill_tool_id" ON "skill" ("tool_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_skill_updated_at" ON "skill" ("updated_at")
;"#,
        (),
    )
    .await?;

    tx.execute(
        &format!(
        r#"
CREATE INDEX IF NOT EXISTS "idx_skill_trigger_sig" ON "skill" (libsql_vector_idx(trigger_sig, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "skill_fts" USING fts5(
    "id"       UNINDEXED,
    "tool_id"  UNINDEXED,
    "trigger")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn delete(db_path: &Path, skill_id: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
DELETE FROM "skill" 
WHERE "id" = ?
;"#,
        [skill_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "skill_fts" 
WHERE "id" = ?
;"#,
        [skill_id],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn delete_by_tool(db_path: &Path, tool_id: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
DELETE FROM "skill" 
WHERE "tool_id" = ?
;"#,
        [tool_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "skill_fts" 
WHERE "tool_id" = ?
;"#,
        [tool_id],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get(db_path: &Path, id: &str) -> AiterResult<Option<SkillEntity>> {
    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "tool_id", "trigger", "created_at", "updated_at"
FROM "skill"
WHERE "id" = ?;
LIMIT 1
;"#,
            [id],
        )
        .await?;

    Ok(SkillEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list(
    db_path: &Path,
    search: &str,
    limit: u64,
    offset: u64,
) -> AiterResult<Vec<SkillEntity>> {
    let search_words = to_words(search, false);

    let conn = open(db_path).await?;
    let mut rows = if search_words.is_empty() {
        conn.query(
            r#"
SELECT "id", "tool_id", "trigger", "created_at", "updated_at"
FROM "skill"
ORDER BY "updated_at" DESC
LIMIT ? 
OFFSET ?
;"#,
            (limit.max(1), offset),
        )
        .await?
    } else {
        conn.query(
            r#"
SELECT t."id", t."tool_id", t."trigger", t."created_at", t."updated_at"
FROM "skill" t JOIN "skill_fts" ON t."id" = "skill_fts"."id"
WHERE "skill_fts" MATCH ?
ORDER BY "updated_at" DESC
LIMIT ?
OFFSET ?
;"#,
            (search_words.join(" "), limit.max(1), offset),
        )
        .await?
    };

    SkillEntity::collect_rows(&mut rows).await
}

pub async fn query_by_search(
    db_path: &Path,
    search: &str,
    limit: u64,
    match_keywords: bool,
) -> AiterResult<Vec<SkillEntity>> {
    let search_words = to_words(search, match_keywords);
    if search_words.is_empty() {
        return Ok(vec![]);
    }

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT t."id", t."tool_id", t."trigger", t."created_at", t."updated_at"
FROM "skill" t JOIN "skill_fts" ON t."id" = "skill_fts"."id"
WHERE "skill_fts" MATCH ?
ORDER BY "skill_fts"."rank" DESC
LIMIT ?
;"#,
            (search_words.join(" "), limit.max(1)),
        )
        .await?;

    SkillEntity::collect_rows(&mut rows).await
}

pub async fn query_by_signature(
    db_path: &Path,
    signature: &[f32],
    limit: u64,
) -> AiterResult<Vec<SkillEntity>> {
    let sig = vec_f32_to_f16_str(signature);

    let conn = open(db_path).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "tool_id", "trigger", "created_at", "updated_at"
FROM "skill"
WHERE rowid IN vector_top_k('idx_skill_trigger_sig', vector16(?), ?)
;"#,
            (sig, limit.max(1)),
        )
        .await?;

    SkillEntity::collect_rows(&mut rows).await
}

pub async fn upsert(db_path: &Path, skill: &Skill, context: &str) -> AiterResult<Option<String>> {
    let signature_dims = get_mem_signature_dims(db_path);
    let tokenizer = get_mem_tokenizer(db_path);

    let trigger_with_context = format!("**{}** {}", context, &skill.trigger);
    let trigger_words = to_words(&trigger_with_context, false);
    let trigger_sig =
        vec_f32_to_f16_str(&minhash(&trigger_with_context, signature_dims, &tokenizer)?);

    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    let return_id = {
        let exists_id: Option<String> = {
            tx.query(
                r#"
SELECT "id" 
FROM "skill" 
WHERE "tool_id" = ?
LIMIT 1
;"#,
                [skill.tool_id.as_str()],
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
UPDATE "skill"
SET "updated_at" = CURRENT_TIMESTAMP, "trigger" = ?, "trigger_sig" = vector16(?)
WHERE "id" = ?
;"#,
                (skill.trigger.as_str(), trigger_sig.as_str(), id.as_str()),
            )
            .await?;

            tx.execute(
                r#"
UPDATE "skill_fts"
SET "trigger" = ?
WHERE "id" = ?
;"#,
                (trigger_words.join(" "), id.as_str()),
            )
            .await?;

            id
        } else {
            tx.execute(
                r#"
INSERT INTO "skill" 
    ("id", "tool_id", "trigger", "trigger_sig") 
VALUES 
    (?, ?, ?, vector16(?))
;"#,
                (
                    skill.id.as_str(),
                    skill.tool_id.as_str(),
                    skill.trigger.as_str(),
                    trigger_sig.as_str(),
                ),
            )
            .await?;

            tx.execute(
                r#"
INSERT INTO "skill_fts" 
    ("id", "tool_id","trigger") 
VALUES 
    (?, ?, ?)
;"#,
                (
                    skill.id.as_str(),
                    skill.tool_id.as_str(),
                    trigger_words.join(" "),
                ),
            )
            .await?;

            skill.id.clone()
        }
    };

    tx.commit().await?;

    Ok(Some(return_id))
}

impl Skill {
    pub fn new(tool_id: &str, trigger: &str) -> Self {
        Self {
            id: Ulid::new().to_string(),
            tool_id: tool_id.to_string(),
            trigger: trigger.to_string(),
        }
    }
}

impl SkillEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                tool_id: row.get(1)?,
                trigger: row.get(2)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(3)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(4)?),
            });
        }

        Ok(vec)
    }
}
