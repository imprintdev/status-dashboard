use crate::ws::messages::WsMessage;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{Mutex, broadcast},
    task::JoinHandle,
};

pub type SchedulerHandles = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub tx: broadcast::Sender<WsMessage>,
    pub scheduler_handles: SchedulerHandles,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            db,
            tx,
            scheduler_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
