use crate::astro::posts;
use crate::state::AppState;
use crate::types::Event;
use crate::ws::dispatch::server_envelope;
#[cfg(test)]
use notify::Config;
#[cfg(test)]
use notify::PollWatcher;
#[cfg(not(test))]
use notify::RecommendedWatcher;
use notify::RecursiveMode;
#[cfg(not(test))]
use notify_debouncer_full::new_debouncer;
#[cfg(test)]
use notify_debouncer_full::new_debouncer_opt;
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const DEBOUNCE_TIMEOUT: Duration = Duration::from_millis(250);

/// Keeps the debouncer alive. Dropping this stops the OS watch for the site.
#[cfg(not(test))]
type WatcherBackend = RecommendedWatcher;
#[cfg(test)]
type WatcherBackend = PollWatcher;

pub struct SiteWatcher(#[allow(dead_code)] Debouncer<WatcherBackend, RecommendedCache>);

/// Start watching `{site_dir}/src/content/blog/` with a 250 ms debounce.
///
/// On any relevant `.md` change, the callback rescans the content dir,
/// diffs against the cached post list in `AppState`, updates the cache, and
/// emits `PostChanged`/`PostRemoved` into `events_tx`.
///
/// The notify callback runs on a native OS thread (sync). Both
/// `broadcast::Sender::send` and `RwLock::blocking_write` are sync-safe here
/// because the tokio multi-thread runtime is running at this point.
///
/// # Errors
///
/// Propagates any `notify` error from constructing the debouncer or starting
/// the watch on the content directory.
pub fn start_site_watcher(
    site_dir: PathBuf,
    site_slug: String,
    state: Arc<AppState>,
) -> notify::Result<SiteWatcher> {
    let content_dir = site_dir.join(posts::CONTENT_DIR);

    let callback = move |result: DebounceEventResult| {
        let Ok(events) = result else { return };

        // The backend's post CRUD and scanner use .md files only.
        let relevant = events.iter().any(|e| {
            e.paths
                .iter()
                .any(|p| p.extension().and_then(|ext| ext.to_str()) == Some("md"))
        });
        if !relevant {
            return;
        }

        // Rescan. If the directory is gone, treat as empty.
        let current = posts::list_posts(&site_dir).unwrap_or_default();

        let mut sites = state.sites.blocking_write();
        let Some(entry) = sites.get_mut(&site_slug) else {
            return;
        };

        // Posts new or modified since last scan → PostChanged.
        for post in &current {
            let changed = entry
                .posts
                .iter()
                .find(|old| old.id == post.id)
                .is_none_or(|old| old != post);
            if changed {
                let _ = state.events_tx.send(server_envelope(Event::PostChanged {
                    site_slug: site_slug.clone(),
                    post: post.clone(),
                }));
            }
        }

        // Posts that no longer exist → PostRemoved.
        for old in &entry.posts {
            if !current.iter().any(|p| p.id == old.id) {
                let _ = state.events_tx.send(server_envelope(Event::PostRemoved {
                    site_slug: site_slug.clone(),
                    id: old.id.clone(),
                }));
            }
        }

        entry.posts = current;
    };

    #[cfg(not(test))]
    let mut debouncer = new_debouncer(DEBOUNCE_TIMEOUT, None, callback)?;

    #[cfg(test)]
    let mut debouncer = new_debouncer_opt::<_, PollWatcher, RecommendedCache>(
        DEBOUNCE_TIMEOUT,
        None,
        callback,
        RecommendedCache::new(),
        Config::default()
            .with_poll_interval(Duration::from_millis(50))
            .with_compare_contents(true),
    )?;

    debouncer.watch(&content_dir, RecursiveMode::NonRecursive)?;
    Ok(SiteWatcher(debouncer))
}

#[cfg(test)]
mod tests {
    use super::start_site_watcher;
    use crate::astro::posts::{self, CONTENT_DIR};
    use crate::state::AppState;
    use crate::types::{Event, WsMessage};
    use std::fs;
    use std::sync::Arc;
    use tempfile::TempDir;

    const POST_CONTENT: &str =
        "---\ntitle: Hello\ndescription: Description\npubDate: 2026-01-01\n---\nBody\n";

    fn state_for(site: &TempDir, posts: Vec<crate::types::PostMeta>) -> Arc<AppState> {
        Arc::new(AppState::new(
            site.path(),
            4321,
            vec![("my-blog".to_string(), "My Blog".to_string(), posts)],
        ))
    }

    fn post_path(site: &TempDir) -> std::path::PathBuf {
        site.path().join(CONTENT_DIR).join("hello.md")
    }

    async fn receive_event(
        receiver: &mut tokio::sync::broadcast::Receiver<crate::types::WsEnvelope>,
    ) -> Event {
        let envelope = tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv())
            .await
            .expect("watcher event timed out")
            .expect("watcher event channel closed");
        match envelope.message {
            WsMessage::Event(event) => event,
            other @ WsMessage::Command(_) => panic!("expected event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn watcher_broadcasts_new_post() {
        let site = TempDir::new().unwrap();
        fs::create_dir_all(site.path().join(CONTENT_DIR)).unwrap();
        let state = state_for(&site, vec![]);
        let mut receiver = state.events_tx.subscribe();
        let _watcher = start_site_watcher(
            site.path().to_path_buf(),
            "my-blog".to_string(),
            Arc::clone(&state),
        )
        .unwrap();

        fs::write(post_path(&site), POST_CONTENT).unwrap();

        assert!(matches!(
            receive_event(&mut receiver).await,
            Event::PostChanged { post, .. } if post.id == "hello"
        ));
    }

    #[tokio::test]
    async fn watcher_broadcasts_changed_post_metadata() {
        let site = TempDir::new().unwrap();
        fs::create_dir_all(site.path().join(CONTENT_DIR)).unwrap();
        fs::write(post_path(&site), POST_CONTENT).unwrap();
        let cached = posts::list_posts(site.path()).unwrap();
        let state = state_for(&site, cached);
        let mut receiver = state.events_tx.subscribe();
        let _watcher = start_site_watcher(
            site.path().to_path_buf(),
            "my-blog".to_string(),
            Arc::clone(&state),
        )
        .unwrap();

        fs::write(
            post_path(&site),
            POST_CONTENT.replace("title: Hello", "title: Changed"),
        )
        .unwrap();

        assert!(matches!(
            receive_event(&mut receiver).await,
            Event::PostChanged { post, .. } if post.title == "Changed"
        ));
    }

    #[tokio::test]
    async fn watcher_broadcasts_removed_post() {
        let site = TempDir::new().unwrap();
        fs::create_dir_all(site.path().join(CONTENT_DIR)).unwrap();
        fs::write(post_path(&site), POST_CONTENT).unwrap();
        let cached = posts::list_posts(site.path()).unwrap();
        let state = state_for(&site, cached);
        let mut receiver = state.events_tx.subscribe();
        let _watcher = start_site_watcher(
            site.path().to_path_buf(),
            "my-blog".to_string(),
            Arc::clone(&state),
        )
        .unwrap();

        fs::remove_file(post_path(&site)).unwrap();

        assert!(matches!(
            receive_event(&mut receiver).await,
            Event::PostRemoved { id, .. } if id == "hello"
        ));
    }
}
