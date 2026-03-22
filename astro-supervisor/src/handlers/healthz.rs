use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tracing::trace;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::handlers::Meta;

#[derive(Debug, Default, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum HealthStatus {
    #[default]
    Healthy,
}

#[derive(Debug, Default, Serialize, ToSchema)]
struct HealthData {
    status: HealthStatus,
    version: &'static str,
}

/// Response envelope for the health endpoint.
#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn healthz_returns_200_with_envelop() {
        let response = healthz().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["data"]["status"], "healthy");
        assert_eq!(json["meta"]["serverName"], "blog-engine-agent");
    }
}