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
    Conflict,
    BuildFailed,
    PreviewTimeout,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Command {
    CreateSite { name: String, slug: String },
    BuildSite { slug: String, force: bool },
    StartPreview { slug: String, port: Option<u16> },
    StopPreview,
    GetStatus { slug: String },
    DeleteSite { slug: String },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    SiteCreated {
        slug: String,
        name: String,
    },
    SiteDeleted,
    BuildStarted {
        build_id: Uuid,
        slug: String,
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
        slug: String,
        url: String,
        port: u16,
    },
    PreviewStopped,
    Pong,
    Error {
        code: ErrorCode,
        message: String,
        command_id: Option<Uuid>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[allow(clippy::option_if_let_else)]
pub struct Envelope<T> {
    pub id: Uuid,
    pub correlation_id: Option<Uuid>,  // links response to request
    pub idempotency_key: Option<Uuid>, // client-generated; same across retries
    pub sequence: u64,                 // monotonic; used for replay-from detection
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: T,
}
