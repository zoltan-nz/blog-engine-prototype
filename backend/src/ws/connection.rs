//! Transport-neutral half of a client connection, shared by the `/ws`
//! socket and the desktop app's Tauri IPC: snapshot, broadcast fan-in and
//! command dispatch. Transports only move `WsEnvelope` JSON.

use crate::state::AppState;
use crate::types::{WsEnvelope, WsMessage};
use crate::ws::dispatch;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

/// Per-connection outbound buffer. Fills only if the client stops reading;
/// senders then apply backpressure by awaiting.
const OUTBOUND_CHANNEL_CAPACITY: usize = 32;

/// Start a connection: the receiver yields the snapshot, then every
/// broadcast event plus replies sent through the returned sender.
pub async fn open(state: &Arc<AppState>) -> (mpsc::Sender<WsEnvelope>, mpsc::Receiver<WsEnvelope>) {
    let (tx, rx) = mpsc::channel::<WsEnvelope>(OUTBOUND_CHANNEL_CAPACITY);

    // Every connection starts from a full snapshot; reconnect = fresh snapshot,
    // never event replay.
    let snapshot = dispatch::snapshot_event(state).await;
    let _ = tx.send(dispatch::server_envelope(snapshot)).await;

    tokio::spawn(forward_broadcasts(state.events_tx.subscribe(), tx.clone()));
    (tx, rx)
}

/// Parse one client message and dispatch its command; replies go to `tx`.
pub fn handle_text(text: &str, tx: &mpsc::Sender<WsEnvelope>, state: &Arc<AppState>) {
    match serde_json::from_str::<WsEnvelope>(text) {
        Ok(envelope) => {
            if let WsMessage::Command(command) = envelope.message {
                // Each command runs as its own task so long operations
                // (scaffold, build, preview poll) don't block the read
                // loop or each other.
                tokio::spawn(dispatch::dispatch_command(
                    envelope.correlation_id,
                    command,
                    tx.clone(),
                    Arc::clone(state),
                ));
            }
        }
        Err(error) => warn!(%error, "discarding unparseable client message"),
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
