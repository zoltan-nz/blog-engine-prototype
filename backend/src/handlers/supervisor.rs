use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;

use crate::state::{AppState, CommandMessage};
use admin_protocol::{Envelope, Event};
use axum::extract::State;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::stream::SplitSink;
use tokio::sync::{broadcast, oneshot};
use uuid::Uuid;

/// WebSocket endpoint the `astro-supervisor` connects to.
///
/// Only one supervisor may be connected at a time. A second connection attempt
/// is rejected with 409 before the WebSocket handshake completes.
///
/// Steps:
///   1. Lock `state.command_rx`.
///   2. Call `.take()` on the inner `Option` to claim the receiver.
///   3. If `None`  → another supervisor is already connected; return a 409
///      response. The rejection MUST happen here, before `ws.on_upgrade()`,
///      because once `on_upgrade` is called the HTTP 101 response is committed.
///   4. If `Some(rx)` → drop the Mutex guard, then call
///      `ws.on_upgrade(move |socket| handle_supervisor_session(socket, rx, state))`
///      and return the result.
pub async fn supervisor_ws(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    let rx = state.command_rx.lock().await.take();
    match rx {
        None => {
            tracing::warn!("supervisor connection rejected — already connected");
            StatusCode::CONFLICT.into_response()
        }
        Some(rx) => {
            tracing::info!("supervisor connected");
            ws.on_upgrade(move |socket| handle_supervisor_session(socket, rx, state))
        }
    }
}

type WsSender = SplitSink<WebSocket, Message>;
type Pending = HashMap<Uuid, oneshot::Sender<Event>>;

async fn forward_command(
    cmd_msg: CommandMessage,
    sender: &mut WsSender,
    pending: &mut Pending,
) -> bool {
    let id = cmd_msg.envelope.id;
    let json =
        serde_json::to_string(&cmd_msg.envelope).expect("Failed to serialize command envelope");
    tracing::debug!(command_id = %id, "forwarding command to supervisor");
    pending.insert(id, cmd_msg.response_tx);
    sender
        .send(Message::Text(Utf8Bytes::from(json)))
        .await
        .is_ok()
}

fn resolve_event(text: &str, pending: &mut Pending, event_tx: &broadcast::Sender<Envelope<Event>>) {
    let Ok(envelope) = serde_json::from_str::<Envelope<Event>>(text) else {
        return;
    };

    if let Some(cid) = envelope.correlation_id {
        if let Some(tx) = pending.remove(&cid) {
            tracing::debug!(correlation_id = %cid, "resolved pending command via oneshot");
            let _ = tx.send(envelope.payload.clone());
        }
    }
    let _ = event_tx.send(envelope);
}
// ---------------------------------------------------------------------------
// Session handler — runs for the lifetime of one supervisor connection
// ---------------------------------------------------------------------------

/// Drives the bidirectional tunnel between backend and supervisor.
///
/// Two concurrent directions via `tokio::select!`:
///
/// Outbound (backend → supervisor):
///   - Receive `CommandMessage` from `command_rx`
///   - Serialize `envelope` as JSON, send over `socket`
///   - Store `(envelope.id, response_tx)` in `pending` HashMap
///
/// Inbound (supervisor → backend):
///   - Receive JSON text frame from `socket`
///   - Deserialize as `Envelope<Event>`
///   - If `correlation_id` is set, find matching entry in `pending` and send
///     the event payload to the `response_tx` (resolves the HTTP handler's wait)
///   - Broadcast the envelope to `state.event_tx` for all browser subscribers
///
/// On disconnect: drain `pending`, send an `Event::Error` for each so HTTP
/// handlers don't hang indefinitely.
///
/// NOTE: `command_rx` is dropped when this function returns. `state.command_rx`
/// stays `None` — reconnect requires a backend restart for now (deferred).
pub(crate) async fn handle_supervisor_session(
    socket: WebSocket,
    mut command_rx: tokio::sync::mpsc::Receiver<CommandMessage>,
    state: Arc<AppState>,
) {
    tracing::info!("supervisor session started");
    let (mut sender, mut receiver) = socket.split();
    let mut pending: HashMap<Uuid, oneshot::Sender<Event>> = HashMap::new();

    loop {
        tokio::select! {
            msg = command_rx.recv() => {
                    match msg {
                    None => break,
                    Some(cmd_msg) => if !forward_command(cmd_msg, &mut sender, &mut pending).await { break }
                }
            }
            frame = receiver.next() => {
                    match frame {
                    None | Some(Err(_)) => break,
                    Some(Ok(Message::Text(text))) => resolve_event(&text, &mut pending, &state.event_tx),
                    _ => {}
                }
            }
        }
    }

    tracing::warn!(pending = pending.len(), "supervisor session ended, draining pending commands");
    for (_, tx) in pending.drain() {
        let _ = tx.send(Event::Error {
            code: admin_protocol::ErrorCode::Internal,
            message: "Supervisor disconnected before responding".to_string(),
            command_id: None,
        });
    }

    let (new_tx, new_rx) = tokio::sync::mpsc::channel(32);
    *state.command_rx.lock().await = Some(new_rx);
    *state.command_tx.lock().await = new_tx;
    tracing::info!("command channel restored - supervisor can reconnect");
}

