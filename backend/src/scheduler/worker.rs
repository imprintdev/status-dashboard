use crate::{
    checkers::build_checker,
    models::{check_result::CheckStatus, incident::Incident, service::Service},
    ws::messages::WsMessage,
};
use chrono::Utc;
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

pub async fn run_service_loop(service_id: String, db: PgPool, tx: broadcast::Sender<WsMessage>) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

        let row = sqlx::query_as::<_, Service>(
            "SELECT id, name, service_type, config, interval_secs, enabled, created_at, updated_at
             FROM services WHERE id = $1",
        )
        .bind(&service_id)
        .fetch_optional(&db)
        .await;

        let row = match row {
            Ok(Some(r)) => r,
            Ok(None) => {
                tracing::warn!("Service {} not found, stopping worker", service_id);
                break;
            }
            Err(e) => {
                tracing::error!("DB error fetching service {}: {}", service_id, e);
                continue;
            }
        };

        if row.enabled == 0 {
            tracing::info!("Service {} disabled, stopping worker", service_id);
            break;
        }

        let interval = std::time::Duration::from_secs(row.interval_secs as u64);
        if ticker.period() != interval {
            ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await;
        }

        let config: serde_json::Value = match serde_json::from_str(&row.config) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Invalid config JSON for service {}: {}", service_id, e);
                continue;
            }
        };

        let checker = match build_checker(&row.service_type, &config) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Cannot build checker for service {}: {}", service_id, e);
                continue;
            }
        };

        let check_timeout = std::time::Duration::from_secs(row.interval_secs.max(30) as u64);
        let output = match tokio::time::timeout(check_timeout, checker.check()).await {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                tracing::error!("Checker error for service {}: {}", service_id, e);
                continue;
            }
            Err(_) => {
                tracing::error!("Checker timed out for service {}", service_id);
                continue;
            }
        };

        let check_id = Uuid::new_v4().to_string();
        let checked_at = Utc::now().to_rfc3339();
        let status_str = output.status.as_str().to_string();
        let response_ms = output.response_ms.map(|v| v as i64);
        let detail_str = output.detail.as_ref().map(|v| v.to_string());

        if let Err(e) = sqlx::query(
            "INSERT INTO check_results (id, service_id, checked_at, status, response_ms, detail, error_message)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&check_id)
        .bind(&service_id)
        .bind(&checked_at)
        .bind(&status_str)
        .bind(response_ms)
        .bind(&detail_str)
        .bind(&output.error_message)
        .execute(&db)
        .await
        {
            tracing::error!("Failed to persist check result for {}: {}", service_id, e);
        }

        handle_incident(&service_id, &output.status, &checked_at, &db, &tx).await;

        let _ = tx.send(WsMessage::CheckCompleted {
            service_id: service_id.clone(),
            check_id,
            checked_at,
            status: status_str,
            response_ms,
            detail: output.detail,
            error_message: output.error_message,
        });
    }
}

async fn handle_incident(
    service_id: &str,
    new_status: &CheckStatus,
    now: &str,
    db: &PgPool,
    tx: &broadcast::Sender<WsMessage>,
) {
    let open = sqlx::query_as::<_, Incident>(
        "SELECT id, service_id, started_at, resolved_at, status, trigger_status, notes
         FROM incidents WHERE service_id = $1 AND status = 'open'
         ORDER BY started_at DESC LIMIT 1",
    )
    .bind(service_id)
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    match (open, new_status) {
        (None, CheckStatus::Up) => {}

        (None, bad_status) => {
            let incident_id = Uuid::new_v4().to_string();
            let trigger = bad_status.as_str().to_string();
            let _ = sqlx::query(
                "INSERT INTO incidents (id, service_id, started_at, resolved_at, status, trigger_status, notes)
                 VALUES ($1, $2, $3, NULL, 'open', $4, NULL)",
            )
            .bind(&incident_id)
            .bind(service_id)
            .bind(now)
            .bind(&trigger)
            .execute(db)
            .await;
            let _ = tx.send(WsMessage::IncidentOpened {
                incident_id,
                service_id: service_id.to_string(),
                started_at: now.to_string(),
                trigger_status: trigger,
            });
        }

        (Some(incident), CheckStatus::Up) => {
            let _ = sqlx::query(
                "UPDATE incidents SET resolved_at = $1, status = 'resolved' WHERE id = $2",
            )
            .bind(now)
            .bind(&incident.id)
            .execute(db)
            .await;
            let _ = tx.send(WsMessage::IncidentResolved {
                incident_id: incident.id,
                service_id: service_id.to_string(),
                resolved_at: now.to_string(),
            });
        }

        (Some(_), _) => {}
    }
}
