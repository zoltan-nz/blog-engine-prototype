use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_sites_dir")]
    pub sites_dir: PathBuf,
}

fn default_sites_dir() -> PathBuf {
    PathBuf::from("/tmp/astro-sites")
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env()
    }
}

