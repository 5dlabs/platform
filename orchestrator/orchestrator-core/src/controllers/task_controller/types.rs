use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use kube::{Client, ResourceExt};
use std::sync::Arc;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
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

    /// Get working directory (defaults to service name if not specified)
    pub fn working_directory(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.working_directory,
            TaskType::Code(cr) => {
                // Default to service name if working_directory is None or empty
                match &cr.spec.working_directory {
                    Some(wd) if !wd.is_empty() => wd,
                    _ => &cr.spec.service,
                }
            },
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

    /// Get retry/versioning information for `CodeRun` (docs don't have retries)
    pub fn context_version(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 1, // Docs don't have context versions
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

    pub fn prompt_mode(&self) -> &str {
        match self {
            TaskType::Docs(_) => "direct", // Docs use direct mode
            TaskType::Code(cr) => &cr.spec.prompt_mode,
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

    /// Get docs repository info (only for CodeRun)
    pub fn docs_repository_url(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => Some(&cr.spec.docs_repository_url),
        }
    }

    /// Always use SSH authentication (we're SSH-only now)
    pub fn uses_ssh(&self) -> bool {
        true
    }

    /// Get SSH secret name for this GitHub user
    pub fn ssh_secret_name(&self) -> String {
        format!("github-ssh-{}", self.github_user())
    }

    /// Get docs branch (only for CodeRun)
    pub fn docs_branch(&self) -> &str {
        match self {
            TaskType::Docs(_) => "main", // Docs use default branch
            TaskType::Code(cr) => &cr.spec.docs_branch,
        }
    }

    /// Get resume session flag (only for CodeRun)
    #[allow(dead_code)]
    pub fn resume_session(&self) -> bool {
        match self {
            TaskType::Docs(_) => false, // Docs don't resume sessions
            TaskType::Code(cr) => cr.spec.resume_session,
        }
    }

    /// Get overwrite memory flag (only for CodeRun)
    pub fn overwrite_memory(&self) -> bool {
        match self {
            TaskType::Docs(_) => true, // Docs always overwrite memory
            TaskType::Code(cr) => cr.spec.overwrite_memory,
        }
    }

    /// Get docs project directory (only for CodeRun)
    pub fn docs_project_directory(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.docs_project_directory.as_deref(),
        }
    }
}