use thiserror::Error;

/// Errors from in-process Astro site and preview management.
/// (Was the supervisor's `AgentError`; same variants.)
#[derive(Error, Debug)]
pub enum AstroError {
    #[error("Site '{0}' does not exist")]
    SiteNotFound(String),

    #[error("Site '{0}' already exists")]
    SiteAlreadyExists(String),

    #[error("Preview already running for site '{0}'")]
    PreviewAlreadyRunning(String),

    #[error("Dev server timed out: {0}")]
    DevServerTimeout(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Post content is not valid: missing '---' frontmatter delimiters")]
    InvalidPostFormat,

    #[error("Post '{0}' does not exist")]
    PostNotFound(String),

    #[error("Post '{0}' already exists")]
    PostAlreadyExists(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml_ng::Error),
}
