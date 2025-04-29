use crate::{db::open, error::*, CURRENT_DB_VERSION, DB_CORE_PATH};

pub async fn ensure_tables() -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
CREATE TABLE IF NOT EXISTS "meta" (
    "key"    TEXT PRIMARY KEY,
    "value"  TEXT NOT NULL)
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
INSERT OR IGNORE INTO "meta" ("key", "value")
VALUES ('db_version', ?)
;"#,
        [CURRENT_DB_VERSION],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_db_version() -> AiterResult<Option<String>> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.query(
        r#"
SELECT "value"
FROM "meta" 
WHERE "key" = 'db_version' 
LIMIT 1
;"#,
        (),
    )
    .await?
    .next()
    .await?
    .map(|row| Ok(row.get::<String>(0)?))
    .transpose()
}
