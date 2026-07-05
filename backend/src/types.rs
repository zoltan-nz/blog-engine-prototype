//! Wire types for the WebSocket protocol. Every type here derives
//! `specta::Type` and is exported to `frontend/src/lib/types/bindings.ts` by
//! `bin/export-types.rs` — the FSM states double as wire types so backend and
//! frontend share one state vocabulary.
use serde::{Deserialize, Serialize};
use specta::Type;

/// Lifecycle of a managed Astro site. Transitions live in `fsm::site`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "payload")]
pub enum SiteState {
    Creating,
    Ready,
    Building,
    BuildFailed { reason: String },
    Deleting,
}

/// Lifecycle of the (single) Astro dev-server preview. Transitions live in
/// `fsm::preview`. `Starting` covers the TCP readiness poll window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "payload")]
pub enum PreviewState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ErrorCode {
    SiteNotFound,
    SiteAlreadyExists,
    InvalidTransition,
    BuildFailed,
    PreviewTimeout,
    Internal,
}

/// Client → server messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "payload")]
pub enum Command {
    CreateSite { name: String, slug: String },
    BuildSite { slug: String },
    StartPreview { slug: String },
    StopPreview,
    DeleteSite { slug: String },
    Ping,
}

/// A site as the frontend sees it: identity plus current FSM state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SiteView {
    pub slug: String,
    pub name: String,
    pub state: SiteState,
}

/// The preview as the frontend sees it. `slug` says which site is previewed;
/// `url` is set only while `Running`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct PreviewView {
    pub state: PreviewState,
    pub slug: Option<String>,
    pub url: Option<String>,
}

/// Server → client messages. `Snapshot` is sent on every connect — reconnect
/// strategy is a fresh snapshot, never event replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    Snapshot {
        sites: Vec<SiteView>,
        preview: PreviewView,
    },
    SiteChanged(SiteView),
    SiteRemoved {
        slug: String,
    },
    PreviewChanged(PreviewView),
    BuildLog {
        slug: String,
        stream: LogStream,
        data: String,
    },
    Error {
        code: ErrorCode,
        message: String,
        correlation_id: Option<String>,
    },
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    Command(Command),
    Event(Event),
}

/// Lean envelope: timestamp + correlation id + flattened message. Commands
/// carry a client-generated `correlation_id`; error events echo it back so the
/// client can match failures to requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WsEnvelope {
    pub unix_timestamp_us: i64,
    pub correlation_id: String,

    #[serde(flatten)]
    pub message: WsMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_flattens_message_tag_to_top_level() {
        let envelope = WsEnvelope {
            unix_timestamp_us: 42,
            correlation_id: "abc".into(),
            message: WsMessage::Event(Event::Pong),
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["type"], "Event");
        assert_eq!(json["payload"]["type"], "Pong");
        assert_eq!(json["correlation_id"], "abc");
    }

    #[test]
    fn command_envelope_roundtrips_from_client_json() {
        let raw = r#"{
            "unix_timestamp_us": 0,
            "correlation_id": "c-1",
            "type": "Command",
            "payload": { "type": "CreateSite", "payload": { "name": "My Blog", "slug": "my-blog" } }
        }"#;
        let envelope: WsEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(
            envelope.message,
            WsMessage::Command(Command::CreateSite {
                name: "My Blog".into(),
                slug: "my-blog".into(),
            })
        );
    }

    #[test]
    fn unit_command_deserializes_without_payload() {
        let raw = r#"{
            "unix_timestamp_us": 0,
            "correlation_id": "c-2",
            "type": "Command",
            "payload": { "type": "Ping" }
        }"#;
        let envelope: WsEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(envelope.message, WsMessage::Command(Command::Ping));
    }

    #[test]
    fn site_state_with_payload_serializes_tagged() {
        let state = SiteState::BuildFailed {
            reason: "boom".into(),
        };
        let json = serde_json::to_value(&state).unwrap();
        assert_eq!(json["type"], "BuildFailed");
        assert_eq!(json["payload"]["reason"], "boom");
    }
}
