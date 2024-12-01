use crate::db::{delete::Delete, retrieve::Retrieve, update::Update};

use super::table::Table;
use super::Database;
use super::{bulk_insert::BulkInsert, insert::Insert};
use acid4sigmas_models::models::db::{
    DatabaseAction, DatabaseRequest, DatabaseResponse, DeleteAction,
};
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;

use serde_json::Value;

//pub async fn async_db_hanlder(db_request: DatabaseRequest) {}

pub trait DbHandler {
    async fn new(db_request: DatabaseRequest) -> Result<Self>
    where
        Self: Sized;
    async fn handle_request(&self) -> Result<DatabaseResponse<Value>>;
    async fn bulk_insert(&self) -> Result<DatabaseResponse<Value>>;
    async fn insert(&self) -> Result<DatabaseResponse<Value>>;
    async fn delete(&self, delete_action: DeleteAction) -> Result<DatabaseResponse<Value>>;
    async fn update(&self) -> Result<DatabaseResponse<Value>>;
    async fn retrieve(&self) -> Result<DatabaseResponse<Value>>;
}

pub struct DatabaseHandler {
    db_request: DatabaseRequest,
    pool: PgPool,
}

impl DbHandler for DatabaseHandler {
    async fn new(db_request: DatabaseRequest) -> Result<Self> {
        let pool = Database::get_pool()
            .await
            .context("Failed to get database pool.")?;

        if !Table::exists(&pool, &db_request.table).await? {
            return Err(anyhow!("No such table exists."));
        }

        Ok(Self { db_request, pool })
    }

    async fn handle_request(&self) -> Result<DatabaseResponse<serde_json::Value>> {
        match &self.db_request.action {
            DatabaseAction::BulkInsert => self.bulk_insert().await,
            DatabaseAction::Insert => self.insert().await,
            DatabaseAction::Delete(action) => self.delete(action.clone()).await, // clone value
            DatabaseAction::Update => self.update().await,
            DatabaseAction::Retrieve => self.retrieve().await,
        }
    }

    async fn bulk_insert(&self) -> Result<DatabaseResponse<Value>> {
        let bulk_values = self
            .db_request
            .bulk_values
            .as_ref()
            .ok_or_else(|| anyhow!("Missing values for insert"))?;
        let table_name = &self.db_request.table;
        let pool = &self.pool;

        BulkInsert::bulk_insert(pool, table_name, bulk_values).await?;
        Ok(DatabaseResponse::Status {
            status: "Insert successful.".to_string(),
        })
    }

    async fn insert(&self) -> Result<DatabaseResponse<Value>> {
        let values = self
            .db_request
            .values
            .as_ref()
            .ok_or_else(|| anyhow!("Missing values for insert"))?;
        let table_name = &self.db_request.table;
        let pool = &self.pool;

        Insert::insert(pool, table_name, values).await?;
        Ok(DatabaseResponse::Status {
            status: "Insert successful.".to_string(),
        })
    }
    async fn delete(
        &self,
        delete_action: DeleteAction,
    ) -> Result<DatabaseResponse<serde_json::Value>> {
        let table_name = &self.db_request.table;
        let pool = &self.pool;
        let filters = self.db_request.filters.clone();

        Delete::delete(pool, table_name, delete_action, filters).await?;

        println!("deleting..");
        // Implement your deletion logic here, returning an appropriate DatabaseResponse.
        Ok(DatabaseResponse::Status {
            status: "Delete successful.".to_string(),
        })
    }

    async fn update(&self) -> Result<DatabaseResponse<serde_json::Value>> {
        println!("updating..");
        let values = self
            .db_request
            .values
            .as_ref()
            .ok_or_else(|| anyhow!("Missing values for update"))?;
        let table_name = &self.db_request.table;
        let pool = &self.pool;
        let filters = self.db_request.filters.clone();

        Update::update(pool, table_name, values.clone(), filters).await?;
        Ok(DatabaseResponse::Status {
            status: "Update successful".to_string(),
        })
    }

    async fn retrieve(&self) -> Result<DatabaseResponse<serde_json::Value>> {
        println!("receiving..");

        let table_name = &self.db_request.table;
        let pool = &self.pool;

        let vals: Vec<serde_json::Value> =
            Retrieve::retrieve(pool, table_name, self.db_request.clone().filters).await?;

        println!("vals: {:?}", vals);

        Ok(DatabaseResponse::Data(vals))
    }
}
