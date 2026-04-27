use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::collections::HashMap;
use crate::{error::AppError, models::check_result::CheckResult, state::AppState};

pub async fn list_checks(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<CheckResult>>, AppError> {
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);

    let results = if let Some(before_id) = params.get("before_id") {
        let before_at = sqlx::query_scalar::<_, String>(
            "SELECT checked_at FROM check_results WHERE id = $1",
        )
        .bind(before_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(at) = before_at {
            sqlx::query_as::<_, CheckResult>(
                "SELECT id, service_id, checked_at, status, response_ms, detail, error_message
                 FROM check_results WHERE service_id = $1 AND checked_at < $2
                 ORDER BY checked_at DESC LIMIT $3",
            )
            .bind(&service_id)
            .bind(&at)
            .bind(limit)
            .fetch_all(&state.db)
            .await?
        } else {
            vec![]
        }
    } else {
        sqlx::query_as::<_, CheckResult>(
            "SELECT id, service_id, checked_at, status, response_ms, detail, error_message
             FROM check_results WHERE service_id = $1
             ORDER BY checked_at DESC LIMIT $2",
        )
        .bind(&service_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(results))
}
