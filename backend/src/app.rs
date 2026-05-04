use std::sync::Arc;
use axum::Router;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use crate::config::Config;
use crate::routes::{create_openapi_routes, create_supervisor_ws_routes};
use crate::state::AppState;

pub fn create_app(config: Config) -> (Router, Arc<AppState>) {
    info!("Initializing application.");
    let (event_tx, _) = broadcast::channel(1000);
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
    let command_rx = Mutex::new(Some(command_rx));

    let state = Arc::new(AppState {
        command_tx: Mutex::new(command_tx),
        event_tx,
        command_rx,
        sites_dir: config.sites_dir,
        active_preview: Mutex::new(None),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(create_supervisor_ws_routes())
        .merge(create_openapi_routes())
        .layer(cors)
    .with_state(Arc::clone(&state));

    (app, state)
}