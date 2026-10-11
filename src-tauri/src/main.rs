// Release builds on Windows would otherwise open a console window too.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;

use backend::app::{create_app, shutdown, shutdown_signal};
use backend::config::Config;
use backend::telemetry;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::info;

/// The webview talks to the backend over IPC (`ipc.rs`), but post images still
/// load from `/site-files` at `PUBLIC_API_BACKEND_URL=http://localhost:8080`,
/// so the HTTP server stays until those move to a custom protocol.
const BACKEND_PORT: u16 = 8080;

fn main() {
    telemetry::init_tracing();

    let config = Config::from_env().expect("Failed to load config");

    // create_app starts per-site watchers that spawn tokio tasks, so it must
    // run inside Tauri's runtime.
    let (router, state) = tauri::async_runtime::block_on(async { create_app(config) });

    // Bind before the window opens: a port already in use (e.g. a dev backend
    // still running) fails startup with a clear error instead of a window
    // whose socket never connects.
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, BACKEND_PORT));
    let listener = tauri::async_runtime::block_on(tokio::net::TcpListener::bind(addr))
        .unwrap_or_else(|error| panic!("Failed to bind backend to {addr}: {error}"));
    info!(%addr, "Backend listening");

    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            tracing::error!(%error, "backend server stopped");
        }
    });

    tauri::Builder::default()
        .manage(Arc::clone(&state))
        .manage(ipc::IpcConnection::default())
        .invoke_handler(tauri::generate_handler![ipc::connect, ipc::send])
        .setup(|app| {
            // SIGINT/SIGTERM would otherwise kill the process without
            // RunEvent::Exit and orphan the Astro preview; route them
            // through the normal exit path instead.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                shutdown_signal().await;
                handle.exit(0);
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to build the Tauri app")
        .run(move |_app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                tauri::async_runtime::block_on(shutdown(&state));
            }
        });
}
