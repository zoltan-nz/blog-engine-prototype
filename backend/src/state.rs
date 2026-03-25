use tokio::sync::mpsc::Sender;

pub struct CommandMessage {
    pub envelope: admin_protocol::Envelope<admin_protocol::Command>,
    pub response_tx: tokio::sync::oneshot::Sender<admin_protocol::Event>
}

pub struct AppState {
    pub command_tx: Sender<CommandMessage>
}