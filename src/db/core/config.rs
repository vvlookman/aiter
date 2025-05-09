use crate::{db::open, error::*, DB_CORE_PATH};

#[derive(strum::Display, strum::EnumString)]
pub enum ConfigKey {
    ActiveChatLlm,
    ActiveReasoningLlm,
    AppDigestBatch,
    AppDigestConcurrent,
    AppDigestDeep,
    AppRemoteUrl,
    AppRemoteToken,
    AppSkipDigest,
}

pub async fn ensure_tables() -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
CREATE TABLE IF NOT EXISTS "config" (
    "key"    TEXT PRIMARY KEY,
    "value"  TEXT NOT NULL)
;"#,
        (),
    )
    .await?;

    Ok(())
}

pub async fn get(key: &ConfigKey) -> AiterResult<Option<String>> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.query(
        r#"
SELECT "value"
FROM "config" 
WHERE "key" = ? 
LIMIT 1
;"#,
        [key.to_string()],
    )
    .await?
    .next()
    .await?
    .map(|row| Ok(row.get::<String>(0)?))
    .transpose()
}

pub async fn set(key: &ConfigKey, value: &str) -> AiterResult<()> {
    let conn = open(&DB_CORE_PATH).await?;
    conn.execute(
        r#"
INSERT INTO "config" ("key", "value")
VALUES (?1, ?2)
ON CONFLICT ("key") DO UPDATE SET
    "value" = ?2
;"#,
        (key.to_string(), value),
    )
    .await?;

    Ok(())
}
