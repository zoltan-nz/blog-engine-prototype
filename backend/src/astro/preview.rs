use std::sync::Arc;
use std::time::Duration;

use crate::astro::error::AstroError;
use crate::state::{ActivePreview, AppState};
use process_wrap::tokio::{CommandWrap, ProcessGroup};
use tokio::process::ChildStdout;
use tokio::sync::oneshot;

/// A preview that passed its readiness check.
#[derive(Debug)]
pub struct StartedPreview {
    pub url: String,
    /// Resolves only if the dev server exits on its own (crash, killed from
    /// outside); dropped without firing when the preview is stopped.
    pub exited: oneshot::Receiver<()>,
}

/// Spawns `pnpm dev --port <port>` in the site's directory and waits until the
/// port accepts connections (max 30s).
///
/// Lock discipline: the preview mutex is held only while claiming the slot and
/// spawning the child, then released before the readiness poll. Holding it across
/// the (up to 30s) poll would block every other caller of the preview mutex —
/// e.g. `list_sites`. (rust-async: never hold a lock across an await you don't
/// need to.)
///
/// # Errors
/// - `PreviewAlreadyRunning` — another site is already being previewed
/// - `SiteNotFound` — `sites_dir/<slug>` does not exist
/// - `PortInUse` — something still listens on `port` after a short grace period
/// - `DevServerTimeout` — port never became reachable within the deadline
/// - `Io` — process spawn failed
///
/// # Panics
/// Never in practice: a just-spawned child always has a PID.
pub async fn start_preview(
    state: &Arc<AppState>,
    slug: &str,
    port: u16,
) -> Result<StartedPreview, AstroError> {
    let url = format!("http://localhost:{port}");
    let (exited_tx, exited) = oneshot::channel();
    let site_dir = state.sites_dir.join(slug);
    if !site_dir.exists() {
        return Err(AstroError::SiteNotFound(slug.to_string()));
    }
    // Cheap early answer; re-checked when claiming the slot below.
    if state.lock_preview().await.is_some() {
        return Err(AstroError::PreviewAlreadyRunning(slug.to_string()));
    }

    // The readiness probe only sees "something answers on the port", so a
    // server still on it (the previous preview finishing its exit, or a stray
    // one) would pass as ready and the iframe would show the wrong site.
    wait_for_port_free(port).await?;

    // Claim the slot + spawn under the lock, then drop it before the poll.
    {
        let mut guard = state.lock_preview().await;
        if guard.is_some() {
            return Err(AstroError::PreviewAlreadyRunning(slug.to_string()));
        }
        // `pnpm dev` spawns the real astro/vite dev server as a separate OS
        // process (not exec'd in place), so tracking just this PID and
        // killing it would orphan that grandchild. ProcessGroup::leader()
        // puts pnpm and everything it spawns in one group; kill() below then
        // signals the whole group.
        //
        // `--ignore-lock` keeps `astro dev` in the foreground: Astro 7.3 detects
        // AI agents (e.g. a `CLAUDECODE` env var) and otherwise detaches into
        // a daemon in its own session, which we could neither kill nor track.
        let mut command = CommandWrap::with_new("pnpm", |cmd| {
            cmd.args([
                "dev",
                "--port",
                &port.to_string(),
                "--host",
                "0.0.0.0",
                "--ignore-lock",
            ])
            .env_remove("PNPM_SCRIPT_SRC_DIR")
            .current_dir(&site_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        });
        command.wrap(ProcessGroup::leader());
        let mut child = command.spawn()?;
        let pid = child.id().expect("a just-spawned child has a PID");
        // Without this the dev server's output (and why it died) is lost.
        if let Some(out) = child.stdout().take() {
            tokio::spawn(watch_exit(
                Arc::clone(state),
                slug.to_string(),
                pid,
                out,
                exited_tx,
            ));
        }
        if let Some(err) = child.stderr().take() {
            tokio::spawn(log_lines(slug.to_string(), err));
        }
        *guard = Some(ActivePreview {
            slug: slug.to_string(),
            url: url.clone(),
            child,
        });
    }

    // Poll readiness without holding the lock.
    let ready = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
    .await;

    if ready.is_err() {
        // Roll back the slot we claimed (only if it's still ours), then kill the
        // child after releasing the lock.
        let orphaned = {
            let mut guard = state.lock_preview().await;
            if guard.as_ref().is_some_and(|p| p.slug == slug) {
                guard.take()
            } else {
                None
            }
        };
        if let Some(mut p) = orphaned {
            Box::into_pin(p.child.kill()).await.ok();
        }
        return Err(AstroError::DevServerTimeout(slug.to_string()));
    }

    tracing::info!(slug = %slug, port = port, url = %url, "preview started");
    Ok(StartedPreview { url, exited })
}

/// Kills the running preview process if one exists. Idempotent — returns
/// `Ok(())` even when nothing is running.
///
/// # Errors
pub async fn stop_preview(state: &Arc<AppState>) -> Result<(), AstroError> {
    // Take the child out under the lock, then release it before killing.
    let active = state.lock_preview().await.take();
    if let Some(mut preview) = active {
        tracing::info!(slug = %preview.slug, "stopping preview");
        if let Err(err) = preview.child.start_kill() {
            if !is_group_already_gone(&err) {
                return Err(err.into());
            }
            tracing::warn!(slug = %preview.slug, "preview had already exited");
        }
        // Reap the leader (and any group zombies) whether we killed it or not.
        preview.child.wait().await?;
    }
    Ok(())
}

/// How long a previous server gets to release the port before we give up.
const PORT_RELEASE_GRACE: Duration = Duration::from_secs(5);

async fn wait_for_port_free(port: u16) -> Result<(), AstroError> {
    tokio::time::timeout(PORT_RELEASE_GRACE, async {
        while tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| AstroError::PortInUse(port))
}

/// Logs the dev server's stdout until it closes, which happens once every
/// process holding it has exited. If the child is then still in the slot,
/// nobody stopped it: clear the slot, reap it, and fire `exited`.
async fn watch_exit(
    state: Arc<AppState>,
    slug: String,
    pid: u32,
    stdout: ChildStdout,
    exited: oneshot::Sender<()>,
) {
    log_lines(slug.clone(), stdout).await;

    let ours = {
        let mut guard = state.lock_preview().await;
        if guard.as_ref().is_some_and(|p| p.child.id() == Some(pid)) {
            guard.take()
        } else {
            None
        }
    };
    let Some(mut dead) = ours else { return };
    tracing::warn!(slug = %slug, "preview exited on its own");
    dead.child.wait().await.ok();
    let _ = exited.send(());
}

/// Forwards each output line of the dev server to the backend log. Ends when
/// the pipe closes, i.e. when the process exits.
async fn log_lines(slug: String, pipe: impl tokio::io::AsyncRead + Unpin) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(pipe).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!(slug = %slug, "preview: {line}");
    }
    tracing::info!(slug = %slug, "preview output closed");
}

