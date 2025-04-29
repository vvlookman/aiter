use std::collections::HashMap;

use libsql::Rows;
use serde::Serialize;
use tabled::Tabled;

use crate::{db::open, error::*, utils::datetime::utc_to_iso_datetime_string, DB_CORE_PATH};

#[derive(Clone, Serialize, Tabled)]
pub struct LlmEntity {
    #[tabled(rename = "Name")]
    pub name: String,

    #[tabled(rename = "Type")]
    pub r#type: String,

    #[tabled(rename = "Protocol")]
    pub protocol: String,

    #[tabled(rename = "Options")]
    #[tabled(format("{:?}", self.options))]
    pub options: HashMap<String, String>,

    #[tabled(rename = "Created Time")]
    pub created_at: String,

    #[tabled(rename = "Updated Time")]
    pub updated_at: String,
}

pub async fn ensure_tables() -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
CREATE TABLE IF NOT EXISTS "llm" (
    "name"        TEXT PRIMARY KEY,
    "type"        TEXT NOT NULL,
    "protocol"    TEXT NOT NULL,
    "options"     TEXT NOT NULL,
    "created_at"  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"  TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
        (),
    )
    .await?;

    Ok(())
}

pub async fn delete(name: &str) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
DELETE FROM "llm"
WHERE "name" = ?
;"#,
        [name],
    )
    .await?;

    Ok(())
}

pub async fn get_by_name(name: &str) -> AiterResult<Option<LlmEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "name", "type", "protocol", "options", "created_at", "updated_at"
FROM "llm" 
WHERE "name" = ? 
LIMIT 1
;"#,
            [name],
        )
        .await?;

    Ok(LlmEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list() -> AiterResult<Vec<LlmEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "name", "type", "protocol", "options", "created_at", "updated_at"
FROM "llm" 
ORDER BY "updated_at" DESC
;"#,
            (),
        )
        .await?;

    LlmEntity::collect_rows(&mut rows).await
}

pub async fn rename(name: &str, new_name: &str) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
UPDATE "llm"
SET "name" = ?2, "updated_at" = CURRENT_TIMESTAMP
WHERE "name" = ?1
;"#,
        (name, new_name),
    )
    .await?;

    Ok(())
}

pub async fn upsert(
    name: &str,
    r#type: &str,
    protocol: &str,
    options: &HashMap<String, String>,
) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
INSERT INTO "llm" ("name", "type", "protocol", "options")
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT ("name") DO UPDATE SET
    "updated_at" = CURRENT_TIMESTAMP,
    "type" = ?2,
    "protocol" = ?3,
    "options" = ?4
;"#,
        (name, r#type, protocol, serde_json::to_string(options)?),
    )
    .await?;

    Ok(())
}

impl LlmEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                name: row.get(0)?,
                r#type: row.get(1)?,
                protocol: row.get(2)?,
                options: serde_json::from_str(&row.get::<String>(3)?)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(4)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(5)?),
            });
        }

        Ok(vec)
    }
}
