use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub backend_ws_url: String,
    pub sites_dir: String,
    pub git_repos_dir: String,
    pub preview_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend_ws_url: "ws://localhost:8080/api/supervisor/ws".to_string(),
            sites_dir: "/tmp/sites".to_string(),
            git_repos_dir: "/tmp/repos".to_string(),
            preview_port: 4321,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok(); // loads environment variables from .env file, silently ignores if missing
        envy::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_defaults_when_no_env_vars_set() {
        let config = envy::from_iter::<_, Config>(std::iter::empty::<(String, String)>()).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn overrides_default_with_env_var() {
        let config = envy::from_iter::<_, Config>(vec![(
            "BACKEND_WS_URL".to_string(),
            "ws://custom.test:9090/ws".to_string(),
        )])
        .unwrap();
        assert_eq!(config.backend_ws_url, "ws://custom.test:9090/ws");
        assert_eq!(config.preview_port, 4321);
    }
}
