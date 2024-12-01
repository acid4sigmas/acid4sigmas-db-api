use crate::cache::{CacheKey, CACHE_MANAGER};
use crate::timer::Timer;
use acid4sigmas_models::db::{ModelRegistry, TableModel};
use acid4sigmas_models::models::db::{BuildQuery, DatabaseAction, Filters, QueryBuilder};
use anyhow::anyhow;
use sqlx::postgres::PgRow;
use sqlx::PgPool;

use crate::MODEL_REGISTRY;

pub struct Retrieve;

impl Retrieve {
    pub async fn retrieve(
        pool: &PgPool,
        table_name: &str,
        filters: Option<Filters>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        println!("filters: {:?}", filters);

        let timer = Timer::new();

        let query_builder: BuildQuery = QueryBuilder::from(QueryBuilder {
            table: table_name.to_string(),
            action: DatabaseAction::Retrieve,
            filters,
            ..Default::default()
        })
        .build_query()?;

        println!("{:?}", query_builder);
        let (query, params) = query_builder;
        let cache_key_gen = CacheKey::generate_cache_key(&table_name, &query, &params);

        if let Some(cache) = CACHE_MANAGER.get(&cache_key_gen) {
            println!("value found in cache in {} µs", timer.elapsed_as_micros());

            return Ok(cache);
        } else {
            println!("value not found in cache");
        }

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

        let rows: Vec<PgRow> = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch data: {}", e))?;

        let registry: &ModelRegistry = MODEL_REGISTRY
            .get()
            .expect("Model registry not initialized");

        if let Some(entry) = registry.get(table_name) {
            let mut models: Vec<serde_json::Value> = Vec::new();

            for row in rows {
                println!("row");
                let model_instance: Box<dyn TableModel + Send + Sync> = (entry.factory)(&row); // call the factory to create the model
                models.push(model_instance.as_value());
            }

            CACHE_MANAGER.insert(cache_key_gen, models.clone());

            println!("finished in {} ms", timer.elapsed_as_millis());
            return Ok(models);
        };

        Ok(Vec::new())
    }
}
