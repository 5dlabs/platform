use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use kube::{Client, ResourceExt};
use std::sync::Arc;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Missing object key")]
    MissingObjectKey,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Context shared across controller operations
pub(crate) struct Context {
    pub client: Client,
    pub namespace: String,
    pub config: Arc<ControllerConfig>,
}

// Finalizer names for cleanup
pub(crate) const DOCS_FINALIZER_NAME: &str = "docsruns.orchestrator.io/finalizer";
pub(crate) const CODE_FINALIZER_NAME: &str = "coderuns.orchestrator.io/finalizer";

// Enum to represent either task type for shared functionality
pub enum TaskType {
    Docs(Arc<DocsRun>),
    Code(Arc<CodeRun>),
}

impl TaskType {
    pub fn name(&self) -> String {
        match self {
            TaskType::Docs(dr) => dr.name_any(),
            TaskType::Code(cr) => cr.name_any(),
        }
    }

    pub fn is_docs(&self) -> bool {
        matches!(self, TaskType::Docs(_))
    }

    pub fn service_name(&self) -> &str {
        match self {
            TaskType::Docs(_) => "docs-generator", // Fixed service name for docs
            TaskType::Code(cr) => &cr.spec.service,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.model,
            TaskType::Code(cr) => &cr.spec.model,
        }
    }

    pub fn github_user(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.github_user,
            TaskType::Code(cr) => &cr.spec.github_user,
        }
    }

    pub fn repository_url(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.repository_url,
            TaskType::Code(cr) => &cr.spec.repository_url,
        }
    }

    pub fn working_directory(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.working_directory,
            TaskType::Code(cr) => &cr.spec.working_directory,
        }
    }

    pub fn task_id(&self) -> Option<u32> {
        match self {
            TaskType::Docs(_) => None, // Docs generation doesn't have a specific task ID
            TaskType::Code(cr) => Some(cr.spec.task_id),
        }
    }

    pub fn branch(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.source_branch,
            TaskType::Code(cr) => &cr.spec.branch,
        }
    }

    /// Get retry/versioning information for CodeRun (docs don't have retries)
    pub fn context_version(&self) -> Option<u32> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.context_version,
        }
    }

    pub fn retry_count(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 0, // Docs don't retry
            TaskType::Code(cr) => cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.session_id.as_deref()),
        }
    }

    pub fn prompt_modification(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.prompt_modification.as_deref(),
        }
    }

    pub fn prompt_mode(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.prompt_mode.as_deref(),
        }
    }

    /// Get tool configuration for the task
    pub fn local_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.local_tools.as_deref(),
        }
    }

    pub fn remote_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.remote_tools.as_deref(),
        }
    }

    pub fn tool_config(&self) -> &str {
        match self {
            TaskType::Docs(_) => "default", // Docs use default config
            TaskType::Code(cr) => &cr.spec.tool_config,
        }
    }

    /// Get platform repository info (only for CodeRun)
    pub fn platform_repository_url(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => Some(&cr.spec.platform_repository_url),
        }
    }

    /// Check if this is using SSH authentication
    pub fn uses_ssh(&self) -> bool {
        let url = self.repository_url();
        url.starts_with("git@") || url.starts_with("ssh://")
    }

    /// Get SSH secret name for this GitHub user
    pub fn ssh_secret_name(&self) -> String {
        format!("github-ssh-{}", self.github_user())
    }
}