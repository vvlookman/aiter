use crate::{
    DATA_DIR, PathBuf, db,
    error::{AiterError, AiterResult},
};

pub mod ai;
pub mod chat;
pub mod config;
pub mod learn;
pub mod llm;
pub mod mem;
pub mod sys;
pub mod tool;

async fn get_docs_dir_path(name: Option<&str>) -> AiterResult<PathBuf> {
    let dir_name = if let Some(name) = name {
        if let Some(ai) = db::core::ai::get_by_name(name).await? {
            format!("docs_{}", ai.id)
        } else {
            return Err(AiterError::Invalid(format!("AI '{name}' is not exists")));
        }
    } else {
        "docs".to_string()
    };

    Ok(DATA_DIR.join(dir_name))
}

async fn get_mem_path(name: Option<&str>) -> AiterResult<PathBuf> {
    let mem_db_filename = if let Some(name) = name {
        if let Some(ai) = db::core::ai::get_by_name(name).await? {
            format!("mem_{}.db", ai.id)
        } else {
            return Err(AiterError::Invalid(format!("AI '{name}' is not exists")));
        }
    } else {
        "mem.db".to_string()
    };

    Ok(DATA_DIR.join(mem_db_filename))
}
