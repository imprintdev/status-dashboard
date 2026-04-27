use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::AppError,
    models::{
        check_result::CheckResult,
        service::{CreateService, Service, UpdateService},
    },
    scheduler,
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_services(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query_as::<_, Service>(
        "SELECT id, name, service_type, config, interval_secs, enabled, created_at, updated_at
         FROM services ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let system_ids: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT system_id FROM service_systems WHERE service_id = $1",
        )
        .bind(&r.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let latest = sqlx::query_as::<_, CheckResult>(
            "SELECT id, service_id, checked_at, status, response_ms, detail, error_message
             FROM check_results WHERE service_id = $1 ORDER BY checked_at DESC LIMIT 1",
        )
        .bind(&r.id)
        .fetch_optional(&state.db)
        .await?;

        let latest_check = latest.map(|c| {
            let detail = c.detail
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
            serde_json::json!({
                "id": c.id,
                "checked_at": c.checked_at,
                "status": c.status,
                "response_ms": c.response_ms,
                "error_message": c.error_message,
                "detail": detail
            })
        });

        result.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "service_type": r.service_type,
            "config": serde_json::from_str::<serde_json::Value>(&r.config).unwrap_or(serde_json::Value::Null),
            "interval_secs": r.interval_secs,
            "enabled": r.enabled,
            "system_ids": system_ids,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
            "latest_check": latest_check
        }));
    }
    Ok(Json(result))
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query_as::<_, Service>(
        "SELECT id, name, service_type, config, interval_secs, enabled, created_at, updated_at
         FROM services WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let system_ids: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT system_id FROM service_systems WHERE service_id = $1",
    )
    .bind(&r.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "id": r.id,
        "name": r.name,
        "service_type": r.service_type,
        "config": serde_json::from_str::<serde_json::Value>(&r.config).unwrap_or(serde_json::Value::Null),
        "interval_secs": r.interval_secs,
        "enabled": r.enabled,
        "system_ids": system_ids,
        "created_at": r.created_at,
        "updated_at": r.updated_at
    })))
}

pub async fn create_service(
    State(state): State<AppState>,
    Json(body): Json<CreateService>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let interval = body.interval_secs.unwrap_or(60);
    let config_str = body.config.to_string();

    crate::checkers::build_checker(&body.service_type, &body.config)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    sqlx::query(
        "INSERT INTO services (id, name, service_type, config, interval_secs, enabled, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, TRUE, $6, $7)",
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.service_type)
    .bind(&config_str)
    .bind(interval)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    if let Some(system_ids) = &body.system_ids {
        for sid in system_ids {
            sqlx::query(
                "INSERT INTO service_systems (service_id, system_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(&id)
            .bind(sid)
            .execute(&state.db)
            .await?;
        }
    }

    scheduler::spawn_service(id.clone(), &state.db, state.tx.clone(), &state.scheduler_handles).await;

    let system_ids = body.system_ids.unwrap_or_default();
    let resp = serde_json::json!({ "id": id, "name": body.name, "system_ids": system_ids, "created_at": now });
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateService>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query_as::<_, Service>(
        "SELECT id, name, service_type, config, interval_secs, enabled, created_at, updated_at
         FROM services WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = body.name.unwrap_or(r.name);
    let config_str = body.config.as_ref().map(|c| c.to_string()).unwrap_or(r.config);
    let interval = body.interval_secs.unwrap_or(r.interval_secs);
    let enabled = body.enabled.unwrap_or(r.enabled);
    let now = Utc::now();

    sqlx::query(
        "UPDATE services SET name = $1, config = $2, interval_secs = $3, enabled = $4, updated_at = $5 WHERE id = $6",
    )
    .bind(&name)
    .bind(&config_str)
    .bind(interval)
    .bind(enabled)
    .bind(now)
    .bind(&id)
    .execute(&state.db)
    .await?;

    if let Some(system_ids) = &body.system_ids {
        sqlx::query("DELETE FROM service_systems WHERE service_id = $1")
            .bind(&id)
            .execute(&state.db)
            .await?;
        for sid in system_ids {
            sqlx::query(
                "INSERT INTO service_systems (service_id, system_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(&id)
            .bind(sid)
            .execute(&state.db)
            .await?;
        }
    }

    if enabled {
        scheduler::spawn_service(id.clone(), &state.db, state.tx.clone(), &state.scheduler_handles).await;
    } else {
        scheduler::abort_service(&id, &state.scheduler_handles).await;
    }

    let effective_system_ids: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT system_id FROM service_systems WHERE service_id = $1",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let _ = state.tx.send(WsMessage::ServiceUpdated {
        service_id: id.clone(),
        fields: serde_json::json!({
            "name": name,
            "interval_secs": interval,
            "enabled": enabled,
            "system_ids": effective_system_ids
        }),
    });

    Ok(Json(serde_json::json!({ "id": id, "updated_at": now })))
}

pub async fn delete_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let rows = sqlx::query("DELETE FROM services WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    scheduler::abort_service(&id, &state.scheduler_handles).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_uptime(
    State(state): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let window = params.get("window").map(String::as_str).unwrap_or("7d");
    let days: i64 = match window {
        "24h" => 1,
        "30d" => 30,
        "90d" => 90,
        _ => 7,
    };
    let since = Utc::now() - chrono::Duration::days(days);

    let total: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM check_results WHERE service_id = $1 AND checked_at >= $2",
    )
    .bind(&id)
    .bind(since)
    .fetch_one(&state.db)
    .await?;

    let up: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM check_results WHERE service_id = $1 AND checked_at >= $2 AND status = 'up'",
    )
    .bind(&id)
    .bind(since)
    .fetch_one(&state.db)
    .await?;

    let pct = if total == 0 { None } else { Some(up as f64 / total as f64 * 100.0) };
    Ok(Json(serde_json::json!({ "window": window, "uptime_pct": pct, "total_checks": total, "up_checks": up })))
}
