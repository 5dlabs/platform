//! Common types and utilities for request handlers

use anyhow::{anyhow, Result};
use axum::http::StatusCode;
use serde_json::Value;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::api::Api;
use kube::Client;
use tracing::{info, warn};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub k8s_client: Client,
    pub namespace: String,
}

/// Error type for API handlers
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            AppError::Conflict(msg) => write!(f, "Conflict: {msg}"),
            AppError::Internal(msg) => write!(f, "Internal Error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<kube::Error> for AppError {
    fn from(e: kube::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<AppError> for StatusCode {
    fn from(err: AppError) -> Self {
        match err {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// API response structure
#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ApiResponse {
    #[must_use]
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    #[must_use]
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Validate user-specified tools exist
pub async fn validate_tools(
    local_tools: &[String],
    remote_tools: &[String],
    k8s_client: Client,
) -> Result<()> {
    // Local tools have fixed set
    const VALID_LOCAL_TOOLS: &[&str] = &["filesystem", "git"];

    for local_tool in local_tools {
        if !VALID_LOCAL_TOOLS.contains(&local_tool.as_str()) {
            return Err(anyhow!("Invalid local tool: {}", local_tool));
        }
    }

    // Remote tools must exist in toolman config
    if !remote_tools.is_empty() {
        info!("Discovering available MCP tools from Toolman ConfigMap");

        let configmaps: Api<ConfigMap> = Api::namespaced(k8s_client, "mcp");

        // Read toolman-config ConfigMap
        match configmaps.get("toolman-config").await {
            Ok(cm) => {
                let servers_json = cm.data
                    .as_ref()
                    .and_then(|d| d.get("servers-config.json"))
                    .ok_or_else(|| anyhow!("servers-config.json not found in toolman-config"))?;

                let config: serde_json::Value = serde_json::from_str(servers_json)?;
                let servers = config.get("servers").and_then(|s| s.as_object())
                    .ok_or_else(|| anyhow!("Invalid servers configuration"))?;

                let available_tools: Vec<String> = servers.keys().cloned().collect();

                info!("Discovered {} available MCP tools", available_tools.len());

                for remote_tool in remote_tools {
                    if !available_tools.contains(remote_tool) {
                        return Err(anyhow!(
                            "Remote tool '{}' not found in toolman configuration. Available tools: {:?}",
                            remote_tool,
                            available_tools
                        ));
                    }
                }
            }
            Err(e) => {
                warn!("Toolman ConfigMap not found, skipping validation: {}", e);
                // If we can't find the ConfigMap, skip validation to not block tasks
            }
        }
    }

    Ok(())
}
