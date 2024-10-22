pub mod db_handler;
pub mod insert;
pub mod retrieve;
pub mod table;
pub mod update;

use crate::secrets::SECRETS;
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::fs;
use std::path::PathBuf;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let url = format!(
            "postgresql://postgres:{}@localhost:{}/{}",
            SECRETS.get("DB_PW").unwrap(),
            SECRETS.get("DB_PORT").unwrap(),
            SECRETS.get("DB_NAME").unwrap()
        );

        let pool = sqlx::postgres::PgPool::connect(&url).await?;
        Ok(Self { pool })
    }

    pub async fn get_pool() -> Result<PgPool> {
        Database::new().await.map(|db| db.pool)
    }

    pub async fn init(schema_path: PathBuf) -> Result<()> {
        let schema = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema file from {:?}", schema_path))?;

        let pool = Self::get_pool().await?;

        for statement in schema.split(";") {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&pool)
                    .await
                    .with_context(|| format!("Failed to execute statement: {}", trimmed))?;
            }
        }

        Ok(())
    }
}
