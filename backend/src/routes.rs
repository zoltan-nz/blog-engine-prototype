use crate::handlers::apidoc::ApiDoc;
use crate::handlers::commands::{__path_dispatch_command, dispatch_command};
use crate::handlers::healthz::__path_healthz;
use crate::handlers::healthz::healthz;
use crate::handlers::sites::{
    __path_create_site, __path_delete_site, __path_list_sites, __path_preview_site,
    __path_stop_preview, create_site, delete_site, list_sites, preview_site, stop_preview,
};
use crate::handlers::supervisor::supervisor_ws;
use crate::state::AppState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_openapi_routes() -> Router<Arc<AppState>> {
    let (router, openapi) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .routes(routes!(healthz))
        .routes(routes!(list_sites, create_site, delete_site))
        .routes(routes!(preview_site))
        .routes(routes!(stop_preview))
        .routes(routes!(dispatch_command))
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
}

pub fn create_supervisor_ws_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/supervisor/ws",
        get(supervisor_ws).connect(supervisor_ws),
    )
}
