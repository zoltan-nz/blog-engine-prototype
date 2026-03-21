pub struct ActivePreview {
    pub slug: String,
    pub child: tokio::process::Child,
}

pub struct AppState {
    preview: tokio::sync::Mutex<Option<ActivePreview>>,
    pub sites_dir: std::path::PathBuf,
    pub git_repos_dir: std::path::PathBuf,
    pub preview_port: u16,
}

impl AppState {
    pub fn new(
        sites_dir: impl Into<std::path::PathBuf>,
        git_repos_dir: impl Into<std::path::PathBuf>,
        preview_port: u16,
    ) -> Self {
        Self {
            preview: tokio::sync::Mutex::new(None),
            sites_dir: sites_dir.into(),
            git_repos_dir: git_repos_dir.into(),
            preview_port,
        }
    }

    pub async fn lock_preview(&self) -> tokio::sync::MutexGuard<'_, Option<ActivePreview>> {
        self.preview.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lock_preview_initially_none() {
        let state = AppState::new("/tmp/sites", "/tmp/repos", 4321);
        let guard = state.lock_preview().await;
        assert!(guard.is_none());
    }
}