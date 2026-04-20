use admin_protocol::{Command, Event};
use astro_supervisor::{state::AppState as SupervisorState, ws_client::connect_and_run};
use axum::Router;
use axum::routing::{get, post};
use axum_test::{TestServer, Transport};
use backend::{
    handlers::{commands::dispatch_command, supervisor::supervisor_ws},
    state::AppState,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, broadcast};

fn build_backend_state() -> Arc<AppState> {
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
    let (event_tx, _) = broadcast::channel(1000);
    Arc::new(AppState {
        command_tx,
        event_tx,
        command_rx: Mutex::new(Some(command_rx)),
        sites_dir: std::path::PathBuf::from("/tmp"),
        active_preview: Mutex::new(None),
    })
}

fn build_backend_server(state: Arc<AppState>) -> TestServer {
    let app = Router::new()
        .route("/api/supervisor/ws", get(supervisor_ws))
        .route("/api/commands", post(dispatch_command))
        .with_state(state);
    TestServer::builder()
        .transport(Transport::HttpRandomPort)
        .build(app)
}

#[tokio::test]
async fn e2e_ping_pong() {
    let state = build_backend_state();
    let server = build_backend_server(state);

    // Build the ws:// URL the supervisor will connect to
    let mut ws_url = server.server_address().expect("server has no address");
    ws_url.set_scheme("ws").expect("url scheme change failed");
    ws_url.set_path("/api/supervisor/ws");

    // Spawn the supervisor client loop — mirrors what main() does
    let supervisor_state = Arc::new(SupervisorState::new("/tmp/sites", "/tmp/repos", 4321));
    tokio::spawn(connect_and_run(ws_url.to_string(), supervisor_state));

    // Give the WebSocket handshake time to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // POST a Ping through the backend HTTP endpoint and assert we get Pong back
    let response = server
        .post("/api/commands")
        .json(&Command::Ping)
        .await;

    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    let event: Event = response.json();
    assert!(matches!(event, Event::Pong));
}
