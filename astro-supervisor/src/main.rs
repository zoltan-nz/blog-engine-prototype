mod config;
mod error;
mod handlers;
mod state;
mod telemetry;
mod ws_client;

use std::sync::Arc;
use crate::config::Config;
use crate::state::AppState;
use crate::telemetry::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();

    let config = Config::from_env().expect("Failed to load configuration");
    let state = Arc::new(AppState::new(config.sites_dir, config.git_repos_dir, config.preview_port));

    ws_client::agent_main_loop(config.backend_ws_url, state).await;
}
