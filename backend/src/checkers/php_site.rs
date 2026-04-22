use async_trait::async_trait;
use serde_json::Value;
use std::time::Instant;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct PhpSiteChecker {
    url: String,
    fpm_status_url: Option<String>,
    expected_content: Option<String>,
    timeout_ms: u64,
    degraded_ms: Option<u64>,
}

impl PhpSiteChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let url = config["url"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("php_site checker requires 'url'".into()))?
            .to_string();
        let fpm_status_url = config["fpm_status_url"].as_str().map(str::to_string);
        let expected_content = config["expected_content"].as_str().map(str::to_string);
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(10_000);
        let degraded_ms = config["degraded_ms"].as_u64();
        Ok(Self { url, fpm_status_url, expected_content, timeout_ms, degraded_ms })
    }
}

#[async_trait]
impl Checker for PhpSiteChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| CheckError::Error(e.to_string()))?;

        let start = Instant::now();
        let resp = match client.get(&self.url).send().await {
            Err(e) => {
                return Ok(CheckOutput {
                    status: CheckStatus::Down,
                    response_ms: Some(start.elapsed().as_millis() as u64),
                    detail: None,
                    error_message: Some(e.to_string()),
                });
            }
            Ok(r) => r,
        };
        let elapsed = start.elapsed().as_millis() as u64;
        let http_status = resp.status().as_u16();

        if !resp.status().is_success() {
            return Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: Some(serde_json::json!({ "http_status": http_status })),
                error_message: Some(format!("HTTP {http_status}")),
            });
        }

        if let Some(expected) = &self.expected_content {
            let body = resp.text().await.unwrap_or_default();
            if !body.contains(expected.as_str()) {
                return Ok(CheckOutput {
                    status: CheckStatus::Degraded,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({ "http_status": http_status })),
                    error_message: Some("Expected content not found in response".to_string()),
                });
            }
        }

        let mut fpm_ok = true;
        let mut fpm_detail: Option<String> = None;
        if let Some(fpm_url) = &self.fpm_status_url {
            match client.get(fpm_url).send().await {
                Err(e) => {
                    fpm_ok = false;
                    fpm_detail = Some(e.to_string());
                }
                Ok(r) if !r.status().is_success() => {
                    fpm_ok = false;
                    fpm_detail = Some(format!("FPM status HTTP {}", r.status().as_u16()));
                }
                Ok(_) => {}
            }
        }

        let status = if !fpm_ok || self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
            CheckStatus::Degraded
        } else {
            CheckStatus::Up
        };

        Ok(CheckOutput {
            status,
            response_ms: Some(elapsed),
            detail: Some(serde_json::json!({
                "http_status": http_status,
                "fpm_ok": fpm_ok,
                "fpm_error": fpm_detail
            })),
            error_message: None,
        })
    }
}
