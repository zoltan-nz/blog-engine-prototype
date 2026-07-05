use axum_test::TestServer;
use backend::app::create_app;
use backend::config::Config;
use tempfile::TempDir;

#[tokio::test]
async fn healthz_endpoint_return_healthy_status() {
    let sites = TempDir::new().unwrap();
    let config = Config {
        sites_dir: sites.path().to_path_buf(),
        preview_port: 4321,
        frontend_dir: std::path::PathBuf::from("/tmp"),
    };

    let (app, state) = create_app(config);
    let server = TestServer::new(app);

    let response = server.get("/healthz").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["status"], "healthy");
    assert!(body["data"]["version"].is_string());
    assert!(body["meta"]["timestamp"].is_string());
    assert!(body["meta"]["requestId"].is_string());
    assert_eq!(body["meta"]["serverName"], "backend");

    assert!(state.lock_preview().await.is_none())
}
