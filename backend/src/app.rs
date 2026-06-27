use crate::config::Config;
use crate::routes::create_routes;
use crate::state::AppState;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub fn create_app(config: Config) -> (Router, Arc<AppState>) {
    info!("Initializing application.");

    let state = Arc::new(AppState::new(config.sites_dir, config.preview_port));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(create_routes())
        .layer(cors)
        .with_state(Arc::clone(&state));

    (app, state)
}
