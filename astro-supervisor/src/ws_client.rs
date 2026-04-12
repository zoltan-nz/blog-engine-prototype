use crate::state::AppState;
use admin_protocol::{Command, Envelope, ErrorCode, Event};
use backon::{ExponentialBuilder, Retryable};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub async fn agent_main_loop(backend_url: String, state: Arc<AppState>) {
    let run = || async { connect_and_run(backend_url.clone(), state.clone()).await };

    run.retry(
        ExponentialBuilder::default()
            .with_jitter()
            .with_max_delay(Duration::from_secs(60))
            .with_max_times(0),
    )
    .sleep(tokio::time::sleep)
    .notify(|err: &anyhow::Error, dur: Duration| {
        tracing::error!(error = %err, delay_secs = dur.as_secs(), "reconnecting to backend");
    })
    .await
    .ok();
}

pub async fn connect_and_run(url: String, state: Arc<AppState>) -> anyhow::Result<()> {
    let (ws_stream, _) = connect_async(&url).await?;
    tracing::info!("connected to backend");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg?;
        let text = match msg {
            Message::Text(t) => t,
            _ => continue,
        };
        let envelope: Envelope<Command> = serde_json::from_str(&text)?;
        tracing::debug!(command = ?envelope.payload, id = %envelope.id, "received command");

        let event = dispatch_command(envelope.payload, &state).await;
        tracing::debug!(event = ?event, "dispatched, sending reply");

        let reply = Envelope {
            id: Uuid::new_v4(),
            correlation_id: Some(envelope.id),
            idempotency_key: None,
            sequence: 0,
            timestamp: chrono::Utc::now(),
            payload: event,
        };
        let json = serde_json::to_string(&reply)?;
        write.send(Message::Text(json.into())).await?;
    }

    tracing::info!("disconnected from backend cleanly");
    Ok(())
}

async fn dispatch_command(cmd: Command, state: &Arc<AppState>) -> Event {
    match cmd {
        Command::Ping => Event::Pong,
        Command::CreateSite { name, domain: _domain } => Event::SiteCreated {
            site_id: Uuid::new_v4(),
            name,
        },
        Command::StartPreview { site_id, port } => Event::PreviewReady {
            site_id,
            url: format!("http://localhost:{}", port.unwrap_or(state.preview_port)),
            port: port.unwrap_or(state.preview_port),
        },
        Command::StopPreview { site_id } => Event::PreviewStopped { site_id },
        Command::GetStatus { site_id: _site_id } => Event::BuildProgress {
            build_id: Uuid::new_v4(),
            phase: "not implemented".into(),
            percent: 0.0,
        },
        _ => Event::Error {
            code: ErrorCode::Internal,
            message: "not implemented".into(),
            command_id: None,
        },
    }
}
