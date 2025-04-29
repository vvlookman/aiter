use std::{
    fs::{copy, remove_dir_all, remove_file},
    path::PathBuf,
};

use crate::{
    api::{get_docs_dir_path, get_mem_path},
    db,
    error::{AiterError, AiterResult},
    DATA_DIR,
};

pub type AiEntity = db::core::ai::AiEntity;

pub async fn add(name: &str) -> AiterResult<Option<AiEntity>> {
    let trimmed_name = name.trim();

    if trimmed_name.is_empty() {
        return Err(AiterError::Invalid("AI name cannot be empty".to_string()));
    } else if trimmed_name.starts_with('~') || trimmed_name.starts_with('@') {
        return Err(AiterError::Invalid(
            "AI name cannot start with ~ or @, they are reserved.".to_string(),
        ));
    }

    if db::core::ai::get_by_name(trimmed_name).await?.is_some() {
        return Err(AiterError::Invalid(format!(
            "AI '{trimmed_name}' already exists"
        )));
    }

    db::core::ai::insert(trimmed_name).await?;

    let mem_path = get_mem_path(Some(trimmed_name)).await?;
    db::ensure_mem_tables(&mem_path).await?;

    db::core::ai::get_by_name(trimmed_name).await
}

pub async fn clone(from_name: Option<&str>, to_name: &str) -> AiterResult<()> {
    let trimmed_to_name = to_name.trim();

    if trimmed_to_name.is_empty() {
        return Err(AiterError::Invalid("AI name cannot be empty".to_string()));
    } else if trimmed_to_name.starts_with('~') || trimmed_to_name.starts_with('@') {
        return Err(AiterError::Invalid(
            "AI name cannot start with ~ or @, they are reserved.".to_string(),
        ));
    }

    if db::core::ai::get_by_name(trimmed_to_name).await?.is_some() {
        return Err(AiterError::Invalid(format!(
            "AI '{trimmed_to_name}' already exists"
        )));
    }

    db::core::ai::insert(trimmed_to_name).await?;

    let from_mem_path = get_mem_path(from_name).await?;

    let to_mem_path = get_mem_path(Some(trimmed_to_name)).await?;
    copy(&from_mem_path, &to_mem_path)?;

    Ok(())
}

pub async fn delete(name: &str) -> AiterResult<Option<db::core::ai::AiEntity>> {
    let item = db::core::ai::get_by_name(name).await?;

    if let Some(deleted_id) = db::core::ai::delete_by_name(name).await? {
        let mem_path = DATA_DIR.join(format!("mem_{}.db", deleted_id));
        if mem_path.exists() {
            let _ = remove_file(&mem_path);
        }

        // For WAL mode
        for suffix in ["wal", "shm"] {
            let path = PathBuf::from(format!("{}-{}", &mem_path.to_string_lossy(), suffix));
            if path.exists() {
                let _ = remove_file(&path);
            }
        }

        if let Ok(docs_path) = get_docs_dir_path(Some(name)).await {
            let _ = remove_dir_all(docs_path);
        }
    }

    Ok(item)
}

pub async fn get(name: &str) -> AiterResult<Option<db::core::ai::AiEntity>> {
    db::core::ai::get_by_name(name).await
}

pub async fn list() -> AiterResult<Vec<db::core::ai::AiEntity>> {
    db::core::ai::list().await
}

pub async fn rename(name: &str, new_name: &str) -> AiterResult<Option<db::core::ai::AiEntity>> {
    let trimmed_new_name = new_name.trim();

    if trimmed_new_name.is_empty() {
        return Err(AiterError::Invalid("AI name cannot be empty".to_string()));
    } else if trimmed_new_name.starts_with('~') || trimmed_new_name.starts_with('@') {
        return Err(AiterError::Invalid(
            "AI name cannot start with ~ or @, they are reserved.".to_string(),
        ));
    }

    if db::core::ai::get_by_name(trimmed_new_name).await?.is_some() {
        return Err(AiterError::Invalid(format!(
            "AI '{trimmed_new_name}' already exists"
        )));
    }

    if let Some(ai) = db::core::ai::get_by_name(name).await? {
        db::core::ai::set_name(&ai.id, trimmed_new_name).await?;
        db::core::ai::get_by_name(trimmed_new_name).await
    } else {
        Err(AiterError::Invalid(format!("AI '{name}' does not exist")))
    }
}
