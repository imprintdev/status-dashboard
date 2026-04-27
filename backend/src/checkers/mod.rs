use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use crate::models::check_result::CheckStatus;

pub mod aws_billing;
pub mod chart_query;
pub mod database;
pub mod http;
pub mod http_body;
pub mod php_site;
pub mod preflight;
pub mod sql_query;

#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub status: CheckStatus,
    pub response_ms: Option<u64>,
    pub detail: Option<Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("Checker error: {0}")]
    Error(String),
}

#[async_trait]
pub trait Checker: Send + Sync {
    async fn check(&self) -> Result<CheckOutput, CheckError>;
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Unknown service type: {0}")]
    UnknownType(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

pub fn infer_driver(connection_string: &str) -> Result<&'static str, ConfigError> {
    if connection_string.starts_with("sqlite:") {
        Ok("sqlite")
    } else if connection_string.starts_with("postgresql://") || connection_string.starts_with("postgres://") {
        Ok("postgresql")
    } else {
        Err(ConfigError::InvalidConfig(
            format!("cannot infer driver from connection_string '{connection_string}' — expected 'sqlite:' or 'postgresql://'")
        ))
    }
}

pub fn build_checker(
    service_type: &str,
    config: &Value,
) -> Result<Box<dyn Checker>, ConfigError> {
    match service_type {
        "http" => Ok(Box::new(http::HttpChecker::from_config(config)?)),
        "http_body" => Ok(Box::new(http_body::HttpBodyChecker::from_config(config)?)),
        "database" => Ok(Box::new(database::DatabaseChecker::from_config(config)?)),
        "aws_billing" => Ok(Box::new(aws_billing::AwsBillingChecker::from_config(config)?)),
        "php_site" => Ok(Box::new(php_site::PhpSiteChecker::from_config(config)?)),
        "preflight" => Ok(Box::new(preflight::PreflightChecker::from_config(config)?)),
        "sql_query" => Ok(Box::new(sql_query::SqlQueryChecker::from_config(config)?)),
        "chart_query" => Ok(Box::new(chart_query::ChartQueryChecker::from_config(config)?)),
        other => Err(ConfigError::UnknownType(other.to_string())),
    }
}
