mod checkers;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod scheduler;
mod state;
mod ws;

use chrono::Utc;
use state::AppState;

const HEARTBEAT_INTERVAL_SECS: u64 = 30;
const RETENTION_CHECK_INTERVAL_SECS: u64 = 86_400; // daily
const CHECK_RESULT_RETENTION_DAYS: i64 = 90;
const INCIDENT_RETENTION_DAYS: i64 = 180;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::from_env();

    let pool = db::create_pool(&cfg.database_url).await;
    // db::run_migrations(&pool).await;

    let app_state = AppState::new(pool.clone());

    scheduler::start_all(&pool, app_state.tx.clone(), &app_state.scheduler_handles).await;

    // Heartbeat: ping all WS clients periodically
    let ping_tx = app_state.tx.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let _ = ping_tx.send(ws::messages::WsMessage::Ping { ts: Utc::now() });
        }
    });

    // Data retention: purge old check results and resolved incidents daily
    let retention_db = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            RETENTION_CHECK_INTERVAL_SECS,
        ));
        loop {
            interval.tick().await;
            let check_cutoff = Utc::now() - chrono::Duration::days(CHECK_RESULT_RETENTION_DAYS);
            let incident_cutoff = Utc::now() - chrono::Duration::days(INCIDENT_RETENTION_DAYS);

            if let Err(e) = sqlx::query("DELETE FROM check_results WHERE checked_at < $1")
                .bind(check_cutoff)
                .execute(&retention_db)
                .await
            {
                tracing::error!("check_results retention failed: {e}");
            }

            if let Err(e) =
                sqlx::query("DELETE FROM incidents WHERE status = 'resolved' AND resolved_at < $1")
                    .bind(incident_cutoff)
                    .execute(&retention_db)
                    .await
            {
                tracing::error!("incidents retention failed: {e}");
            }
        }
    });

    let router = routes::router(app_state);
    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, draining connections");
}
