use std::{path::Path, time::Duration};

use half::f16;
use libsql::{Builder, Connection};

use crate::{
    CURRENT_DB_VERSION, CURRENT_SIGNATURE_DIMS, CURRENT_TOKENIZER, DB_BUSY_SECS, DB_CORE_PATH,
    Tokenizer, error::AiterResult,
};

pub mod core;
pub mod mem;
pub mod updates;

pub async fn ensure_core_tables() -> AiterResult<()> {
    if DB_CORE_PATH.exists() {
        if let Some(db_version_str) = core::meta::get_db_version().await? {
            let db_version = db_version_str.parse::<u64>().unwrap_or(CURRENT_DB_VERSION);
            if db_version < CURRENT_DB_VERSION {
                for i in db_version..CURRENT_DB_VERSION {
                    if let Some(sqls) = updates::SQLS_UPDATE_CORE.get(i as usize) {
                        let conn = open(&DB_CORE_PATH.clone()).await?;
                        let tx = conn.transaction().await?;

                        for sql in *sqls {
                            tx.execute(sql, ()).await?;
                        }

                        tx.execute(updates::SQL_UPDATE_DB_VERSION, [i]).await?;
                        tx.commit().await?;
                    }
                }
            }
        }
    } else {
        core::ai::ensure_tables().await?;
        core::config::ensure_tables().await?;
        core::llm::ensure_tables().await?;
        core::meta::ensure_tables().await?;
        core::tool::ensure_tables().await?;
    }

    Ok(())
}

pub async fn ensure_mem_tables(db_path: &Path) -> AiterResult<()> {
    if db_path.exists() {
        if let Some(db_version_str) = mem::meta::get_db_version(db_path).await? {
            let db_version = db_version_str.parse::<u64>().unwrap_or(CURRENT_DB_VERSION);
            if db_version < CURRENT_DB_VERSION {
                for i in db_version..CURRENT_DB_VERSION {
                    if let Some(sqls) = updates::SQLS_UPDATE_MEM.get(i as usize) {
                        let conn = open(db_path).await?;
                        let tx = conn.transaction().await?;

                        for sql in *sqls {
                            tx.execute(sql, ()).await?;
                        }

                        tx.execute(updates::SQL_UPDATE_DB_VERSION, [i]).await?;
                        tx.commit().await?;
                    }
                }
            }
        }

        if let Some(signature_dims_str) = mem::meta::get_signature_dims(db_path).await? {
            let signature_dims = signature_dims_str
                .parse::<usize>()
                .unwrap_or(CURRENT_SIGNATURE_DIMS);
            mem::MEM_SIGNATURE_DIMS_MAP.insert(db_path.to_path_buf(), signature_dims);
        }

        if let Some(tokenizer_str) = mem::meta::get_tokenizer(db_path).await? {
            let tokenizer = tokenizer_str
                .parse::<Tokenizer>()
                .unwrap_or(CURRENT_TOKENIZER);
            mem::MEM_TOKENIZER_MAP.insert(db_path.to_path_buf(), tokenizer);
        }
    } else {
        mem::doc::ensure_tables(db_path).await?;
        mem::doc_frag::ensure_tables(db_path).await?;
        mem::doc_implicit::ensure_tables(db_path).await?;
        mem::doc_knl::ensure_tables(db_path).await?;
        mem::doc_part::ensure_tables(db_path).await?;
        mem::doc_seg::ensure_tables(db_path).await?;
        mem::history_chat::ensure_tables(db_path).await?;
        mem::meta::ensure_tables(db_path).await?;
        mem::skill::ensure_tables(db_path).await?;
    }

    Ok(())
}

pub async fn open(db_path: &Path) -> AiterResult<Connection> {
    let db = Builder::new_local(db_path).build().await?;
    let conn = db.connect()?;
    conn.busy_timeout(Duration::from_secs(DB_BUSY_SECS))?;
    conn.query("PRAGMA journal_mode=WAL;", ()).await?;

    Ok(conn)
}

