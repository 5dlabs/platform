//! PM task submission models

use serde::{Deserialize, Serialize};

/// PM task request structure according to design document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskRequest {
    // Task Master schema fields
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,

    // PM-specific fields
    pub service_name: String,
    pub agent_name: String,

    // Claude model selection (sonnet, opus)
    #[serde(default = "default_model")]
    pub model: String,

    // Markdown files as structured payloads
    pub markdown_files: Vec<MarkdownPayload>,

    // Agent tools specification
    #[serde(default)]
    pub agent_tools: Vec<AgentToolSpec>,

    // Repository specification for code access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,

    // Working directory within target repository (defaults to service_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    // Additional prompt instructions for retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_modification: Option<String>,

    // How to apply prompt_modification: 'append' or 'replace'
    #[serde(default = "default_prompt_mode", skip_serializing_if = "is_default_prompt_mode")]
    pub prompt_mode: String,

    // Local Claude Code tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_tools: Vec<String>,

    // Remote MCP tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_tools: Vec<String>,

    // Tool configuration preset
    #[serde(default = "default_tool_config", skip_serializing_if = "is_default_tool_config")]
    pub tool_config: String,
}

/// Subtask structure from Task Master
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<u32>,
    pub details: String,
    pub status: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
}

/// Markdown file payload for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownPayload {
    pub content: String,
    pub filename: String,
    pub file_type: String,
}

/// Agent tool specification for PM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolSpec {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

/// Repository specification for cloning source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySpec {
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub github_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>, // Reserved for future use - TODO: Implement direct token submission
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_model() -> String {
    "sonnet".to_string()
}

fn default_prompt_mode() -> String {
    "append".to_string()
}

fn is_default_prompt_mode(mode: &str) -> bool {
    mode == "append"
}

fn default_tool_config() -> String {
    "default".to_string()
}

fn is_default_tool_config(config: &str) -> bool {
    config == "default"
}

/// Documentation generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsGenerationRequest {
    /// Repository URL to clone
    pub repository_url: String,

    /// Working directory within the repository (path to .taskmaster)
    pub working_directory: String,

    /// Source branch to checkout and base new branch from
    pub source_branch: String,

    /// Target branch for the PR
    pub target_branch: String,

    /// Service name for the job
    pub service_name: String,

    /// Agent name for the job
    pub agent_name: String,

    /// Claude model selection (sonnet, opus)
    #[serde(default = "default_model")]
    pub model: String,

    /// GitHub user for authentication
    pub github_user: String,

    /// Optional specific task ID to generate docs for (if None, generates all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,

    /// Force overwrite existing documentation
    #[serde(default)]
    pub force: bool,

    /// Dry run mode (preview only)
    #[serde(default)]
    pub dry_run: bool,
}

/// Task Master JSON file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMasterFile {
    pub master: TaskMaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMaster {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,
}

impl PmTaskRequest {
    /// Create a new PM task request from Task Master task and markdown files
    pub fn new(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
    ) -> Self {
        Self::new_with_tools(
            task,
            service_name,
            agent_name,
            model,
            markdown_files,
            Vec::new(),
        )
    }

    /// Create a new PM task request with agent tools specification
    pub fn new_with_tools(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository: None,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request from Task Master task with repository support
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_repository(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full specification including working directory
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_full_spec(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with prompt modification support for retries
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_prompt_modification(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full tool configuration support
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_tool_config(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
        local_tools: Vec<String>,
        remote_tools: Vec<String>,
        tool_config: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools,
            remote_tools,
            tool_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_task_request_creation() {
        let task = Task {
            id: 1001,
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            details: "Test details".to_string(),
            test_strategy: "Test strategy".to_string(),
            priority: "high".to_string(),
            dependencies: vec![],
            status: "pending".to_string(),
            subtasks: vec![],
        };

        let markdown_files = vec![MarkdownPayload {
            content: "# Task Content".to_string(),
            filename: "task.md".to_string(),
            file_type: "task".to_string(),
        }];

        let request = PmTaskRequest::new(
            task,
            "test-service".to_string(),
            "claude-agent-1".to_string(),
            "sonnet".to_string(),
            markdown_files,
        );

        assert_eq!(request.id, 1001);
        assert_eq!(request.service_name, "test-service");
        assert_eq!(request.model, "sonnet");
        assert_eq!(request.markdown_files.len(), 1);
    }
}
