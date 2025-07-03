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

    // Markdown files as structured payloads
    pub markdown_files: Vec<MarkdownPayload>,

    // Agent tools specification
    #[serde(default)]
    pub agent_tools: Vec<AgentToolSpec>,

    // Repository specification for code access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<RepositoryAuth>,
}

/// Repository authentication specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAuth {
    #[serde(rename = "type")]
    pub auth_type: RepositoryAuthType,
    pub secret_name: String,
    #[serde(default = "default_secret_key")]
    pub secret_key: String,
}

/// Repository authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryAuthType {
    Token,
    SshKey,
    BasicAuth,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_secret_key() -> String {
    "token".to_string()
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
        markdown_files: Vec<MarkdownPayload>,
    ) -> Self {
        Self::new_with_tools(task, service_name, agent_name, markdown_files, Vec::new())
    }

    /// Create a new PM task request with agent tools specification
    pub fn new_with_tools(
        task: Task,
        service_name: String,
        agent_name: String,
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
            markdown_files,
            agent_tools,
            repository: None,
        }
    }

    /// Create a new PM task request from Task Master task with repository support
    pub fn new_with_repository(
        task: Task,
        service_name: String,
        agent_name: String,
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
            markdown_files,
            agent_tools,
            repository,
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
            markdown_files,
        );

        assert_eq!(request.id, 1001);
        assert_eq!(request.service_name, "test-service");
        assert_eq!(request.markdown_files.len(), 1);
    }
}
