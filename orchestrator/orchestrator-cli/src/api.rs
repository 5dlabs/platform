//! HTTP API client for communicating with the orchestrator service

use anyhow::{Context, Result};
use orchestrator_common::models::{
    pm_task::PmTaskRequest,
    request::CreateTaskRequest,
    response::{ApiResponse, JobResponse, TaskResponse},
};
use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};

/// API client for the orchestrator service
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Submit a new task
    #[allow(dead_code)]
    pub async fn submit_task(
        &self,
        microservice: &str,
        task_file: &str,
    ) -> Result<ApiResponse<TaskResponse>> {
        let task_content = tokio::fs::read_to_string(task_file)
            .await
            .with_context(|| format!("Failed to read task file: {task_file}"))?;

        // For simplicity, use the task content as both title and description
        let lines: Vec<&str> = task_content.lines().collect();
        let title = lines.first().unwrap_or(&"Untitled Task").trim().to_string();
        let description = if lines.len() > 1 {
            lines[1..].join("\n").trim().to_string()
        } else {
            task_content.clone()
        };

        let create_request = CreateTaskRequest {
            microservice: microservice.to_string(),
            title,
            description,
            acceptance_criteria: vec![],
            priority: None,
            agent_type: None,
            metadata: None,
        };

        info!("Submitting task for microservice: {microservice}");
        debug!("Task request: {:?}", create_request);

        let response = self
            .client
            .post(format!("{}/api/v1/tasks", self.base_url))
            .json(&create_request)
            .send()
            .await
            .context("Failed to send task submission request")?;

        self.handle_response(response).await
    }

    /// Submit a PM task with design specification
    pub async fn submit_pm_task(&self, request: &PmTaskRequest) -> Result<ApiResponse<Value>> {
        info!(
            "Submitting PM task: {} for service: {}",
            request.id, request.service_name
        );
        debug!("PM task request: {:?}", request);

        let response = self
            .client
            .post(format!("{}/api/v1/pm/tasks", self.base_url))
            .json(request)
            .send()
            .await
            .context("Failed to send PM task submission request")?;

        self.handle_response(response).await
    }

    /// Get task status by ID
    pub async fn get_task(&self, task_id: &str) -> Result<ApiResponse<TaskResponse>> {
        info!("Getting task status: {task_id}");

        let response = self
            .client
            .get(format!("{}/api/v1/tasks/{}", self.base_url, task_id))
            .send()
            .await
            .context("Failed to send get task request")?;

        self.handle_response(response).await
    }

    /// List tasks with optional filtering
    pub async fn list_tasks(
        &self,
        microservice: Option<&str>,
    ) -> Result<ApiResponse<Vec<TaskResponse>>> {
        let mut url = format!("{}/api/v1/tasks", self.base_url);
        let mut params = vec![];

        if let Some(microservice) = microservice {
            params.push(format!("microservice={microservice}"));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        info!("Listing tasks with URL: {url}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send list tasks request")?;

        self.handle_response(response).await
    }

    /// List jobs with optional filtering
    pub async fn list_jobs(
        &self,
        microservice: Option<&str>,
        status: Option<&str>,
    ) -> Result<ApiResponse<Vec<JobResponse>>> {
        let mut url = format!("{}/api/v1/jobs", self.base_url);
        let mut params = vec![];

        if let Some(microservice) = microservice {
            params.push(format!("microservice={microservice}"));
        }

        if let Some(status) = status {
            params.push(format!("status={status}"));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        info!("Listing jobs with URL: {url}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send list jobs request")?;

        self.handle_response(response).await
    }

    /// Get job details by ID
    pub async fn get_job(&self, job_id: &str) -> Result<ApiResponse<JobResponse>> {
        info!("Getting job details: {job_id}");

        let response = self
            .client
            .get(format!("{}/api/v1/jobs/{job_id}", self.base_url))
            .send()
            .await
            .context("Failed to send get job request")?;

        self.handle_response(response).await
    }

    /// Get job logs
    pub async fn get_job_logs(&self, job_id: &str, follow: bool) -> Result<String> {
        let mut url = format!("{}/api/v1/jobs/{job_id}/logs", self.base_url);
        if follow {
            url = format!("{url}?follow=true");
        }

        info!("Getting job logs: {job_id} (follow: {follow})");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send get job logs request")?;

        if response.status().is_success() {
            response
                .text()
                .await
                .context("Failed to read logs response body")
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ))
        }
    }

    /// Create a ConfigMap
    pub async fn create_configmap(
        &self,
        name: &str,
        files: &[String],
    ) -> Result<ApiResponse<Value>> {
        let mut data = HashMap::new();

        // Read all files and add to ConfigMap data
        for file_path in files {
            let content = tokio::fs::read_to_string(file_path)
                .await
                .with_context(|| format!("Failed to read file: {file_path}"))?;

            let file_name = std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_path);

            data.insert(file_name.to_string(), content);
        }

        let request_body = serde_json::json!({
            "name": name,
            "data": data
        });

        info!("Creating ConfigMap: {name}");
        debug!("ConfigMap data keys: {:?}", data.keys().collect::<Vec<_>>());

        let response = self
            .client
            .post(format!("{}/api/v1/configmaps", self.base_url))
            .json(&request_body)
            .send()
            .await
            .context("Failed to send create ConfigMap request")?;

        self.handle_response(response).await
    }

    /// Get ConfigMap by name
    pub async fn get_configmap(&self, name: &str) -> Result<ApiResponse<Value>> {
        info!("Getting ConfigMap: {name}");

        let response = self
            .client
            .get(format!("{}/api/v1/configmaps/{name}", self.base_url))
            .send()
            .await
            .context("Failed to send get ConfigMap request")?;

        self.handle_response(response).await
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<ApiResponse<Value>> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .context("Failed to send health check request")?;

        self.handle_response(response).await
    }

    /// Generic response handler
    async fn handle_response<T>(&self, response: Response) -> Result<ApiResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        debug!("API response status: {}", status);
        debug!("API response body: {}", response_text);

        if status.is_success() {
            serde_json::from_str(&response_text)
                .with_context(|| format!("Failed to parse successful response: {response_text}"))
        } else {
            // Try to parse as error response first
            if let Ok(error_response) = serde_json::from_str::<ApiResponse<T>>(&response_text) {
                Ok(error_response)
            } else {
                Err(anyhow::anyhow!(
                    "API request failed with status {}: {}",
                    status,
                    response_text
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_url_construction() {
        let client = ApiClient::new("http://localhost:8080".to_string());

        // Test task endpoint
        let task_url = format!("{}/api/v1/tasks", client.base_url);
        assert_eq!(task_url, "http://localhost:8080/api/v1/tasks");

        // Test job endpoint
        let job_url = format!("{}/api/v1/jobs/123", client.base_url);
        assert_eq!(job_url, "http://localhost:8080/api/v1/jobs/123");
    }
}
