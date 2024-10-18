use std::collections::HashMap;

use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct Table;

impl Table {
    pub async fn exists(pool: &PgPool, table_name: &str) -> Result<bool> {
        let query = r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name = $1
            )
        "#;

        let exists: (bool,) = sqlx::query_as(query)
            .bind(table_name)
            .fetch_one(pool)
            .await?;

        Ok(exists.0)
    }

    pub async fn get_table_columns_and_types(
        pool: &PgPool,
        table_name: &str,
    ) -> Result<HashMap<String, String>> {
        let query = r#"
            SELECT column_name, data_type
            FROM information_schema.columns
            WHERE table_name = $1
        "#;

        let rows = sqlx::query(query).bind(table_name).fetch_all(pool).await?;

        let mut columns_and_types = HashMap::new();

        for row in rows {
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            columns_and_types.insert(column_name, data_type);
        }

        Ok(columns_and_types)
    }
}
