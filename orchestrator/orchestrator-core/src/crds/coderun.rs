//! CodeRun Custom Resource Definition for code implementation tasks

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default function for tool_config field
fn default_tool_config() -> String {
    "default".to_string()
}

/// Default function for context_version field
fn default_context_version() -> Option<u32> {
    Some(1)
}

/// Default function for prompt_mode field
fn default_prompt_mode() -> Option<String> {
    Some("append".to_string())
}

/// CodeRun CRD for code implementation tasks
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.platform", version = "v1", kind = "CodeRun")]
#[kube(namespaced)]
#[kube(status = "CodeRunStatus")]
#[kube(printcolumn = r#"{"name":"Task","type":"integer","jsonPath":".spec.taskId"}"#)]
#[kube(printcolumn = r#"{"name":"Service","type":"string","jsonPath":".spec.service"}"#)]
#[kube(printcolumn = r#"{"name":"Model","type":"string","jsonPath":".spec.model"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct CodeRunSpec {
    /// Task ID to implement
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    pub repository_url: String,

    /// Platform repository URL (where Task Master definitions come from)
    pub platform_repository_url: String,

    /// Git branch to work on in target repository
    pub branch: String,

    /// Working directory within target repository
    pub working_directory: String,

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// GitHub username for authentication and commits
    pub github_user: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub remote_tools: Option<String>,

    /// Tool configuration preset (default, minimal, advanced)
    #[serde(default = "default_tool_config")]
    pub tool_config: String,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version")]
    pub context_version: Option<u32>,

    /// Additional context for retry attempts
    #[serde(default)]
    pub prompt_modification: Option<String>,

    /// How to apply prompt modifications (append, replace)
    #[serde(default = "default_prompt_mode")]
    pub prompt_mode: Option<String>,
}

/// Status of the CodeRun
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeRunStatus {
    /// Current phase of the code implementation
    pub phase: String,

    /// Human-readable message about the current state
    pub message: Option<String>,

    /// Timestamp when this phase was reached
    pub last_update: Option<String>,

    /// Associated Kubernetes Job name
    pub job_name: Option<String>,

    /// Pull request URL if created
    pub pull_request_url: Option<String>,

    /// Current retry attempt (if applicable)
    pub retry_count: Option<u32>,

    /// Conditions for the CodeRun
    pub conditions: Option<Vec<CodeRunCondition>>,

    /// Name of the ConfigMap containing the prompt and context
    pub configmap_name: Option<String>,

    /// Version of the context and prompt used
    pub context_version: Option<u32>,

    /// Modification to the prompt if any
    pub prompt_modification: Option<String>,

    /// Mode of prompt (e.g., "direct", "indirect")
    pub prompt_mode: Option<String>,

    /// Session ID for tracking
    pub session_id: Option<String>,
}

/// Condition for the CodeRun
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

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