use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// TaskRun is a custom resource that represents a task to be executed by an agent
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[kube(
    group = "orchestrator.io",
    version = "v1",
    kind = "TaskRun",
    namespaced,
    status = "TaskRunStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunSpec {
    /// Unique identifier for the task
    pub task_id: u32,

    /// Target service for the task
    pub service_name: String,

    /// Agent to execute the task
    pub agent_name: String,

    /// Claude model to use (sonnet, opus)
    #[serde(default = "default_model")]
    pub model: String,

    /// Version of the context, incremented on updates
    #[serde(default = "default_context_version")]
    pub context_version: u32,

    /// Markdown files containing task context
    pub markdown_files: Vec<MarkdownFile>,

    /// Tools available to the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_tools: Vec<AgentTool>,

    /// Repository information for code access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,
}

fn default_context_version() -> u32 {
    1
}

fn default_model() -> String {
    "sonnet".to_string()
}

/// Markdown file containing task context
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownFile {
    /// Filename for the markdown content
    pub filename: String,

    /// Markdown content
    pub content: String,

    /// Type of markdown file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<MarkdownFileType>,
}

/// Type of markdown file
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownFileType {
    Task,
    DesignSpec,
    Prompt,
    Context,
    AcceptanceCriteria,
}

/// Agent tool specification
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentTool {
    /// Tool name (e.g., "bash", "edit", "read")
    pub name: String,

    /// Whether the tool is enabled
    #[serde(default = "default_tool_enabled")]
    pub enabled: bool,

    /// Tool-specific configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,

    /// Tool restrictions or limitations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restrictions: Vec<String>,
}

fn default_tool_enabled() -> bool {
    true
}

/// Repository specification for cloning source code
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySpec {
    /// Repository URL (HTTPS or SSH)
    pub url: String,

    /// Branch or tag to checkout
    #[serde(default = "default_branch")]
    pub branch: String,

    /// GitHub username for authentication (used to auto-resolve secret name)
    pub github_user: String,

    /// Optional token for direct authentication (reserved for future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>, // TODO: Implement direct token submission in future
}

fn default_branch() -> String {
    "main".to_string()
}

/// Status of the TaskRun
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunStatus {
    /// Current phase of the TaskRun
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<TaskRunPhase>,

    /// Name of the Kubernetes Job created for this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_name: Option<String>,

    /// Name of the ConfigMap containing task files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map_name: Option<String>,

    /// Number of execution attempts
    #[serde(default)]
    pub attempts: u32,

    /// Last time the status was updated (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,

    /// Human-readable message about the current status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Claude session ID for resuming conversations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Detailed conditions for the TaskRun
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<TaskRunCondition>,
}

/// Phase of the TaskRun execution
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub enum TaskRunPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl std::fmt::Display for TaskRunPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskRunPhase::Pending => write!(f, "Pending"),
            TaskRunPhase::Running => write!(f, "Running"),
            TaskRunPhase::Succeeded => write!(f, "Succeeded"),
            TaskRunPhase::Failed => write!(f, "Failed"),
        }
    }
}

/// Condition for the TaskRun
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition
    pub status: ConditionStatus,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Status of a condition
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

impl std::fmt::Display for ConditionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionStatus::True => write!(f, "True"),
            ConditionStatus::False => write!(f, "False"),
            ConditionStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_taskrun_serialization() {
        let taskrun = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                markdown_files: vec![MarkdownFile {
                    filename: "task.md".to_string(),
                    content: "# Task content".to_string(),
                    file_type: Some(MarkdownFileType::Task),
                }],
                agent_tools: vec![],
                repository: None,
            },
            status: None,
        };

        let json = serde_json::to_string_pretty(&taskrun).unwrap();
        let deserialized: TaskRun = serde_json::from_str(&json).unwrap();
        assert_eq!(taskrun.spec.task_id, deserialized.spec.task_id);
    }

    #[test]
    fn test_status_serialization() {
        let status = TaskRunStatus {
            phase: Some(TaskRunPhase::Running),
            job_name: Some("test-job".to_string()),
            config_map_name: Some("test-cm".to_string()),
            attempts: 1,
            last_updated: Some(Utc::now().to_rfc3339()),
            message: Some("Job is running".to_string()),
            session_id: None,
            conditions: vec![],
        };

        let json = serde_json::to_string_pretty(&status).unwrap();
        let deserialized: TaskRunStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status.phase, deserialized.phase);
    }
}
