use crate::Meta;
use crate::astro::error::AstroError;
use crate::astro::sites as astro_sites;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// A single Astro site managed by the CMS (API response shape).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteData {
    pub slug: String,
    pub name: String,
    pub git_url: String,
    pub preview_url: Option<String>,
}

/// Request body for creating a new site.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSiteRequest {
    pub name: String,
    pub slug: String,
}

/// Response envelope for a single site.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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

/// Preview URL for an active dev server.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewData {
    pub preview_url: String,
}

/// Response envelope for a preview start.
#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    data: PreviewData,
    meta: Meta,
}

pub async fn list_sites(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("List sites requested");

    let sites = match astro_sites::list_sites(&state.sites_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list sites");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let active = state.lock_preview().await;
    let data = sites
        .into_iter()
        .map(|s| {
            let preview_url = active
                .as_ref()
                .filter(|p| p.slug == s.folder)
                .map(|p| p.url.clone());
            SiteData {
                slug: s.folder,
                name: s.name,
                git_url: s.git_url,
                preview_url,
            }
        })
        .collect();

    Json(SiteListResponse::new(data)).into_response()
}

pub async fn create_site(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSiteRequest>,
) -> impl IntoResponse {
    info!(slug = %req.slug, "Create site requested");

    match astro_sites::create_site(&state.sites_dir, &req.name, &req.slug) {
        Ok(_) => {}
        Err(AstroError::SiteAlreadyExists(_)) => {
            return (
                StatusCode::CONFLICT,
                format!("site '{}' already exists", req.slug),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(slug = %req.slug, error = %e, "create site failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let site_dir = state.sites_dir.join(&req.slug);
    if let Err(e) = astro_sites::scaffold_site(&site_dir).await {
        tracing::error!(slug = %req.slug, error = %e, "Astro scaffold failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("scaffold failed: {e}"),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(SiteResponse::new(SiteData {
            slug: req.slug,
            name: req.name,
            git_url: String::new(),
            preview_url: None,
        })),
    )
        .into_response()
}

pub async fn delete_site(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!(slug = %slug, "Delete site requested");

    // Stop the preview first if it belongs to this site.
    let active_slug = state.lock_preview().await.as_ref().map(|p| p.slug.clone());
    if active_slug.as_deref() == Some(slug.as_str())
        && let Err(e) = crate::astro::preview::stop_preview(&state).await
    {
        tracing::warn!(slug = %slug, error = %e, "could not stop preview before delete");
    }

    match astro_sites::delete_site(&state.sites_dir, &slug) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(AstroError::SiteNotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(slug = %slug, error = %e, "delete site failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn preview_site(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!(slug = %slug, "Preview site requested");

    let port = state.preview_port;
    match crate::astro::preview::start_preview(&state, &slug, port).await {
        Ok(url) => Json(PreviewResponse {
            data: PreviewData { preview_url: url },
            meta: Meta::default(),
        })
        .into_response(),
        Err(AstroError::SiteNotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(AstroError::PreviewAlreadyRunning(_)) => StatusCode::CONFLICT.into_response(),
        Err(AstroError::DevServerTimeout(_)) => StatusCode::GATEWAY_TIMEOUT.into_response(),
        Err(e) => {
            tracing::error!(slug = %slug, error = %e, "start preview failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn stop_preview(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Stop preview requested");

    match crate::astro::preview::stop_preview(&state).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "stop preview failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
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

    // --- handler tests (in-process, tempdir-backed) ---
    //
    // The happy paths for create/preview spawn `create-astro`/`pnpm dev` and are
    // covered by the Playwright integration suite, not here. These unit tests
    // exercise the error-mapping branches that return before any subprocess runs.

    use axum::Router;
    use axum::routing::{delete, get, post};
    use axum_test::{TestServer, Transport};
    use tempfile::TempDir;

    fn build_server(state: Arc<AppState>) -> TestServer {
        let app = Router::new()
            .route("/sites", get(list_sites).post(create_site))
            .route("/sites/{slug}", delete(delete_site))
            .route("/sites/{slug}/preview", post(preview_site))
            .route("/preview", delete(stop_preview))
            .with_state(state);
        TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app)
    }

    #[tokio::test]
    async fn list_sites_returns_empty_for_fresh_dir() {
        let tmp = TempDir::new().unwrap();
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server.get("/sites").await;
        assert_eq!(resp.status_code(), StatusCode::OK);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["data"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn create_site_returns_409_when_site_already_exists() {
        let tmp = TempDir::new().unwrap();
        astro_sites::create_site(tmp.path(), "My Blog", "my-blog").unwrap();
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server
            .post("/sites")
            .json(&serde_json::json!({ "name": "My Blog", "slug": "my-blog" }))
            .await;

        assert_eq!(resp.status_code(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn preview_site_returns_404_when_site_not_found() {
        let tmp = TempDir::new().unwrap();
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server.post("/sites/no-site/preview").await;
        assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_site_returns_404_when_not_found() {
        let tmp = TempDir::new().unwrap();
        astro_sites::list_sites(tmp.path()).unwrap(); // init manifest
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server.delete("/sites/ghost").await;
        assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_site_returns_204_when_present() {
        let tmp = TempDir::new().unwrap();
        astro_sites::create_site(tmp.path(), "X", "x").unwrap();
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server.delete("/sites/x").await;
        assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn stop_preview_returns_204_when_nothing_running() {
        let tmp = TempDir::new().unwrap();
        let state = Arc::new(AppState::new(tmp.path(), 4321));
        let server = build_server(state);

        let resp = server.delete("/preview").await;
        assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
    }
}