pub async fn vacuum(db_path: &Path) -> AiterResult<()> {
    let db = Builder::new_local(db_path).build().await?;
    let conn = db.connect()?;
    conn.execute("VACUUM;", ()).await?;

    Ok(())
}

pub fn vec_f32_to_f16_str(vec: &[f32]) -> String {
    format!(
        "[{}]",
        vec.iter()
            .map(|f| f16::from_f32(*f).to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DB_VECTOR_NEIGHBORS;

    #[tokio::test]
    async fn test_fts5() {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();

        {
            conn.execute(
                r#"
CREATE VIRTUAL TABLE "test_fts" USING fts5(
    "id" UNINDEXED, "name")
;"#,
                (),
            )
            .await
            .unwrap();

            conn.execute(
                r#"
INSERT INTO "test_fts"("id", "name") VALUES (?, ?)
;"#,
                ("x", r#"Hello 我是一个有用的 AI 助理"#),
            )
            .await
            .unwrap();
        }

        {
            let id: Option<String> = conn
                .query(
                    r#"
SELECT "id" FROM "test_fts"(?) ORDER BY rank
;"#,
                    ["hello 助理"],
                )
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .map(|row| row.get::<String>(0).ok())
                .unwrap_or(None);
            assert_eq!(id, Some("x".to_string()));
        }

        {
            let id: Option<String> = conn
                .query(
                    r#"
SELECT "id" FROM "test_fts"(?)
;"#,
                    ["A"],
                )
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .map(|row| row.get::<String>(0).ok())
                .unwrap_or(None);
            assert_eq!(id, None);
        }
    }

    #[tokio::test]
    async fn test_vec() {
        let db = Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();

        {
            conn.execute(
                r#"
CREATE TABLE "test_vec" (
    id TEXT PRIMARY KEY, 
    user_id TEXT, 
    embedding F16_BLOB(8))
;"#,
                (),
            )
            .await
            .unwrap();

            conn.execute(
                &format!(
                r#"
CREATE INDEX "idx_test_vec_embedding" ON "test_vec" (libsql_vector_idx(embedding, 'compress_neighbors=float8', 'max_neighbors={}'))
;"#, *DB_VECTOR_NEIGHBORS),
                (),
            )
            .await
            .unwrap();

            conn.execute(
                r#"
INSERT INTO "test_vec" ("id", "user_id", "embedding") VALUES 
    ('1', 'A', vector16('[-0.200, 0.250, 0.341, -0.211, 0.645, 0.935, -0.316, -0.924]')),
    ('2', 'A', vector16('[1.001, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]')),
    ('3', 'B', vector16('[0.716, -0.927, 0.134, 0.052, -0.669, 0.793, -0.634, -0.162]')),
    ('4', 'B', vector16('[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]'))
;"#,
                (),
            )
            .await
            .unwrap();
        }

        {
            let id: Option<String> = conn
                .query(
                    r#"
SELECT "id" 
FROM "test_vec" 
WHERE rowid IN vector_top_k('idx_test_vec_embedding', vector16(?1), 10) AND "user_id" = 'A'
ORDER BY vector_distance_cos("embedding", vector16(?1))
;"#,
                    [vec_f32_to_f16_str(&[
                        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                    ])],
                )
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .map(|row| row.get::<String>(0).ok())
                .unwrap_or(None);
            assert_eq!(id, Some("2".to_string()));
        }

        {
            let id: Option<String> = conn
                .query(
                    r#"
SELECT "id" 
FROM "test_vec" 
WHERE rowid IN vector_top_k('idx_test_vec_embedding', vector16(?1), 10) AND vector_distance_cos("embedding", vector16(?1)) < 1e-8
;"#,
                    [vec_f32_to_f16_str(&[
                        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                    ])],
                )
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .map(|row| row.get::<String>(0).ok())
                .unwrap_or(None);
            assert_eq!(id, Some("4".to_string()));
        }
    }
}
