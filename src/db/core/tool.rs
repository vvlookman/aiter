use libsql::Rows;
use serde::Serialize;
use tabled::Tabled;

use crate::{
    db::{core::tool::datetime::utc_to_iso_datetime_string, open},
    error::AiterResult,
    utils::{datetime, text::to_words},
    DB_CORE_PATH,
};

#[derive(Debug, Serialize)]
pub struct Tool {
    pub id: String,
    pub toolset_id: String,
    pub toolset_title: String,
    pub toolset_options: serde_json::Value,
    pub r#type: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Tabled)]
pub struct ToolEntity {
    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Toolset ID")]
    pub toolset_id: String,

    #[tabled(rename = "Toolset")]
    pub toolset_title: String,

    #[tabled(skip)]
    pub toolset_options: String,

    #[tabled(rename = "Type")]
    pub r#type: String,

    #[tabled(rename = "Name")]
    pub name: String,

    #[tabled(rename = "Description")]
    pub description: String,

    #[tabled(skip)]
    pub parameters: String,

    #[tabled(rename = "Created Time")]
    pub created_at: String,

    #[tabled(rename = "Updated Time")]
    pub updated_at: String,
}

#[derive(Serialize, Tabled)]
pub struct ToolsetEntity {
    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Title")]
    pub title: String,
}

pub async fn ensure_tables() -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
CREATE TABLE IF NOT EXISTS "tool" (
    "id"               TEXT PRIMARY KEY,
    "toolset_id"       TEXT NOT NULL,
    "toolset_title"    TEXT NOT NULL,
    "toolset_options"  TEXT,
    "type"             TEXT NOT NULL,
    "name"             TEXT NOT NULL,
    "description"      TEXT NOT NULL,
    "parameters"       TEXT,
    "created_at"       TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"       TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_tool_toolset_id" ON "tool" ("toolset_id")
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_tool_updated_at" ON "tool" ("updated_at")
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE VIRTUAL TABLE IF NOT EXISTS "tool_fts" USING fts5(
    "id"             UNINDEXED,
    "toolset_title",
    "description")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn delete(tool_id: &str) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
DELETE FROM "tool" 
WHERE "id" = ?
;"#,
        [tool_id],
    )
    .await?;

    tx.execute(
        r#"
DELETE FROM "tool_fts" 
WHERE "id" = ?
;"#,
        [tool_id],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get(tool_id: &str) -> AiterResult<Option<ToolEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "toolset_id", "toolset_title", "toolset_options", "type", "name", "description", "parameters", "created_at", "updated_at" 
FROM "tool" 
WHERE "id" = ? 
LIMIT 1
;"#,
            [tool_id],
        )
    .await?;

    Ok(ToolEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn insert(tool: &Tool) -> AiterResult<()> {
    let toolset_title_words = to_words(&tool.toolset_title, false);
    let description_words = to_words(&tool.description, false);

    let conn = open(&DB_CORE_PATH).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
INSERT INTO "tool" 
    ("id", "toolset_id", "toolset_title", "toolset_options", "type", "name", "description", "parameters") 
VALUES 
    (?, ?, ?, ?, ?, ?, ?, ?)
;"#,
        (
            tool.id.as_str(),
            tool.toolset_id.as_str(),
            tool.toolset_title.as_str(),
            tool.toolset_options.to_string(),
            tool.r#type.as_str(),
            tool.name.as_str(),
            tool.description.as_str(),
            tool.parameters.to_string(),
        ),
    ).await?;

    tx.execute(
        r#"
INSERT INTO "tool_fts" 
    ("id", "toolset_title", "description") 
VALUES 
    (?, ?, ?)
;"#,
        (
            tool.id.as_str(),
            toolset_title_words.join(" "),
            description_words.join(" "),
        ),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn list(search: &str, limit: u64, offset: u64) -> AiterResult<Vec<ToolEntity>> {
    let search_words = to_words(search, false);

    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = if search_words.is_empty() {
        conn.query(
            r#"
SELECT "id", "toolset_id", "toolset_title", "toolset_options", "type", "name", "description", "parameters", "created_at", "updated_at" 
FROM "tool"
ORDER BY "updated_at" DESC
LIMIT ? 
OFFSET ?
;"#, (limit.max(1), offset)).await?
    } else {
        conn.query(
            r#"
SELECT t."id", t."toolset_id", t."toolset_title", t."toolset_options", t."type", t."name", t."description", t."parameters", t."created_at", t."updated_at" 
FROM "tool" t JOIN "tool_fts" ON t."id" = "tool_fts"."id"
WHERE "tool_fts" MATCH ?
ORDER BY "updated_at" DESC
LIMIT ?
OFFSET ?
;"#, (search_words.join(" "), limit.max(1), offset)
        ).await?
    };

    ToolEntity::collect_rows(&mut rows).await
}

pub async fn list_by_ids(ids: &[String]) -> AiterResult<Vec<ToolEntity>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = vec!["?"; ids.len()].join(",");

    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn.query(&format!(r#"
SELECT "id", "toolset_id", "toolset_title", "toolset_options", "type", "name", "description", "parameters", "created_at", "updated_at" 
FROM "tool"
WHERE "id" IN ({placeholders})
ORDER BY "updated_at" DESC
;"#
), ids.to_vec()).await?;

    ToolEntity::collect_rows(&mut rows).await
}

pub async fn list_toolsets() -> AiterResult<Vec<ToolsetEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT DISTINCT "toolset_id", "toolset_title" 
FROM "tool"
ORDER BY "updated_at" DESC
;"#,
            (),
        )
        .await?;

    ToolsetEntity::collect_rows(&mut rows).await
}

pub async fn query_by_toolset(toolset_id: &str) -> AiterResult<Vec<ToolEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn.query(
            r#"
SELECT "id", "toolset_id", "toolset_title", "toolset_options", "type", "name", "description", "parameters", "created_at", "updated_at" 
FROM "tool"
WHERE "toolset_id" = ?
;"#, [toolset_id]
        ).await?;

    ToolEntity::collect_rows(&mut rows).await
}

impl ToolEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                toolset_id: row.get(1)?,
                toolset_title: row.get(2)?,
                toolset_options: row.get::<Option<String>>(3)?.unwrap_or_default(),
                r#type: row.get(4)?,
                name: row.get(5)?,
                description: row.get(6)?,
                parameters: row.get::<Option<String>>(7)?.unwrap_or_default(),
                created_at: utc_to_iso_datetime_string(&row.get::<String>(8)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(9)?),
            });
        }

        Ok(vec)
    }
}

impl ToolsetEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                title: row.get(1)?,
            });
        }

        Ok(vec)
    }
}
