use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub config: String,
    pub interval_secs: i64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateService {
    pub name: String,
    pub service_type: String,
    pub config: serde_json::Value,
    pub interval_secs: Option<i64>,
    pub system_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateService {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub interval_secs: Option<i64>,
    pub enabled: Option<bool>,
    /// Some(ids) = replace membership, None = no change
    pub system_ids: Option<Vec<String>>,
}
