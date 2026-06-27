use crate::handlers::healthz::healthz;
use crate::handlers::sites::{create_site, delete_site, list_sites, preview_site, stop_preview};
use crate::state::AppState;
use axum::Router;
use axum::routing::{delete, get, post};
use std::sync::Arc;

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/sites", get(list_sites).post(create_site))
        .route("/sites/{slug}", delete(delete_site))
        .route("/sites/{slug}/preview", post(preview_site))
        .route("/preview", delete(stop_preview))
}
