use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
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

// Helper to run orchestrator CLI command and capture output.
fn run_orchestrator_cli(args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("orchestrator-cli");
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
                .unwrap_or("2024-11-05");

            // Validate that required fields are present (as per MCP schema)
            if params_map.get("capabilities").is_none()
                || params_map.get("clientInfo").is_none()
                || params_map.get("protocolVersion").is_none()
            {
                return Some(Err(anyhow!("Missing required initialize parameters: capabilities, clientInfo, and protocolVersion are required")));
            }

            Some(Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "orchestrator-mcp-server",
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
        "ping" => {
            // MCP ping request - respond with empty object for connection health
            Some(Ok(json!({})))
        }
        "tools/list" => {
            // Return list of available tools with schemas
            Some(Ok(get_capabilities()))
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
        "ping" => {
            // Test connectivity and show tool information
            Some(Ok(json!({
                "status": "OK",
                "server": "orchestrator-mcp-server",
                "version": "1.0.0",
                "environment": {
                    "working_directory": env::current_dir().unwrap_or_default().to_string_lossy(),
                    "orchestrator_cli_available": std::path::Path::new("./target/release/orchestrator-cli").exists()
                },
                "usage_examples": {
                    "init_docs": [
                        "init_docs()",
                        "init_docs({model: 'opus'})",
                        "init_docs({working_directory: '/absolute/path/to/project'})",
                        "init_docs({task_id: 5})",
                        "init_docs({force: true})"
                    ]
                }
            })))
        }
        "init_docs" => {
            // Initialize documentation for Task Master tasks
            eprintln!("DEBUG: MCP init_docs called with raw args: {:?}", params_map);

            // Extract parameters with defaults
            let model = params_map
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("opus");

            let working_directory = params_map
                .get("working_directory")
                .and_then(|v| v.as_str());

            let force = params_map
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let task_id = params_map
                .get("task_id")
                .and_then(|v| v.as_u64());

            // Validate model parameter
            if !["opus", "sonnet"].contains(&model) {
                return Some(Err(anyhow!("Invalid model '{}'. Must be 'opus' or 'sonnet'", model)));
            }

            // Build CLI arguments
            let mut args = vec!["task", "init-docs"];

            // Add model
            args.extend(&["--model", model]);

            // Add working directory if specified
            if let Some(wd) = working_directory {
                args.extend(&["--working-directory", wd]);
            }

            // Add force flag if true
            if force {
                args.push("--force");
            }

            // Add task ID if specified
            let task_id_str = task_id.map(|tid| tid.to_string());
            if let Some(ref tid_str) = task_id_str {
                args.extend(&["--task-id", tid_str]);
            }

            eprintln!("DEBUG: Running orchestrator-cli with args: {:?}", args);

            // Execute the CLI command
            match run_orchestrator_cli(&args) {
                Ok(output) => {
                    Some(Ok(json!({
                        "success": true,
                        "message": "Documentation generation initiated successfully",
                        "output": output,
                        "parameters_used": {
                            "model": model,
                            "working_directory": working_directory,
                            "force": force,
                            "task_id": task_id
                        }
                    })))
                }
                Err(e) => {
                    Some(Err(anyhow!("Failed to execute init-docs: {}", e)))
                }
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
fn handle_core_methods(
    method: &str,
    params_map: &HashMap<String, Value>,
) -> Option<Result<Value>> {
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