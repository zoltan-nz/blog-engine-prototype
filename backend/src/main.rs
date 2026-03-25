use backend::handlers::apidoc::ApiDoc;
use backend::handlers::healthz::{__path_healthz, healthz};
use backend::handlers::sites::{
    __path_create_site, __path_list_sites, __path_preview_site, __path_stop_preview,
    create_site, list_sites, preview_site, stop_preview,
};
use backend::state::AppState;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(32);
    // tokio.spawn(command_processor(command_rx))
    let state = Arc::new(AppState { command_tx });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .routes(routes!(healthz))
        .routes(routes!(list_sites, create_site))
        .routes(routes!(preview_site))
        .routes(routes!(stop_preview))
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    info!(%addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Server error");
}
