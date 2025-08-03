/*
 * 5D Labs Agent Platform - Argo Workflows MCP Client
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

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

/// Argo Workflows client for submitting and monitoring workflows
pub struct ArgoWorkflowsClient {
    client: reqwest::Client,
    base_url: String,
    namespace: String,
}

impl ArgoWorkflowsClient {
    pub fn new(base_url: String, namespace: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            base_url,
            namespace,
        }
    }
    
    pub async fn new_with_timeout(base_url: String, namespace: String, timeout_secs: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;
            
        Ok(Self {
            client,
            base_url,
            namespace,
        })
    }

    /// Submit a CodeRun workflow to Argo Workflows
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_coderun_workflow(
        &self,
        task_id: u64,
        service: &str,
        repository: &str,
        docs_repository: &str,
        docs_project_directory: &str,
        docs_branch: Option<&str>,
        working_directory: &str,
        github_user: &str,
        model: Option<&str>,
        continue_session: bool,
        env: Option<&Value>,
        env_from_secrets: Option<&Value>,
    ) -> Result<String> {
        let uuid_str = Uuid::new_v4().to_string();
        let workflow_name = format!("coderun-{}-{}", service, &uuid_str[..8]);
        
        let mut parameter_pairs = vec![
            format!("task-id={}", task_id),
            format!("service-id={}", service),
            format!("repository-url={}", repository),
            format!("docs-repository-url={}", docs_repository),
            format!("docs-project-directory={}", docs_project_directory),
            format!("working-directory={}", working_directory),
            format!("github-user={}", github_user),
            format!("continue-session={}", continue_session),
            format!("overwrite-memory=false"), // Add missing overwrite-memory parameter
            format!("docs-branch={}", docs_branch.unwrap_or("main")), // Add docs-branch parameter
        ];

        // Always include model parameter, use default if not provided
        let model_value = model.unwrap_or("claude-3-5-sonnet-20241022");
        parameter_pairs.push(format!("model={}", model_value));

        // TODO: Handle env and env_from_secrets parameters
        if let Some(env_obj) = env {
            if let Some(env_map) = env_obj.as_object() {
                for (key, value) in env_map {
                    if let Some(val_str) = value.as_str() {
                        parameter_pairs.push(format!("env-{}={}", key, val_str));
                    }
                }
            }
        }
        
        if let Some(secrets_arr) = env_from_secrets {
            if let Some(secrets_array) = secrets_arr.as_array() {
                for (i, secret) in secrets_array.iter().enumerate() {
                    if let Some(secret_obj) = secret.as_object() {
                        if let (Some(name), Some(secret_name), Some(secret_key)) = (
                            secret_obj.get("name").and_then(|v| v.as_str()),
                            secret_obj.get("secretName").and_then(|v| v.as_str()),
                            secret_obj.get("secretKey").and_then(|v| v.as_str()),
                        ) {
                            parameter_pairs.push(format!("secret-{}-{}={}:{}:{}", i, name, name, secret_name, secret_key));
                        }
                    }
                }
            }
        }

        let submit_request = json!({
            "namespace": self.namespace,
            "resourceKind": "WorkflowTemplate",
            "resourceName": "coderun-template",
            "submitOptions": {
                "parameters": parameter_pairs,
                "name": workflow_name
            }
        });

        self.submit_workflow_request(submit_request).await?;
        
        Ok(workflow_name)
    }

    /// Submit a DocsRun workflow to Argo Workflows
    pub async fn submit_docsrun_workflow(
        &self,
        working_directory: &str,
        github_user: &str,
        source_branch: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let uuid_str = Uuid::new_v4().to_string();
        let workflow_name = format!("docsrun-{}", &uuid_str[..8]);
        
        let mut parameter_pairs = vec![
            format!("working-directory={}", working_directory),
            format!("github-user={}", github_user),
            format!("source-branch={}", source_branch),
        ];

        // Always include model parameter, use default if not provided
        let model_value = model.unwrap_or("claude-opus-4-20250514");
        parameter_pairs.push(format!("model={}", model_value));

        let submit_request = json!({
            "namespace": self.namespace,
            "resourceKind": "WorkflowTemplate", 
            "resourceName": "docsrun-template",
            "submitOptions": {
                "parameters": parameter_pairs,
                "name": workflow_name
            }
        });

        self.submit_workflow_request(submit_request).await?;
        
        Ok(workflow_name)
    }

    /// Get workflow status
    pub async fn get_workflow_status(&self, workflow_name: &str) -> Result<Value> {
        let url = format!("{}/api/v1/workflows/{}/{}", self.base_url, self.namespace, workflow_name);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get workflow status: {}", error_text));
        }

        let workflow: Value = response.json().await?;
        Ok(workflow)
    }

    /// Get workflow logs
    pub async fn get_workflow_logs(&self, workflow_name: &str) -> Result<String> {
        let url = format!("{}/api/v1/workflows/{}/{}/log", self.base_url, self.namespace, workflow_name);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get workflow logs: {}", error_text));
        }

        let logs = response.text().await?;
        Ok(logs)
    }

    /// List workflows with optional filters
    pub async fn list_workflows(&self, label_selector: Option<&str>) -> Result<Value> {
        let mut url = format!("{}/api/v1/workflows/{}", self.base_url, self.namespace);
        
        if let Some(labels) = label_selector {
            url.push_str(&format!("?listOptions.labelSelector={}", labels));
        }
        
        let response = timeout(Duration::from_secs(10), self.client.get(&url).send()).await??;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to list workflows: {}", error_text));
        }

        let workflows: Value = response.json().await?;
        Ok(workflows)
    }
    
    /// Check if Argo Workflows server is accessible
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/v1/version", self.base_url);
        
        match timeout(Duration::from_secs(5), self.client.get(&url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }
    
    /// Get workflow events
    pub async fn get_workflow_events(&self, workflow_name: &str) -> Result<Value> {
        let url = format!("{}/api/v1/workflows/{}/{}/events", self.base_url, self.namespace, workflow_name);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get workflow events: {}", error_text));
        }

        let events: Value = response.json().await?;
        Ok(events)
    }

    /// Internal method to submit workflow requests
    async fn submit_workflow_request(&self, submit_request: Value) -> Result<Value> {
        let url = format!("{}/api/v1/workflows/{}/submit", self.base_url, self.namespace);
        
        let response = timeout(
            Duration::from_secs(30),
            self.client
                .post(&url)
                .json(&submit_request)
                .send()
        ).await??;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Workflow submission failed: {}", error_text));
        }

        let workflow: Value = response.json().await?;
        Ok(workflow)
    }
}