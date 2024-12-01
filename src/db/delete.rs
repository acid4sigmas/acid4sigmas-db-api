use acid4sigmas_models::models::db::{
    BuildQuery, DatabaseAction, DeleteAction, Filters, QueryBuilder,
};
use serde_json::Value;
use sqlx::PgPool;

pub struct Delete;

impl Delete {
    pub async fn delete(
        pool: &PgPool,
        table_name: &str,
        delete_action: DeleteAction,
        filters: Option<Filters>,
    ) -> anyhow::Result<()> {
        let query_builder: BuildQuery = QueryBuilder::from(QueryBuilder {
            table: table_name.to_string(),
            action: DatabaseAction::Delete(delete_action),
            filters,
            ..Default::default()
        })
        .build_query()?;

        println!("{:?}", query_builder);
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
            }
        }

        query_builder.execute(&mut *txn).await?;
        txn.commit().await?;

        Ok(())
    }
}
