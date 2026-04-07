use admin_protocol::{Envelope, Event};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, broadcast};

pub struct CommandMessage {
    pub envelope: admin_protocol::Envelope<admin_protocol::Command>,
    pub response_tx: tokio::sync::oneshot::Sender<admin_protocol::Event>,
}

pub struct AppState {
    pub command_tx: Sender<CommandMessage>,
    pub event_tx: broadcast::Sender<Envelope<Event>>,
    pub command_rx: Mutex<Option<Receiver<CommandMessage>>>,
}
