use crate::astro;
use crate::config::Config;
use crate::routes::create_routes;
use crate::state::AppState;
use crate::types;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// Builds the Axum router and shared `AppState`: hydrates sites from disk and
/// starts a filesystem watcher per pre-existing site.
///
/// # Panics
///
/// If the `watchers` mutex is poisoned while starting watchers — only possible
/// if another thread panicked holding it, which cannot happen during
/// single-threaded construction.
pub fn create_app(config: Config) -> (Router, Arc<AppState>) {
    info!("Initializing application.");

    let initial_sites = hydrate_sites(&config.sites_dir);

    // Collect slugs before initial_sites is consumed by AppState::new.
    let site_slugs: Vec<String> = initial_sites
        .iter()
        .map(|(slug, _, _)| slug.clone())
        .collect();

    let state = Arc::new(AppState::new(
        config.sites_dir,
        config.preview_port,
        initial_sites,
    ));

    // Start per-site watchers for pre-existing sites.
    {
        let mut watchers = state.watchers.lock().expect("watchers mutex");
        for slug in site_slugs {
            let site_dir = state.sites_dir.join(&slug);
            match astro::watch::start_site_watcher(site_dir, slug.clone(), Arc::clone(&state)) {
                Ok(watcher) => {
                    watchers.insert(slug.clone(), watcher);
                }
                Err(error) => warn!(%error, %slug, "could not start watcher for site"),
            }
        }
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(create_routes())
        .fallback_service(spa_service(&config.frontend_dir))
        .layer(cors)
        .with_state(Arc::clone(&state));

    (app, state)
}

/// Load `(slug, name, posts)` triples from the manifest so pre-existing
/// sites enter the FSM map as `Ready` (with their post cache warm) at
/// startup. Best-effort: a broken manifest, or a site whose posts fail to
/// list, logs a warning and contributes an empty list rather than refusing
/// to boot.
fn hydrate_sites(sites_dir: &std::path::Path) -> Vec<(String, String, Vec<types::PostMeta>)> {
    if let Err(error) = std::fs::create_dir_all(sites_dir) {
        warn!(%error, dir = %sites_dir.display(), "could not create sites dir");
        return Vec::new();
    }
    match astro::sites::list_sites(sites_dir) {
        Ok(sites) => sites
            .into_iter()
            .map(|s| {
                let posts =
                    astro::posts::list_posts(&sites_dir.join(&s.folder)).unwrap_or_else(|error| {
                        warn!(%error, slug = %s.folder, "could not list posts for site");
                        Vec::new()
                    });
                (s.folder, s.name, posts)
            })
            .collect(),
        Err(error) => {
            warn!(%error, "could not hydrate sites from manifest");
            Vec::new()
        }
    }
}

/// Dev/test builds serve the SPA from disk; `--features embed` builds serve
/// the frontend compiled into the binary (single-binary deployment).
#[cfg(not(feature = "embed"))]
fn spa_service(
    frontend_dir: &std::path::Path,
) -> tower_http::services::ServeDir<tower_http::services::ServeFile> {
    use tower_http::services::{ServeDir, ServeFile};
    ServeDir::new(frontend_dir).fallback(ServeFile::new(frontend_dir.join("index.html")))
}

#[cfg(feature = "embed")]
fn spa_service(_frontend_dir: &std::path::Path) -> axum::routing::MethodRouter {
    axum::routing::get(embedded::serve_embedded)
}

#[cfg(feature = "embed")]
mod embedded {
    use axum::http::{StatusCode, Uri, header};
    use axum::response::{IntoResponse, Response};
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "../frontend/build/"]
    struct Assets;

    /// SPA semantics: serve the asset if it exists, otherwise fall back to
    /// index.html so client-side routes deep-link correctly.
    pub async fn serve_embedded(uri: Uri) -> Response {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };

        let file = Assets::get(path).or_else(|| Assets::get("index.html"));
        match file {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "frontend not embedded").into_response(),
        }
    }
}
