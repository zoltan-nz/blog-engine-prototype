use std::path::PathBuf;
use tokio::sync::{Mutex, MutexGuard};

/// A running `pnpm dev` preview process. Owns the child so it can be killed on
/// stop. `url` lets `list_sites` report `previewUrl` without a second field.
pub struct ActivePreview {
    pub slug: String,
    pub url: String,
    pub child: tokio::process::Child,
}

pub struct AppState {
    pub sites_dir: PathBuf,
    pub preview_port: u16,
    /// At most one preview runs at a time; the mutex serialises start/stop.
    preview: Mutex<Option<ActivePreview>>,
}

impl AppState {
    pub fn new(sites_dir: impl Into<PathBuf>, preview_port: u16) -> Self {
        Self {
            sites_dir: sites_dir.into(),
            preview_port,
            preview: Mutex::new(None),
        }
    }

    pub async fn lock_preview(&self) -> MutexGuard<'_, Option<ActivePreview>> {
        self.preview.lock().await
    }
}
