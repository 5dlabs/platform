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
                    "name": "orchestrator-mcp",
                    "title": "Orchestrator MCP Server",
                    "version": "1.0.0"
                }
            })))
        }
        "tools/list" => {
            Some(Ok(json!({
                "tools": [
                    {
                        "name": "docs",
                        "description": "Submit documentation generation workflow",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "working_directory": {
                                    "type": "string",
                                    "description": "Working directory"
                                }
                            }
                        }
                    },
                    {
                        "name": "task", 
                        "description": "Submit task implementation workflow",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": {
                                    "type": "integer",
                                    "description": "Task ID"
                                }
                            }
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
        .unwrap_or(".");
    
    let working_dir_param = format!("workingDirectory={working_directory}");
    let args = vec![
        "submit",
        "--from", "workflowtemplate/docsrun-template",
        "-n", "orchestrator",
        "-p", &working_dir_param,
    ];

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Documentation generation workflow submitted successfully",
            "output": output,
            "working_directory": working_directory
        })),
        Err(e) => Err(anyhow!("Failed to submit docs workflow: {}", e)),
    }
}

fn handle_task_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let task_id = arguments
        .get("task_id")
        .and_then(|v| v.as_u64())
        .ok_or(anyhow!("Missing required parameter: task_id"))?;
    
    let task_id_param = format!("taskId={task_id}");
    let args = vec![
        "submit",
        "--from", "workflowtemplate/coderun-template", 
        "-n", "orchestrator",
        "-p", &task_id_param,
    ];

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Task implementation workflow submitted successfully",
            "output": output,
            "task_id": task_id
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