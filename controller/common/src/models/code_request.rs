//! Clean code task submission request structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export SecretEnvVar from orchestrator-core crate to avoid duplication
// For now, we'll define it locally until we can reorganize the type sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEnvVar {
    /// Name of the environment variable
    pub name: String,
    /// Name of the secret
    #[serde(rename = "secretName")]
    pub secret_name: String,
    /// Key within the secret
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    /// Task ID to implement
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version")]
    pub context_version: u32,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default)]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default)]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default)]
    pub env_from_secrets: Vec<SecretEnvVar>,
}

/// Default context version
fn default_context_version() -> u32 {
    1
}

/// Default docs branch
fn default_docs_branch() -> String {
    "main".to_string()
}
