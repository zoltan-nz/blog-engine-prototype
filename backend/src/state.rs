use crate::types::{PreviewState, PreviewView, SiteState, WsEnvelope};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::{Mutex, MutexGuard, RwLock, broadcast};

/// Broadcast capacity: slow readers lag and skip ahead (see `ws::socket`),
/// so this only needs to absorb bursts like build-log floods.
const EVENTS_CHANNEL_CAPACITY: usize = 256;

/// A running `pnpm dev` preview process. Owns the child so it can be killed on
/// stop.
pub struct ActivePreview {
    pub slug: String,
    pub url: String,
    pub child: tokio::process::Child,
}

/// A managed site: display name plus current FSM state. Existence in the map
/// is existence in the system; `sites.json` persists identity across restarts.
#[derive(Debug, Clone)]
pub struct SiteEntry {
    pub name: String,
    pub state: SiteState,
}

pub struct AppState {
    pub sites_dir: PathBuf,
    pub preview_port: u16,
    /// Site FSM states, keyed by slug.
    pub sites: RwLock<HashMap<String, SiteEntry>>,
    /// Preview FSM state as the frontend sees it (state + slug + url).
    pub preview_view: RwLock<PreviewView>,
    /// Server → client fan-out; every WS connection subscribes.
    pub events_tx: broadcast::Sender<WsEnvelope>,
    /// At most one preview runs at a time; the mutex serialises start/stop.
    preview: Mutex<Option<ActivePreview>>,
}

impl AppState {
    /// `initial_sites` hydrates the FSM map from the manifest at startup; all
    /// pre-existing sites start `Ready`.
    pub fn new(
        sites_dir: impl Into<PathBuf>,
        preview_port: u16,
        initial_sites: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        let sites = initial_sites
            .into_iter()
            .map(|(slug, name)| {
                (
                    slug,
                    SiteEntry {
                        name,
                        state: SiteState::Ready,
                    },
                )
            })
            .collect();

        let (events_tx, _) = broadcast::channel(EVENTS_CHANNEL_CAPACITY);

        Self {
            sites_dir: sites_dir.into(),
            preview_port,
            sites: RwLock::new(sites),
            preview_view: RwLock::new(PreviewView {
                state: PreviewState::Stopped,
                slug: None,
                url: None,
            }),
            events_tx,
            preview: Mutex::new(None),
        }
    }

    pub async fn lock_preview(&self) -> MutexGuard<'_, Option<ActivePreview>> {
        self.preview.lock().await
    }
}
