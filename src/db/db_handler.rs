use crate::db::{retrieve::Retrieve, update::Update};

use super::insert::Insert;
use super::table::Table;
use super::Database;
use acid4sigmas_models::models::db::{DatabaseAction, DatabaseRequest};
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;

//pub async fn async_db_hanlder(db_request: DatabaseRequest) {}

pub trait DbHandler {
    async fn new(db_request: DatabaseRequest) -> Result<Self>
    where
        Self: Sized;
    async fn handle_request(&self) -> Result<Option<Vec<serde_json::Value>>>;
    async fn insert(&self) -> Result<()>;
    async fn delete(&self) -> Result<()>;
    async fn update(&self) -> Result<()>;
    async fn retrieve(&self) -> Result<Vec<serde_json::Value>>;
}

pub struct DatabaseHandler {
    db_request: DatabaseRequest,
    pool: PgPool,
}

impl DbHandler for DatabaseHandler {
    async fn new(db_request: DatabaseRequest) -> Result<Self> {
        let pool = Database::get_pool()
            .await
            .context("Failed to get database pool")?;

        if !Table::exists(&pool, &db_request.table).await? {
            return Err(anyhow!("No such table exists."));
        }

        Ok(Self { db_request, pool })
    }

    async fn handle_request(&self) -> Result<Option<Vec<serde_json::Value>>> {
        match self.db_request.action {
            DatabaseAction::Insert => {
                self.insert().await?;
            }
            DatabaseAction::Delete => {
                self.delete().await?;
            }
            DatabaseAction::Update => {
                self.update().await?;
            }
            DatabaseAction::Retrieve => {
                println!("retrieve");
                let result = self.retrieve().await?;
                if result.is_empty() {
                    return Err(anyhow!("no rows were returned"));
                } else {
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    async fn insert(&self) -> Result<()> {
        let values = self.db_request.values.as_ref().unwrap();
        let table_name = &self.db_request.table;
        let pool = &self.pool;

        Insert::insert(pool, table_name, values).await?;

        Ok(())
    }

    async fn delete(&self) -> Result<()> {
        println!("deleting..");
        Ok(())
    }

    async fn update(&self) -> Result<()> {
        println!("updating..");
        let values = self.db_request.values.as_ref().unwrap();
        let table_name = &self.db_request.table;
        let pool = &self.pool;
        let filters = self.db_request.filters.clone();

        Update::update(pool, table_name, values.clone(), filters).await?;
        Ok(())
    }

    async fn retrieve(&self) -> Result<Vec<serde_json::Value>> {
        println!("receiving..");

        let table_name = &self.db_request.table;
        let pool = &self.pool;

        let vals: Vec<serde_json::Value> =
            Retrieve::retrieve(pool, table_name, self.db_request.clone().filters).await?;

        Ok(vals)
    }
}
