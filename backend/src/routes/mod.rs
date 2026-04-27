use crate::state::AppState;
use crate::ws;
use axum::{
    Router,
    routing::{get, patch},
};

pub mod check_results;
pub mod incidents;
pub mod services;
pub mod systems;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/health", get(health))
                // Systems
                .route(
                    "/systems",
                    get(systems::list_systems).post(systems::create_system),
                )
                .route(
                    "/systems/{id}",
                    axum::routing::put(systems::update_system).delete(systems::delete_system),
                )
                // Services
                .route(
                    "/services",
                    get(services::list_services).post(services::create_service),
                )
                .route(
                    "/services/{id}",
                    get(services::get_service)
                        .put(services::update_service)
                        .delete(services::delete_service),
                )
                .route("/services/{id}/checks", get(check_results::list_checks))
                .route("/services/{id}/uptime", get(services::get_uptime))
                .route("/services/{id}/incidents", get(incidents::list_incidents))
                .route(
                    "/services/:service_id/incidents/:incident_id",
                    patch(incidents::resolve_incident),
                ),
        )
        .route("/ws", get(ws::ws_handler))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
