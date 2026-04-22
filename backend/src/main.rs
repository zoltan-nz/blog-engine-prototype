use axum::routing::get;
use backend::handlers::apidoc::ApiDoc;
use backend::handlers::commands::{__path_dispatch_command, dispatch_command};
use backend::handlers::healthz::{__path_healthz, healthz};
use backend::handlers::sites::{
    __path_create_site, __path_list_sites, __path_preview_site, __path_stop_preview, create_site,
    list_sites, preview_site, stop_preview,
};
use backend::handlers::supervisor::supervisor_ws;
use backend::state::AppState;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_sites_dir")]
    sites_dir: PathBuf,
}

fn default_sites_dir() -> PathBuf {
    PathBuf::from("/tmp/astro-sites")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = envy::from_env::<Config>().expect("Failed to load configuration");

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

    let (router, api) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .routes(routes!(healthz))
        .routes(routes!(list_sites, create_site))
        .routes(routes!(preview_site))
        .routes(routes!(stop_preview))
        .routes(routes!(dispatch_command))
        .split_for_parts();

    let app = router
        .route("/api/supervisor/ws", get(supervisor_ws))
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
