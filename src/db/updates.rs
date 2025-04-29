use crate::CURRENT_DB_VERSION;

pub static SQL_UPDATE_DB_VERSION: &str = r#"
INSERT INTO "meta" ("key", "value")
VALUES ('db_version', ?1)
ON CONFLICT ("key") DO UPDATE SET
    "value" = ?1
;"#;

pub static SQLS_UPDATE_CORE: [&[&str]; CURRENT_DB_VERSION as usize] = [
    // 0 -> 1
    &[],
];

pub static SQLS_UPDATE_MEM: [&[&str]; CURRENT_DB_VERSION as usize] = [
    // 0 -> 1
    &[],
];
