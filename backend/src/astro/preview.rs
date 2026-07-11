use std::sync::Arc;
use std::time::Duration;

use crate::astro::error::AstroError;
use crate::state::{ActivePreview, AppState};

/// Spawns `pnpm dev --port <port>` in the site's directory and waits until the
/// port accepts connections (max 30s).
///
/// Lock discipline: the preview mutex is held only while claiming the slot and
/// spawning the child, then released before the readiness poll. Holding it across
/// the (up to 30s) poll would block every other caller of the preview mutex —
/// e.g. `list_sites`. (rust-async: never hold a lock across an await you don't
/// need to.)
///
/// # Errors
/// - `PreviewAlreadyRunning` — another site is already being previewed
/// - `SiteNotFound` — `sites_dir/<slug>` does not exist
/// - `DevServerTimeout` — port never became reachable within the deadline
/// - `Io` — process spawn failed
pub async fn start_preview(
    state: &Arc<AppState>,
    slug: &str,
    port: u16,
) -> Result<String, AstroError> {
    let url = format!("http://localhost:{port}");

    // Claim the slot + spawn under the lock, then drop it before the poll.
    {
        let mut guard = state.lock_preview().await;
        if guard.is_some() {
            return Err(AstroError::PreviewAlreadyRunning(slug.to_string()));
        }
        let site_dir = state.sites_dir.join(slug);
        if !site_dir.exists() {
            return Err(AstroError::SiteNotFound(slug.to_string()));
        }
        let child = tokio::process::Command::new("pnpm")
            .args(["dev", "--port", &port.to_string(), "--host", "0.0.0.0"])
            .env_remove("PNPM_SCRIPT_SRC_DIR")
            .current_dir(&site_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        *guard = Some(ActivePreview {
            slug: slug.to_string(),
            url: url.clone(),
            child,
        });
    }

    // Poll readiness without holding the lock.
    let ready = tokio::time::timeout(Duration::from_secs(30), async {
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
    .await;

    if ready.is_err() {
        // Roll back the slot we claimed (only if it's still ours), then kill the
        // child after releasing the lock.
        let stale = {
            let mut guard = state.lock_preview().await;
            if guard.as_ref().is_some_and(|p| p.slug == slug) {
                guard.take()
            } else {
                None
            }
        };
        if let Some(mut p) = stale {
            p.child.kill().await.ok();
        }
        return Err(AstroError::DevServerTimeout(slug.to_string()));
    }

    tracing::info!(slug = %slug, port = port, url = %url, "preview started");
    Ok(url)
}

/// Kills the running preview process if one exists. Idempotent — returns
/// `Ok(())` even when nothing is running.
///
/// # Errors
pub async fn stop_preview(state: &Arc<AppState>) -> Result<(), AstroError> {
    // Take the child out under the lock, then release it before killing.
    let active = state.lock_preview().await.take();
    if let Some(mut preview) = active {
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
        Arc::new(AppState::new(sites_dir, 4321, Vec::new()))
    }

    #[tokio::test]
    async fn start_preview_returns_site_not_found_when_dir_missing() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());

        let err = start_preview(&state, "nonexistent", 4321)
            .await
            .unwrap_err();
        assert!(matches!(err, AstroError::SiteNotFound(_)));
    }

    #[tokio::test]
    async fn start_preview_returns_already_running_when_preview_active() {
        let tmp = TempDir::new().unwrap();
        // Create a placeholder site dir so it passes the existence check.
        std::fs::create_dir(tmp.path().join("my-site")).unwrap();
        let state = make_state(tmp.path());

        // Simulate a running preview by putting something in the mutex.
        let fake_child = tokio::process::Command::new("sleep")
            .arg("100")
            .spawn()
            .unwrap();
        *state.lock_preview().await = Some(ActivePreview {
            slug: "other-site".to_string(),
            url: "http://localhost:4321".to_string(),
            child: fake_child,
        });

        let err = start_preview(&state, "my-site", 4321).await.unwrap_err();
        assert!(matches!(err, AstroError::PreviewAlreadyRunning(_)));

        // Cleanup — kill the sleep process. Bind the taken value first so the
        // MutexGuard temporary drops before the await.
        let running = state.lock_preview().await.take();
        if let Some(mut p) = running {
            p.child.kill().await.ok();
        }
    }

    #[tokio::test]
    async fn stop_preview_is_idempotent_when_nothing_running() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        stop_preview(&state).await.unwrap();
    }
}
