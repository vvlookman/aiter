use std::path::Path;

use crate::{CURRENT_DB_VERSION, CURRENT_SIGNATURE_DIMS, CURRENT_TOKENIZER, db::open, error::*};

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
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
        [CURRENT_DB_VERSION.to_string()],
    )
    .await?;

    tx.execute(
        r#"
INSERT OR IGNORE INTO "meta" ("key", "value")
VALUES ('signature_dims', ?)
;"#,
        [CURRENT_SIGNATURE_DIMS.to_string()],
    )
    .await?;

    tx.execute(
        r#"
INSERT OR IGNORE INTO "meta" ("key", "value")
VALUES ('tokenizer', ?)
;"#,
        [CURRENT_TOKENIZER.to_string()],
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_db_version(db_path: &Path) -> AiterResult<Option<String>> {
    let conn = open(db_path).await?;
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

pub async fn get_signature_dims(db_path: &Path) -> AiterResult<Option<String>> {
    let conn = open(db_path).await?;
    conn.query(
        r#"
SELECT "value"
FROM "meta" 
WHERE "key" = 'signature_dims' 
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

pub async fn get_tokenizer(db_path: &Path) -> AiterResult<Option<String>> {
    let conn = open(db_path).await?;
    conn.query(
        r#"
SELECT "value"
FROM "meta" 
WHERE "key" = 'tokenizer' 
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
