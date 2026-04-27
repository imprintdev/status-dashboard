use crate::state::AppState;
use crate::ws::messages::WsMessage;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

pub mod messages;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let mut rx: broadcast::Receiver<WsMessage> = state.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(ws_msg) => {
                        if let Ok(json) = serde_json::to_string(&ws_msg)
                            && sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged by {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = receiver.next() => {
                match msg {
                    None | Some(Err(_)) => break,
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}
