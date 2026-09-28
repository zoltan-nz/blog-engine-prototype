//! Read-only access to a site's `src/` folder, so the admin editor can show
//! images that posts reference by relative path.
use axum_test::TestServer;
use backend::app::create_app;
use backend::astro::sites as astro_sites;
use backend::config::Config;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const SLUG: &str = "my-blog";
const IMAGE_BYTES: &[u8] = b"fake-jpeg";

/// A registered site with one image under `src/assets/` and a file outside
/// `src/` that must never be served.
fn server_with_site() -> (TestServer, TempDir) {
    let tmp = TempDir::new().unwrap();
    astro_sites::create_site(tmp.path(), "My Blog", SLUG).unwrap();

    let site_dir = tmp.path().join(SLUG);
    fs::create_dir_all(site_dir.join("src/assets")).unwrap();
    fs::write(site_dir.join("src/assets/photo.jpg"), IMAGE_BYTES).unwrap();
    fs::write(site_dir.join("package.json"), "{}").unwrap();

    (TestServer::new(app(tmp.path())), tmp)
}

fn app(sites_dir: &Path) -> axum::Router {
    let config = Config {
        sites_dir: sites_dir.to_path_buf(),
        preview_port: 4321,
        frontend_dir: std::path::PathBuf::from("/nonexistent"),
    };
    create_app(config).0
}

#[tokio::test]
async fn site_files_serves_file_under_src() {
    let (server, _tmp) = server_with_site();

    let response = server.get("/site-files/my-blog/assets/photo.jpg").await;

    response.assert_status_ok();
    assert_eq!(response.as_bytes().as_ref(), IMAGE_BYTES);
}

#[tokio::test]
async fn site_files_returns_not_found_for_unknown_site() {
    let (server, _tmp) = server_with_site();

    let response = server.get("/site-files/other-blog/assets/photo.jpg").await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn site_files_rejects_path_outside_src() {
    let (server, _tmp) = server_with_site();

    let response = server.get("/site-files/my-blog/%2e%2e/package.json").await;

    response.assert_status_not_found();
}
