mod config;
mod error;
mod handlers;
mod state;
mod telemetry;
mod ws_client;

use std::fs::create_dir_all;
use std::process::ExitStatus;
use std::sync::Arc;
use tokio::io;
use tracing::error;
use crate::config::Config;
use crate::state::AppState;
use crate::telemetry::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();

    let config = Config::from_env().expect("Failed to load configuration");

    create_dir_all(&config.sites_dir).expect("Failed to create SITES_DIR");
    create_dir_all(&config.git_repos_dir).expect("Failed to create GIT_REPOS_DIR");

    let state = Arc::new(AppState::new(config.sites_dir, config.git_repos_dir, config.preview_port));

    check_required_command_exist("pnpm", "npm install -g pnpm");
    check_required_command_exist("create-astro", "pnpm add -g create-astro");

    ws_client::agent_main_loop(config.backend_ws_url, state).await;
}

fn check_required_command_exist(name: &str, install_hint: &str) -> () {
    match which::which(name) {
        Err(_) => {
            error!("Required command not found: {}", name);
            error!("Hint: {}", install_hint);
            std::process::exit(1)
        }
        Ok(_) => (),
    }
}