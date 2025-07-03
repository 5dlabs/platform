//! Shared types and utilities for the Orchestrator project

pub mod error;
pub mod models;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Re-export commonly used types for convenience
pub use models::{AgentType, Job, JobStatus, JobType, Request, RequestSource, Task, TaskStatus};
