use libsql::Rows;
use serde::Serialize;
use tabled::Tabled;
use ulid::Ulid;

use crate::{
    DB_CORE_PATH, db::open, error::AiterResult, utils::datetime::utc_to_iso_datetime_string,
};

#[derive(Serialize, Tabled)]
pub struct AiEntity {
    #[tabled(skip)]
    pub id: String,

    #[tabled(rename = "Name")]
    pub name: String,

    #[tabled(rename = "Created Time")]
    pub created_at: String,

    #[tabled(rename = "Updated Time")]
    pub updated_at: String,
}

pub async fn ensure_tables() -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
CREATE TABLE IF NOT EXISTS "ai" (
    "id"          TEXT PRIMARY KEY,
    "name"        TEXT NOT NULL,
    "created_at"  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "updated_at"  TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
        (),
    )
    .await?;

    Ok(())
}

pub async fn insert(name: &str) -> AiterResult<String> {
    let id = Ulid::new().to_string();

    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
INSERT INTO "ai" 
    ("id", "name") 
VALUES 
    (?, ?)
;"#,
        (id.as_str(), name),
    )
    .await?;

    Ok(id)
}

pub async fn delete_by_name(name: &str) -> AiterResult<Option<String>> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.query(
        r#"
DELETE FROM "ai" 
WHERE "name" = ? 
RETURNING id
;"#,
        [name],
    )
    .await?
    .next()
    .await?
    .map(|row| Ok(row.get::<String>(0)?))
    .transpose()
}

pub async fn get_by_name(name: &str) -> AiterResult<Option<AiEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "name", "created_at", "updated_at" 
FROM "ai" 
WHERE "name" = ? 
LIMIT 1
;"#,
            [name],
        )
        .await?;

    Ok(AiEntity::collect_rows(&mut rows).await?.pop())
}

pub async fn list() -> AiterResult<Vec<AiEntity>> {
    let conn = open(&DB_CORE_PATH).await?;
    let mut rows = conn
        .query(
            r#"
SELECT "id", "name", "created_at", "updated_at" 
FROM "ai" 
ORDER BY "updated_at" DESC
;"#,
            (),
        )
        .await?;

    AiEntity::collect_rows(&mut rows).await
}

pub async fn set_name(id: &str, name: &str) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
UPDATE "ai" 
SET 
    "name" = ?, "updated_at" = CURRENT_TIMESTAMP
WHERE 
    "id" = ?
;"#,
        (name, id),
    )
    .await?;

    Ok(())
}

impl AiEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(2)?),
                updated_at: utc_to_iso_datetime_string(&row.get::<String>(3)?),
            });
        }

        Ok(vec)
    }
}
