use crate::Meta;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;

/// A single Astro site managed by the CMS.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SiteData {
    pub slug: String,
    pub name: String,
    pub git_url: String,
    pub preview_url: Option<String>,
}

/// Request body for creating a new site.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateSiteRequest {
    pub name: String,
    pub slug: String,
}

/// Response envelope for a single site.
#[derive(Debug, Serialize, ToSchema)]
pub struct SiteResponse {
    data: SiteData,
    meta: Meta,
}

impl SiteResponse {
    fn new(data: SiteData) -> Self {
        Self {
            data,
            meta: Meta::default(),
        }
    }
}

/// Response envelope for a list of sites.
#[derive(Debug, Serialize, ToSchema)]
pub struct SiteListResponse {
    data: Vec<SiteData>,
    meta: Meta,
}

impl SiteListResponse {
    fn new(data: Vec<SiteData>) -> Self {
        Self {
            data,
            meta: Meta::default(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/sites",
    responses(
        (status = 200, description = "List of all sites", body = SiteListResponse)
    )
)]
pub async fn list_sites(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("List sites requested");
    StatusCode::NOT_IMPLEMENTED
}

#[utoipa::path(
    post,
    path = "/sites",
    request_body = CreateSiteRequest,
    responses(
        (status = 201, description = "Site created", body = SiteResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Management API error"),
    )
)]
pub async fn create_site(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateSiteRequest>,
) -> impl IntoResponse {
    info!(slug = %req.slug, "Create site requested");
    StatusCode::NOT_IMPLEMENTED
}

#[utoipa::path(
    post,
    path = "/sites/{slug}/preview",
    params(
        ("slug" = String, Path, description = "Site slug")
    ),
    responses(
        (status = 200, description = "Dev server started", body = SiteResponse),
        (status = 404, description = "Site not found"),
        (status = 500, description = "Dev server failed to start"),
    )
)]
pub async fn preview_site(
    Path(slug): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!(slug = %slug, "Preview site requested");
    StatusCode::NOT_IMPLEMENTED
}

#[utoipa::path(
    delete,
    path = "/preview",
    responses(
        (status = 204, description = "Preview stopped (or was not running)"),
    )
)]
pub async fn stop_preview(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Stop preview requested");
    StatusCode::NOT_IMPLEMENTED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_data_serializes_git_url_as_camel_case() {
        let site = SiteData {
            slug: "my-blog".into(),
            name: "My Blog".into(),
            git_url: "/app/git-repos/my-blog.git".into(),
            preview_url: None,
        };
        let json = serde_json::to_value(&site).unwrap();
        assert_eq!(json["gitUrl"], "/app/git-repos/my-blog.git");
        assert_eq!(json["slug"], "my-blog");
        assert_eq!(json["name"], "My Blog");
        assert!(
            json.get("git_url").is_none(),
            "snake_case key must not appear"
        );
    }

    #[test]
    fn create_site_request_deserializes() {
        let raw = r#"{"name": "My Blog", "slug": "my-blog"}"#;
        let req: CreateSiteRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.name, "My Blog");
        assert_eq!(req.slug, "my-blog");
    }

    #[test]
    fn site_list_response_wraps_vec_with_meta() {
        let sites = vec![SiteData {
            slug: "test".into(),
            name: "Test".into(),
            git_url: "/repos/test.git".into(),
            preview_url: None,
        }];
        let resp = SiteListResponse::new(sites);
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["data"].is_array());
        assert_eq!(json["data"].as_array().unwrap().len(), 1);
        assert!(json["meta"].is_object());
    }

    #[test]
    fn site_data_serializes_preview_url_as_null_when_none() {
        let site = SiteData {
            slug: "s".into(),
            name: "S".into(),
            git_url: "g".into(),
            preview_url: None,
        };
        let json = serde_json::to_value(&site).unwrap();
        assert_eq!(json["previewUrl"], serde_json::Value::Null);
        assert!(
            json.get("preview_url").is_none(),
            "snake_case must not appear"
        );
    }

    #[test]
    fn site_data_serializes_preview_url_when_some() {
        let site = SiteData {
            slug: "s".into(),
            name: "S".into(),
            git_url: "g".into(),
            preview_url: Some("http://localhost:4321".into()),
        };
        let json = serde_json::to_value(&site).unwrap();
        assert_eq!(json["previewUrl"], "http://localhost:4321");
    }
}
