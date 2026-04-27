use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct HttpBodyChecker {
    url: String,
    timeout_ms: u64,
    degraded_ms: Option<u64>,
    headers: HashMap<String, String>,
    body_limit: usize,
}

impl HttpBodyChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let url = config["url"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("http_body checker requires 'url'".into()))?
            .to_string();
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(10_000);
        let degraded_ms = config["degraded_ms"].as_u64();
        let body_limit = config["body_limit"].as_u64().unwrap_or(4096) as usize;
        let headers = config["headers"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { url, timeout_ms, degraded_ms, headers, body_limit })
    }
}

#[async_trait]
impl Checker for HttpBodyChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| CheckError::Error(e.to_string()))?;

        let mut req = client.get(&self.url);
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }

        let start = Instant::now();
        let result = req.send().await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e.to_string()),
            }),
            Ok(resp) => {
                let http_status = resp.status().as_u16();
                let is_success = resp.status().is_success();
                let body = resp.text().await.unwrap_or_default();
                let truncated = if body.len() > self.body_limit {
                    format!("{}\n[truncated]", &body[..self.body_limit])
                } else {
                    body
                };

                let status = if !is_success {
                    CheckStatus::Down
                } else if self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };

                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({
                        "http_status": http_status,
                        "body": truncated,
                    })),
                    error_message: if !is_success { Some(format!("HTTP {http_status}")) } else { None },
                })
            }
        }
    }
}
