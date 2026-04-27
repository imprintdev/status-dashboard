use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::AppError,
    models::system::{CreateSystem, System, UpdateSystem},
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_systems(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query_as::<_, System>(
        "SELECT id, name, description, created_at, updated_at FROM systems ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for r in &rows {
        let health = derive_health(&r.id, &state).await;
        let count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM service_systems WHERE system_id = $1",
        )
        .bind(&r.id)
        .fetch_one(&state.db)
        .await?;

        result.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "health": health,
            "service_count": count,
            "created_at": r.created_at,
            "updated_at": r.updated_at
        }));
    }

    Ok(Json(result))
}

pub async fn create_system(
    State(state): State<AppState>,
    Json(body): Json<CreateSystem>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO systems (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    let resp = serde_json::json!({
        "id": id,
        "name": body.name,
        "description": body.description,
        "health": "unknown",
        "service_count": 0,
        "created_at": now,
        "updated_at": now
    });
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_system(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSystem>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query_as::<_, System>(
        "SELECT id, name, description, created_at, updated_at FROM systems WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = body.name.unwrap_or(r.name);
    let description = body.description.or(r.description);
    let now = Utc::now();

    sqlx::query("UPDATE systems SET name = $1, description = $2, updated_at = $3 WHERE id = $4")
        .bind(&name)
        .bind(&description)
        .bind(now)
        .bind(&id)
        .execute(&state.db)
        .await?;

    let _ = state.tx.send(WsMessage::SystemUpdated {
        system_id: id.clone(),
        fields: serde_json::json!({ "name": name, "description": description, "updated_at": now }),
    });

    Ok(Json(serde_json::json!({ "id": id, "name": name, "updated_at": now })))
}

pub async fn delete_system(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let rows = sqlx::query("DELETE FROM systems WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn derive_health(system_id: &str, state: &AppState) -> &'static str {
    // DISTINCT ON gives exactly one row per service_id, the most recent by checked_at
    let statuses = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT ON (service_id) status
         FROM check_results
         WHERE service_id IN (SELECT service_id FROM service_systems WHERE system_id = $1)
         ORDER BY service_id, checked_at DESC",
    )
    .bind(system_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if statuses.is_empty() {
        return "unknown";
    }

    let mut worst = "up";
    for (status,) in &statuses {
        match status.as_str() {
            "down" => return "down",
            "degraded" => worst = "degraded",
            _ => {}
        }
    }
    worst
}
