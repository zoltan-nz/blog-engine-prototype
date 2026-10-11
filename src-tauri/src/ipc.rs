//! Tauri IPC transport for the client protocol. The webview calls `connect`
//! once per page load and `send` per command; both drive the same
//! `ws::connection` as the `/ws` socket, carrying the same `WsEnvelope` JSON.

use backend::state::AppState;
use backend::types::WsEnvelope;
use backend::ws::connection;
use std::sync::{Arc, Mutex};
use tauri::State;
use tauri::async_runtime::JoinHandle;
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use tracing::info;

/// The webview's current connection. A page reload calls `connect` again,
/// which replaces the previous connection and stops its pump.
#[derive(Default)]
pub struct IpcConnection(Mutex<Option<Active>>);

struct Active {
    tx: mpsc::Sender<WsEnvelope>,
    pump: JoinHandle<()>,
}

#[tauri::command]
pub async fn connect(
    on_message: Channel<WsEnvelope>,
    app: State<'_, Arc<AppState>>,
    ipc: State<'_, IpcConnection>,
) -> Result<(), String> {
    let (tx, mut rx) = connection::open(&app).await;

    let pump = tauri::async_runtime::spawn(async move {
        while let Some(envelope) = rx.recv().await {
            if on_message.send(envelope).is_err() {
                break; // webview gone
            }
        }
    });

    let previous = ipc
        .0
        .lock()
        .map_err(|_| "IPC connection lock poisoned")?
        .replace(Active { tx, pump });
    if let Some(previous) = previous {
        // Dropping its receiver also ends that connection's broadcast fan-in.
        previous.pump.abort();
    }
    info!("IPC client connected");
    Ok(())
}

// Tauri deserializes command arguments into owned values. `async` so Tauri
// runs it on its tokio runtime: `handle_text` spawns the dispatch task, and a
// sync command would run on the main thread, where `tokio::spawn` panics.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub async fn send(
    envelope: String,
    app: State<'_, Arc<AppState>>,
    ipc: State<'_, IpcConnection>,
) -> Result<(), String> {
    // Copy the sender out so the lock isn't held while dispatching.
    let tx = ipc
        .0
        .lock()
        .map_err(|_| "IPC connection lock poisoned")?
        .as_ref()
        .map(|active| active.tx.clone())
        .ok_or("send before connect")?;
    connection::handle_text(&envelope, &tx, &app);
    Ok(())
}
