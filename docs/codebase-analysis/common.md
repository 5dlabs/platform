# common Analysis

**Path:** `orchestrator/common`
**Type:** RustLibrary
**Lines of Code:** 1888
**Description:** No description available

## Dependencies

- serde
- serde_json
- anyhow
- thiserror
- chrono
- k8s-openapi
- async-trait
- uuid

## Source Files

### src/error.rs (37 lines)

**Key Definitions:**
```rust
6:pub enum Error {
33:impl From<anyhow::Error> for Error {
```

**Full Content:**
```rust
//! Common error types for the orchestrator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes operation failed: {0}")]
    Kubernetes(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Job failed: {0}")]
    JobFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversion from anyhow::Error for easier error handling
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

```

### src/lib.rs (28 lines)

**Full Content:**
```rust
/*
 * 5D Labs Agent Platform - Common Types and Utilities
 * Copyright (C) 2025 5D Labs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Shared types and utilities for the Orchestrator project

pub mod error;
pub mod models;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Re-export commonly used types for convenience
pub use models::{AgentType, Job, JobStatus, JobType, Request, RequestSource, Task, TaskStatus};

```

### src/models/response.rs (239 lines)

**Key Definitions:**
```rust
10:pub struct ApiResponse<T> {
20:pub enum ResponseStatus {
28:pub struct ErrorDetails {
36:pub struct ResponseMetadata {
45:pub struct TaskResponse {
59:pub struct JobResponse {
74:pub struct JobListResponse {
83:pub struct TaskListResponse {
92:pub struct HealthResponse {
102:pub enum HealthStatus {
110:pub struct ComponentHealth {
118:pub struct WebhookResponse {
127:pub fn success(data: T, request_id: String) -> Self {
143:pub fn error(error: ErrorDetails, request_id: String) -> Self {
158:pub fn with_duration(mut self, duration_ms: u64) -> Self {
164:impl From<super::task::Task> for TaskResponse {
180:impl From<super::job::Job> for JobResponse {
```

**Full Content:**
```rust
//! Response models for API endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: ResponseStatus,
    pub data: Option<T>,
    pub error: Option<ErrorDetails>,
    pub metadata: ResponseMetadata,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Error,
    Partial,
}

/// Error details in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub version: String,
}

/// Task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: super::task::TaskStatus,
    pub priority: super::task::TaskPriority,
    pub microservice: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub job_ids: Vec<String>,
}

/// Job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub task_id: String,
    pub job_type: super::job::JobType,
    pub status: super::job::JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub logs_url: Option<String>,
}

/// Job list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Task list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Webhook processing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub accepted: bool,
    pub task_id: Option<String>,
    pub job_ids: Vec<String>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// Create a success response
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(error: ErrorDetails, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: None,
            error: Some(error),
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Set the duration in milliseconds
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.metadata.duration_ms = Some(duration_ms);
        self
    }
}

impl From<super::task::Task> for TaskResponse {
    fn from(task: super::Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            microservice: task.microservice,
            created_at: task.created_at,
            updated_at: task.updated_at,
            job_ids: Vec::new(), // To be populated by service layer
        }
    }
}

impl From<super::job::Job> for JobResponse {
    fn from(job: super::Job) -> Self {
        Self {
            id: job.id,
            task_id: job.task_id,
            job_type: job.job_type,
            status: job.status,
            k8s_job_name: job.k8s_job_name,
            namespace: job.namespace,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            logs_url: None, // To be populated by service layer
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::{TaskPriority, TaskStatus};

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(
            TaskResponse {
                id: "task-123".to_string(),
                title: "Test Task".to_string(),
                description: "A test task".to_string(),
                status: TaskStatus::Pending,
                priority: TaskPriority::Medium,
                microservice: "auth".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                job_ids: vec![],
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<TaskResponse> = ApiResponse::error(
            ErrorDetails {
                code: "TASK_NOT_FOUND".to_string(),
                message: "Task not found".to_string(),
                details: None,
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Error);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}

```

### src/models/request.rs (201 lines)

**Key Definitions:**
```rust
9:pub struct Request {
20:pub enum RequestSource {
31:pub enum RequestAction {
44:pub struct RequestMetadata {
56:pub struct ParsedRequest {
69:pub struct CliRequest {
77:pub struct CreateTaskRequest {
89:pub struct UpdateTaskRequest {
99:pub struct AssistanceRequest {
110:pub enum AssistanceType {
121:pub enum AssistancePriority {
128:impl Request {
131:pub fn new(source: RequestSource, action: RequestAction, payload: Value) -> Self {
149:pub fn with_trace_id(mut self, trace_id: String) -> Self {
156:pub fn with_user(mut self, user: String) -> Self {
```

