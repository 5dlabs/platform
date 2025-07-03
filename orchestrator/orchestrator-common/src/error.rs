//! Common error types for the orchestrator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes operation failed: {0}")]
    Kubernetes(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Job failed: {0}")]
    JobFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversion from anyhow::Error for easier error handling
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}
