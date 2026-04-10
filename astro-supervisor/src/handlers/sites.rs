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
}