**Full Content:**
```rust
//! Request models for unified orchestration

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Unified request interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub source: RequestSource,
    pub action: RequestAction,
    pub payload: Value,
    pub metadata: RequestMetadata,
}

/// Source of incoming requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestSource {
    Cli,
    PmAgent,
    GitHub,
    Grafana,
    Discord,
}

/// Action to be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestAction {
    CreateTask,
    UpdateTask,
    GetTaskStatus,
    TriggerAssistance,
    ListJobs,
    GetJobLogs,
    ReviewPR,
    HandleAlert,
}

/// Additional request metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestMetadata {
    pub user: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub channel: Option<String>,
    pub timestamp: String,
    pub trace_id: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Parsed request after normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequest {
    pub action: RequestAction,
    pub task_id: Option<String>,
    pub microservice: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub metadata: Value,
}

/// CLI request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub command: String,
    pub args: Vec<String>,
    pub options: HashMap<String, String>,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub microservice: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub agent_type: Option<super::AgentType>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Task update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<super::TaskStatus>,
    pub priority: Option<super::task::TaskPriority>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Assistance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceRequest {
    pub task_id: String,
    pub reason: String,
    pub assist_type: AssistanceType,
    pub context: Option<Value>,
    pub priority: AssistancePriority,
}

/// Type of assistance needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistanceType {
    ImplementationGuidance,
    ArchitectureReview,
    ErrorDiagnosis,
    TestDebugging,
    PerformanceOptimization,
}

/// Priority of assistance request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistancePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Request {
    /// Create a new request
    #[must_use]
    pub fn new(source: RequestSource, action: RequestAction, payload: Value) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        Self {
            id: Uuid::new_v4().to_string(),
            source,
            action,
            payload,
            metadata: RequestMetadata {
                timestamp: Utc::now().to_rfc3339(),
                ..Default::default()
            },
        }
    }

    /// Add trace ID for distributed tracing
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }

    /// Add user information
    #[must_use]
    pub fn with_user(mut self, user: String) -> Self {
        self.metadata.user = Some(user);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_creation() {
        let request = Request::new(
            RequestSource::Cli,
            RequestAction::CreateTask,
            json!({
                "title": "Test Task",
                "description": "A test task"
            }),
        );

        assert_eq!(request.source, RequestSource::Cli);
        assert_eq!(request.action, RequestAction::CreateTask);
        assert!(!request.id.is_empty());
        assert!(!request.metadata.timestamp.is_empty());
    }

    #[test]
    fn test_create_task_request_serialization() {
        let req = CreateTaskRequest {
            microservice: "auth".to_string(),
            title: "Implement JWT validation".to_string(),
            description: "Add JWT token validation".to_string(),
            acceptance_criteria: vec!["Validate tokens".to_string()],
            priority: Some("high".to_string()),
            agent_type: Some(super::super::AgentType::Claude),
            metadata: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: CreateTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.title, deserialized.title);
        assert_eq!(req.microservice, deserialized.microservice);
    }
}

```

### src/models/job.rs (244 lines)

**Key Definitions:**
```rust
10:pub struct Job {
26:pub enum JobType {
40:pub enum JobStatus {
50:pub struct JobSpec {
73:pub struct VolumeSpec {
83:pub enum VolumeType {
94:impl Job {
97:pub fn new(
120:pub fn update_from_k8s_job(&mut self, k8s_job: &K8sJob) {
147:pub fn is_terminal(&self) -> bool {
153:pub fn duration(&self) -> Option<chrono::Duration> {
161:impl Default for JobSpec {
177:impl std::fmt::Display for JobType {
188:impl std::fmt::Display for JobStatus {
```

