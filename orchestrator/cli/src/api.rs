//! HTTP API client for orchestrator TaskRun submissions

use anyhow::{Context, Result};
use common::models::{CodeRequest, DocsRequest};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

/// API response structure used by PM endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

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

    /// Submit a code task
    pub async fn submit_code_task(&self, request: &CodeRequest) -> Result<ApiResponse> {
        info!(
            "Submitting code task: {} for service: {}",
            request.task_id, request.service
        );
        debug!("Code task request: {:?}", request);

        let response = self
            .client
            .post(format!("{}/pm/tasks", self.base_url))
            .json(request)
            .send()
            .await
            .context("Failed to send code task submission request")?;

        self.handle_response(response).await
    }

    /// Submit a documentation generation job
    pub async fn submit_docs_generation(&self, request: &DocsRequest) -> Result<ApiResponse> {
        info!(
            "Submitting documentation generation job for repository: {}",
            request.repository_url
        );
        debug!("Docs generation request: {:?}", request);

        let response = self
            .client
            .post(format!("{}/pm/docs/generate", self.base_url))
            .json(request)
            .send()
            .await
            .context("Failed to send documentation generation request")?;

        self.handle_response(response).await
    }

    /// Generic response handler for API responses
    async fn handle_response(&self, response: Response) -> Result<ApiResponse> {
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
            if let Ok(error_response) = serde_json::from_str::<ApiResponse>(&response_text) {
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
