//! Where the desktop app keeps its sites. The backend's own default
//! (`/tmp/astro-sites`) is shared with a dev backend and wiped on reboot, so
//! the desktop defaults to its app-data folder unless `SITES_DIR` is set.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Folder under the app-data directory that holds `sites.json` and the sites.
const SITES_FOLDER: &str = "sites";

/// Desktop-only env overrides, read alongside the backend's `Config`.
#[derive(Debug, Default, Deserialize)]
pub struct DesktopEnv {
    sites_dir: Option<PathBuf>,
}

impl DesktopEnv {
    /// `SITES_DIR` if set, else `{data_dir}/{identifier}/sites`, the same
    /// folder Tauri's `app_data_dir()` would give.
    pub fn sites_dir(self, data_dir: &Path, identifier: &str) -> PathBuf {
        self.sites_dir
            .unwrap_or_else(|| data_dir.join(identifier).join(SITES_FOLDER))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(vars: &[(&str, &str)]) -> DesktopEnv {
        envy::from_iter(vars.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))).unwrap()
    }

    #[test]
    fn defaults_to_the_app_data_folder() {
        let dir = env(&[]).sites_dir(Path::new("/data"), "app.example");

        assert_eq!(dir, PathBuf::from("/data/app.example/sites"));
    }

    #[test]
    fn sites_dir_env_wins() {
        let dir =
            env(&[("SITES_DIR", "/tmp/astro-sites")]).sites_dir(Path::new("/data"), "app.example");

        assert_eq!(dir, PathBuf::from("/tmp/astro-sites"));
    }
}
