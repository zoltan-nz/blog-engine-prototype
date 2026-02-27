use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use tracing::info;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
enum MetaServerName {
    // For the Open API specification
    #[allow(dead_code)]
    BackendNode,
    BackendRust,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct Meta {
    timestamp: String,
    request_id: String,
    version: &'static str,
    server_name: MetaServerName,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            request_id: Uuid::new_v4().to_string(),
            version: env!("CARGO_PKG_VERSION"),
            server_name: MetaServerName::BackendRust,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum HealthStatus {
    #[default]
    Healthy,
    // Degraded,
    // Unhealthy,
}

#[derive(Debug, Serialize, ToSchema)]
struct Envelop<T: ToSchema> {
    data: T,
    meta: Meta,
}

impl<T: ToSchema> Envelop<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            meta: Meta::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, ToSchema)]
struct HealthData {
    status: HealthStatus,
    version: &'static str,
}

type HealthResponse = Envelop<HealthData>;

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn healthz() -> impl IntoResponse {
    info!("Health check requested");
    let response = HealthResponse::new(HealthData {
        status: HealthStatus::Healthy,
        version: env!("CARGO_PKG_VERSION"),
    });
    (StatusCode::OK, Json(response))
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Blog Engine API", version = env!("CARGO_PKG_VERSION")),
    components(schemas(HealthResponse, HealthData, HealthStatus, Meta, MetaServerName))
)]
pub struct ApiDoc;
