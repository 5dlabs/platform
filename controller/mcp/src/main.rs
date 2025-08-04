use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;
mod agents;

use agents::AgentsConfig;

// Global agents configuration loaded once at startup
static AGENTS_CONFIG: OnceLock<AgentsConfig> = OnceLock::new();

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
            let default_config = AgentsConfig::default();
            let agents_config = AGENTS_CONFIG.get().unwrap_or(&default_config);
            Some(Ok(tools::get_enhanced_tool_schemas(agents_config)))
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
        format!("working-directory={working_directory}"),
        format!("source-branch=main"), // Default branch
    ];

    // Add model parameter with default if not provided
    let model = arguments.get("model").and_then(|v| v.as_str()).unwrap_or("claude-opus-4-20250514");
    params.push(format!("model={model}"));

    // Handle GitHub App authentication with agent name resolution
    let default_config = AgentsConfig::default();
    let agents_config = AGENTS_CONFIG.get().unwrap_or(&default_config);
    let github_app = if let Some(input) = arguments.get("github_app").and_then(|v| v.as_str()) {
        // Try to resolve agent name (e.g., "Morgan" -> "5DLabs-Morgan")
        if let Some(agent) = agents_config.resolve_agent(input) {
            agent.github_app.clone()
        } else {
            input.to_string() // Use as-is if not found
        }
    } else if let Ok(env_app) = std::env::var("FDL_DEFAULT_GITHUB_APP") {
        env_app
    } else if let Some(default_agent) = agents_config.get_docs_agent() {
        default_agent.github_app.clone()
    } else {
        return Err(anyhow!("No GitHub App configured for docs workflow and no default docs agent found"));
    };
    params.push(format!("github-app={github_app}"));
    
    // For backward compatibility, check github_user but default to empty
    let github_user = arguments
        .get("github_user")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    params.push(format!("github-user={github_user}"));
    
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
        
    // GitHub App resolution with agent intelligence
    let agents_config = load_agents_config()?;
    
    // Determine which agent to use for this task
    let agent_name = arguments.get("agent")
        .and_then(|v| v.as_str())
        .or_else(|| {
            // Try to get default code agent from Helm configuration
            agents_config.get_code_agent().map(|agent| agent.name.as_str())
        })
        .unwrap_or("rex"); // Fallback to Rex as default code agent
    
    // Get GitHub App from the selected agent
    let github_app = if let Some(agent) = agents_config.agents.get(agent_name) {
        agent.github_app.clone()
    } else if let Ok(env_app) = std::env::var("FDL_DEFAULT_GITHUB_APP") {
        env_app
    } else if let Some(default_agent) = agents_config.get_code_agent() {
        default_agent.github_app.clone()
    } else {
        return Err(anyhow!("No GitHub App configured for agent '{}' and no default code agent found", agent_name));
    };
    
    // For backward compatibility, check github_user but default to empty
    let github_user = arguments
        .get("github_user")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let mut params = vec![
        format!("task-id={task_id}"),
        format!("service-id={service}"),
        format!("repository-url={repository}"),
        format!("docs-repository-url={docs_repository}"),
        format!("docs-project-directory={docs_project_directory}"),
        format!("github-app={github_app}"),
        format!("github-user={github_user}"),
    ];
    
    // Add optional parameters
    if let Some(working_directory) = arguments.get("working_directory").and_then(|v| v.as_str()) {
        params.push(format!("working-directory={working_directory}"));
    }
    
    if let Some(model) = arguments.get("model").and_then(|v| v.as_str()) {
        params.push(format!("model={model}"));
    }
    
    if let Some(continue_session) = arguments.get("continue_session").and_then(|v| v.as_bool()) {
        params.push(format!("continue-session={continue_session}"));
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
            "github_app": github_app,
            "agent": agent_name,
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
    eprintln!("🚀 Starting 5D Labs MCP Server...");
    
    // Initialize agents configuration
    let agents_config = AgentsConfig::load().unwrap_or_else(|e| {
        eprintln!("⚠️  Failed to load agents config: {}. Using defaults.", e);
        AgentsConfig::default()
    });
    
    // Store in global static
    AGENTS_CONFIG.set(agents_config).map_err(|_| anyhow!("Failed to set agents config"))?;
    eprintln!("✅ Agents configuration loaded");
    
    eprintln!("Creating runtime...");
    let rt = Runtime::new()?;
    eprintln!("Runtime created, starting RPC loop");
    rt.block_on(rpc_loop())?;
    eprintln!("RPC loop completed");
    Ok(())
}