//! Core data models module

pub mod code_request;
pub mod config;
pub mod docs_request;
pub mod job;
pub mod pm_task;
pub mod request;
pub mod response;
pub mod task;
pub mod webhook;

// Re-export commonly used types
pub use code_request::CodeRequest;
pub use config::{AgentConfig, AgentType, ResourceLimits};
pub use docs_request::DocsRequest;
pub use job::{Job, JobSpec, JobStatus, JobType};
pub use pm_task::{
    DocsGenerationRequest, MarkdownPayload, PmTaskRequest, Subtask, Task as PmTask, TaskMaster,
    TaskMasterFile,
};
pub use request::{ParsedRequest, Request, RequestAction, RequestSource};
pub use response::{ApiResponse, JobResponse, TaskResponse};
pub use task::{Task, TaskMetadata, TaskStatus};
pub use webhook::{GitHubWebhookPayload, GrafanaAlert, WebhookPayload};
