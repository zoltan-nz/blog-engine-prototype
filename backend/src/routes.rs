use crate::handlers::healthz::healthz;
use crate::handlers::site_files::{SITE_FILES_PREFIX, site_file};
use crate::state::AppState;
use crate::ws::socket::upgrade_ws;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(upgrade_ws))
        .route(
            &format!("{SITE_FILES_PREFIX}/{{slug}}/{{*path}}"),
            get(site_file),
        )
}