**Full Content:**
```rust
//! Job-related data models for Kubernetes job orchestration

use chrono::{DateTime, Utc};
use k8s_openapi::api::batch::v1::Job as K8sJob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Kubernetes job for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub task_id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub spec: JobSpec,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Type of job in the orchestration pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Prepare job that sets up workspace and context files
    Prepare,
    /// Execute job that runs the primary agent (Claude)
    Execute,
    /// Assist job that runs helper agent (Gemini)
    Assist,
    /// Review job for code review tasks
    Review,
}

/// Job execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

/// Job specification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// Container image to use
    pub image: String,
    /// Agent type for execution jobs
    pub agent: Option<super::AgentType>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Resource limits and requests
    pub resources: super::ResourceLimits,
    /// Volume mounts
    pub volumes: Vec<VolumeSpec>,
    /// Command to execute
    pub command: Option<Vec<String>>,
    /// Working directory
    pub working_dir: Option<String>,
    /// Job timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Number of retries
    pub retry_limit: Option<u32>,
}

/// Volume specification for job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub mount_path: String,
    pub volume_type: VolumeType,
    pub read_only: bool,
}

/// Types of volumes that can be mounted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeType {
    /// `ConfigMap` volume
    ConfigMap { name: String },
    /// `PersistentVolumeClaim`
    Pvc { claim_name: String },
    /// `EmptyDir` volume
    EmptyDir,
    /// Secret volume
    Secret { name: String },
}

impl Job {
    /// Create a new job
    #[must_use]
    pub fn new(
        id: String,
        task_id: String,
        job_type: JobType,
        k8s_job_name: String,
        namespace: String,
        spec: JobSpec,
    ) -> Self {
        Self {
            id,
            task_id,
            job_type,
            status: JobStatus::Pending,
            k8s_job_name,
            namespace,
            spec,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Update job status based on Kubernetes job status
    pub fn update_from_k8s_job(&mut self, k8s_job: &K8sJob) {
        if let Some(status) = &k8s_job.status {
            if status.succeeded == Some(1) {
                self.status = JobStatus::Succeeded;
                self.completed_at = status.completion_time.as_ref().map(|t| {
                    DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                        .unwrap()
                        .with_timezone(&Utc)
                });
            } else if status.failed.unwrap_or(0) > 0 {
                self.status = JobStatus::Failed;
                self.completed_at = Some(Utc::now());
            } else if status.active == Some(1) {
                self.status = JobStatus::Running;
                if self.started_at.is_none() {
                    self.started_at = status.start_time.as_ref().map(|t| {
                        DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                            .unwrap()
                            .with_timezone(&Utc)
                    });
                }
            }
        }
    }

    /// Check if the job is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Succeeded | JobStatus::Failed)
    }

    /// Get job duration if available
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            image: "busybox:latest".to_string(),
            agent: None,
            env_vars: HashMap::new(),
            resources: super::ResourceLimits::default(),
            volumes: Vec::new(),
            command: None,
            working_dir: None,
            timeout_seconds: Some(1800), // 30 minutes default
            retry_limit: Some(2),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Prepare => write!(f, "Prepare"),
            JobType::Execute => write!(f, "Execute"),
            JobType::Assist => write!(f, "Assist"),
            JobType::Review => write!(f, "Review"),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Succeeded => write!(f, "Succeeded"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let spec = JobSpec::default();
        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(!job.is_terminal());
    }

    #[test]
    fn test_job_serialization() {
        let spec = JobSpec {
            image: "claude:latest".to_string(),
            agent: Some(super::super::AgentType::Claude),
            ..Default::default()
        };

        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job.id, deserialized.id);
        assert_eq!(job.job_type, deserialized.job_type);
    }
}

```

### src/models/config.rs (147 lines)

**Key Definitions:**
```rust
9:pub enum AgentType {
17:pub struct AgentConfig {
29:pub struct ResourceLimits {
40:pub struct McpServerConfig {
50:pub struct OrchestratorConfig {
60:impl Default for ResourceLimits {
73:impl AgentType {
76:pub fn display_name(&self) -> &'static str {
85:pub fn default_image(&self) -> &'static str {
94:pub fn can_implement(&self) -> bool {
103:pub fn can_assist(&self) -> bool {
```

