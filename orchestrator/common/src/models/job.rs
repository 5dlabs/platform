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
    /// 
    /// # Panics
    /// 
    /// Panics if the Kubernetes job completion time cannot be parsed as RFC3339
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