/// POSIX errnos `killpg` returns for a group with nothing left to signal.
const EPERM: i32 = 1;
const ESRCH: i32 = 3;

/// `ESRCH`: the group no longer exists. `EPERM`: on macOS, `killpg` returns
/// this when every member of the group is an unreaped zombie, i.e. the
/// process already exited on its own.
fn is_group_already_gone(err: &std::io::Error) -> bool {
    matches!(err.raw_os_error(), Some(EPERM | ESRCH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as StdDuration;
    use tempfile::TempDir;

    /// Blocks until `pidfile` contains a parseable PID, or panics after ~2s.
    async fn read_pid_file(pidfile: &std::path::Path) -> i32 {
        for _ in 0..40 {
            if let Ok(contents) = std::fs::read_to_string(pidfile)
                && let Ok(pid) = contents.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(StdDuration::from_millis(50)).await;
        }
        panic!("pidfile never appeared: {}", pidfile.display());
    }

    fn process_is_alive(pid: i32) -> bool {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    }

    #[tokio::test]
    async fn stop_preview_kills_the_whole_process_group_not_just_the_leader() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        let pidfile = tmp.path().join("grandchild.pid");

        // `sh` is the tracked leader; `sleep` is a grandchild it backgrounds —
        // mirrors pnpm (leader) spawning the real astro/vite dev server
        // (grandchild) as a separate OS process.
        let mut command = process_wrap::tokio::CommandWrap::with_new("sh", |cmd| {
            cmd.arg("-c")
                .arg(format!("sleep 60 & echo $! > {}; wait", pidfile.display()));
        });
        command.wrap(process_wrap::tokio::ProcessGroup::leader());
        let child = command.spawn().unwrap();

        *state.lock_preview().await = Some(ActivePreview {
            slug: "test-site".to_string(),
            url: "http://localhost:1".to_string(),
            child,
        });

        let grandchild_pid = read_pid_file(&pidfile).await;
        assert!(
            process_is_alive(grandchild_pid),
            "precondition: grandchild running"
        );

        stop_preview(&state).await.unwrap();
        tokio::time::sleep(StdDuration::from_millis(200)).await;

        assert!(
            !process_is_alive(grandchild_pid),
            "grandchild process should be dead after stop_preview, not orphaned"
        );
    }

    fn make_state(sites_dir: &std::path::Path) -> Arc<AppState> {
        Arc::new(AppState::new(
            sites_dir,
            4321,
            Vec::<(String, String, Vec<crate::types::PostMeta>, Option<String>)>::new(),
        ))
    }

    #[tokio::test]
    async fn start_preview_returns_site_not_found_when_dir_missing() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());

        let err = start_preview(&state, "nonexistent", 4321)
            .await
            .unwrap_err();
        assert!(matches!(err, AstroError::SiteNotFound(_)));
    }

    #[tokio::test]
    async fn start_preview_returns_already_running_when_preview_active() {
        let tmp = TempDir::new().unwrap();
        // Create a placeholder site dir so it passes the existence check.
        std::fs::create_dir(tmp.path().join("my-site")).unwrap();
        let state = make_state(tmp.path());

        // Simulate a running preview by putting something in the mutex.
        let fake_child = CommandWrap::with_new("sleep", |cmd| {
            cmd.arg("100");
        })
        .spawn()
        .unwrap();
        *state.lock_preview().await = Some(ActivePreview {
            slug: "other-site".to_string(),
            url: "http://localhost:4321".to_string(),
            child: fake_child,
        });

        let err = start_preview(&state, "my-site", 4321).await.unwrap_err();
        assert!(matches!(err, AstroError::PreviewAlreadyRunning(_)));

        // Cleanup — kill the sleep process. Bind the taken value first so the
        // MutexGuard temporary drops before the await.
        let running = state.lock_preview().await.take();
        if let Some(mut p) = running {
            Box::into_pin(p.child.kill()).await.ok();
        }
    }

    /// Spawns `sh -c script` in its own group with stdout piped, puts it in
    /// the preview slot, and returns its stdout and PID.
    async fn occupy_slot(state: &Arc<AppState>, script: &str) -> (ChildStdout, u32) {
        let mut command = CommandWrap::with_new("sh", |cmd| {
            cmd.arg("-c")
                .arg(script)
                .stdout(std::process::Stdio::piped());
        });
        command.wrap(ProcessGroup::leader());
        let mut child = command.spawn().unwrap();
        let stdout = child.stdout().take().unwrap();
        let pid = child.id().unwrap();
        *state.lock_preview().await = Some(ActivePreview {
            slug: "test-site".to_string(),
            url: "http://localhost:1".to_string(),
            child,
        });
        (stdout, pid)
    }

    #[tokio::test]
    async fn watch_exit_reports_a_server_that_exits_on_its_own() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        let (stdout, pid) = occupy_slot(&state, "echo crashing").await;
        let (tx, rx) = tokio::sync::oneshot::channel();

        watch_exit(Arc::clone(&state), "test-site".to_string(), pid, stdout, tx).await;

        assert!(rx.await.is_ok(), "an unexpected exit is reported");
        assert!(
            state.lock_preview().await.is_none(),
            "the dead child is cleared"
        );
    }

    #[tokio::test]
    async fn watch_exit_stays_quiet_when_the_preview_was_stopped() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        let (stdout, pid) = occupy_slot(&state, "sleep 60").await;
        let (tx, rx) = tokio::sync::oneshot::channel();

        let watcher = tokio::spawn(watch_exit(
            Arc::clone(&state),
            "test-site".to_string(),
            pid,
            stdout,
            tx,
        ));
        stop_preview(&state).await.unwrap();
        watcher.await.unwrap();

        assert!(rx.await.is_err(), "a requested stop is not an exit");
    }

    #[tokio::test]
    async fn start_preview_refuses_a_port_another_server_holds() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("my-site")).unwrap();
        let state = make_state(tmp.path());

        // A stale server on the port would pass the readiness probe and the
        // iframe would show the wrong site.
        let squatter = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = squatter.local_addr().unwrap().port();

        let err = start_preview(&state, "my-site", port).await.unwrap_err();
        assert!(matches!(err, AstroError::PortInUse(p) if p == port));
        assert!(state.lock_preview().await.is_none());
    }

    #[tokio::test]
    async fn stop_preview_succeeds_when_the_preview_already_exited() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());

        // Leader exits immediately and is never reaped: the group holds only a
        // zombie, which is what a crashed `pnpm dev` leaves behind.
        let mut command = CommandWrap::with_new("true", |_| {});
        command.wrap(ProcessGroup::leader());
        let child = command.spawn().unwrap();
        tokio::time::sleep(StdDuration::from_millis(300)).await;

        *state.lock_preview().await = Some(ActivePreview {
            slug: "test-site".to_string(),
            url: "http://localhost:1".to_string(),
            child,
        });

        stop_preview(&state).await.unwrap();
        assert!(state.lock_preview().await.is_none());
    }

    #[tokio::test]
    async fn stop_preview_is_idempotent_when_nothing_running() {
        let tmp = TempDir::new().unwrap();
        let state = make_state(tmp.path());
        stop_preview(&state).await.unwrap();
    }
}
