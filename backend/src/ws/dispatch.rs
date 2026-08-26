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
    let sites_guard = state.sites.read().await;

    let mut sites: Vec<SiteView> = sites_guard
        .iter()
        .map(|(slug, entry)| SiteView {
            slug: slug.clone(),
            name: entry.name.clone(),
            state: entry.state.clone(),
        })
        .collect();
    sites.sort_by(|a, b| a.slug.cmp(&b.slug));

    let posts = sites_guard
        .iter()
        .map(|(slug, entry)| (slug.clone(), entry.posts.clone()))
        .collect();
    drop(sites_guard);

    let preview = state.preview_view.read().await.clone();
    Event::Snapshot {
        sites,
        preview,
        posts,
    }
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
        Command::CreatePost {
            site_slug,
            id,
            title,
            description,
            body,
        } => {
            create_post(
                correlation_id,
                site_slug,
                id,
                title,
                description,
                body,
                tx,
                state,
            )
            .await;
        }
        Command::UpdatePost {
            site_slug,
            id,
            title,
            description,
            body,
        } => {
            update_post(
                correlation_id,
                site_slug,
                id,
                title,
                description,
                body,
                tx,
                state,
            )
            .await;
        }
        Command::DeletePost { site_slug, id } => {
            delete_post(correlation_id, site_slug, id, tx, state).await;
        }
        Command::GetPost { site_slug, id } => {
            get_post(correlation_id, site_slug, id, tx, state).await;
        }
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
                posts: Vec::new(),
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

// --- post command flows --------------------------------------------------------

/// Confirms `site_slug` is a known site and `id` is a safe path segment,
/// replying `InvalidInput`/`SiteNotFound` and returning `None` otherwise.
/// Every post command arm runs this before touching the filesystem.
async fn validated_site_dir(
    state: &Arc<AppState>,
    tx: &mpsc::Sender<WsEnvelope>,
    correlation_id: &str,
    site_slug: &str,
    id: &str,
) -> Option<std::path::PathBuf> {
    if !validate_id(site_slug) || !validate_id(id) {
        send_error(
            tx,
            correlation_id.to_string(),
            ErrorCode::InvalidInput,
            "invalid site_slug or id".to_string(),
        )
        .await;
        return None;
    }
    if !state.sites.read().await.contains_key(site_slug) {
        send_error(
            tx,
            correlation_id.to_string(),
            ErrorCode::SiteNotFound,
            format!("site '{site_slug}' does not exist"),
        )
        .await;
        return None;
    }
    Some(state.sites_dir.join(site_slug))
}

/// Replaces this post's cache entry (or inserts it), keeping the site's post
/// list sorted `pub_date` descending — the same order every `Snapshot` and
/// `list_posts` scan guarantees.
async fn upsert_post_cache(state: &Arc<AppState>, site_slug: &str, post: &crate::types::PostMeta) {
    if let Some(entry) = state.sites.write().await.get_mut(site_slug) {
        entry.posts.retain(|p| p.id != post.id);
        entry.posts.push(post.clone());
        entry.posts.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
    }
}

