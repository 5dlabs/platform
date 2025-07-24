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

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// GitHub username for authentication
    pub github_user: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub remote_tools: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version")]
    pub context_version: u32,

    /// Additional context for retry attempts
    #[serde(default)]
    pub prompt_modification: Option<String>,

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_code_request_parsing() {
        let json = r#"{"
          "task_id": 1,
          "service": "test-service",
          "repository_url": "https://github.com/test/repo",
          "docs_repository_url": "https://github.com/test/docs",
          "docs_project_directory": "_projects/test-service",
          "working_directory": "test-service",
          "model": "claude-3-5-sonnet-20241022",
          "github_user": "testuser",
          "local_tools": "mcp-server-git,taskmaster",
          "remote_tools": "api-docs-tool",
          "context_version": 2,
          "prompt_modification": "Focus on error handling",
          "docs_branch": "feature/test",
          "continue_session": true,
          "overwrite_memory": false,
          "env": {
            "DEBUG": "true",
            "LOG_LEVEL": "info"
          },
          "env_from_secrets": [
            {
              "name": "API_KEY",
              "secretName": "api-secrets",
              "secretKey": "key"
            }
          ]
        }"#;

        let request: CodeRequest = serde_json::from_str(json).expect("Failed to parse JSON");

        assert_eq!(request.task_id, 1);
        assert_eq!(request.service, "test-service");
        assert_eq!(request.repository_url, "https://github.com/test/repo");
        assert_eq!(request.docs_repository_url, "https://github.com/test/docs");
        assert_eq!(request.docs_project_directory, Some("_projects/test-service".to_string()));
        assert_eq!(request.working_directory, Some("test-service".to_string()));
        assert_eq!(request.model, "claude-3-5-sonnet-20241022");
        assert_eq!(request.github_user, "testuser");
        assert_eq!(request.local_tools, Some("mcp-server-git,taskmaster".to_string()));
        assert_eq!(request.remote_tools, Some("api-docs-tool".to_string()));
        assert_eq!(request.context_version, 2);
        assert_eq!(request.prompt_modification, Some("Focus on error handling".to_string()));
        assert_eq!(request.docs_branch, "feature/test");
        assert_eq!(request.continue_session, true);
        assert_eq!(request.overwrite_memory, false);

        let mut expected_env = HashMap::new();
        expected_env.insert("DEBUG".to_string(), "true".to_string());
        expected_env.insert("LOG_LEVEL".to_string(), "info".to_string());
        assert_eq!(request.env, expected_env);

        assert_eq!(request.env_from_secrets.len(), 1);
        assert_eq!(request.env_from_secrets[0].name, "API_KEY");
        assert_eq!(request.env_from_secrets[0].secret_name, "api-secrets");
        assert_eq!(request.env_from_secrets[0].secret_key, "key");
    }

    #[test]
    fn test_code_request_defaults() {
        let json = r#"{"
          "task_id": 1,
          "service": "test-service",
          "repository_url": "https://github.com/test/repo",
          "docs_repository_url": "https://github.com/test/docs",
          "model": "claude-3-5-sonnet-20241022",
          "github_user": "testuser"
        }"#;

        let request: CodeRequest = serde_json::from_str(json).expect("Failed to parse JSON");

        // Test defaults
        assert_eq!(request.context_version, 1);
        assert_eq!(request.docs_branch, "main");
        assert_eq!(request.continue_session, false);
        assert_eq!(request.overwrite_memory, false);
        assert!(request.env.is_empty());
        assert!(request.env_from_secrets.is_empty());
        assert!(request.prompt_modification.is_none());
        assert!(request.local_tools.is_none());
        assert!(request.remote_tools.is_none());
        assert!(request.docs_project_directory.is_none());
        assert!(request.working_directory.is_none());
    }
}