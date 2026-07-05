use crate::astro;
use crate::astro::error::AstroError;
use crate::fsm;
use crate::state::{AppState, SiteEntry};
use crate::types::{
    Command, ErrorCode, Event, PreviewState, PreviewView, SiteState, SiteView, WsEnvelope,
    WsMessage,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

/// Wraps a server event in an envelope with a fresh correlation id (used for
/// broadcasts, which no single request owns).
#[must_use]
pub fn server_envelope(event: Event) -> WsEnvelope {
    envelope(Uuid::new_v4().to_string(), event)
}

fn envelope(correlation_id: String, event: Event) -> WsEnvelope {
    WsEnvelope {
        unix_timestamp_us: chrono::Utc::now().timestamp_micros(),
        correlation_id,
        message: WsMessage::Event(event),
    }
}

/// Full state snapshot for a (re)connecting client.
pub async fn snapshot_event(state: &Arc<AppState>) -> Event {
    let mut sites: Vec<SiteView> = state
        .sites
        .read()
        .await
        .iter()
        .map(|(slug, entry)| SiteView {
            slug: slug.clone(),
            name: entry.name.clone(),
            state: entry.state.clone(),
        })
        .collect();
    sites.sort_by(|a, b| a.slug.cmp(&b.slug));

    let preview = state.preview_view.read().await.clone();
    Event::Snapshot { sites, preview }
}

pub async fn dispatch_command(
    correlation_id: String,
    command: Command,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    info!(?command, %correlation_id, "dispatching command");
    match command {
        Command::Ping => send_to_client(&tx, correlation_id, Event::Pong).await,
        Command::CreateSite { name, slug } => {
            create_site(correlation_id, name, slug, tx, state).await;
        }
        Command::BuildSite { slug } => build_site(correlation_id, slug, tx, state).await,
        Command::StartPreview { slug } => start_preview(correlation_id, slug, tx, state).await,
        Command::StopPreview => stop_preview(correlation_id, tx, state).await,
        Command::DeleteSite { slug } => delete_site(correlation_id, slug, tx, state).await,
    }
}

// --- command flows -----------------------------------------------------------

async fn create_site(
    correlation_id: String,
    name: String,
    slug: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    // Claim the slug in the FSM map before any I/O so a concurrent duplicate
    // request fails fast.
    {
        let mut sites = state.sites.write().await;
        if sites.contains_key(&slug) {
            drop(sites);
            send_error(
                &tx,
                correlation_id,
                ErrorCode::SiteAlreadyExists,
                format!("site '{slug}' already exists"),
            )
            .await;
            return;
        }
        sites.insert(
            slug.clone(),
            SiteEntry {
                name: name.clone(),
                state: SiteState::Creating,
            },
        );
    }
    broadcast_site(&state, &slug, &name, SiteState::Creating);

    let result = match astro::sites::create_site(&state.sites_dir, &name, &slug) {
        Ok(_) => astro::sites::scaffold_site(&state.sites_dir.join(&slug)).await,
        Err(e) => Err(e),
    };

    match result {
        Ok(()) => {
            match apply_site_event(&state, &slug, fsm::site::SiteEvent::ScaffoldSucceeded).await {
                Ok(new_state) => broadcast_site(&state, &slug, &name, new_state),
                Err(reply) => send_reply(&tx, correlation_id, reply).await,
            }
        }
        Err(error) => {
            warn!(%slug, %error, "create site failed, rolling back");
            state.sites.write().await.remove(&slug);
            // Best-effort manifest/folder cleanup; the site may have been
            // half-created before the scaffold failed.
            if let Err(cleanup) = astro::sites::delete_site(&state.sites_dir, &slug) {
                warn!(%slug, %cleanup, "rollback cleanup failed");
            }
            broadcast(&state, Event::SiteRemoved { slug: slug.clone() });
            send_error(
                &tx,
                correlation_id,
                astro_error_code(&error),
                error.to_string(),
            )
            .await;
        }
    }
}

async fn build_site(
    correlation_id: String,
    slug: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    let (name, new_state) =
        match apply_site_event_named(&state, &slug, fsm::site::SiteEvent::BuildRequested).await {
            Ok(ok) => ok,
            Err(reply) => {
                send_reply(&tx, correlation_id, reply).await;
                return;
            }
        };
    broadcast_site(&state, &slug, &name, new_state);

    // Forward build output lines to all clients as they arrive.
    let (log_tx, mut log_rx) = mpsc::unbounded_channel();
    let log_state = Arc::clone(&state);
    let log_slug = slug.clone();
    let forwarder = tokio::spawn(async move {
        while let Some((stream, data)) = log_rx.recv().await {
            broadcast(
                &log_state,
                Event::BuildLog {
                    slug: log_slug.clone(),
                    stream,
                    data,
                },
            );
        }
    });

    let result = astro::build::build_site(&state.sites_dir.join(&slug), log_tx).await;
    let _ = forwarder.await;

    let outcome_event = match &result {
        Ok(()) => fsm::site::SiteEvent::BuildSucceeded,
        Err(error) => fsm::site::SiteEvent::BuildFailed {
            reason: error.to_string(),
        },
    };
    match apply_site_event(&state, &slug, outcome_event).await {
        Ok(final_state) => broadcast_site(&state, &slug, &name, final_state),
        Err(reply) => send_reply(&tx, correlation_id.clone(), reply).await,
    }
    if let Err(error) = result {
        send_error(
            &tx,
            correlation_id,
            ErrorCode::BuildFailed,
            error.to_string(),
        )
        .await;
    }
}

async fn start_preview(
    correlation_id: String,
    slug: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    if !state.sites.read().await.contains_key(&slug) {
        send_error(
            &tx,
            correlation_id,
            ErrorCode::SiteNotFound,
            format!("site '{slug}' does not exist"),
        )
        .await;
        return;
    }

    let starting = apply_preview_event(
        &state,
        fsm::preview::PreviewEvent::StartRequested,
        Some(slug.clone()),
        None,
    )
    .await;
    match starting {
        Ok(view) => broadcast(&state, Event::PreviewChanged(view)),
        Err(reply) => {
            send_reply(&tx, correlation_id, reply).await;
            return;
        }
    }

    match astro::preview::start_preview(&state, &slug, state.preview_port).await {
        Ok(url) => {
            let running = apply_preview_event(
                &state,
                fsm::preview::PreviewEvent::ServerReady,
                Some(slug),
                Some(url),
            )
            .await;
            match running {
                Ok(view) => broadcast(&state, Event::PreviewChanged(view)),
                Err(reply) => send_reply(&tx, correlation_id, reply).await,
            }
        }
        Err(error) => {
            let failed = apply_preview_event(
                &state,
                fsm::preview::PreviewEvent::Failed {
                    reason: error.to_string(),
                },
                Some(slug),
                None,
            )
            .await;
            if let Ok(view) = failed {
                broadcast(&state, Event::PreviewChanged(view));
            }
            send_error(
                &tx,
                correlation_id,
                astro_error_code(&error),
                error.to_string(),
            )
            .await;
        }
    }
}

async fn stop_preview(correlation_id: String, tx: mpsc::Sender<WsEnvelope>, state: Arc<AppState>) {
    let slug = state.preview_view.read().await.slug.clone();
    let stopping = apply_preview_event(
        &state,
        fsm::preview::PreviewEvent::StopRequested,
        slug,
        None,
    )
    .await;
    match stopping {
        Ok(view) => broadcast(&state, Event::PreviewChanged(view)),
        Err(reply) => {
            send_reply(&tx, correlation_id, reply).await;
            return;
        }
    }

    if let Err(error) = astro::preview::stop_preview(&state).await {
        // The child handle is gone either way; report but continue to Stopped.
        warn!(%error, "stop preview reported an error");
        send_error(&tx, correlation_id, ErrorCode::Internal, error.to_string()).await;
    }

    let stopped = apply_preview_event(
        &state,
        fsm::preview::PreviewEvent::ProcessStopped,
        None,
        None,
    )
    .await;
    if let Ok(view) = stopped {
        broadcast(&state, Event::PreviewChanged(view));
    }
}

async fn delete_site(
    correlation_id: String,
    slug: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    // Stop the preview first if it belongs to this site. This is internal
    // cleanup, not a user command, so the machine is reset directly rather
    // than walked through Stopping.
    let preview_slug = state.preview_view.read().await.slug.clone();
    if preview_slug.as_deref() == Some(slug.as_str()) {
        if let Err(error) = astro::preview::stop_preview(&state).await {
            warn!(%slug, %error, "could not stop preview before delete");
        }
        let view = PreviewView {
            state: PreviewState::Stopped,
            slug: None,
            url: None,
        };
        *state.preview_view.write().await = view.clone();
        broadcast(&state, Event::PreviewChanged(view));
    }

    let entry = state
        .sites
        .read()
        .await
        .get(&slug)
        .map(|entry| (entry.name.clone(), entry.state.clone()));
    let Some((name, previous)) = entry else {
        send_error(
            &tx,
            correlation_id,
            ErrorCode::SiteNotFound,
            format!("site '{slug}' does not exist"),
        )
        .await;
        return;
    };

    match apply_site_event(&state, &slug, fsm::site::SiteEvent::DeleteRequested).await {
        Ok(new_state) => broadcast_site(&state, &slug, &name, new_state),
        Err(reply) => {
            send_reply(&tx, correlation_id, reply).await;
            return;
        }
    }

    match astro::sites::delete_site(&state.sites_dir, &slug) {
        Ok(()) => {
            state.sites.write().await.remove(&slug);
            broadcast(&state, Event::SiteRemoved { slug });
        }
        Err(error) => {
            // Deletion failed: restore the pre-delete state so the site is
            // not stuck in Deleting.
            if let Some(entry) = state.sites.write().await.get_mut(&slug) {
                entry.state = previous.clone();
            }
            broadcast_site(&state, &slug, &name, previous);
            send_error(
                &tx,
                correlation_id,
                astro_error_code(&error),
                error.to_string(),
            )
            .await;
        }
    }
}

// --- FSM application helpers --------------------------------------------------

/// An error destined for the requesting client only.
struct ErrorReply {
    code: ErrorCode,
    message: String,
}

async fn apply_site_event(
    state: &Arc<AppState>,
    slug: &str,
    event: fsm::site::SiteEvent,
) -> Result<SiteState, ErrorReply> {
    apply_site_event_named(state, slug, event)
        .await
        .map(|(_, new_state)| new_state)
}

/// Transition a site under the write lock; returns `(name, new_state)`.
async fn apply_site_event_named(
    state: &Arc<AppState>,
    slug: &str,
    event: fsm::site::SiteEvent,
) -> Result<(String, SiteState), ErrorReply> {
    let mut sites = state.sites.write().await;
    let entry = sites.get_mut(slug).ok_or_else(|| ErrorReply {
        code: ErrorCode::SiteNotFound,
        message: format!("site '{slug}' does not exist"),
    })?;
    let new_state = fsm::site::transition(entry.state.clone(), event).map_err(|e| ErrorReply {
        code: ErrorCode::InvalidTransition,
        message: e.to_string(),
    })?;
    entry.state = new_state.clone();
    Ok((entry.name.clone(), new_state))
}

/// Transition the preview under the write lock, updating slug/url alongside
/// the state so the view is always internally consistent.
async fn apply_preview_event(
    state: &Arc<AppState>,
    event: fsm::preview::PreviewEvent,
    slug: Option<String>,
    url: Option<String>,
) -> Result<PreviewView, ErrorReply> {
    let mut view = state.preview_view.write().await;
    let new_state =
        fsm::preview::transition(view.state.clone(), event).map_err(|e| ErrorReply {
            code: ErrorCode::InvalidTransition,
            message: e.to_string(),
        })?;
    *view = PreviewView {
        state: new_state,
        slug,
        url,
    };
    Ok(view.clone())
}

// --- outbound helpers ----------------------------------------------------------

fn broadcast(state: &Arc<AppState>, event: Event) {
    // Send fails only when no client is connected; state is still correct and
    // the next connection gets it via snapshot.
    let _ = state.events_tx.send(server_envelope(event));
}

fn broadcast_site(state: &Arc<AppState>, slug: &str, name: &str, site_state: SiteState) {
    broadcast(
        state,
        Event::SiteChanged(SiteView {
            slug: slug.to_string(),
            name: name.to_string(),
            state: site_state,
        }),
    );
}

async fn send_to_client(tx: &mpsc::Sender<WsEnvelope>, correlation_id: String, event: Event) {
    let _ = tx.send(envelope(correlation_id, event)).await;
}

async fn send_reply(tx: &mpsc::Sender<WsEnvelope>, correlation_id: String, reply: ErrorReply) {
    send_error(tx, correlation_id, reply.code, reply.message).await;
}

async fn send_error(
    tx: &mpsc::Sender<WsEnvelope>,
    correlation_id: String,
    code: ErrorCode,
    message: String,
) {
    warn!(?code, %message, "command failed");
    let event = Event::Error {
        code,
        message,
        correlation_id: Some(correlation_id.clone()),
    };
    let _ = tx.send(envelope(correlation_id, event)).await;
}

const fn astro_error_code(error: &AstroError) -> ErrorCode {
    match error {
        AstroError::SiteNotFound(_) => ErrorCode::SiteNotFound,
        AstroError::SiteAlreadyExists(_) => ErrorCode::SiteAlreadyExists,
        AstroError::PreviewAlreadyRunning(_) => ErrorCode::InvalidTransition,
        AstroError::DevServerTimeout(_) => ErrorCode::PreviewTimeout,
        AstroError::CommandFailed(_) => ErrorCode::BuildFailed,
        AstroError::Io(_) | AstroError::Json(_) => ErrorCode::Internal,
    }
}
