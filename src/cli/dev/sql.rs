use std::path::PathBuf;

use aiter::{error::AiterResult, utils::text::truncate_format};
use colored::Colorize;
use libsql::{Builder, OpenFlags, Value};

#[derive(clap::Args)]
pub struct DevSqlCommand {
    #[arg(long = "allow-create", help = "Create database file if not exists")]
    allow_create: bool,

    #[arg(
        short = 'f',
        long = "db",
        value_name = "DB_PATH",
        help = "Database file path"
    )]
    db_path: PathBuf,

    #[arg(
        long = "limit",
        default_value = "10",
        help = "Limit the maximum number of rows to be returned, if set to 0 means there is no limit, default value is 10"
    )]
    limit: u64,

    #[clap(required = true, help = "SQL to execute")]
    sql: String,
}

impl DevSqlCommand {
    pub async fn exec(&self) {
        if let Err(err) = self.exec_sql().await {
            println!("{}", err.to_string().red());
        }
    }

    async fn exec_sql(&self) -> AiterResult<()> {
        let db = if self.allow_create {
            Builder::new_local(&self.db_path).build().await?
        } else {
            Builder::new_local(&self.db_path)
                .flags(OpenFlags::SQLITE_OPEN_READ_WRITE)
                .build()
                .await?
        };
        let conn = db.connect()?;

        let mut stmt = conn.prepare(&self.sql).await?;
        let column_names: Vec<String> = stmt
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let column_count = column_names.len();

        let mut table_data: Vec<Vec<String>> = vec![];
        table_data.push(column_names);

        let mut rows = stmt.query(()).await?;
        while let Some(row) = rows.next().await? {
            let mut row_data: Vec<String> = vec![];

            for i in 0..column_count {
                if let Ok(value) = row.get_value(i as i32) {
                    let s = match value {
                        Value::Null => "[Null]",
                        Value::Integer(i) => &format!("{}", i),
                        Value::Real(f) => &format!("{}", f),
                        Value::Text(t) => &t.to_string(),
                        Value::Blob(_) => "[Blob]",
                    };

                    row_data.push(truncate_format(s, 100, true));
                } else {
                    row_data.push("".to_string());
                }
            }

            table_data.push(row_data);

            if self.limit > 0 && table_data.len() > self.limit as usize {
                break;
            }
        }

        let table = tabled::builder::Builder::from_iter(&table_data).build();
        println!("{table}");

        Ok(())
    }
}
