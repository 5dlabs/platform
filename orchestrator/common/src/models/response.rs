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
