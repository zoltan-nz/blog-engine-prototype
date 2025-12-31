use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::get};
use serde::Serialize;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    timestamp: String,
    request_id: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            request_id: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum HealthStatus {
    #[default]
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize)]
struct Envelop<T> {
    data: T,
    meta: Meta,
}

impl<T> Envelop<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            meta: Meta::default(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct HealthData {
    status: HealthStatus,
    version: &'static str,
}

type HealthResponse = Envelop<HealthData>;

async fn healthz() -> impl IntoResponse {
    info!("Health check requested");
    let response = HealthResponse::new(HealthData {
        status: HealthStatus::Healthy,
        version: env!("CARGO_PKG_VERSION"),
    });
    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend_rust=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/healthz", get(healthz));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    info!(%addr, "Server listening");


    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Server error");
}
