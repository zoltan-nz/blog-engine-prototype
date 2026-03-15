use backend_rust::{ApiDoc, AppState};
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
                .unwrap_or_else(|_| "backend_rust=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let astro_management_url = std::env::var("ASTRO_MANAGEMENT_URL")
        .unwrap_or_else(|_| "http://localhost:4320".to_string());

    let state = Arc::new(AppState::new(astro_management_url));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .routes(routes!(backend_rust::healthz))
        .routes(routes!(backend_rust::list_sites, backend_rust::create_site))
        .routes(routes!(backend_rust::preview_site))
        .routes(routes!(backend_rust::stop_preview))
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
