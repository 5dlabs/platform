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
