use thiserror::Error;

/**
 * User-facing failure while opening a URL with the configured browser.
 */
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UrlOpenError {
    #[error("opening URLs is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("open_url.command is empty")]
    EmptyCommand,
    #[error("URL opener {tool} was not found")]
    NoToolFound { tool: String },
    #[error("failed to start {tool}: {message}")]
    SpawnFailed { tool: String, message: String },
    #[error("failed waiting for {tool}: {message}")]
    WaitFailed { tool: String, message: String },
    #[error("{tool} exited with status {status}")]
    CommandFailed { tool: String, status: String },
}
