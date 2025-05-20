use std::path::Path;

use libsql::Rows;
use serde::Serialize;
use ulid::Ulid;

use crate::{
    AiterError, db::open, error::AiterResult, utils::datetime::utc_to_iso_datetime_string,
};

#[derive(Debug, Serialize)]
pub struct HistoryChatEntity {
    pub rowid: i64,
    pub role: String,
    pub content: String,
    pub exchange: String,
    pub created_at: String,
}

pub async fn ensure_tables(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    let tx = conn.transaction().await?;

    tx.execute(
        r#"
CREATE TABLE IF NOT EXISTS "history_chat" (
    "role"        TEXT NOT NULL,
    "content"     TEXT NOT NULL,
    "session"     TEXT,
    "exchange"    TEXT NOT NULL,
    "created_at"  TIMESTAMP DEFAULT CURRENT_TIMESTAMP)
;"#,
        (),
    )
    .await?;

    tx.execute(
        r#"
CREATE INDEX IF NOT EXISTS "idx_history_chat_session" ON "history_chat" ("session")
;"#,
        (),
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn delete(db_path: &Path, rowid: i64) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
DELETE FROM "history_chat" WHERE "rowid" = ?
;"#,
        [rowid],
    )
    .await?;

    Ok(())
}

pub async fn delete_all(db_path: &Path) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
DELETE FROM "history_chat"
;"#,
        (),
    )
    .await?;

    Ok(())
}

pub async fn delete_by_session(db_path: &Path, session: Option<&str>) -> AiterResult<()> {
    let conn = open(db_path).await?;

    if let Some(session) = session {
        conn.execute(
            r#"
DELETE FROM "history_chat" WHERE "session" = ?
;"#,
            [session],
        )
        .await?;
    } else {
        conn.execute(
            r#"
DELETE FROM "history_chat" WHERE "session" IS NULL
;"#,
            (),
        )
        .await?;
    }

    Ok(())
}

pub async fn delete_by_session_exchange(
    db_path: &Path,
    session: Option<&str>,
    exchange: &str,
) -> AiterResult<()> {
    let conn = open(db_path).await?;

    if let Some(session) = session {
        conn.execute(
            r#"
DELETE FROM "history_chat" WHERE "session" = ? AND "exchange"=?
;"#,
            (session, exchange),
        )
        .await?;
    } else {
        conn.execute(
            r#"
DELETE FROM "history_chat" WHERE "session" IS NULL AND "exchange"=?
;"#,
            [exchange],
        )
        .await?;
    }

    Ok(())
}

pub async fn insert(
    db_path: &Path,
    role: &str,
    content: &str,
    exchange: Option<&str>,
    session: Option<&str>,
) -> AiterResult<i64> {
    let exchange = exchange
        .map(|s| s.to_string())
        .unwrap_or(Ulid::new().to_string());

    let conn = open(db_path).await?;
    conn.query(
        r#"
INSERT INTO "history_chat" 
    ("role", "content", "exchange", "session") 
VALUES 
    (?, ?, ?, ?)
RETURNING rowid
;"#,
        (role, content, exchange, session),
    )
    .await?
    .next()
    .await?
    .map(|row| Ok(row.get::<i64>(0)?))
    .ok_or(AiterError::None)?
}

pub async fn retrace(
    db_path: &Path,
    limit: u64,
    session: Option<&str>,
) -> AiterResult<Vec<HistoryChatEntity>> {
    let conn = open(db_path).await?;

    let sql = if session.is_some() {
        r#"
SELECT "rowid", "role", "content", "exchange", "created_at"
FROM (
    SELECT "rowid", *
    FROM "history_chat"
    WHERE "session" = ?
    ORDER BY "rowid" DESC
    LIMIT ?
)
ORDER BY "rowid" ASC
;"#
    } else {
        r#"
SELECT "rowid", "role", "content", "exchange", "created_at"
FROM (
    SELECT "rowid", *
    FROM "history_chat"
    WHERE "session" IS NULL
    ORDER BY "rowid" DESC
    LIMIT ?2
)
ORDER BY "rowid" ASC
;"#
    };

    let mut rows = conn
        .query(sql, (session.unwrap_or_default(), limit))
        .await?;

    HistoryChatEntity::collect_rows(&mut rows).await
}

pub async fn set_content(db_path: &Path, rowid: i64, content: &str) -> AiterResult<()> {
    let conn = open(db_path).await?;
    conn.execute(
        r#"
UPDATE "history_chat"
SET "content" = ?
WHERE "rowid" = ?
;"#,
        (content, rowid),
    )
    .await?;

    Ok(())
}

impl HistoryChatEntity {
    async fn collect_rows(rows: &mut Rows) -> AiterResult<Vec<Self>> {
        let mut vec = vec![];

        while let Some(row) = rows.next().await? {
            vec.push(Self {
                rowid: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                exchange: row.get(3)?,
                created_at: utc_to_iso_datetime_string(&row.get::<String>(4)?),
            });
        }

        Ok(vec)
    }
}
