use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_sites_dir")]
    pub sites_dir: PathBuf,
    #[serde(default = "default_preview_port")]
    pub preview_port: u16,
    #[serde(default = "default_frontend_dir")]
    pub frontend_dir: PathBuf,
}

fn default_sites_dir() -> PathBuf {
    PathBuf::from("/tmp/astro-sites")
}

fn default_frontend_dir() -> PathBuf {
    PathBuf::from("../frontend/build")
}

const fn default_preview_port() -> u16 {
    4321
}

impl Config {
    #[allow(clippy::missing_errors_doc)]
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}
