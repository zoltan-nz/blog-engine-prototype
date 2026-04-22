use admin_protocol::{Envelope, Event};
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, broadcast};

pub struct CommandMessage {
    pub envelope: admin_protocol::Envelope<admin_protocol::Command>,
    pub response_tx: tokio::sync::oneshot::Sender<admin_protocol::Event>,
}

pub struct AppState {
    pub command_tx: Mutex<Sender<CommandMessage>>,
    pub event_tx: broadcast::Sender<Envelope<Event>>,
    pub command_rx: Mutex<Option<Receiver<CommandMessage>>>,
    pub sites_dir: PathBuf,
    /// Tracks the slug and URL of the currently active preview server.
    /// Set by `preview_site`, cleared by `stop_preview`, read by `list_sites`.
    pub active_preview: Mutex<Option<(String, String)>>,
}
