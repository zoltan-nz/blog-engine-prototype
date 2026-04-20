use crate::error::AgentError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const MANIFEST_FILE_NAME: &str = "sites.json";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SiteData {
    pub folder: String,
    pub name: String,
    pub git_url: String,
}

#[derive(Deserialize, Serialize)]
struct SitesManifest {
    sites: Vec<SiteData>,
}

pub fn list_sites(sites_dir: &Path) -> Result<Vec<SiteData>, AgentError> {
    let manifest_file = sites_dir.join(MANIFEST_FILE_NAME);
    if !manifest_file.exists() {
        std::fs::write(&manifest_file, r#"{ "sites": [] }"#)?;
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&manifest_file)?;
    let sites_manifest: SitesManifest = serde_json::from_str(&content)?;

    Ok(sites_manifest.sites)
}

/// Scaffolds a minimal Astro project in `site_dir` and installs dependencies.
///
/// Runs sequentially:
///   1. `create-astro . --template minimal --no-git --yes --skip-houston --no-install`
///   2. `pnpm install`
///
/// Both commands are assumed to be on `PATH` (Node.js + pnpm must be installed).
pub async fn scaffold_site(site_dir: &Path) -> Result<(), AgentError> {
    let create = tokio::process::Command::new("create-astro")
        .args([".", "--template", "minimal", "--no-git", "--yes", "--skip-houston", "--no-install"])
        .current_dir(site_dir)
        .status()
        .await?;

    if !create.success() {
        return Err(AgentError::CommandFailed(format!(
            "create-astro exited with {create}"
        )));
    }

    let install = tokio::process::Command::new("pnpm")
        .args(["install"])
        .current_dir(site_dir)
        .status()
        .await?;

    if !install.success() {
        return Err(AgentError::CommandFailed(format!(
            "pnpm install exited with {install}"
        )));
    }

    tracing::info!(dir = %site_dir.display(), "Astro project scaffolded");
    Ok(())
}

pub fn create_site(sites_dir: &Path, name: &str, slug: &str) -> Result<SiteData, AgentError> {
    let existing = list_sites(sites_dir)?;
    if existing.iter().any(|s| s.folder == slug) {
        return Err(AgentError::SiteAlreadyExists(slug.into()));
    }

    fs::create_dir_all(sites_dir.join(slug))?;

    let manifest_path = sites_dir.join(MANIFEST_FILE_NAME);
    let content = fs::read_to_string(&manifest_path)?;
    let mut manifest: SitesManifest = serde_json::from_str(&content)?;
    let site = SiteData { folder: slug.into(), name: name.into(), git_url: "".into() };
    manifest.sites.push(site.clone());
    fs::write(&manifest_path, serde_json::to_string(&manifest)?)?;

    Ok(site)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn list_sites_creates_manifest_file_when_missing() {
        let sites = TempDir::new().unwrap();

        let result = list_sites(sites.path()).unwrap();
        assert_eq!(result.len(), 0);
        let manifest_file = fs::read(sites.path().join(MANIFEST_FILE_NAME));
        assert!(manifest_file.is_ok());
    }

    #[test]
    fn list_sites_returns_sites_from_manifest() {
        let sites = TempDir::new().unwrap();

        let site_data = SiteData {
            folder: "my-site".to_string(),
            name: "My Site".to_string(),
            git_url: "/repos/my-blog.git".to_string(),
        };

        let manifest_file = sites.path().join(MANIFEST_FILE_NAME);
        let _ = fs::write(
            &manifest_file,
            serde_json::to_string(&SitesManifest {
                sites: vec![site_data.clone()],
            })
            .unwrap(),
        );

        let result = list_sites(sites.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], site_data);
    }

    #[test]
    fn create_site_creates_folder_in_sites_dir() {
        let sites = TempDir::new().unwrap();

        create_site(sites.path(), "My Site", "my-site").unwrap();

        assert!(sites.path().join("my-site").is_dir())
    }

    #[test]
    fn create_site_adds_entry_to_manifest() {
        let sites = TempDir::new().unwrap();

        create_site(sites.path(), "My Site", "my-site").unwrap();
        let result = list_sites(sites.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "My Site");
        assert_eq!(result[0].folder, "my-site");
        assert_eq!(result[0].git_url, "");
    }

    #[test]
    fn create_site_returns_err_when_slug_already_exists() {
        let sites = TempDir::new().unwrap();
        create_site(sites.path(), "My Site", "my-site").unwrap();
        let result = create_site(sites.path(), "Another Site", "my-site");

        assert!(matches!(result, Err(AgentError::SiteAlreadyExists(_))));
    }
}
