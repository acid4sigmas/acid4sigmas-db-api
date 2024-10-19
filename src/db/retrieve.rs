use acid4sigmas_models::db::{TableModel, ModelRegistry};
use acid4sigmas_models::models::db::Filters;
use acid4sigmas_models::models::db::QueryBuilder;
use anyhow::anyhow;
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use super::table::Table;


use crate::MODEL_REGISTRY;

pub struct Retrieve;

impl Retrieve {
    pub async fn retrieve(
        pool: &PgPool,
        table_name: &str,
        filters: Option<Filters>,
    ) -> anyhow::Result<Vec<Box<dyn TableModel + Send + Sync>>> {

        println!("filters: {:?}", filters);

        let query_builder: (String, Vec<serde_json::Value>) = QueryBuilder::new(table_name.to_string(), filters).build_query()?;
        

        println!("{:?}", query_builder);
        let (query, params) = query_builder;

        let mut query_builder = sqlx::query(&query);

        for param in params {
            match param {
                serde_json::Value::Number(num) => {
                    if let Some(int_value) = num.as_i64() {
                        query_builder = query_builder.bind(int_value); 
                    } else if let Some(float_value) = num.as_f64() {
                        query_builder = query_builder.bind(float_value);
                    }
                }
                serde_json::Value::String(s) => {
                    query_builder = query_builder.bind(s);
                }
                serde_json::Value::Bool(b) => {
                    query_builder = query_builder.bind(b);
                }
                _ => return Err(anyhow!("Unsupported JSON type for parameter binding")),
            
            }
        }
        
        let rows: Vec<PgRow> = query_builder.fetch_all(pool).await
            .map_err(|e| anyhow!("Failed to fetch data: {}", e))?;

        let registry: &ModelRegistry = MODEL_REGISTRY
            .get()
            .expect("Model registry not initialized");

        if let Some(entry) = registry.get(table_name) {
            let mut models: Vec<Box<dyn TableModel + Send + Sync>> = Vec::new();
            for row in rows {
                let model_instance = (entry.factory)(&row); // Call the factory to create the model
                models.push(model_instance);
            }
            return Ok(models);
        };

        Ok(Vec::new())
    }
}
