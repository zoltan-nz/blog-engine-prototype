// Release builds on Windows would otherwise open a console window too.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;
mod site_files;

use backend::app::{create_app, shutdown, shutdown_signal};
use backend::config::Config;
use backend::telemetry;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::info;

/// The webview uses IPC (`ipc.rs`) and the `site-files://` protocol
/// (`site_files.rs`), so nothing in the app needs this server any more; it
/// stays until a follow-up step removes it.
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

    let site_files_router = router.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            tracing::error!(%error, "backend server stopped");
        }
    });

    tauri::Builder::default()
        .manage(Arc::clone(&state))
        .manage(ipc::IpcConnection::default())
        .invoke_handler(tauri::generate_handler![ipc::connect, ipc::send])
        .register_asynchronous_uri_scheme_protocol(
            site_files::SCHEME,
            move |_ctx, request, responder| {
                let router = site_files_router.clone();
                tauri::async_runtime::spawn(async move {
                    responder.respond(site_files::serve(router, request).await);
                });
            },
        )
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