async fn remove_post_cache(state: &Arc<AppState>, site_slug: &str, id: &str) {
    if let Some(entry) = state.sites.write().await.get_mut(site_slug) {
        entry.posts.retain(|p| p.id != id);
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_post(
    correlation_id: String,
    site_slug: String,
    id: String,
    title: String,
    description: String,
    body: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    let Some(site_dir) = validated_site_dir(&state, &tx, &correlation_id, &site_slug, &id).await
    else {
        return;
    };

    let pub_date = chrono::Local::now().date_naive().to_string();
    match astro::posts::create_post(&site_dir, &id, &title, &description, &body, &pub_date) {
        Ok(post) => {
            upsert_post_cache(&state, &site_slug, &post).await;
            broadcast(&state, Event::PostChanged { site_slug, post });
        }
        Err(error) => {
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

#[allow(clippy::too_many_arguments)]
async fn update_post(
    correlation_id: String,
    site_slug: String,
    id: String,
    title: String,
    description: String,
    body: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    let Some(site_dir) = validated_site_dir(&state, &tx, &correlation_id, &site_slug, &id).await
    else {
        return;
    };

    let updated_date = chrono::Local::now().date_naive().to_string();
    match astro::posts::update_post(&site_dir, &id, &title, &description, &body, &updated_date) {
        Ok(post) => {
            upsert_post_cache(&state, &site_slug, &post).await;
            broadcast(&state, Event::PostChanged { site_slug, post });
        }
        Err(error) => {
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

async fn delete_post(
    correlation_id: String,
    site_slug: String,
    id: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    let Some(site_dir) = validated_site_dir(&state, &tx, &correlation_id, &site_slug, &id).await
    else {
        return;
    };

    match astro::posts::delete_post(&site_dir, &id) {
        Ok(()) => {
            remove_post_cache(&state, &site_slug, &id).await;
            broadcast(&state, Event::PostRemoved { site_slug, id });
        }
        Err(error) => {
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

async fn get_post(
    correlation_id: String,
    site_slug: String,
    id: String,
    tx: mpsc::Sender<WsEnvelope>,
    state: Arc<AppState>,
) {
    let Some(site_dir) = validated_site_dir(&state, &tx, &correlation_id, &site_slug, &id).await
    else {
        return;
    };

    match astro::posts::read_post(&site_dir, &id) {
        Ok((_, body)) => {
            send_to_client(
                &tx,
                correlation_id,
                Event::PostBody {
                    site_slug,
                    id,
                    body,
                },
            )
            .await;
        }
        Err(error) => {
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
        AstroError::Io(_)
        | AstroError::Json(_)
        | AstroError::InvalidPostFormat
        | AstroError::Yaml(_) => ErrorCode::Internal,
        AstroError::PostNotFound(_) => ErrorCode::PostNotFound,
        AstroError::PostAlreadyExists(_) => ErrorCode::PostAlreadyExists,
    }
}

/// Checks a client-supplied post or site id against `^[a-z0-9-]{1,64}$`
/// before it is joined into any filesystem path — no regex crate needed for
/// a character class this simple.
fn validate_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PostMeta;
    use tempfile::TempDir;

    fn make_state_with_site(sites_dir: &std::path::Path, slug: &str) -> Arc<AppState> {
        std::fs::create_dir_all(sites_dir.join(slug)).unwrap();
        Arc::new(AppState::new(
            sites_dir,
            4321,
            vec![(slug.to_string(), "My Blog".to_string(), Vec::new())],
        ))
    }

    fn recv_broadcast(state: &Arc<AppState>) -> tokio::sync::broadcast::Receiver<WsEnvelope> {
        state.events_tx.subscribe()
    }

    #[tokio::test]
    async fn create_post_broadcasts_post_changed_and_updates_cache() {
        let sites = TempDir::new().unwrap();
        let state = make_state_with_site(sites.path(), "my-blog");
        let mut broadcast_rx = recv_broadcast(&state);
        let (tx, _rx) = mpsc::channel(8);

        dispatch_command(
            "c-1".to_string(),
            Command::CreatePost {
                site_slug: "my-blog".to_string(),
                id: "hello-world".to_string(),
                title: "Hello World".to_string(),
                description: "desc".to_string(),
                body: "body\n".to_string(),
            },
            tx,
            Arc::clone(&state),
        )
        .await;

        let envelope = broadcast_rx.recv().await.unwrap();
        match envelope.message {
            WsMessage::Event(Event::PostChanged { site_slug, post }) => {
                assert_eq!(site_slug, "my-blog");
                assert_eq!(post.id, "hello-world");
                assert_eq!(post.title, "Hello World");
            }
            other => panic!("expected PostChanged, got {other:?}"),
        }

        let cache = state.sites.read().await;
        let posts = &cache.get("my-blog").unwrap().posts;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "hello-world");
    }

    #[tokio::test]
    async fn create_post_with_traversal_id_sends_invalid_input_and_writes_nothing() {
        let sites = TempDir::new().unwrap();
        let state = make_state_with_site(sites.path(), "my-blog");
        let (tx, mut rx) = mpsc::channel(8);

        dispatch_command(
            "c-2".to_string(),
            Command::CreatePost {
                site_slug: "my-blog".to_string(),
                id: "../evil".to_string(),
                title: "Hello".to_string(),
                description: "desc".to_string(),
                body: "body".to_string(),
            },
            tx,
            Arc::clone(&state),
        )
        .await;

        let envelope = rx.recv().await.unwrap();
        match envelope.message {
            WsMessage::Event(Event::Error { code, .. }) => {
                assert_eq!(code, ErrorCode::InvalidInput);
            }
            other => panic!("expected Error, got {other:?}"),
        }
        assert!(
            !sites
                .path()
                .join("my-blog")
                .join(astro::posts::CONTENT_DIR)
                .exists()
        );
    }

    #[tokio::test]
    async fn get_post_replies_post_body_to_requester_only() {
        let sites = TempDir::new().unwrap();
        let state = make_state_with_site(sites.path(), "my-blog");
        astro::posts::create_post(
            &sites.path().join("my-blog"),
            "hello-world",
            "Hello World",
            "desc",
            "the body\n",
            "2026-01-01",
        )
        .unwrap();
        let mut broadcast_rx = recv_broadcast(&state);
        let (tx, mut rx) = mpsc::channel(8);

        dispatch_command(
            "c-3".to_string(),
            Command::GetPost {
                site_slug: "my-blog".to_string(),
                id: "hello-world".to_string(),
            },
            tx,
            Arc::clone(&state),
        )
        .await;

        let envelope = rx.recv().await.unwrap();
        match envelope.message {
            WsMessage::Event(Event::PostBody {
                site_slug,
                id,
                body,
            }) => {
                assert_eq!(site_slug, "my-blog");
                assert_eq!(id, "hello-world");
                assert_eq!(body, "the body\n");
            }
            other => panic!("expected PostBody, got {other:?}"),
        }
        assert!(broadcast_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn update_post_missing_id_sends_post_not_found() {
        let sites = TempDir::new().unwrap();
        let state = make_state_with_site(sites.path(), "my-blog");
        let (tx, mut rx) = mpsc::channel(8);

        dispatch_command(
            "c-4".to_string(),
            Command::UpdatePost {
                site_slug: "my-blog".to_string(),
                id: "ghost".to_string(),
                title: "New".to_string(),
                description: "d".to_string(),
                body: "b".to_string(),
            },
            tx,
            Arc::clone(&state),
        )
        .await;

        let envelope = rx.recv().await.unwrap();
        match envelope.message {
            WsMessage::Event(Event::Error { code, .. }) => {
                assert_eq!(code, ErrorCode::PostNotFound);
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_post_broadcasts_post_removed() {
        let sites = TempDir::new().unwrap();
        let state = make_state_with_site(sites.path(), "my-blog");
        astro::posts::create_post(
            &sites.path().join("my-blog"),
            "hello-world",
            "Hello World",
            "desc",
            "body",
            "2026-01-01",
        )
        .unwrap();
        {
            let mut cache = state.sites.write().await;
            cache.get_mut("my-blog").unwrap().posts =
                astro::posts::list_posts(&sites.path().join("my-blog")).unwrap();
        }
        let mut broadcast_rx = recv_broadcast(&state);
        let (tx, _rx) = mpsc::channel(8);

        dispatch_command(
            "c-5".to_string(),
            Command::DeletePost {
                site_slug: "my-blog".to_string(),
                id: "hello-world".to_string(),
            },
            tx,
            Arc::clone(&state),
        )
        .await;

        let envelope = broadcast_rx.recv().await.unwrap();
        match envelope.message {
            WsMessage::Event(Event::PostRemoved { site_slug, id }) => {
                assert_eq!(site_slug, "my-blog");
                assert_eq!(id, "hello-world");
            }
            other => panic!("expected PostRemoved, got {other:?}"),
        }
        let cache = state.sites.read().await;
        assert!(cache.get("my-blog").unwrap().posts.is_empty());
    }

    #[tokio::test]
    async fn snapshot_includes_hydrated_posts_sorted_desc() {
        let sites = TempDir::new().unwrap();
        std::fs::create_dir_all(sites.path().join("my-blog")).unwrap();
        let posts = vec![
            PostMeta {
                id: "newer".into(),
                title: "Newer".into(),
                description: "d".into(),
                pub_date: "2026-06-01".into(),
                updated_date: None,
            },
            PostMeta {
                id: "older".into(),
                title: "Older".into(),
                description: "d".into(),
                pub_date: "2026-01-01".into(),
                updated_date: None,
            },
        ];
        let state = Arc::new(AppState::new(
            sites.path(),
            4321,
            vec![("my-blog".to_string(), "My Blog".to_string(), posts)],
        ));

        let event = snapshot_event(&state).await;

        match event {
            Event::Snapshot { posts, .. } => {
                let my_blog_posts = &posts["my-blog"];
                assert_eq!(my_blog_posts[0].id, "newer");
                assert_eq!(my_blog_posts[1].id, "older");
            }
            other => panic!("expected Snapshot, got {other:?}"),
        }
    }

    #[test]
    fn validate_id_accepts_slug_chars() {
        assert!(validate_id("hello-world-123"));
    }

    #[test]
    fn validate_id_rejects_traversal() {
        assert!(!validate_id("../x"));
    }

    #[test]
    fn validate_id_rejects_empty_and_overlong() {
        assert!(!validate_id(""));
        assert!(!validate_id(&"a".repeat(65)));
        assert!(validate_id(&"a".repeat(64)));
    }

    #[test]
    fn validate_id_rejects_uppercase_and_dots() {
        assert!(!validate_id("Hello"));
        assert!(!validate_id("hello.world"));
    }
}
