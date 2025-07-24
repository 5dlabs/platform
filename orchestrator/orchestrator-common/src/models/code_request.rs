//! Clean code task submission request structure

use serde::{Deserialize, Serialize};

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

    /// Git branch to work on in target repository
    pub branch: String,

    /// GitHub username for authentication
    pub github_user: String,

    /// Working directory within target repository (defaults to service name)
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub remote_tools: Option<String>,

    /// Tool configuration preset (default, minimal, advanced)
    #[serde(default = "default_tool_config")]
    pub tool_config: String,
}

/// Default tool configuration
fn default_tool_config() -> String {
    "default".to_string()
}
