use acid4sigmas_models::models::db::{BuildQuery, DatabaseAction, QueryBuilder, Values};
use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;

use crate::cache::{CacheKey, CACHE_MANAGER};

use super::table::Table;

pub struct Insert;

impl Insert {
    pub async fn insert(pool: &PgPool, table_name: &str, values: &Values) -> Result<()> {
        let table_columns = Table::get_table_columns_and_types(&pool, &table_name).await?;

        let query_builder: BuildQuery = QueryBuilder::from(QueryBuilder {
            table: table_name.to_string(),
            action: DatabaseAction::Insert,
            values: Some(values.clone()),
            table_columns: Some(table_columns),
            ..Default::default()
        })
        .build_query()?;

        let (query, params) = query_builder;

        let mut txn = pool.begin().await?;
        let mut query_builder = sqlx::query::<sqlx::Postgres>(&query);

        for value in params {
            query_builder = match value {
                Value::String(s) => query_builder.bind(s),
                Value::Number(n) => {
                    if let Some(num) = n.as_i64() {
                        query_builder.bind(num)
                    } else {
                        return Err(anyhow::anyhow!("Invalid number type for binding"));
                    }
                }
                Value::Bool(b) => query_builder.bind(b),
                _ => return Err(anyhow::anyhow!("Unsupported value type")),
            };
        }

        query_builder.execute(&mut *txn).await?;
        txn.commit().await?;

        let cache_key_table = CacheKey::generate_table_cache_hash(&table_name);

        CACHE_MANAGER.remove_by_prefix(&cache_key_table);

        Ok(())
    }
}
