//! Helm binary wrapper for deploying Claude Code agents

use anyhow::{Context, Result};
use orchestrator_common::models::pm_task::PmTaskRequest;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tempfile::TempDir;
use tokio::fs;
use tracing::{debug, info};

/// Helm client for deploying charts
#[derive(Clone)]
pub struct HelmClient {
    chart_path: String,
    namespace: String,
}

/// Helm values for Claude Code deployment
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmValues {
    pub service_name: String,
    pub task_id: String,
    pub attempt: String,
    pub agent_name: String,
    pub task_config_map_name: String,
    pub job: JobConfig,
    pub shared_workspace: WorkspaceConfig,
    pub node_affinity: NodeAffinityConfig,
    pub persistence: PersistenceConfig,
    pub replica_count: u32, // Set to 0 to disable deployment
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub secrets: SecretsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsConfig {
    pub anthropic_api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfig {
    pub enabled: bool,
    pub task_type: String,
    pub task_id: String,
    pub microservice: String,
    pub agent_type: String,
    pub backoff_limit: u32,
    pub active_deadline_seconds: u32,
    pub ttl_seconds_after_finished: u32,
    pub skip_prepare: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    pub enabled: bool,
    pub claim_name: String,
    pub existing_claim: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeAffinityConfig {
    pub enabled: bool,
    pub required: bool,
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub storage_class: String,
    pub access_mode: String,
    pub size: String,
}

impl HelmClient {
    /// Create a new Helm client
    pub fn new(chart_path: String, namespace: String) -> Self {
        Self {
            chart_path,
            namespace,
        }
    }

    /// Deploy a task using Helm
    pub async fn deploy_task(&self, release_name: &str, request: &PmTaskRequest) -> Result<String> {
        // Get the next attempt number (for now, hardcode to 1)
        let attempt_number = 1;

        // Generate values for the deployment
        let values = self.generate_values(request, attempt_number)?;

        // Create temporary directory for values file
        let temp_dir = TempDir::new()?;
        let values_file = temp_dir.path().join("values.yaml");

        // Write values to file
        let values_yaml =
            serde_yaml::to_string(&values).context("Failed to serialize values to YAML")?;
        fs::write(&values_file, values_yaml)
            .await
            .context("Failed to write values file")?;

        info!("Deploying Helm release: {}", release_name);
        debug!("Using values file: {:?}", values_file);

        // Execute helm install command
        let output = Command::new("helm")
            .args([
                "install",
                release_name,
                &self.chart_path,
                "--namespace",
                &self.namespace,
                "--values",
                values_file.to_str().unwrap(),
                "--wait",
                "--timeout",
                "5m",
            ])
            .output()
            .context("Failed to execute helm command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Helm deployment failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Helm deployment successful: {}", stdout);

        Ok(release_name.to_string())
    }

    /// Generate Helm values from PM task request
    fn generate_values(&self, request: &PmTaskRequest, attempt: u32) -> Result<HelmValues> {
        // Get API key from environment
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .unwrap_or_else(|_| "sk-ant-api03-your-key-here".to_string());

        let values = HelmValues {
            service_name: request.service_name.clone(),
            task_id: request.id.to_string(),
            attempt: attempt.to_string(),
            agent_name: request.agent_name.clone(),
            task_config_map_name: format!("{}-task-{}-files", request.service_name, request.id),
            job: JobConfig {
                enabled: true,
                task_type: "implementation".to_string(),
                task_id: request.id.to_string(),
                microservice: request.service_name.clone(),
                agent_type: "claude".to_string(),
                backoff_limit: 3,
                active_deadline_seconds: 3600,     // 1 hour
                ttl_seconds_after_finished: 86400, // 24 hours
                skip_prepare: false,
            },
            shared_workspace: WorkspaceConfig {
                enabled: true,
                claim_name: "shared-workspace".to_string(),
                existing_claim: "shared-workspace".to_string(),
            },
            node_affinity: NodeAffinityConfig {
                enabled: false, // Disable node affinity for now
                required: false,
                key: "kubernetes.io/hostname".to_string(),
                values: vec![], // Empty values since enabled is false
            },
            persistence: PersistenceConfig {
                enabled: false, // Disable as we're using shared workspace
                storage_class: "local-path".to_string(),
                access_mode: "ReadWriteOnce".to_string(),
                size: "10Gi".to_string(),
            },
            replica_count: 0, // Disable deployment, only use Job
            command: vec!["claude".to_string()],
            args: vec![
                "-p".to_string(),
                "Read the task context in CLAUDE.md and begin implementing the requested service. Focus on the acceptance criteria and follow the autonomous agent instructions.".to_string(),
            ],
            secrets: SecretsConfig {
                anthropic_api_key: api_key,
            },
        };

        Ok(values)
    }

    /// Get the status of a Helm release
    #[allow(dead_code)]
    pub async fn get_release_status(&self, release_name: &str) -> Result<String> {
        let output = Command::new("helm")
            .args([
                "status",
                release_name,
                "--namespace",
                &self.namespace,
                "--output",
                "json",
            ])
            .output()
            .context("Failed to execute helm status command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to get release status: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Delete a Helm release
    #[allow(dead_code)]
    pub async fn delete_release(&self, release_name: &str) -> Result<()> {
        let output = Command::new("helm")
            .args(["uninstall", release_name, "--namespace", &self.namespace])
            .output()
            .context("Failed to execute helm uninstall command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to delete release: {}", stderr));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orchestrator_common::models::pm_task::Task;

    #[test]
    fn test_helm_client_creation() {
        let client = HelmClient::new("/path/to/chart".to_string(), "orchestrator".to_string());
        assert_eq!(client.chart_path, "/path/to/chart");
        assert_eq!(client.namespace, "orchestrator");
    }

    #[test]
    fn test_values_generation() {
        let client = HelmClient::new("/path/to/chart".to_string(), "orchestrator".to_string());

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

        let request = PmTaskRequest::new(
            task,
            "test-service".to_string(),
            "claude-agent-1".to_string(),
            "sonnet".to_string(),
            vec![],
        );

        let values = client.generate_values(&request, 1).unwrap();
        assert_eq!(values.service_name, "test-service");
        assert_eq!(values.task_id, "1001");
        assert_eq!(values.attempt, "1");
        assert_eq!(values.agent_name, "claude-agent-1");
        assert!(!values.node_affinity.enabled);
        assert!(!values.persistence.enabled);
        assert_eq!(values.shared_workspace.existing_claim, "shared-workspace");
        assert_eq!(values.command, vec!["claude"]);
        assert_eq!(values.args.len(), 2);
        assert_eq!(values.args[0], "-p");
        assert!(!values.secrets.anthropic_api_key.is_empty());
    }
}
