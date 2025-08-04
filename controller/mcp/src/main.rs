use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcSuccessResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Serialize)]
struct RpcErrorResponse {
    jsonrpc: String,
    error: RpcError,
    id: Option<Value>,
}

fn extract_params(params: Option<&Value>) -> HashMap<String, Value> {
    params
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn handle_mcp_methods(method: &str, _params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "initialize" => {
            Some(Ok(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "agent-platform-mcp",
                    "title": "Agent Platform MCP Server",
                    "version": "1.0.0"
                }
            })))
        }
        "tools/list" => {
            Some(Ok(json!({
                "tools": [
                    {
                        "name": "docs",
                        "description": "Initialize documentation for Task Master tasks using Claude",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "working_directory": {
                                    "type": "string",
                                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like '_projects/simple-api'."
                                },
                                "model": {
                                    "type": "string",
                                    "description": "Claude model to use (optional, defaults to Helm configuration value)"
                                },
                                "github_user": {
                                    "type": "string",
                                    "description": "GitHub username for authentication (optional if FDL_DEFAULT_DOCS_USER environment variable is set, which takes precedence)"
                                }
                            },
                            "required": ["working_directory"]
                        }
                    },
                    {
                        "name": "task", 
                        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": {
                                    "type": "integer",
                                    "description": "REQUIRED: Task ID to implement from task files",
                                    "minimum": 1
                                },
                                "service": {
                                    "type": "string",
                                    "description": "REQUIRED: Target service name (creates workspace-{service} PVC) - can be overridden by FDL_DEFAULT_SERVICE environment variable",
                                    "pattern": "^[a-z0-9-]+$"
                                },
                                "repository": {
                                    "type": "string",
                                    "description": "REQUIRED: Target repository in format 'org/repo' or 'user/repo' (e.g., '5dlabs/platform')"
                                },
                                "docs_repository": {
                                    "type": "string",
                                    "description": "REQUIRED: Documentation repository in format 'org/repo' or 'user/repo' where Task Master definitions are stored"
                                },
                                "docs_project_directory": {
                                    "type": "string",
                                    "description": "REQUIRED: Project directory within docs repository (e.g., '_projects/simple-api', use '.' for repo root)"
                                },
                                "github_user": {
                                    "type": "string",
                                    "description": "REQUIRED: GitHub username for authentication and task assignment"
                                },
                                "working_directory": {
                                    "type": "string",
                                    "description": "Working directory within target repository (optional, defaults to '.' for repo root)"
                                },
                                "model": {
                                    "type": "string",
                                    "description": "Claude model to use (optional, defaults to Helm configuration value)"
                                },
                                "continue_session": {
                                    "type": "boolean",
                                    "description": "Whether to continue a previous session (optional, defaults to false)"
                                },
                                "env": {
                                    "type": "object",
                                    "description": "Environment variables to set in the container (optional)",
                                    "additionalProperties": {
                                        "type": "string"
                                    }
                                },
                                "env_from_secrets": {
                                    "type": "array",
                                    "description": "Environment variables from secrets (optional)",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": {
                                                "type": "string",
                                                "description": "Name of the environment variable"
                                            },
                                            "secretName": {
                                                "type": "string",
                                                "description": "Name of the secret"
                                            },
                                            "secretKey": {
                                                "type": "string",
                                                "description": "Key within the secret"
                                            }
                                        },
                                        "required": ["name", "secretName", "secretKey"]
                                    }
                                }
                            },
                            "required": ["task_id", "service", "repository", "docs_repository", "docs_project_directory", "github_user"]
                        }
                    }
                ]
            })))
        }
        _ => None,
    }
}

fn run_argo_cli(args: &[&str]) -> Result<String> {
    let output = Command::new("argo")
        .args(args)
        .output()
        .context("Failed to execute argo command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Argo command failed: {}", stderr))
    }
}

fn handle_docs_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: working_directory"))?;
    
    let mut params = vec![
        format!("workingDirectory={working_directory}")
    ];
    
    // Add optional model parameter
    if let Some(model) = arguments.get("model").and_then(|v| v.as_str()) {
        params.push(format!("model={model}"));
    }
    
    // Add optional github_user parameter
    if let Some(github_user) = arguments.get("github_user").and_then(|v| v.as_str()) {
        params.push(format!("githubUser={github_user}"));
    }
    
    let mut args = vec![
        "submit",
        "--from", "workflowtemplate/docsrun-template",
        "-n", "agent-platform",
    ];
    
    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Documentation generation workflow submitted successfully",
            "output": output,
            "working_directory": working_directory,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit docs workflow: {}", e)),
    }
}

