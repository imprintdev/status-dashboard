use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Incident {
    pub id: String,
    pub service_id: String,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub status: String,
    pub trigger_status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveIncident {
    pub notes: Option<String>,
}
