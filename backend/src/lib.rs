#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod handlers;
pub mod state;

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Shared meta infrastructure
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MetaServerName {
    Backend,
}

#[derive(Debug, Serialize, ToSchema)]
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