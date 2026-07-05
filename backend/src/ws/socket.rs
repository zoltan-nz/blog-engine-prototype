use crate::state::AppState;
use crate::types::{WsEnvelope, WsMessage};
use crate::ws::dispatch;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

/// Per-connection outbound buffer. Fills only if the client stops reading;
/// senders then apply backpressure by awaiting.
const OUTBOUND_CHANNEL_CAPACITY: usize = 32;

pub async fn upgrade_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("WS connection upgrading");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, receiver) = socket.split();
    let (tx, rx) = mpsc::channel::<WsEnvelope>(OUTBOUND_CHANNEL_CAPACITY);

    // Every connection starts from a full snapshot; reconnect = fresh snapshot,
    // never event replay.
    let snapshot = dispatch::snapshot_event(&state).await;
    let _ = tx.send(dispatch::server_envelope(snapshot)).await;

    tokio::spawn(forward_broadcasts(state.events_tx.subscribe(), tx.clone()));
    tokio::spawn(write(sender, rx));
    tokio::spawn(read(receiver, tx, state));
}

async fn read(
    mut receiver: SplitStream<WebSocket>,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => match serde_json::from_str::<WsEnvelope>(&text) {
                Ok(envelope) => {
                    if let WsMessage::Command(command) = envelope.message {
                        // Each command runs as its own task so long operations
                        // (scaffold, build, preview poll) don't block the read
                        // loop or each other.
                        tokio::spawn(dispatch::dispatch_command(
                            envelope.correlation_id,
                            command,
                            tx.clone(),
                            Arc::clone(&state),
                        ));
                    }
                }
                Err(error) => warn!(%error, "discarding unparseable WS message"),
            },
            Ok(Message::Close(frame)) => {
                debug!(?frame, "WS close received");
                break;
            }
            Ok(Message::Ping(_) | Message::Pong(_) | Message::Binary(_)) => {}
            Err(error) => {
                debug!(%error, "WS receive error, dropping connection");
                break;
            }
        }
    }
}

/// Drain the shared event broadcast into this connection's outbound channel.
async fn forward_broadcasts(mut rx: broadcast::Receiver<WsEnvelope>, tx: mpsc::Sender<WsEnvelope>) {
    loop {
        match rx.recv().await {
            Ok(envelope) => {
                if tx.send(envelope).await.is_err() {
                    break; // client gone
                }
            }
            // Lagging just means this client missed intermediate states; the
            // next event carries current state, so skip ahead.
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(skipped, "WS client lagged behind event broadcast");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn write(mut sender: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<WsEnvelope>) {
    while let Some(envelope) = rx.recv().await {
        match serde_json::to_string(&envelope) {
            Ok(json) => {
                if sender
                    .send(Message::Text(Utf8Bytes::from(json)))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(error) => warn!(%error, "failed to encode WS envelope"),
        }
    }
}