fn handle_task_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let task_id = arguments
        .get("task_id")
        .and_then(|v| v.as_u64())
        .ok_or(anyhow!("Missing required parameter: task_id"))?;
    
    let service = arguments
        .get("service")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: service"))?;
        
    let repository = arguments
        .get("repository")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: repository"))?;
        
    let docs_repository = arguments
        .get("docs_repository")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: docs_repository"))?;
        
    let docs_project_directory = arguments
        .get("docs_project_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: docs_project_directory"))?;
        
    let github_user = arguments
        .get("github_user")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: github_user"))?;
    
    let mut params = vec![
        format!("taskId={task_id}"),
        format!("service={service}"),
        format!("repository={repository}"),
        format!("docsRepository={docs_repository}"),
        format!("docsProjectDirectory={docs_project_directory}"),
        format!("githubUser={github_user}"),
    ];
    
    // Add optional parameters
    if let Some(working_directory) = arguments.get("working_directory").and_then(|v| v.as_str()) {
        params.push(format!("workingDirectory={working_directory}"));
    }
    
    if let Some(model) = arguments.get("model").and_then(|v| v.as_str()) {
        params.push(format!("model={model}"));
    }
    
    if let Some(continue_session) = arguments.get("continue_session").and_then(|v| v.as_bool()) {
        params.push(format!("continueSession={continue_session}"));
    }
    
    // Handle env object - convert to JSON string for workflow parameter
    if let Some(env) = arguments.get("env").and_then(|v| v.as_object()) {
        let env_json = serde_json::to_string(env)?;
        params.push(format!("env={env_json}"));
    }
    
    // Handle env_from_secrets array - convert to JSON string for workflow parameter
    if let Some(env_from_secrets) = arguments.get("env_from_secrets").and_then(|v| v.as_array()) {
        let env_from_secrets_json = serde_json::to_string(env_from_secrets)?;
        params.push(format!("envFromSecrets={env_from_secrets_json}"));
    }
    
    let mut args = vec![
        "submit",
        "--from", "workflowtemplate/coderun-template", 
        "-n", "agent-platform",
    ];
    
    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Task implementation workflow submitted successfully",
            "output": output,
            "task_id": task_id,
            "service": service,
            "repository": repository,
            "docs_repository": docs_repository,
            "docs_project_directory": docs_project_directory,
            "github_user": github_user,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit task workflow: {}", e)),
    }
}

fn handle_tool_calls(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => {
            let name = params_map
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(anyhow!("Missing tool name"));
            
            let arguments = params_map
                .get("arguments")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();
            
            match name {
                Ok("docs") => Some(handle_docs_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("task") => Some(handle_task_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok(unknown) => Some(Err(anyhow!("Unknown tool: {}", unknown))),
                Err(e) => Some(Err(e)),
            }
        }
        _ => None,
    }
}

fn handle_method(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
    let params_map = extract_params(params);

    // Try MCP protocol methods first
    if let Some(result) = handle_mcp_methods(method, &params_map) {
        return Some(result);
    }

    // Handle notifications (no response)
    if method.starts_with("notifications/") {
        return None;
    }

    // Try tool calls
    if let Some(result) = handle_tool_calls(method, &params_map) {
        return Some(result);
    }

    Some(Err(anyhow!("Unknown method: {}", method)))
}

#[allow(clippy::disallowed_macros)]
async fn rpc_loop() -> Result<()> {
    eprintln!("Starting RPC loop");
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        eprintln!("Received line: {line}");
        let request: RpcRequest = serde_json::from_str(&line).context("Invalid JSON request")?;
        eprintln!("Parsed request for method: {}", request.method);

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
    }
    Ok(())
}

#[allow(clippy::disallowed_macros)]
fn main() -> Result<()> {
    eprintln!("Creating runtime...");
    let rt = Runtime::new()?;
    eprintln!("Runtime created, starting block_on");
    rt.block_on(rpc_loop())?;
    eprintln!("block_on completed");
    Ok(())
}