use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeDir;

/// Route prefix; the router mounts `{SITE_FILES_PREFIX}/{slug}/{*path}`.
pub const SITE_FILES_PREFIX: &str = "/site-files";

/// Only the site's source folder is exposed: it holds the images posts
/// reference, and nothing like `node_modules` or `.env`.
const SITE_SRC_DIR: &str = "src";

/// Serves a file from a registered site's `src/` folder, read-only.
///
/// The admin editor needs this to display images a post references by a
/// path relative to its markdown file (Astro resolves those at build time,
/// so the browser cannot load them from the SPA origin). `ServeDir` rejects
/// `..` and other non-normal path components, so requests cannot leave
/// `src/`.
pub async fn site_file(
    State(state): State<Arc<AppState>>,
    Path((slug, _path)): Path<(String, String)>,
    mut request: Request,
) -> Response {
    if !state.sites.read().await.contains_key(&slug) {
        return StatusCode::NOT_FOUND.into_response();
    }

    // ServeDir resolves the request URI path against its root, so drop the
    // route prefix and keep the raw (still percent-encoded) remainder.
    let prefix = format!("{SITE_FILES_PREFIX}/{slug}");
    let Some(file_uri) = request
        .uri()
        .path()
        .strip_prefix(&prefix)
        .and_then(|rest| rest.parse::<Uri>().ok())
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    *request.uri_mut() = file_uri;

    let src_dir = state.sites_dir.join(&slug).join(SITE_SRC_DIR);
    match ServeDir::new(src_dir).oneshot(request).await {
        Ok(response) => response.map(Body::new).into_response(),
        Err(infallible) => match infallible {},
    }
}
