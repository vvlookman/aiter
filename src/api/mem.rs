use std::{
    fs,
    fs::{copy, remove_dir_all, remove_file},
    path::PathBuf,
};

use tokio::sync::mpsc::Sender;

use crate::{
    api::{get_docs_dir_path, get_mem_path},
    db,
    error::AiterResult,
};

pub mod doc;
pub mod history;
pub mod skill;

pub type MemWriteEvent = db::mem::MemWriteEvent;

pub async fn erase(ai_name: Option<&str>) -> AiterResult<()> {
    let mem_path = get_mem_path(ai_name).await?;
    if mem_path.exists() {
        remove_file(&mem_path)?;
    }

    // For WAL mode
    for suffix in ["wal", "shm"] {
        let path = PathBuf::from(format!("{}-{}", &mem_path.to_string_lossy(), suffix));
        if path.exists() {
            remove_file(&path)?;
        }
    }

    // Recreate mem tables
    db::ensure_mem_tables(&mem_path).await?;

    if let Ok(docs_path) = get_docs_dir_path(ai_name).await {
        let _ = remove_dir_all(docs_path);
    }

    Ok(())
}

pub async fn merge(ai_name: Option<&str>, from_ai_name: Option<&str>) -> AiterResult<()> {
    let mem_path = get_mem_path(ai_name).await?;
    let from_mem_path = get_mem_path(from_ai_name).await?;

    let bak_mem_path = mem_path.with_extension("bak");
    if let Err(err) = copy(&mem_path, &bak_mem_path) {
        Err(err.into())
    } else {
        let process = async || -> AiterResult<()> {
            let conn = db::open(&mem_path).await?;
            conn.execute(
                "ATTACH ? AS merge_db",
                [from_mem_path.to_string_lossy().to_string()],
            )
            .await?;

            let mut table_names_rows = conn
                .query("SELECT name FROM sqlite_master WHERE type='table'", ())
                .await?;

            let mut table_names: Vec<String> = vec![];
            while let Some(row) = table_names_rows.next().await? {
                table_names.push(row.get(0)?);
            }

            for table_name in table_names {
                let insert_sql = format!(
                    "INSERT OR IGNORE INTO {} SELECT * FROM merge_db.{}",
                    table_name, table_name
                );

                conn.execute(&insert_sql, ()).await?;
            }

            conn.execute("DETACH merge_db", ()).await?;

            Ok(())
        };

        let process_result = process().await;

        if process_result.is_err() {
            copy(&bak_mem_path, &mem_path)?;
        }
        remove_file(&bak_mem_path)?;

        process_result
    }
}

pub async fn spawn_mem_write(name: Option<&str>) -> AiterResult<Sender<MemWriteEvent>> {
    let mem_path = get_mem_path(name).await?;
    Ok(db::mem::spawn_mem_write(&mem_path))
}

pub async fn stats(ai_name: Option<&str>) -> AiterResult<u64> {
    let mem_path = get_mem_path(ai_name).await?;
    if mem_path.exists() {
        let metadata = fs::metadata(mem_path)?;
        return Ok(metadata.len());
    }

    Ok(0)
}

pub async fn vacuum(ai_name: Option<&str>) -> AiterResult<()> {
    let mem_path = get_mem_path(ai_name).await?;
    db::vacuum(&mem_path).await?;

    Ok(())
}
