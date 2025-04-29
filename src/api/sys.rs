use crate::{api, db, error::AiterResult, DATA_DIR, DB_DEFAULT_MEM_PATH};

pub async fn get_data_path() -> String {
    DATA_DIR.to_string_lossy().to_string()
}

pub async fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn update() -> AiterResult<()> {
    db::ensure_core_tables().await?;
    db::ensure_mem_tables(&DB_DEFAULT_MEM_PATH).await?;

    if let Ok(ais) = api::ai::list().await {
        for ai in ais {
            let mem_path = DATA_DIR.join(format!("mem_{}.db", ai.id));
            db::ensure_mem_tables(&mem_path).await?;
        }
    }

    Ok(())
}
