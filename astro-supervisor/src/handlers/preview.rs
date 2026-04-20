use std::sync::Arc;
use std::time::Duration;

use crate::error::AgentError;
use crate::state::{ActivePreview, AppState};

/// Spawns `pnpm dev --port <port>` in the site's directory, then polls TCP
/// until the port accepts connections (max 30 s). Stores the child process in
/// `state.preview`. Returns the preview URL on success.
///
/// Errors:
/// - `PreviewAlreadyRunning` — another site is already being previewed
/// - `SiteNotFound` — `sites_dir/<slug>` does not exist
/// - `DevServerTimeout` — port never became reachable within the deadline
/// - `Io` — process spawn failed
pub async fn start_preview(
    state: &Arc<AppState>,
    slug: &str,
    port: u16,
) -> Result<String, AgentError> {
    let mut guard = state.lock_preview().await;

    if guard.is_some() {
        return Err(AgentError::PreviewAlreadyRunning(slug.to_string()));
    }

    let site_dir = state.sites_dir.join(slug);
    if !site_dir.exists() {
        return Err(AgentError::SiteNotFound(slug.to_string()));
    }

    let child = tokio::process::Command::new("pnpm")
        .args(["dev", "--port", &port.to_string(), "--host", "0.0.0.0"])
        .current_dir(&site_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await
    .map_err(|_| AgentError::DevServerTimeout(slug.to_string()))?;

    let url = format!("http://localhost:{port}");
    tracing::info!(slug = %slug, port = port, url = %url, "preview started");
    *guard = Some(ActivePreview {
        slug: slug.to_string(),
        child,
    });

    Ok(url)
}

/// Kills the running preview process if one exists. Idempotent — returns
/// `Ok(())` even when nothing is running.
pub async fn stop_preview(state: &Arc<AppState>) -> Result<(), AgentError> {
    let mut guard = state.lock_preview().await;
    if let Some(mut preview) = guard.take() {
        tracing::info!(slug = %preview.slug, "stopping preview");
        preview.child.kill().await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_state(sites_dir: &std::path::Path) -> Arc<AppState> {
        Arc::new(AppState::new(sites_dir, "/tmp/git-repos", 4321))
    }

    #[tokio::test]
    async fn start_preview_returns_site_not_found_when_dir_missing() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());

        let err = start_preview(&state, "nonexistent", 4321).await.unwrap_err();
        assert!(matches!(err, AgentError::SiteNotFound(_)));
    }

    #[tokio::test]
    async fn start_preview_returns_already_running_when_preview_active() {
        let tmp = TempDir::new().unwrap();
        // Create a placeholder site dir so it passes the existence check
        std::fs::create_dir(tmp.path().join("my-site")).unwrap();
        let state = make_state(tmp.path());

        // Manually put something in the preview mutex to simulate a running preview
        let fake_child = tokio::process::Command::new("sleep")
            .arg("100")
            .spawn()
            .unwrap();
        *state.lock_preview().await = Some(ActivePreview {
            slug: "other-site".to_string(),
            child: fake_child,
        });

        let err = start_preview(&state, "my-site", 4321).await.unwrap_err();
        assert!(matches!(err, AgentError::PreviewAlreadyRunning(_)));

        // Cleanup — kill the sleep process
        if let Some(mut p) = state.lock_preview().await.take() {
            p.child.kill().await.ok();
        }
    }

    #[tokio::test]
    async fn stop_preview_is_idempotent_when_nothing_running() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        // Should not error even though no preview is active
        stop_preview(&state).await.unwrap();
    }
}
