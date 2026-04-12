use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ErrorCode {
    SiteNotFound,
    BuildFailed,
    PreviewTimeout,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Command {
    CreateSite { name: String, domain: String },
    BuildSite { site_id: Uuid, force: bool },
    StartPreview { site_id: Uuid, port: Option<u16> },
    StopPreview { site_id: Uuid },
    GetStatus { site_id: Uuid },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    SiteCreated {
        site_id: Uuid,
        name: String,
    },
    BuildStarted {
        build_id: Uuid,
        site_id: Uuid,
    },
    BuildProgress {
        build_id: Uuid,
        phase: String,
        percent: f32,
    },
    BuildLog {
        build_id: Uuid,
        stream: LogStream,
        data: String,
    },
    BuildCompleted {
        build_id: Uuid,
        duration_ms: u64,
    },
    BuildFailed {
        build_id: Uuid,
        error: String,
        retryable: bool,
    },
    PreviewReady {
        site_id: Uuid,
        url: String,
        port: u16,
    },
    PreviewStopped {
        site_id: Uuid,
    },
    Pong,
    Error {
        code: ErrorCode,
        message: String,
        command_id: Option<Uuid>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Envelope<T: ToSchema> {
    pub id: uuid::Uuid,
    pub correlation_id: Option<Uuid>,  // links response to request
    pub idempotency_key: Option<Uuid>, // client-generated; same across retries
    pub sequence: u64,                 // monotonic; used for replay-from detection
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: T,
}
