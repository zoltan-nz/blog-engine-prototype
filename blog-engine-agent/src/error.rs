use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
}

impl ProblemDetails {
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank".to_string(),
            title: "Not Found".to_string(),
            status: 404,
            detail: detail.into(),
        }
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank".to_string(),
            title: "Conflict".to_string(),
            status: 409,
            detail: detail.into(),
        }
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank".to_string(),
            title: "Internal Server Error".to_string(),
            status: 500,
            detail: detail.into(),
        }
    }

    pub fn dev_server_failed(detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank".to_string(),
            title: "Astro Dev Server Failed".to_string(),
            status: 500,
            detail: detail.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Site '{0}' does not exist")]
    SiteNotFound(String),

    #[error("Site '{0}' already exists")]
    SiteAlreadyExists(String),

    #[error("Dev server timed out: {0}")]
    DevServerTimeout(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for AgentError {
    fn into_response(self) -> Response {
        let detail = self.to_string();
        let problem = match self {
            AgentError::SiteNotFound(_) => ProblemDetails::not_found(detail),
            AgentError::SiteAlreadyExists(_) => ProblemDetails::conflict(detail),
            AgentError::DevServerTimeout(_) => ProblemDetails::dev_server_failed(detail),
            AgentError::CommandFailed(_) | AgentError::Io(_) | AgentError::Json(_) => {
                ProblemDetails::internal(detail)
            }
        };

        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(problem)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_details_not_found_serializes_correctly() {
        let not_found_problem = ProblemDetails::not_found("Site 'foo' does not exist");
        let json = serde_json::to_value(&not_found_problem).unwrap();

        assert_eq!(json["type"], "about:blank");
        assert_eq!(json["title"], "Not Found");
        assert_eq!(json["status"], 404);
        assert_eq!(json["detail"], "Site 'foo' does not exist");
    }

    #[tokio::test]
    async fn site_not_found_returns_404_problem_details() {
        let response = AgentError::SiteNotFound("foo".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["type"], "about:blank");
        assert_eq!(json["title"], "Not Found");
        assert_eq!(json["status"], 404);
        assert_eq!(json["detail"], "Site 'foo' does not exist");
    }

    #[tokio::test]
    async fn site_already_exists_returns_409_problem_details() {
        let response = AgentError::SiteAlreadyExists("foo".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["type"], "about:blank");
        assert_eq!(json["title"], "Conflict");
        assert_eq!(json["status"], 409);
        assert_eq!(json["detail"], "Site 'foo' already exists");
    }

    #[tokio::test]
    async fn dev_server_timeout_returns_500_problem_details() {
        let response = AgentError::DevServerTimeout("foo".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["type"], "about:blank");
        assert_eq!(json["title"], "Astro Dev Server Failed");
        assert_eq!(json["status"], 500);
        assert_eq!(json["detail"], "Dev server timed out: foo");
    }

    #[tokio::test]
    async fn command_failed_returns_500_problem_details() {
        let response = AgentError::CommandFailed("foo".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["type"], "about:blank");
        assert_eq!(json["title"], "Internal Server Error");
        assert_eq!(json["status"], 500);
        assert_eq!(json["detail"], "Command failed: foo");
    }
}
