use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Site '{0}' does not exist")]
    SiteNotFound(String),

    #[error("Site '{0}' already exists")]
    SiteAlreadyExists(String),

    #[error("Dev server timed out: {0}")]
    DevServerTimeout(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
