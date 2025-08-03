/*
 * 5D Labs Agent Platform - MCP Tools for AI Coding Agents
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

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;
mod workflow_client;

use workflow_client::ArgoWorkflowsClient;

// Custom error type for production-ready error handling.
#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

// JSON-RPC Success Response structure.
#[derive(Serialize)]
struct RpcSuccessResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

// JSON-RPC Error Response structure.
#[derive(Serialize)]
struct RpcErrorResponse {
    jsonrpc: String,
    error: RpcError,
    id: Option<Value>,
}

// JSON-RPC Request structure.
#[derive(Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

/// Validate repository format (org/repo or user/repo)
fn validate_repository_format(repo: &str) -> Result<()> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(anyhow!(
            "Repository must be in format 'org/repo' or 'user/repo'"
        ));
    }
    Ok(())
}

/// Convert org/repo format to HTTPS URL
fn repo_to_https_url(repo: &str) -> String {
    format!("https://github.com/{repo}.git")
}

/// Get Argo Workflows client configuration from environment
fn get_argo_config() -> Result<(String, String)> {
    let base_url = env::var("ARGO_WORKFLOWS_URL")
        .unwrap_or_else(|_| "http://argo-workflows-server.argo.svc.cluster.local:2746".to_string());
    let namespace = env::var("ARGO_WORKFLOWS_NAMESPACE")
        .unwrap_or_else(|_| "orchestrator".to_string());
    
    Ok((base_url, namespace))
}

/// Create an Argo Workflows client
fn create_argo_client() -> Result<ArgoWorkflowsClient> {
    let (base_url, namespace) = get_argo_config()?;
    Ok(ArgoWorkflowsClient::new(base_url, namespace))
}

// Capabilities advertised by the server with full MCP tool schemas.
fn get_capabilities() -> Value {
    tools::get_all_tool_schemas()
}

// Extract parameters from JSON value into HashMap
fn extract_params(params: Option<&Value>) -> HashMap<String, Value> {
    params
        .and_then(|p| {
            p.as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        })
        .unwrap_or_default()
}

// Handle MCP protocol methods
fn handle_mcp_protocol_methods(
    method: &str,
    params_map: &HashMap<String, Value>,
) -> Option<Result<Value>> {
    match method {
        "initialize" => {
            // MCP initialization - validate required fields and return proper server capabilities
            let _protocol_version = params_map
                .get("protocolVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("2025-06-18");

            // Validate that required fields are present (as per MCP schema)
            if params_map.get("capabilities").is_none()
                || params_map.get("clientInfo").is_none()
                || params_map.get("protocolVersion").is_none()
            {
                return Some(Err(anyhow!("Missing required initialize parameters: capabilities, clientInfo, and protocolVersion are required")));
            }

            Some(Ok(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "orchestrator-mcp-argo",
                    "title": "Orchestrator MCP Server (Argo Workflows)",
                    "version": "1.0.0"
                }
            })))
        }
        "notifications/initialized" => {
            // MCP initialized notification - no response should be sent
            None
        }
        method if method.starts_with("notifications/") => {
            // Debug: catch any notifications we might be missing
            None
        }

        "tools/list" => {
            // Return list of available tools with schemas
            let capabilities = get_capabilities();
            // Debug output removed to satisfy clippy
            Some(Ok(capabilities))
        }
        _ => None,
    }
}

// Handle orchestrator tool methods (async version)
async fn handle_orchestrator_tools_async(
    method: &str,
    params_map: &HashMap<String, Value>,
) -> Option<Result<Value>> {
    match method {
        "docs" => {
            // Initialize documentation for Task Master tasks using Argo Workflows

            // Extract required working directory parameter
            let working_directory =
                match params_map.get("working_directory").and_then(|v| v.as_str()) {
                    Some(wd) => wd,
                    None => return Some(Err(anyhow!("working_directory parameter is required"))),
                };

            // Extract model parameter (no default - let workflow template handle it)
            let model = params_map.get("model").and_then(|v| v.as_str());

            // Get GitHub user from environment variable (takes precedence) or parameter
            let env_user = env::var("FDL_DEFAULT_DOCS_USER").ok();
            let github_user = match env_user
                .as_deref()
                .or_else(|| params_map.get("github_user").and_then(|v| v.as_str()))
            {
                Some(user) => user,
                None => return Some(Err(anyhow!("github_user parameter is required or FDL_DEFAULT_DOCS_USER environment variable must be set"))),
            };

            // Validate model parameter if provided - allow any model that starts with "claude-"
            if let Some(m) = model {
                if !m.starts_with("claude-") {
                    return Some(Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name (e.g., 'claude-opus-4-20250514')", m)));
                }
            }

            // Create Argo Workflows client
            let client = match create_argo_client() {
                Ok(c) => c,
                Err(e) => return Some(Err(anyhow!("Failed to create Argo client: {}", e))),
            };

            // Submit DocsRun workflow
            match client.submit_docsrun_workflow(working_directory, github_user, model).await {
                Ok(workflow_name) => Some(Ok(json!({
                    "success": true,
                    "message": "Documentation generation workflow submitted successfully",
                    "workflow_name": workflow_name,
                    "parameters_used": {
                        "model": model.unwrap_or("default from workflow template"),
                        "working_directory": working_directory,
                        "github_user": github_user
                    }
                }))),
                Err(e) => Some(Err(anyhow!("Failed to submit docs workflow: {}", e))),
            }
        }
        "task" => {
            // Submit a Task Master task for implementation using Argo Workflows

            // Extract required parameters
            let task_id = match params_map
                .get("task_id")
                .and_then(serde_json::Value::as_u64)
            {
                Some(id) => id,
                None => return Some(Err(anyhow!("Missing required parameter: task_id"))),
            };

            let service = params_map.get("service").and_then(|v| v.as_str());

            // Get service from environment variable (takes precedence) or parameter
            let env_service = env::var("FDL_DEFAULT_SERVICE").ok();
            let service = match env_service.as_deref().or(service) {
                Some(s) => s,
                None => return Some(Err(anyhow!("Missing required parameter: service (can also be set via FDL_DEFAULT_SERVICE environment variable)"))),
            };

            // Extract required repository parameters in org/repo format
            let repository = match params_map.get("repository").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return Some(Err(anyhow!("Missing required parameter: repository"))),
            };

            let docs_repository = match params_map.get("docs_repository").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return Some(Err(anyhow!("Missing required parameter: docs_repository"))),
            };

            // Extract required directory parameters
            let docs_project_directory = match params_map
                .get("docs_project_directory")
                .and_then(|v| v.as_str())
            {
                Some(d) => d,
                None => {
                    return Some(Err(anyhow!(
                        "Missing required parameter: docs_project_directory"
                    )))
                }
            };

            let working_directory = params_map
                .get("working_directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".");

            // Validate repository format (org/repo)
            if let Err(e) = validate_repository_format(repository) {
                return Some(Err(anyhow!(
                    "Invalid repository format '{}': {}",
                    repository,
                    e
                )));
            }
            if let Err(e) = validate_repository_format(docs_repository) {
                return Some(Err(anyhow!(
                    "Invalid docs_repository format '{}': {}",
                    docs_repository,
                    e
                )));
            }

            // Extract optional model parameter
            let model = params_map.get("model").and_then(|v| v.as_str());

            let github_user = match params_map.get("github_user").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => return Some(Err(anyhow!("Missing required parameter: github_user"))),
            };

            let continue_session = params_map
                .get("continue_session")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            let env = params_map.get("env");
            let env_from_secrets = params_map.get("env_from_secrets");

            // Validate model parameter if provided
            if let Some(m) = model {
                if !m.starts_with("claude-") {
                    return Some(Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name (e.g., 'claude-sonnet-4-20250514')", m)));
                }
            }

            // Validate service name (must be valid for PVC naming)
            if !service
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Some(Err(anyhow!("Invalid service name '{}'. Must contain only lowercase letters, numbers, and hyphens", service)));
            }

            // Create Argo Workflows client
            let client = match create_argo_client() {
                Ok(c) => c,
                Err(e) => return Some(Err(anyhow!("Failed to create Argo client: {}", e))),
            };

            // Submit CodeRun workflow
            match client.submit_coderun_workflow(
                task_id,
                service,
                repository,
                docs_repository,
                docs_project_directory,
                working_directory,
                github_user,
                model,
                continue_session,
                env,
                env_from_secrets,
            ).await {
                Ok(workflow_name) => Some(Ok(json!({
                    "success": true,
                    "message": "Implementation task workflow submitted successfully",
                    "workflow_name": workflow_name,
                    "parameters_used": {
                        "task_id": task_id,
                        "service": service,
                        "repository": repository,
                        "docs_repository": docs_repository,
                        "docs_project_directory": docs_project_directory,
                        "working_directory": working_directory,
                        "model": model.unwrap_or("default from workflow template"),
                        "github_user": github_user,
                        "continue_session": continue_session,
                        "env": env,
                        "env_from_secrets": env_from_secrets
                    }
                }))),
                Err(e) => Some(Err(anyhow!("Failed to submit code workflow: {}", e))),
            }
        }
        _ => None,
    }
}

// Handle tool invocation (async version)
async fn handle_tool_invocation_async(params_map: &HashMap<String, Value>) -> Result<Value> {
    let name = params_map
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing tool name"))?;
    let default_args = json!({});
    let arguments = params_map.get("arguments").unwrap_or(&default_args);

    // Extract arguments as a map for the tool handlers
    let args_map = extract_params(Some(arguments));

    // Try orchestrator tools
    if let Some(result) = handle_orchestrator_tools_async(name, &args_map).await {
        match result {
            Ok(content) => Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string())
                    }
                ]
            })),
            Err(e) => Err(e),
        }
    } else {
        Err(anyhow!("Unknown tool: {}", name))
    }
}

// Handle core MCP methods (including tool calls) (async version)
async fn handle_core_methods_async(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => Some(handle_tool_invocation_async(params_map).await),
        _ => None,
    }
}

// Handler for each method (following MCP specification) (async version).
async fn handle_method_async(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
    let params_map = extract_params(params);

    // Try MCP protocol methods FIRST (ping, initialize, tools/list, etc.)
    if let Some(result) = handle_mcp_protocol_methods(method, &params_map) {
        return Some(result); // Found a matching MCP method
    }

    // Special handling for notifications that should return None
    if method.starts_with("notifications/") {
        return None; // Notifications should not have responses
    }

    // Try core methods (tools/call)
    if let Some(result) = handle_core_methods_async(method, &params_map).await {
        return Some(result);
    }

    // Try orchestrator tools directly (for debugging)
    if let Some(result) = handle_orchestrator_tools_async(method, &params_map).await {
        return Some(result);
    }

    Some(Err(anyhow!("Unknown method: {}", method)))
}

// Main async RPC loop over stdio (from MCP specification).
async fn rpc_loop() -> Result<()> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let request: RpcRequest = serde_json::from_str(&line).context("Invalid JSON request")?;

        let result = handle_method_async(&request.method, request.params.as_ref()).await;
        if let Some(method_result) = result {
            let resp_json = match method_result {
                Ok(res) => {
                    let response = RpcSuccessResponse {
                        jsonrpc: "2.0".to_string(),
                        result: res,
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
                Err(err) => {
                    let response = RpcErrorResponse {
                        jsonrpc: "2.0".to_string(),
                        error: RpcError {
                            code: -32600,
                            message: err.to_string(),
                            data: None,
                        },
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
            };
            stdout.write_all((resp_json + "\n").as_bytes()).await?;
            stdout.flush().await?;
        }
        // If result is None, it's a notification - no response should be sent
    }
    Ok(())
}

fn main() -> Result<()> {
    let rt = Runtime::new()?;
    rt.block_on(rpc_loop())?;

    Ok(())
}