#[cfg(test)]
mod tests {
    use super::*;
    use admin_protocol::{Command, Envelope, Event};
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum_test::{TestServer, Transport};
    use chrono::Utc;
    use std::time::Duration;
    use tokio::sync::mpsc::Receiver;
    use tokio::sync::{Mutex, broadcast, oneshot};
    use uuid::Uuid;

    #[tokio::test]
    async fn returns_409_when_supervisor_already_connected() {
        let (event_tx, _) = broadcast::channel(1000);
        let (command_tx, _) = tokio::sync::mpsc::channel(32);
        let command_rx: Mutex<Option<Receiver<CommandMessage>>> = Mutex::new(None);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx,
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: tokio::sync::Mutex::new(None),
        });

        let app = Router::new()
            .route("/api/supervisor/ws", get(supervisor_ws))
            .with_state(state);
        let server = TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app);
        let response = server
            .get("/api/supervisor/ws")
            .add_header("Connection", "Upgrade")
            .add_header("Upgrade", "websocket")
            .add_header("Sec-Websocket-Version", "13")
            .add_header("Sec-Websocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .await;

        assert_eq!(response.status_code(), StatusCode::CONFLICT)
    }

    #[tokio::test]
    async fn returns_101_when_supervisor_connects_successfully() {
        let (event_tx, _) = broadcast::channel(1000);
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
        let command_rx: Mutex<Option<Receiver<CommandMessage>>> = Mutex::new(Some(command_rx));
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx,
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: tokio::sync::Mutex::new(None),
        });

        let app = Router::new()
            .route("/api/supervisor/ws", get(supervisor_ws))
            .with_state(state);
        let server = TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app);
        let response = server
            .get("/api/supervisor/ws")
            .add_header("Connection", "Upgrade")
            .add_header("Upgrade", "websocket")
            .add_header("Sec-Websocket-Version", "13")
            .add_header("Sec-Websocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
            .await;

        assert_eq!(response.status_code(), StatusCode::SWITCHING_PROTOCOLS);
    }

    #[tokio::test]
    async fn command_is_forwarded_to_supervisor_and_event_is_broadcast_back() {
        let (event_tx, _) = broadcast::channel(1000);
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
        let command_rx: Mutex<Option<Receiver<CommandMessage>>> = Mutex::new(Some(command_rx));
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx.clone()),
            event_tx,
            command_rx,
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: tokio::sync::Mutex::new(None),
        });

        let app = Router::new()
            .route("/api/supervisor/ws", get(supervisor_ws))
            .with_state(state);
        let server = TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app);

        // fake supervisor side
        let mut supervisor = server
            .get_websocket("/api/supervisor/ws")
            .await
            .into_websocket()
            .await;

        let ping_envelope = Envelope {
            id: Uuid::new_v4(),
            correlation_id: None,
            idempotency_key: None,
            sequence: 0,
            timestamp: Utc::now(),
            payload: Command::Ping,
        };

        // push a command from the "backend handler" side
        let (response_tx, response_rx) = oneshot::channel();
        command_tx
            .send(CommandMessage {
                envelope: ping_envelope,
                response_tx,
            })
            .await
            .unwrap();

        // supervisor receives + check payload only
        let received: Envelope<Command> = supervisor.receive_json::<Envelope<Command>>().await;
        assert!(matches!(received.payload, Command::Ping));

        // supervisor sends the pong back
        let pong_envelope = Envelope {
            id: Uuid::new_v4(),
            correlation_id: Some(received.id), // <-- this is what resolves the pending oneshot
            idempotency_key: None,
            sequence: 0,
            timestamp: Utc::now(),
            payload: Event::Pong,
        };
        supervisor.send_json(&pong_envelope).await;

        // backend side resolves
        let event = tokio::time::timeout(Duration::from_millis(500), response_rx).await;
        assert!(matches!(event, Ok(Ok(Event::Pong))));
    }
}
