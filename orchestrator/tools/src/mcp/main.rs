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
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;

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
        return Err(anyhow!("Repository must be in format 'org/repo' or 'user/repo'"));
    }
    Ok(())
}

/// Convert org/repo format to HTTPS URL
fn repo_to_https_url(repo: &str) -> String {
    format!("https://github.com/{repo}.git")
}

/// Run the orchestrator CLI command
fn run_orchestrator_cli(args: &[&str]) -> Result<String> {
    // Use the local build in the same directory as this MCP binary
    let mut cmd = Command::new("fdl");
    cmd.args(args);
    cmd.stderr(Stdio::piped());
    let output = cmd.output().context("Failed to execute orchestrator-cli")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!("orchestrator-cli failed: {}", err));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
                    "name": "orchestrator-mcp",
                    "title": "Orchestrator MCP Server",
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

// Handle orchestrator tool methods
fn handle_orchestrator_tools(
    method: &str,
    params_map: &HashMap<String, Value>,
) -> Option<Result<Value>> {
    match method {
        "docs" => {
            // Initialize documentation for Task Master tasks
            // Debug output removed to satisfy clippy

            // Extract required working directory parameter
            let working_directory =
                match params_map.get("working_directory").and_then(|v| v.as_str()) {
                    Some(wd) => wd,
                    None => return Some(Err(anyhow!("working_directory parameter is required"))),
                };

            // Extract model parameter (no default - let CLI/backend handle it)
            let model = params_map
                .get("model")
                .and_then(|v| v.as_str());

            // Get GitHub user from environment variable (takes precedence) or parameter
            let env_user = std::env::var("FDL_DEFAULT_DOCS_USER").ok();
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

            // Build CLI arguments
            let mut args = vec!["task", "docs"];

            // Add required parameters
            args.extend(&["--working-directory", working_directory]);
            args.extend(&["--github-user", github_user]);
            
            // Add model parameter only if provided
            if let Some(m) = model {
                args.extend(&["--model", m]);
            }

            // Debug output removed to satisfy clippy

            // Execute the CLI command
            match run_orchestrator_cli(&args) {
                Ok(output) => Some(Ok(json!({
                    "success": true,
                    "message": "Documentation generation initiated successfully",
                    "output": output,
                    "parameters_used": {
                        "model": model.unwrap_or("default from Helm configuration"),
                        "working_directory": working_directory,
                        "github_user": github_user
                    }
                }))),
                Err(e) => Some(Err(anyhow!("Failed to execute docs command: {}", e))),
            }
        }
        "task" => {
            // Submit a Task Master task for implementation
            // Debug output removed to satisfy clippy

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
            let env_service = std::env::var("FDL_DEFAULT_SERVICE").ok();
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
            let docs_project_directory = match params_map.get("docs_project_directory").and_then(|v| v.as_str()) {
                Some(d) => d,
                None => return Some(Err(anyhow!("Missing required parameter: docs_project_directory"))),
            };
            
            let working_directory = params_map
                .get("working_directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            
            // Validate repository format (org/repo)
            if let Err(e) = validate_repository_format(repository) {
                return Some(Err(anyhow!("Invalid repository format '{}': {}", repository, e)));
            }
            if let Err(e) = validate_repository_format(docs_repository) {
                return Some(Err(anyhow!("Invalid docs_repository format '{}': {}", docs_repository, e)));
            }


            // Extract optional model parameter (no default - let CLI/backend handle it)
            let model = params_map
                .get("model")
                .and_then(|v| v.as_str());

            let github_user = match params_map.get("github_user").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => return Some(Err(anyhow!("Missing required parameter: github_user"))),
            };



            let continue_session = params_map
                .get("continue_session")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            let env = params_map.get("env").and_then(|v| v.as_object());

            let env_from_secrets = params_map
                .get("env_from_secrets")
                .and_then(|v| v.as_array());

            // Validate model parameter if provided - allow any model that starts with "claude-"
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

            // Convert org/repo format to HTTPS URLs for CLI 
            let repository_url = repo_to_https_url(repository);
            let docs_repository_url = repo_to_https_url(docs_repository);

            // Build CLI arguments using the new CLI interface
            let mut args = vec!["task", "code"];

            // Add required parameters (task_id is positional, not a flag)
            let task_id_str = task_id.to_string();
            args.push(&task_id_str);
            args.extend(&["--service", service]);
            
            // Add required repository URLs (converted from org/repo format)
            args.extend(&["--repository-url", &repository_url]);
            args.extend(&["--docs-repository-url", &docs_repository_url]);
            
            // Add required directory parameters
            args.extend(&["--docs-project-directory", docs_project_directory]);
            args.extend(&["--working-directory", working_directory]);
            
            // Add model parameter only if provided
            if let Some(m) = model {
                args.extend(&["--model", m]);
            }

            // Add GitHub user (now required)
            args.extend(&["--github-user", github_user]);

            // Docs branch will be auto-detected by CLI

            // Add session flags
            if continue_session {
                args.push("--continue-session");
            }

            // Prepare environment variables string if specified
            #[allow(unused_assignments)]
            let mut env_string = String::new();
            if let Some(env_obj) = env {
                let mut env_pairs = Vec::new();
                for (key, value) in env_obj {
                    if let Some(val_str) = value.as_str() {
                        env_pairs.push(format!("{key}={val_str}"));
                    }
                }
                if !env_pairs.is_empty() {
                    env_string = env_pairs.join(",");
                    args.extend(&["--env", &env_string]);
                }
            }

            // Prepare environment variables from secrets string if specified
            #[allow(unused_assignments)]
            let mut secrets_string = String::new();
            if let Some(env_secrets_arr) = env_from_secrets {
                let mut secret_specs = Vec::new();
                for secret in env_secrets_arr {
                    if let Some(secret_obj) = secret.as_object() {
                        if let (Some(name), Some(secret_name), Some(secret_key)) = (
                            secret_obj.get("name").and_then(|v| v.as_str()),
                            secret_obj.get("secretName").and_then(|v| v.as_str()),
                            secret_obj.get("secretKey").and_then(|v| v.as_str()),
                        ) {
                            secret_specs.push(format!("{name}:{secret_name}:{secret_key}"));
                        }
                    }
                }
                if !secret_specs.is_empty() {
                    secrets_string = secret_specs.join(",");
                    args.extend(&["--env-from-secrets", &secrets_string]);
                }
            }

            // Debug output removed to satisfy clippy

            // Execute the CLI command
            match run_orchestrator_cli(&args) {
                Ok(output) => Some(Ok(json!({
                    "success": true,
                    "message": "Implementation task submitted successfully",
                    "output": output,
                    "parameters_used": {
                        "task_id": task_id,
                        "service": service,
                        "repository": repository,
                        "docs_repository": docs_repository,
                        "docs_project_directory": docs_project_directory,
                        "working_directory": working_directory,
                        "model": model.unwrap_or("default from Helm configuration"),
                        "github_user": github_user,
                        "continue_session": continue_session,
                        "env": env,
                        "env_from_secrets": env_from_secrets
                    }
                }))),
                Err(e) => Some(Err(anyhow!("Failed to execute submit task: {}", e))),
            }
        }
        _ => None,
    }
}

// Handle tool invocation
fn handle_tool_invocation(params_map: &HashMap<String, Value>) -> Result<Value> {
    let name = params_map
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing tool name"))?;
    let default_args = json!({});
    let arguments = params_map.get("arguments").unwrap_or(&default_args);

    // Extract arguments as a map for the tool handlers
    let args_map = extract_params(Some(arguments));

    // Try orchestrator tools
    if let Some(result) = handle_orchestrator_tools(name, &args_map) {
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

// Handle core MCP methods (including tool calls)
fn handle_core_methods(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => Some(handle_tool_invocation(params_map)),
        _ => None,
    }
}

// Handler for each method (following MCP specification).
fn handle_method(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
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
    if let Some(result) = handle_core_methods(method, &params_map) {
        return Some(result);
    }

    // Try orchestrator tools directly (for debugging)
    if let Some(result) = handle_orchestrator_tools(method, &params_map) {
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

        let result = handle_method(&request.method, request.params.as_ref());
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
