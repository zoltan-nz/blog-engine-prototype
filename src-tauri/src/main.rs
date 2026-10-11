// Release builds on Windows would otherwise open a console window too.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;
mod site_files;

use backend::app::{create_app, shutdown, shutdown_signal};
use backend::config::Config;
use backend::telemetry;
use std::sync::Arc;

fn main() {
    telemetry::init_tracing();

    let config = Config::from_env().expect("Failed to load config");

    // create_app starts per-site watchers that spawn tokio tasks, so it must
    // run inside Tauri's runtime.
    // No HTTP server: the webview talks over IPC (`ipc.rs`) and loads
    // images through the `site-files://` protocol, which runs the router
    // in-process.
    let (router, state) = tauri::async_runtime::block_on(async { create_app(config) });

    tauri::Builder::default()
        .manage(Arc::clone(&state))
        .manage(ipc::IpcConnection::default())
        .invoke_handler(tauri::generate_handler![ipc::connect, ipc::send])
        .register_asynchronous_uri_scheme_protocol(
            site_files::SCHEME,
            move |_ctx, request, responder| {
                let router = router.clone();
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
