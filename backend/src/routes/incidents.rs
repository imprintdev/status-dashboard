use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use crate::{
    error::AppError,
    models::incident::{Incident, ResolveIncident},
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_incidents(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
) -> Result<Json<Vec<Incident>>, AppError> {
    let incidents = sqlx::query_as::<_, Incident>(
        "SELECT id, service_id, started_at, resolved_at, status, trigger_status, notes
         FROM incidents WHERE service_id = $1
         ORDER BY started_at DESC LIMIT 100",
    )
    .bind(&service_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(incidents))
}

pub async fn resolve_incident(
    State(state): State<AppState>,
    Path((service_id, incident_id)): Path<(String, String)>,
    Json(body): Json<ResolveIncident>,
) -> Result<Json<Incident>, AppError> {
    let now = Utc::now();

    let rows = sqlx::query(
        "UPDATE incidents SET resolved_at = $1, status = 'resolved', notes = $2
         WHERE id = $3 AND service_id = $4 AND status = 'open'",
    )
    .bind(now)
    .bind(&body.notes)
    .bind(&incident_id)
    .bind(&service_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    let incident = sqlx::query_as::<_, Incident>(
        "SELECT id, service_id, started_at, resolved_at, status, trigger_status, notes
         FROM incidents WHERE id = $1",
    )
    .bind(&incident_id)
    .fetch_one(&state.db)
    .await?;

    let _ = state.tx.send(WsMessage::IncidentResolved {
        incident_id: incident.id.clone(),
        service_id: service_id.clone(),
        resolved_at: now,
    });

    Ok(Json(incident))
}
