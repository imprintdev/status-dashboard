use crate::{state::SchedulerHandles, ws::messages::WsMessage};
use sqlx::PgPool;
use tokio::sync::broadcast;

pub mod worker;

pub async fn start_all(db: &PgPool, tx: broadcast::Sender<WsMessage>, handles: &SchedulerHandles) {
    let services = sqlx::query_as::<_, (i64,)>(r#"SELECT id FROM services WHERE enabled = 1"#)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let mut map = handles.lock().await;
    for (id,) in services {
        let handle = tokio::spawn(supervised_loop(id.to_string(), db.clone(), tx.clone()));
        map.insert(id.to_string(), handle);
    }
}

pub async fn spawn_service(
    service_id: String,
    db: &PgPool,
    tx: broadcast::Sender<WsMessage>,
    handles: &SchedulerHandles,
) {
    let mut map = handles.lock().await;
    if let Some(old) = map.remove(&service_id) {
        old.abort();
    }
    let handle = tokio::spawn(supervised_loop(service_id.clone(), db.clone(), tx));
    map.insert(service_id, handle);
}

/// Wraps `run_service_loop` so that an unexpected panic or early return causes
/// a short backoff and restart rather than silently killing the worker.
async fn supervised_loop(service_id: String, db: PgPool, tx: broadcast::Sender<WsMessage>) {
    let mut backoff = std::time::Duration::from_secs(5);
    loop {
        let result = tokio::spawn(worker::run_service_loop(
            service_id.clone(),
            db.clone(),
            tx.clone(),
        ))
        .await;

        match result {
            // Worker returned normally (service disabled/deleted) — stop supervising.
            Ok(()) => break,
            // Worker panicked — log and restart after backoff.
            Err(e) => {
                tracing::error!(
                    "Worker for service {} panicked: {:?}; restarting in {:?}",
                    service_id,
                    e,
                    backoff
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(std::time::Duration::from_secs(60));
            }
        }
    }
}

pub async fn abort_service(service_id: &str, handles: &SchedulerHandles) {
    let mut map = handles.lock().await;
    if let Some(handle) = map.remove(service_id) {
        handle.abort();
    }
}
