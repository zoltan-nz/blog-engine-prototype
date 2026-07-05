use crate::astro::error::AstroError;
use crate::types::LogStream;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// Runs `pnpm build` in `site_dir`, streaming stdout/stderr lines through
/// `log_tx` as they arrive (the dispatcher forwards them as `BuildLog`
/// events). Returns once the process exits.
pub async fn build_site(
    site_dir: &Path,
    log_tx: mpsc::UnboundedSender<(LogStream, String)>,
) -> Result<(), AstroError> {
    let mut child = tokio::process::Command::new("pnpm")
        .args(["build"])
        .current_dir(site_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .expect("stdout piped above, always present");
    let stderr = child
        .stderr
        .take()
        .expect("stderr piped above, always present");

    let out_task = forward_lines(stdout, LogStream::Stdout, log_tx.clone());
    let err_task = forward_lines(stderr, LogStream::Stderr, log_tx);

    // Drain both pipes concurrently so neither can fill up and stall the child.
    let (status, (), ()) = tokio::join!(child.wait(), out_task, err_task);

    let status = status?;
    if status.success() {
        Ok(())
    } else {
        Err(AstroError::CommandFailed(format!(
            "pnpm build exited with {status}"
        )))
    }
}

async fn forward_lines(
    reader: impl tokio::io::AsyncRead + Unpin,
    stream: LogStream,
    log_tx: mpsc::UnboundedSender<(LogStream, String)>,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        // Receiver gone means the dispatcher stopped listening; keep draining
        // the pipe so the child is not blocked on a full buffer.
        let _ = log_tx.send((stream, line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn build_site_fails_in_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();

        let result = build_site(tmp.path(), tx).await;
        assert!(result.is_err());
    }
}
