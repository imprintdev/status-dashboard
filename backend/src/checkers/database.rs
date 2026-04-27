use super::{CheckError, CheckOutput, Checker, ConfigError};
use crate::models::check_result::CheckStatus;
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};

pub struct DatabaseChecker {
    connection_string: String,
    probe_query: String,
    degraded_ms: Option<u64>,
    timeout_ms: u64,
}

impl DatabaseChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let connection_string = config["connection_string"]
            .as_str()
            .ok_or_else(|| {
                ConfigError::InvalidConfig("database checker requires 'connection_string'".into())
            })?
            .to_string();
        let probe_query = config["probe_query"]
            .as_str()
            .unwrap_or("SELECT 1")
            .to_string();
        let degraded_ms = config["degraded_ms"].as_u64();
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(10_000);
        Ok(Self {
            connection_string,
            probe_query,
            degraded_ms,
            timeout_ms,
        })
    }
}

#[async_trait]
impl Checker for DatabaseChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let start = Instant::now();
        let result = tokio::time::timeout(Duration::from_millis(self.timeout_ms), self.run_probe())
            .await
            .unwrap_or_else(|_| Err("connection timed out".to_string()));
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e),
            }),
            Ok(()) => {
                let status = if self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: None,
                    error_message: None,
                })
            }
        }
    }
}

impl DatabaseChecker {
    async fn run_probe(&self) -> Result<(), String> {
        use sqlx::Connection;
        let mut conn = sqlx::PgConnection::connect(&self.connection_string)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query(&self.probe_query)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
