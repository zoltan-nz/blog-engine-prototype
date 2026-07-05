//! WS protocol integration tests. Happy paths that spawn real
//! `create-astro`/`pnpm` processes are covered by the Playwright suite; these
//! tests exercise the snapshot, FSM rejection, and error-mapping paths.
use axum_test::{TestServer, Transport};
use backend::app::create_app;
use backend::astro::sites as astro_sites;
use backend::config::Config;
use serde_json::{Value, json};
use std::path::Path;

fn ws_server(sites_dir: &Path) -> TestServer {
    let config = Config {
        sites_dir: sites_dir.to_path_buf(),
        preview_port: 4321,
        frontend_dir: std::path::PathBuf::from("/tmp"),
    };
    let (app, _state) = create_app(config);
    TestServer::builder()
        .transport(Transport::HttpRandomPort)
        .build(app)
}

fn command(correlation_id: &str, payload: Value) -> Value {
    json!({
        "unix_timestamp_us": 0,
        "correlation_id": correlation_id,
        "type": "Command",
        "payload": payload,
    })
}

#[tokio::test]
async fn connect_receives_snapshot_with_empty_state() {
    let tmp = tempfile::TempDir::new().unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let snapshot: Value = ws.receive_json().await;

    assert_eq!(snapshot["type"], "Event");
    assert_eq!(snapshot["payload"]["type"], "Snapshot");
    assert_eq!(snapshot["payload"]["payload"]["sites"], json!([]));
    assert_eq!(
        snapshot["payload"]["payload"]["preview"]["state"]["type"],
        "Stopped"
    );
}

#[tokio::test]
async fn snapshot_hydrates_existing_sites_as_ready() {
    let tmp = tempfile::TempDir::new().unwrap();
    astro_sites::create_site(tmp.path(), "My Blog", "my-blog").unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let snapshot: Value = ws.receive_json().await;

    let sites = snapshot["payload"]["payload"]["sites"].as_array().unwrap();
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0]["slug"], "my-blog");
    assert_eq!(sites[0]["name"], "My Blog");
    assert_eq!(sites[0]["state"]["type"], "Ready");
}

#[tokio::test]
async fn ping_returns_pong_with_same_correlation_id() {
    let tmp = tempfile::TempDir::new().unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command("ping-1", json!({ "type": "Ping" })))
        .await;
    let reply: Value = ws.receive_json().await;

    assert_eq!(reply["payload"]["type"], "Pong");
    assert_eq!(reply["correlation_id"], "ping-1");
}

#[tokio::test]
async fn create_site_rejects_duplicate_slug() {
    let tmp = tempfile::TempDir::new().unwrap();
    astro_sites::create_site(tmp.path(), "My Blog", "my-blog").unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command(
        "create-1",
        json!({ "type": "CreateSite", "payload": { "name": "Again", "slug": "my-blog" } }),
    ))
    .await;
    let reply: Value = ws.receive_json().await;

    assert_eq!(reply["payload"]["type"], "Error");
    assert_eq!(reply["payload"]["payload"]["code"], "SiteAlreadyExists");
    assert_eq!(reply["payload"]["payload"]["correlation_id"], "create-1");
}

#[tokio::test]
async fn start_preview_for_missing_site_returns_site_not_found() {
    let tmp = tempfile::TempDir::new().unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command(
        "preview-1",
        json!({ "type": "StartPreview", "payload": { "slug": "ghost" } }),
    ))
    .await;
    let reply: Value = ws.receive_json().await;

    assert_eq!(reply["payload"]["type"], "Error");
    assert_eq!(reply["payload"]["payload"]["code"], "SiteNotFound");
}

#[tokio::test]
async fn stop_preview_when_stopped_is_invalid_transition() {
    let tmp = tempfile::TempDir::new().unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command("stop-1", json!({ "type": "StopPreview" })))
        .await;
    let reply: Value = ws.receive_json().await;

    assert_eq!(reply["payload"]["type"], "Error");
    assert_eq!(reply["payload"]["payload"]["code"], "InvalidTransition");
    assert_eq!(reply["payload"]["payload"]["correlation_id"], "stop-1");
}

#[tokio::test]
async fn delete_missing_site_returns_site_not_found() {
    let tmp = tempfile::TempDir::new().unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command(
        "delete-1",
        json!({ "type": "DeleteSite", "payload": { "slug": "ghost" } }),
    ))
    .await;
    let reply: Value = ws.receive_json().await;

    assert_eq!(reply["payload"]["type"], "Error");
    assert_eq!(reply["payload"]["payload"]["code"], "SiteNotFound");
}

#[tokio::test]
async fn delete_site_broadcasts_deleting_then_removed() {
    let tmp = tempfile::TempDir::new().unwrap();
    astro_sites::create_site(tmp.path(), "Doomed", "doomed").unwrap();
    let server = ws_server(tmp.path());

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;
    let _snapshot: Value = ws.receive_json().await;

    ws.send_json(&command(
        "delete-2",
        json!({ "type": "DeleteSite", "payload": { "slug": "doomed" } }),
    ))
    .await;

    let changed: Value = ws.receive_json().await;
    assert_eq!(changed["payload"]["type"], "SiteChanged");
    assert_eq!(changed["payload"]["payload"]["state"]["type"], "Deleting");

    let removed: Value = ws.receive_json().await;
    assert_eq!(removed["payload"]["type"], "SiteRemoved");
    assert_eq!(removed["payload"]["payload"]["slug"], "doomed");
}
