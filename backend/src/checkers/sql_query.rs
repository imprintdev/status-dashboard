use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct SqlQueryChecker {
    driver: String,
    connection_string: String,
    query: String,
    down_threshold: Option<Threshold>,
    degraded_threshold: Option<Threshold>,
    timeout_ms: Option<u64>,
}

struct Threshold {
    gt:  Option<f64>,
    lt:  Option<f64>,
    gte: Option<f64>,
    lte: Option<f64>,
    eq:  Option<f64>,
    neq: Option<f64>,
}

impl Threshold {
    fn from_value(v: &Value) -> Option<Self> {
        if v.is_null() { return None; }
        Some(Self {
            gt:  v["gt"].as_f64(),
            lt:  v["lt"].as_f64(),
            gte: v["gte"].as_f64(),
            lte: v["lte"].as_f64(),
            eq:  v["eq"].as_f64(),
            neq: v["neq"].as_f64(),
        })
    }

    fn matches(&self, n: f64) -> bool {
        if let Some(v) = self.gt  { if !(n >  v) { return false; } }
        if let Some(v) = self.lt  { if !(n <  v) { return false; } }
        if let Some(v) = self.gte { if !(n >= v) { return false; } }
        if let Some(v) = self.lte { if !(n <= v) { return false; } }
        if let Some(v) = self.eq  { if n != v    { return false; } }
        if let Some(v) = self.neq { if n == v    { return false; } }
        true
    }
}

impl SqlQueryChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let driver = config["driver"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("sql_query checker requires 'driver'".into()))?
            .to_string();
        let connection_string = config["connection_string"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("sql_query checker requires 'connection_string'".into()))?
            .to_string();
        let query = config["query"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("sql_query checker requires 'query'".into()))?
            .to_string();
        let down_threshold = Threshold::from_value(&config["down_threshold"]);
        let degraded_threshold = Threshold::from_value(&config["degraded_threshold"]);
        let timeout_ms = config["timeout_ms"].as_u64();
        Ok(Self { driver, connection_string, query, down_threshold, degraded_threshold, timeout_ms })
    }
}

#[async_trait]
impl Checker for SqlQueryChecker {
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
            Ok(value) => {
                let status = if self.down_threshold.as_ref().map(|t| t.matches(value)).unwrap_or(false) {
                    CheckStatus::Down
                } else if self.degraded_threshold.as_ref().map(|t| t.matches(value)).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({ "value": value })),
                    error_message: None,
                })
            }
        }
    }
}

impl SqlQueryChecker {
    async fn run_query(&self) -> Result<f64, String> {
        let timeout = Duration::from_millis(self.timeout_ms.unwrap_or(10_000));
        tokio::time::timeout(timeout, self.run_query_inner())
            .await
            .map_err(|_| "query timed out".to_string())?
    }

    async fn run_query_inner(&self) -> Result<f64, String> {
        match self.driver.as_str() {
            "sqlite" => {
                let pool = sqlx::SqlitePool::connect(&self.connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                let row: (f64,) = sqlx::query_as(&self.query)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                pool.close().await;
                Ok(row.0)
            }
            "postgresql" | "postgres" => {
                let pool = sqlx::PgPool::connect(&self.connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                let row: (f64,) = sqlx::query_as(&self.query)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                pool.close().await;
                Ok(row.0)
            }
            other => Err(format!("Unsupported database driver: {other}")),
        }
    }
}