**Full Content:**
```rust
//! Configuration-related models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent types that can execute tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    #[default]
    Claude,
    Gemini,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub image: String,
    pub version: String,
    pub env_vars: HashMap<String, String>,
    pub resources: ResourceLimits,
    pub capabilities: Vec<String>,
    pub mcp_servers: Vec<String>,
}

/// Resource limits and requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub ephemeral_storage_request: Option<String>,
    pub ephemeral_storage_limit: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub capabilities: Vec<String>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub namespace: String,
    pub agents: HashMap<AgentType, AgentConfig>,
    pub default_timeout_seconds: u32,
    pub max_retry_attempts: u32,
    pub workspace_pvc_template: String,
    pub prepare_job_image: String,
    pub node_selector: HashMap<String, String>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "2Gi".to_string(),
            ephemeral_storage_request: None,
            ephemeral_storage_limit: None,
        }
    }
}

impl AgentType {
    /// Get display name for the agent
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::Claude => "Claude Code",
            AgentType::Gemini => "Gemini CLI",
        }
    }

    /// Get the default image for the agent
    #[must_use]
    pub fn default_image(&self) -> &'static str {
        match self {
            AgentType::Claude => "anthropic/claude-code:latest",
            AgentType::Gemini => "google/gemini-cli:latest",
        }
    }

    /// Check if this agent can be a primary implementer
    #[must_use]
    pub fn can_implement(&self) -> bool {
        match self {
            AgentType::Claude => true,
            AgentType::Gemini => false, // Gemini is assistance-only in our pattern
        }
    }

    /// Check if this agent can provide assistance
    #[must_use]
    pub fn can_assist(&self) -> bool {
        match self {
            AgentType::Claude => false, // Claude is implementation-only in our pattern
            AgentType::Gemini => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_capabilities() {
        assert!(AgentType::Claude.can_implement());
        assert!(!AgentType::Claude.can_assist());
        assert!(!AgentType::Gemini.can_implement());
        assert!(AgentType::Gemini.can_assist());
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu_request, "100m");
        assert_eq!(limits.memory_limit, "2Gi");
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            agent_type: AgentType::Claude,
            image: "claude:v1".to_string(),
            version: "1.0.0".to_string(),
            env_vars: HashMap::new(),
            resources: ResourceLimits::default(),
            capabilities: vec!["code".to_string(), "test".to_string()],
            mcp_servers: vec!["taskmaster".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.agent_type, deserialized.agent_type);
        assert_eq!(config.capabilities, deserialized.capabilities);
    }
}

```

### src/models/webhook.rs (232 lines)

**Key Definitions:**
```rust
9:pub struct WebhookPayload {
17:pub struct GitHubWebhookPayload {
27:pub struct GitHubIssue {
41:pub struct GitHubPullRequest {
56:pub struct GitHubRepository {
67:pub struct GitHubUser {
76:pub struct GitHubLabel {
83:pub struct GitHubRef {
91:pub struct GrafanaAlert {
106:pub struct GrafanaWebhookPayload {
120:pub struct PmAgentPayload {
128:pub struct PmTaskData {
141:pub struct DiscordPayload {
```

**Full Content:**
```rust
//! Webhook payload models for various sources

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub source: super::RequestSource,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

/// GitHub webhook payload for issue events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub pull_request: Option<GitHubPullRequest>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// GitHub issue structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub pull request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub head: GitHubRef,
    pub base: GitHubRef,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub private: bool,
    pub default_branch: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(rename = "type")]
    pub user_type: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

/// GitHub ref (branch/tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    pub label: String,
    pub ref_field: String,
    pub sha: String,
}

/// Grafana alert webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaAlert {
    pub status: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub values: HashMap<String, f64>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
}

/// Grafana webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaWebhookPayload {
    pub receiver: String,
    pub status: String,
    pub alerts: Vec<GrafanaAlert>,
    #[serde(rename = "groupLabels")]
    pub group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    pub common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    pub common_annotations: HashMap<String, String>,
}

/// PM Agent webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmAgentPayload {
    pub action: String,
    pub project_id: String,
    pub task: PmTaskData,
}

/// PM Agent task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Discord webhook payload (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub command: String,
    pub args: Vec<String>,
    pub message_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_webhook_deserialization() {
        let json = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 42,
                "title": "Test Issue",
                "body": "Test body",
                "state": "open",
                "labels": [],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "user": {
                    "login": "testuser",
                    "id": 456,
                    "type": "User"
                }
            },
            "repository": {
                "id": 789,
                "name": "test-repo",
                "full_name": "org/test-repo",
                "owner": {
                    "login": "org",
                    "id": 999,
                    "type": "Organization"
                },
                "private": false,
                "default_branch": "main"
            },
            "sender": {
                "login": "testuser",
                "id": 456,
                "type": "User"
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, "opened");
        assert!(payload.issue.is_some());
        assert_eq!(payload.issue.unwrap().number, 42);
    }

    #[test]
    fn test_grafana_alert_deserialization() {
        let json = r#"{
            "receiver": "webhook",
            "status": "firing",
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "HighErrorRate",
                    "task_id": "123"
                },
                "annotations": {
                    "summary": "High error rate detected"
                },
                "values": {
                    "error_rate": 0.45
                },
                "startsAt": "2024-01-01T00:00:00Z",
                "endsAt": null,
                "generatorURL": "http://grafana/alert"
            }],
            "groupLabels": {},
            "commonLabels": {},
            "commonAnnotations": {}
        }"#;

        let payload: GrafanaWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.status, "firing");
        assert_eq!(payload.alerts.len(), 1);
        assert_eq!(
            payload.alerts[0].labels.get("task_id"),
            Some(&"123".to_string())
        );
    }
}

```

