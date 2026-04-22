use crate::Meta;
use crate::state::{AppState, CommandMessage};
use admin_protocol::{Command, ErrorCode, Event};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

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

/// Preview URL for an active dev server.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PreviewData {
    pub preview_url: String,
}

/// Response envelope for a preview start.
#[derive(Debug, Serialize, ToSchema)]
pub struct PreviewResponse {
    data: PreviewData,
    meta: Meta,
}

#[derive(Deserialize)]
struct ManifestEntry {
    folder: String,
    name: String,
    git_url: String,
}

#[derive(Deserialize)]
struct SitesManifest {
    sites: Vec<ManifestEntry>,
}

#[utoipa::path(
    get,
    path = "/sites",
    responses(
        (status = 200, description = "List of all sites", body = SiteListResponse)
    )
)]
pub async fn list_sites(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("List sites requested");

    let manifest_path = state.sites_dir.join("sites.json");
    if !manifest_path.exists() {
        return Json(SiteListResponse::new(vec![])).into_response();
    }

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to read sites manifest");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let manifest: SitesManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse sites manifest");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let active = state.active_preview.lock().await.clone();
    let sites = manifest
        .sites
        .into_iter()
        .map(|e| {
            let preview_url = active
                .as_ref()
                .filter(|(slug, _)| slug == &e.folder)
                .map(|(_, url)| url.clone());
            SiteData {
                slug: e.folder,
                name: e.name,
                git_url: e.git_url,
                preview_url,
            }
        })
        .collect();

    Json(SiteListResponse::new(sites)).into_response()
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSiteRequest>,
) -> impl IntoResponse {
    info!(slug = %req.slug, "Create site requested");

    let (response_tx, response_rx) = oneshot::channel::<Event>();
    let envelope = admin_protocol::Envelope {
        id: Uuid::new_v4(),
        correlation_id: None,
        idempotency_key: None,
        sequence: 0,
        timestamp: chrono::Utc::now(),
        payload: Command::CreateSite { name: req.name.clone(), slug: req.slug.clone() },
    };

    if state.command_tx.lock().await.send(CommandMessage { envelope, response_tx }).await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    match tokio::time::timeout(Duration::from_secs(110), response_rx).await {
        Ok(Ok(Event::SiteCreated { name, .. })) => (
            StatusCode::CREATED,
            Json(SiteResponse::new(SiteData {
                slug: req.slug,
                name,
                git_url: "".into(),
                preview_url: None,
            })),
        ).into_response(),
        Ok(Ok(Event::Error { code: ErrorCode::Conflict, message, .. })) => {
            (StatusCode::CONFLICT, message).into_response()
        }
        Ok(Ok(_)) | Ok(Err(_)) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        Err(_) => StatusCode::GATEWAY_TIMEOUT.into_response(),
    }
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
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!(slug = %slug, "Preview site requested");

    let (response_tx, response_rx) = oneshot::channel::<Event>();
    let envelope = admin_protocol::Envelope {
        id: Uuid::new_v4(),
        correlation_id: None,
        idempotency_key: None,
        sequence: 0,
        timestamp: chrono::Utc::now(),
        payload: Command::StartPreview { slug: slug.clone(), port: None },
    };

    if state
        .command_tx
        .lock().await
        .send(CommandMessage { envelope, response_tx })
        .await
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    match tokio::time::timeout(Duration::from_secs(60), response_rx).await {
        Ok(Ok(Event::PreviewReady { url, .. })) => {
            *state.active_preview.lock().await = Some((slug, url.clone()));
            Json(PreviewResponse {
                data: PreviewData { preview_url: url },
                meta: Meta::default(),
            })
            .into_response()
        }
        Ok(Ok(Event::Error { code: ErrorCode::SiteNotFound, .. })) => {
            StatusCode::NOT_FOUND.into_response()
        }
        Ok(Ok(Event::Error { code: ErrorCode::Conflict, .. })) => {
            StatusCode::CONFLICT.into_response()
        }
        Ok(Ok(Event::Error { code: ErrorCode::PreviewTimeout, .. })) => {
            StatusCode::GATEWAY_TIMEOUT.into_response()
        }
        Ok(Ok(_)) | Ok(Err(_)) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        Err(_) => StatusCode::GATEWAY_TIMEOUT.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/preview",
    responses(
        (status = 204, description = "Preview stopped (or was not running)"),
    )
)]
pub async fn stop_preview(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Stop preview requested");

    let (response_tx, response_rx) = oneshot::channel::<Event>();
    let envelope = admin_protocol::Envelope {
        id: Uuid::new_v4(),
        correlation_id: None,
        idempotency_key: None,
        sequence: 0,
        timestamp: chrono::Utc::now(),
        payload: Command::StopPreview,
    };

    if state
        .command_tx
        .lock().await
        .send(CommandMessage { envelope, response_tx })
        .await
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    match tokio::time::timeout(Duration::from_secs(10), response_rx).await {
        Ok(Ok(Event::PreviewStopped)) => {
            *state.active_preview.lock().await = None;
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Ok(_)) | Ok(Err(_)) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        Err(_) => StatusCode::GATEWAY_TIMEOUT.into_response(),
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

    // --- create_site handler tests ---

    use admin_protocol::Event;
    use axum::Router;
    use axum::routing::post;
    use axum_test::{TestServer, Transport};
    use tokio::sync::{Mutex, broadcast, mpsc};
    use uuid::Uuid;

    fn build_server(state: Arc<AppState>) -> TestServer {
        let app = Router::new()
            .route("/sites", post(create_site))
            .with_state(state);
        TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app)
    }

    #[tokio::test]
    async fn create_site_returns_201_with_site_data() {
        let (command_tx, mut command_rx) = mpsc::channel::<CommandMessage>(32);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_server(state);

        tokio::spawn(async move {
            let msg = command_rx.recv().await.unwrap();
            let _ = msg.response_tx.send(Event::SiteCreated {
                site_id: Uuid::new_v4(),
                name: "My Blog".into(),
            });
        });

        let resp = server
            .post("/sites")
            .json(&serde_json::json!({ "name": "My Blog", "slug": "my-blog" }))
            .await;

        assert_eq!(resp.status_code(), StatusCode::CREATED);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["data"]["slug"], "my-blog");
        assert_eq!(body["data"]["name"], "My Blog");
    }

    #[tokio::test]
    async fn create_site_returns_503_when_supervisor_not_connected() {
        // command_tx with no active receiver — send will fail immediately
        let (command_tx, command_rx) = mpsc::channel::<CommandMessage>(32);
        drop(command_rx);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_server(state);

        let resp = server
            .post("/sites")
            .json(&serde_json::json!({ "name": "My Blog", "slug": "my-blog" }))
            .await;

        assert_eq!(resp.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn create_site_returns_409_when_site_already_exists() {
        let (command_tx, mut command_rx) = mpsc::channel::<CommandMessage>(32);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_server(state);

        tokio::spawn(async move {
            let msg = command_rx.recv().await.unwrap();
            let _ = msg.response_tx.send(Event::Error {
                code: ErrorCode::Conflict,
                message: "site 'my-blog' already exists".into(),
                command_id: None,
            });
        });

        let resp = server
            .post("/sites")
            .json(&serde_json::json!({ "name": "My Blog", "slug": "my-blog" }))
            .await;

        assert_eq!(resp.status_code(), StatusCode::CONFLICT);
    }

    // --- preview_site handler tests ---

    fn build_preview_server(state: Arc<AppState>) -> TestServer {
        use axum::routing::{delete, post};
        let app = Router::new()
            .route("/sites/{slug}/preview", post(preview_site))
            .route("/preview", delete(stop_preview))
            .with_state(state);
        TestServer::builder()
            .transport(Transport::HttpRandomPort)
            .build(app)
    }

    #[tokio::test]
    async fn preview_site_returns_200_with_preview_url() {
        let (command_tx, mut command_rx) = mpsc::channel::<CommandMessage>(32);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_preview_server(state);

        tokio::spawn(async move {
            let msg = command_rx.recv().await.unwrap();
            let _ = msg.response_tx.send(Event::PreviewReady {
                slug: "my-site".into(),
                url: "http://localhost:4321".into(),
                port: 4321,
            });
        });

        let resp = server.post("/sites/my-site/preview").await;
        assert_eq!(resp.status_code(), StatusCode::OK);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["data"]["previewUrl"], "http://localhost:4321");
    }

    #[tokio::test]
    async fn preview_site_returns_404_when_site_not_found() {
        let (command_tx, mut command_rx) = mpsc::channel::<CommandMessage>(32);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_preview_server(state);

        tokio::spawn(async move {
            let msg = command_rx.recv().await.unwrap();
            let _ = msg.response_tx.send(Event::Error {
                code: ErrorCode::SiteNotFound,
                message: "site 'no-site' not found".into(),
                command_id: None,
            });
        });

        let resp = server.post("/sites/no-site/preview").await;
        assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn preview_site_returns_503_when_supervisor_disconnected() {
        let (command_tx, command_rx) = mpsc::channel::<CommandMessage>(32);
        drop(command_rx);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_preview_server(state);

        let resp = server.post("/sites/my-site/preview").await;
        assert_eq!(resp.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn stop_preview_returns_204() {
        let (command_tx, mut command_rx) = mpsc::channel::<CommandMessage>(32);
        let (event_tx, _) = broadcast::channel(1000);
        let state = Arc::new(AppState {
            command_tx: Mutex::new(command_tx),
            event_tx,
            command_rx: Mutex::new(None),
            sites_dir: std::path::PathBuf::from("/tmp"),
            active_preview: Mutex::new(None),
        });
        let server = build_preview_server(state);

        tokio::spawn(async move {
            let msg = command_rx.recv().await.unwrap();
            let _ = msg.response_tx.send(Event::PreviewStopped);
        });

        let resp = server.delete("/preview").await;
        assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
    }
}
