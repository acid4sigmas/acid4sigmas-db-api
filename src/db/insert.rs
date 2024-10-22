use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::cache::{CacheKey, CACHE_MANAGER};

use super::table::Table;

pub struct Insert;

impl Insert {
    pub async fn insert(
        pool: &PgPool,
        table_name: &str,
        values: &HashMap<String, Value>,
    ) -> Result<()> {
        let table_columns = Table::get_table_columns_and_types(&pool, &table_name).await?;

        let columns: Vec<String> = values.keys().cloned().collect();

        if columns.len() != table_columns.len() {
            return Err(anyhow!("The number of keys doesnt match the schema."));
        }

        for column in &columns {
            if !table_columns.contains_key(column) {
                return Err(anyhow!(format!(
                    "Column {} does not exist in table {}",
                    column, table_name
                )));
            }
        }

        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        let mut bind_values: Vec<Value> = Vec::new();

        for key in &columns {
            if let Some(value) = values.get(key) {
                if let Some(expected_type) = table_columns.get(key) {
                    let converted_value = match expected_type.as_str() {
                        "bigint" => {
                            if let Some(s) = value.as_str() {
                                if let Ok(parsed) = s.parse::<i64>() {
                                    Value::Number(parsed.into())
                                } else {
                                    return Err(anyhow!("failed to convert {} to bigint", s));
                                }
                            } else if value.is_i64() || value.is_u64() {
                                Value::Number(value.as_i64().unwrap().into())
                            } else {
                                return Err(anyhow!("expected a string to convert to bigint"));
                            }
                        }
                        "text" => {
                            if value.is_string() {
                                value.clone()
                            } else {
                                return Err(anyhow::anyhow!("Expected string for text"));
                            }
                        }
                        "boolean" => {
                            if let Some(b) = value.as_bool() {
                                Value::Bool(b)
                            } else {
                                return Err(anyhow::anyhow!("Expected bool for boolean column"));
                            }
                        }
                        _ => {
                            return Err(anyhow::anyhow!(
                                "Unsupported column type: {}",
                                expected_type
                            ));
                        }
                    };

                    bind_values.push(converted_value);
                }
            }
        }

        let mut txn = pool.begin().await?;
        let mut query_builder = sqlx::query::<sqlx::Postgres>(&query);

        for value in bind_values {
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
