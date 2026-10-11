//! `site-files://` custom protocol: post-editor images without the HTTP
//! server. Requests are forwarded to the backend router's `/site-files`
//! route, so the site lookup and `ServeDir` path guard stay in one place.

use axum::Router;
use axum::body::Body;
use backend::handlers::site_files::SITE_FILES_PREFIX;
use tauri::http::{Request, Response, StatusCode};
use tower::ServiceExt;

/// Scheme the webview loads; the frontend builds URLs with `convertFileSrc`.
pub const SCHEME: &str = "site-files";

/// Rewrites `site-files://localhost/{slug}/{path}` (`http://site-files.localhost/…`
/// on Windows) to `/site-files/{slug}/{path}` and runs it through `router`.
/// The path stays percent-encoded; the backend route decodes it.
pub async fn serve(router: Router, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let (mut parts, body) = request.into_parts();
    let Ok(uri) = format!("{SITE_FILES_PREFIX}{}", parts.uri.path()).parse() else {
        return status(StatusCode::BAD_REQUEST);
    };
    let request_uri = std::mem::replace(&mut parts.uri, uri);

    let response = match router
        .oneshot(Request::from_parts(parts, Body::from(body)))
        .await
    {
        Ok(response) => response,
        Err(infallible) => match infallible {},
    };

    let (parts, body) = response.into_parts();
    tracing::debug!(uri = %request_uri, status = %parts.status, "site file served");
    match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => Response::from_parts(parts, bytes.to_vec()),
        Err(error) => {
            tracing::warn!(%error, "site file body failed");
            status(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn status(code: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = code;
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;
    use axum::routing::get;

    // Stand-in for the backend route: echoes the path it was asked for.
    fn echo_router() -> Router {
        Router::new().route(
            "/site-files/{slug}/{*path}",
            get(|uri: Uri| async move { uri.path().to_owned() }),
        )
    }

    #[tokio::test]
    async fn forwards_to_the_site_files_route() {
        let request = Request::get("site-files://localhost/my-blog/assets/a%20b.png")
            .body(Vec::new())
            .unwrap();

        let response = serve(echo_router(), request).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body(), b"/site-files/my-blog/assets/a%20b.png");
    }

    #[tokio::test]
    async fn passes_through_not_found() {
        let request = Request::get("site-files://localhost/")
            .body(Vec::new())
            .unwrap();

        let response = serve(echo_router(), request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
