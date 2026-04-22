use super::{CheckError, CheckOutput, Checker, ConfigError, infer_driver};
use crate::models::check_result::CheckStatus;
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};

pub struct ChartQueryChecker {
    driver: &'static str,
    connection_string: String,
    query: String,
    timeout_ms: Option<u64>,
}

impl ChartQueryChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let connection_string = config["connection_string"]
            .as_str()
            .ok_or_else(|| {
                ConfigError::InvalidConfig("chart_query requires 'connection_string'".into())
            })?
            .to_string();
        let driver = infer_driver(&connection_string)?;
        let query = config["query"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("chart_query requires 'query'".into()))?
            .to_string();
        let timeout_ms = config["timeout_ms"].as_u64();
        Ok(Self {
            driver,
            connection_string,
            query,
            timeout_ms,
        })
    }
}

#[async_trait]
impl Checker for ChartQueryChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let start = Instant::now();
        let result = self.run_query().await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e),
            }),
            Ok(rows) => Ok(CheckOutput {
                status: CheckStatus::Up,
                response_ms: Some(elapsed),
                detail: Some(serde_json::json!({ "rows": rows })),
                error_message: None,
            }),
        }
    }
}

#[derive(serde::Serialize)]
struct ChartRow {
    label: String,
    value: f64,
    color: Option<String>,
}

impl ChartQueryChecker {
    async fn run_query(&self) -> Result<Vec<ChartRow>, String> {
        let timeout = Duration::from_millis(self.timeout_ms.unwrap_or(10_000));
        tokio::time::timeout(timeout, self.run_query_inner())
            .await
            .map_err(|_| "query timed out".to_string())?
    }

    async fn run_query_inner(&self) -> Result<Vec<ChartRow>, String> {
        match self.driver {
            "sqlite" => {
                let pool = sqlx::SqlitePool::connect(&self.connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                let rows = self.fetch_sqlite(&pool).await?;
                pool.close().await;
                Ok(rows)
            }
            _ => {
                let pool = sqlx::PgPool::connect(&self.connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                let rows = self.fetch_postgres(&pool).await?;
                pool.close().await;
                Ok(rows)
            }
        }
    }

    async fn fetch_sqlite(&self, pool: &sqlx::SqlitePool) -> Result<Vec<ChartRow>, String> {
        use sqlx::Row;
        let raw = sqlx::query(&self.query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        raw.iter()
            .map(|row| {
                let label: String = row
                    .try_get("label")
                    .map_err(|_| "missing 'label' column".to_string())?;
                let value: f64 = row
                    .try_get("value")
                    .map_err(|_| "missing 'value' column".to_string())?;
                let color: Option<String> = row.try_get("color").ok();
                Ok(ChartRow {
                    label,
                    value,
                    color,
                })
            })
            .collect()
    }

    async fn fetch_postgres(&self, pool: &sqlx::PgPool) -> Result<Vec<ChartRow>, String> {
        use sqlx::Row;
        let raw = sqlx::query(&self.query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        raw.iter()
            .map(|row| {
                let label: String = row
                    .try_get("label")
                    .map_err(|_| "missing 'label' column".to_string())?;
                let value: f64 = row
                    .try_get("value")
                    .map_err(|_| "missing 'value' column".to_string())?;
                let color: Option<String> = row.try_get("color").ok();
                Ok(ChartRow {
                    label,
                    value,
                    color,
                })
            })
            .collect()
    }
}
