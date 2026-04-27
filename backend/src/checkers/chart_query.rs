use super::{CheckError, CheckOutput, Checker, ConfigError};
use crate::models::check_result::CheckStatus;
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};

pub struct ChartQueryChecker {
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
        let query = config["query"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("chart_query requires 'query'".into()))?
            .to_string();
        let timeout_ms = config["timeout_ms"].as_u64();
        Ok(Self {
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

fn pg_label(row: &sqlx::postgres::PgRow) -> Result<String, String> {
    use sqlx::Row;
    if let Ok(s) = row.try_get::<String, _>("label") {
        return Ok(s);
    }
    if let Ok(d) = row.try_get::<chrono::NaiveDate, _>("label") {
        return Ok(d.to_string());
    }
    if let Ok(dt) = row.try_get::<chrono::NaiveDateTime, _>("label") {
        return Ok(dt.to_string());
    }
    if let Ok(n) = row.try_get::<i64, _>("label") {
        return Ok(n.to_string());
    }
    if let Ok(n) = row.try_get::<f64, _>("label") {
        return Ok(n.to_string());
    }
    Err("missing 'label' column (expected text, date, or number)".to_string())
}

fn pg_value(row: &sqlx::postgres::PgRow) -> Result<f64, String> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<f64, _>("value") {
        return Ok(v);
    }
    if let Ok(v) = row.try_get::<i64, _>("value") {
        return Ok(v as f64);
    }
    if let Ok(v) = row.try_get::<i32, _>("value") {
        return Ok(v as f64);
    }
    Err("missing 'value' column (expected number)".to_string())
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
        use sqlx::Row;
        let pool = sqlx::PgPool::connect(&self.connection_string)
            .await
            .map_err(|e| e.to_string())?;
        let raw = sqlx::query(&self.query)
            .fetch_all(&pool)
            .await
            .map_err(|e| e.to_string())?;
        pool.close().await;

        raw.iter()
            .map(|row| {
                let label = pg_label(row)?;
                let value = pg_value(row)?;
                let color: Option<String> = row.try_get("color").ok();
                Ok(ChartRow { label, value, color })
            })
            .collect()
    }
}
