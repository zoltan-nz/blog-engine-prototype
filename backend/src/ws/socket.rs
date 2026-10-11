use crate::state::AppState;
use crate::types::{WsEnvelope, WsMessage};
use crate::ws::dispatch;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

/// Per-connection outbound buffer. Fills only if the client stops reading;
/// senders then apply backpressure by awaiting.
const OUTBOUND_CHANNEL_CAPACITY: usize = 32;

/// Browsers attach `Origin` to every WebSocket handshake but skip CORS for it,
/// so without this check any website could drive the local backend.
pub async fn upgrade_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Response {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if let Some(origin) = origin.filter(|origin| !is_allowed_origin(origin)) {
        warn!(%origin, "WS upgrade rejected: foreign origin");
        return StatusCode::FORBIDDEN.into_response();
    }

    info!("WS connection upgrading");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Local pages (Vite dev server, the SPA on :8080, the Tauri webview) only.
/// A missing `Origin` means a non-browser client and is handled by the caller.
fn is_allowed_origin(origin: &str) -> bool {
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    let scheme = uri.scheme_str();
    let host = uri.host();

    matches!(
        (scheme, host),
        (Some("tauri"), Some("localhost"))
            | (
                Some("http" | "https"),
                Some("localhost" | "127.0.0.1" | "[::1]" | "tauri.localhost")
            )
    )
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

#[cfg(test)]
mod tests {
    use super::is_allowed_origin;

    #[test]
    fn local_and_tauri_origins_are_allowed() {
        for origin in [
            "http://localhost:5173",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
            "tauri://localhost",
            "http://tauri.localhost",
        ] {
            assert!(is_allowed_origin(origin), "{origin}");
        }
    }

    #[test]
    fn foreign_and_malformed_origins_are_rejected() {
        for origin in [
            "https://evil.example",
            "http://localhost.evil.example",
            "null",
            "file://",
        ] {
            assert!(!is_allowed_origin(origin), "{origin}");
        }
    }
}
