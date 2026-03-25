use crate::Meta;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use tracing::trace;
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    #[default]
    Healthy,
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct HealthData {
    status: HealthStatus,
    version: &'static str,
}

/// Response envelope for the health endpoint.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    data: HealthData,
    meta: Meta,
}

impl HealthResponse {
    fn new(data: HealthData) -> Self {
        Self {
            data,
            meta: Meta::default(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn healthz() -> impl IntoResponse {
    trace!("Health check requested");
    let response = HealthResponse::new(HealthData {
        status: HealthStatus::Healthy,
        version: env!("CARGO_PKG_VERSION"),
    });
    (StatusCode::OK, Json(response))
}
