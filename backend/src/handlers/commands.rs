use crate::state::{AppState, CommandMessage};
use admin_protocol::{Command, Envelope, Event};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;
use std::time::Duration;
use axum::http::StatusCode;
use tokio::sync::oneshot;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/commands",
    request_body = Command,
    responses(
        (status = 200, body = Event),
        (status = 503, description = "Supervisor not connected"),
        (status = 500, description = "Command timed out")
    )
)]
pub async fn dispatch_command(
    State(state): State<Arc<AppState>>,
    Json(command): Json<Command>,
) -> impl IntoResponse {
    let envelope = Envelope {
        id: Uuid::new_v4(),
        correlation_id: None,
        idempotency_key: None,
        sequence: 0,
        timestamp: chrono::Utc::now(),
        payload: command,
    };

    let (response_tx, response_rx) = oneshot::channel::<Event>();
    tracing::info!(command = ?envelope.payload, id = %envelope.id, "dispatching command to supervisor");

    if state.command_tx
        .lock().await
        .send(CommandMessage { envelope, response_tx})
        .await
        .is_err()
    {
        tracing::warn!("supervisor not connected — command_tx send failed");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    match tokio::time::timeout(Duration::from_secs(1), response_rx).await {
        Ok(Ok(event)) => {
            tracing::info!(event = ?event, "command resolved");
            (StatusCode::OK, Json(event)).into_response()
        }
        Ok(Err(_)) => {
            tracing::warn!("supervisor disconnected before responding");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        Err(_) => {
            tracing::warn!("command timed out after 30s");
            StatusCode::GATEWAY_TIMEOUT.into_response()
        }
    }
}