### src/models/code_request.rs (91 lines)

**Key Definitions:**
```rust
9:pub struct SecretEnvVar {
21:pub struct CodeRequest {
```

**Full Content:**
```rust
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

```

### src/models/task.rs (176 lines)

**Key Definitions:**
```rust
9:pub struct Task {
26:pub enum TaskStatus {
38:pub enum TaskPriority {
48:pub struct TaskMetadata {
62:impl Task {
65:pub fn new(id: String, title: String, description: String, microservice: String) -> Self {
84:pub fn is_terminal(&self) -> bool {
92:pub fn update_status(&mut self, status: TaskStatus) {
98:impl std::fmt::Display for TaskStatus {
111:impl std::fmt::Display for TaskPriority {
```

**Full Content:**
```rust
//! Task-related data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub microservice: String,
    pub agent_type: Option<super::AgentType>,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Additional metadata for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskMetadata {
    /// Source that created this task
    pub source: Option<String>,
    /// GitHub issue number if applicable
    pub github_issue: Option<u64>,
    /// Task Master task ID if applicable
    pub task_master_id: Option<String>,
    /// Custom labels
    pub labels: HashMap<String, String>,
    /// Additional arbitrary data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Task {
    /// Create a new task with default values
    #[must_use]
    pub fn new(id: String, title: String, description: String, microservice: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            acceptance_criteria: Vec::new(),
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            microservice,
            agent_type: None,
            metadata: TaskMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the task is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }

    /// Update the task status and timestamp
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert_eq!(task.id, "test-123");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(!task.is_terminal());
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.status, deserialized.status);
    }

    #[test]
    fn test_terminal_states() {
        let mut task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert!(!task.is_terminal());

        task.update_status(TaskStatus::Completed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::Failed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::InProgress);
        assert!(!task.is_terminal());
    }
}

```

### src/models/docs_request.rs (21 lines)

**Key Definitions:**
```rust
6:pub struct DocsRequest {
```

**Full Content:**
```rust
//! Clean documentation generation request structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsRequest {
    /// Git repository URL
    pub repository_url: String,

    /// Working directory within the repository
    pub working_directory: String,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Source branch (auto-detected)
    pub source_branch: String,
}

```

### src/models/mod.rs (25 lines)

**Full Content:**
```rust
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

```

### src/models/pm_task.rs (447 lines)

**Key Definitions:**
```rust
7:pub struct PmTaskRequest {
70:pub struct Subtask {
83:pub struct MarkdownPayload {
91:pub struct AgentToolSpec {
102:pub struct RepositorySpec {
133:pub struct DocsGenerationRequest {
173:pub struct TaskMasterFile {
178:pub struct TaskMaster {
183:pub struct Task {
196:impl PmTaskRequest {
199:pub fn new(
218:pub fn new_with_tools(
254:pub fn new_with_repository(
291:pub fn new_with_full_spec(
329:pub fn new_with_prompt_modification(
369:pub fn new_with_tool_config(
```

**Full Content:**
```rust
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
    #[serde(
        default = "default_prompt_mode",
        skip_serializing_if = "is_default_prompt_mode"
    )]
    pub prompt_mode: String,

    // Local Claude Code tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_tools: Vec<String>,

    // Remote MCP tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_tools: Vec<String>,

    // Tool configuration preset
    #[serde(
        default = "default_tool_config",
        skip_serializing_if = "is_default_tool_config"
    )]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

```

