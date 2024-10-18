use acid4sigmas_models::db::TableModel;
use acid4sigmas_models::models::db::Filters;
use sqlx::postgres::PgRow;
use sqlx::PgPool;

use crate::MODEL_REGISTRY;

pub struct Retrieve;

impl Retrieve {
    pub async fn retrieve(
        pool: &PgPool,
        table_name: &str,
        filters: Option<Filters>,
    ) -> Option<Box<dyn TableModel + Send + Sync>> {
        // be careful!, here an sql injection would be possible.
        let row: PgRow = sqlx::query(&format!("SELECT * FROM {}", table_name))
            .bind(table_name)
            .fetch_one(pool)
            .await
            .expect("Failed to fetch user");

        let registry = MODEL_REGISTRY
            .get()
            .expect("Model registry not initialized");

        if let Some(entry) = registry.get(table_name) {
            let model_instance = (entry.factory)(&row); // call the factory to create the model
            return Some(model_instance);
        };

        None
    }
}
