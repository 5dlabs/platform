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
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }

    /// Add user information
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
