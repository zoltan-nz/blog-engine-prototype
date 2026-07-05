#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod app;
pub mod astro;
pub mod config;
pub mod fsm;
pub mod handlers;
pub mod routes;
pub mod state;
pub mod telemetry;
pub mod types;
pub mod ws;

use serde::Serialize;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared meta infrastructure (HTTP healthz only; the WS protocol uses
// `types::WsEnvelope`)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetaServerName {
    Backend,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub timestamp: String,
    pub request_id: String,
    pub version: &'static str,
    pub server_name: MetaServerName,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            request_id: Uuid::new_v4().to_string(),
            version: env!("CARGO_PKG_VERSION"),
            server_name: MetaServerName::Backend,
        }
    }
}
