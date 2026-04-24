use sqlx::SqlitePool;
use tokio::sync::broadcast;
use uuid::Uuid;
use chrono::Utc;
use crate::{
    checkers::build_checker,
    models::{check_result::CheckStatus, incident::Incident},
    ws::messages::WsMessage,
};

pub async fn run_service_loop(
    service_id: String,
    db: SqlitePool,
    tx: broadcast::Sender<WsMessage>,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

        let row = sqlx::query!(
            r#"SELECT id as "id!", service_type as "service_type!", config as "config!",
               interval_secs as "interval_secs!", enabled as "enabled!"
               FROM services WHERE id = ?"#,
            service_id
        )
        .fetch_optional(&db)
        .await;

        let row = match row {
            Ok(Some(r)) => r,
            Ok(None) => { tracing::warn!("Service {} not found, stopping worker", service_id); break; }
            Err(e)    => { tracing::error!("DB error fetching service {}: {}", service_id, e); continue; }
        };

        if row.enabled == 0 {
            tracing::info!("Service {} disabled, stopping worker", service_id);
            break;
        }

        let interval = std::time::Duration::from_secs(row.interval_secs as u64);
        if ticker.period() != interval {
            ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            ticker.tick().await; // consume the immediate first tick
        }

        let config: serde_json::Value = match serde_json::from_str(&row.config) {
            Ok(v)  => v,
            Err(e) => { tracing::error!("Invalid config JSON for service {}: {}", service_id, e); continue; }
        };

        let checker = match build_checker(&row.service_type, &config) {
            Ok(c)  => c,
            Err(e) => { tracing::error!("Cannot build checker for service {}: {}", service_id, e); continue; }
        };

        let check_timeout = std::time::Duration::from_secs(row.interval_secs.max(30) as u64);
        let output = match tokio::time::timeout(check_timeout, checker.check()).await {
            Ok(Ok(o))  => o,
            Ok(Err(e)) => { tracing::error!("Checker error for service {}: {}", service_id, e); continue; }
            Err(_)     => { tracing::error!("Checker timed out for service {}", service_id); continue; }
        };

        let check_id = Uuid::new_v4().to_string();
        let checked_at = Utc::now().to_rfc3339();
        let status_str = output.status.as_str().to_string();
        let response_ms = output.response_ms.map(|v| v as i64);
        let detail_str = output.detail.as_ref().map(|v| v.to_string());

        if let Err(e) = sqlx::query!(
            "INSERT INTO check_results (id, service_id, checked_at, status, response_ms, detail, error_message)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            check_id, service_id, checked_at, status_str, response_ms, detail_str, output.error_message
        )
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
    db: &SqlitePool,
    tx: &broadcast::Sender<WsMessage>,
) {
    let open = sqlx::query!(
        r#"SELECT id as "id!", service_id as "service_id!", started_at as "started_at!",
           resolved_at, status as "status!", trigger_status as "trigger_status!", notes
           FROM incidents WHERE service_id = ? AND status = 'open'
           ORDER BY started_at DESC LIMIT 1"#,
        service_id
    )
    .fetch_optional(db)
    .await
    .unwrap_or(None)
    .map(|r| Incident {
        id: r.id,
        service_id: r.service_id,
        started_at: r.started_at,
        resolved_at: r.resolved_at,
        status: r.status,
        trigger_status: r.trigger_status,
        notes: r.notes,
    });

    match (open, new_status) {
        (None, CheckStatus::Up) => {}

        (None, bad_status) => {
            let incident_id = Uuid::new_v4().to_string();
            let trigger = bad_status.as_str().to_string();
            let _ = sqlx::query!(
                "INSERT INTO incidents (id, service_id, started_at, resolved_at, status, trigger_status, notes)
                 VALUES (?, ?, ?, NULL, 'open', ?, NULL)",
                incident_id, service_id, now, trigger
            )
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
            let _ = sqlx::query!(
                "UPDATE incidents SET resolved_at = ?, status = 'resolved' WHERE id = ?",
                now, incident.id
            )
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
