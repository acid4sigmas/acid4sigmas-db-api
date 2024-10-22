use super::table::Table;
use acid4sigmas_models::models::db::{DatabaseAction, Filters, QueryBuilder};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

pub struct Update;

impl Update {
    pub async fn update(
        pool: &PgPool,
        table_name: &str,
        values: HashMap<String, Value>,
        filters: Option<Filters>,
    ) -> anyhow::Result<()> {
        let table_columns = Table::get_table_columns_and_types(&pool, &table_name).await?;

        let query_builder = QueryBuilder::new(
            table_name.to_string(),
            DatabaseAction::Update,
            Some(values),
            Some(table_columns),
            filters,
        )
        .build_query()?;

        println!("{:?}", query_builder);

        let (_query, _params) = query_builder;

        Ok(())
    }
}
