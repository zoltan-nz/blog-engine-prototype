use crate::handlers::healthz::healthz;
use crate::state::AppState;
use crate::ws::socket::upgrade_ws;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(upgrade_ws))
}
