use crate::astro::error::AstroError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const MANIFEST_FILE_NAME: &str = "sites.json";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SiteData {
    pub folder: String,
    pub name: String,
    pub git_url: String,
}

#[derive(Deserialize, Serialize)]
struct SitesManifest {
    sites: Vec<SiteData>,
}

pub fn list_sites(sites_dir: &Path) -> Result<Vec<SiteData>, AstroError> {
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
pub async fn scaffold_site(site_dir: &Path) -> Result<(), AstroError> {
    let create = tokio::process::Command::new("create-astro")
        .args([
            ".",
            "--template",
            "minimal",
            "--no-git",
            "--yes",
            "--skip-houston",
            "--no-install",
        ])
        .current_dir(site_dir)
        .status()
        .await?;

    if !create.success() {
        return Err(AstroError::CommandFailed(format!(
            "create-astro exited with {create}"
        )));
    }

    // pnpm v10+ blocks build scripts by default. esbuild and sharp are native binaries
    // required by Vite/Astro — without this file pnpm install exits with ERR_PNPM_IGNORED_BUILDS.
    fs::write(
        site_dir.join("pnpm-workspace.yaml"),
        "allowBuilds:\n  esbuild: true\n  sharp: true\nonlyBuiltDependencies:\n  - esbuild\n  - sharp\n",
    )?;

    let install = tokio::process::Command::new("pnpm")
        .args(["install"])
        .current_dir(site_dir)
        .status()
        .await?;

    if !install.success() {
        return Err(AstroError::CommandFailed(format!(
            "pnpm install exited with {install}"
        )));
    }

    tracing::info!(dir = %site_dir.display(), "Astro project scaffolded");
    Ok(())
}

pub fn create_site(sites_dir: &Path, name: &str, slug: &str) -> Result<SiteData, AstroError> {
    let existing = list_sites(sites_dir)?;
    if existing.iter().any(|s| s.folder == slug) {
        return Err(AstroError::SiteAlreadyExists(slug.into()));
    }

    fs::create_dir_all(sites_dir.join(slug))?;

    let manifest_path = sites_dir.join(MANIFEST_FILE_NAME);
    let content = fs::read_to_string(&manifest_path)?;
    let mut manifest: SitesManifest = serde_json::from_str(&content)?;
    let site = SiteData {
        folder: slug.into(),
        name: name.into(),
        git_url: String::new(),
    };
    manifest.sites.push(site.clone());
    fs::write(&manifest_path, serde_json::to_string(&manifest)?)?;

    Ok(site)
}

pub fn delete_site(sites_dir: &Path, slug: &str) -> Result<(), AstroError> {
    let sites = list_sites(sites_dir)?;
    if !sites.iter().any(|s| s.folder == slug) {
        return Err(AstroError::SiteNotFound(slug.into()));
    }

    fs::remove_dir_all(sites_dir.join(slug))?;

    let manifest_path = sites_dir.join(MANIFEST_FILE_NAME);
    let content = fs::read_to_string(&manifest_path)?;
    let mut manifest: SitesManifest = serde_json::from_str(&content)?;
    manifest.sites.retain(|s| s.folder != slug);
    fs::write(&manifest_path, serde_json::to_string(&manifest)?)?;

    Ok(())
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

        assert!(sites.path().join("my-site").is_dir());
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

        assert!(matches!(result, Err(AstroError::SiteAlreadyExists(_))));
    }

    #[test]
    fn delete_site_removes_folder_and_manifest_entry() {
        let sites = TempDir::new().unwrap();
        create_site(sites.path(), "My Site", "my-site").unwrap();

        delete_site(sites.path(), "my-site").unwrap();

        assert!(!sites.path().join("my-site").exists());
        let remaining = list_sites(sites.path()).unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn delete_site_returns_err_when_slug_not_found() {
        let sites = TempDir::new().unwrap();
        // Initialise manifest so list_sites doesn't fail.
        list_sites(sites.path()).unwrap();

        let result = delete_site(sites.path(), "ghost");
        assert!(matches!(result, Err(AstroError::SiteNotFound(_))));
    }
}
