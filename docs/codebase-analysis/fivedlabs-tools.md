# fivedlabs-tools Analysis

**Path:** `orchestrator/tools`
**Type:** RustLibrary
**Lines of Code:** 2413
**Description:** 5D Labs platform tools: CLI and MCP server for AI development workflows

## Dependencies

- clap
- colored
- reqwest
- serde
- serde_json
- anyhow
- tracing
- tracing-subscriber
- tokio
- chrono
- common

## Source Files

### src/mcp/tools.rs (143 lines)

**Key Definitions:**
```rust
4:pub fn get_all_tool_schemas() -> Value {
```

**Full Content:**
```rust
use serde_json::{json, Value};

/// Get all tool schemas with descriptions and parameter definitions
pub fn get_all_tool_schemas() -> Value {
    json!({
        "tools": [
            get_init_docs_schema(),
            get_submit_implementation_task_schema()
        ]
    })
}

fn get_init_docs_schema() -> Value {
    json!({
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
                    "description": "Claude model to use (default: 'claude-opus-4-20250514')",
                    "default": "claude-opus-4-20250514"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (optional if FDL_DEFAULT_DOCS_USER environment variable is set, which takes precedence)"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_submit_implementation_task_schema() -> Value {
    json!({
        "name": "task",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "REQUIRED: Task ID to implement from tasks.json",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "REQUIRED: Target service name (creates workspace-{service} PVC)",
                    "pattern": "^[a-z0-9-]+$"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (required)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (default: 'claude-sonnet-4-20250514')",
                    "default": "claude-sonnet-4-20250514"
                },
                "docs_repository_url": {
                    "type": "string",
                    "description": "Documentation repository URL (where Task Master definitions come from)"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g. '_projects/simple-api')"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (optional if FDL_DEFAULT_CODE_USER environment variable is set, which takes precedence)"
                },
                "local_tools": {
                    "type": "string",
                    "description": "Comma-separated list of local MCP tools/servers to enable (e.g., 'mcp-server-git,taskmaster')"
                },
                "remote_tools": {
                    "type": "string",
                    "description": "Comma-separated list of remote MCP tools/servers to enable (e.g., 'api-docs-tool')"
                },
                "context_version": {
                    "type": "integer",
                    "description": "Context version for retry attempts (incremented on each retry, default: 1)",
                    "minimum": 1,
                    "default": 1
                },
                "prompt_modification": {
                    "type": "string",
                    "description": "Additional context for retry attempts"
                },
                "docs_branch": {
                    "type": "string",
                    "description": "Docs branch to use (e.g., 'main', 'feature/branch', default: 'main')",
                    "default": "main"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (auto-continue on retries or user-requested, default: false)",
                    "default": false
                },
                "overwrite_memory": {
                    "type": "boolean",
                    "description": "Whether to overwrite memory before starting (default: false)",
                    "default": false
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables to set in the container (key-value pairs)",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "env_from_secrets": {
                    "type": "array",
                    "description": "Environment variables from secrets",
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
            "required": ["task_id", "service", "working_directory"]
        }
    })
}

```

### src/mcp/main.rs (548 lines)

**Full Content:**
```rust
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

            // Extract model with default
            let model = params_map
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("claude-opus-4-20250514");

            // Get GitHub user from environment variable (takes precedence) or parameter
            let env_user = std::env::var("FDL_DEFAULT_DOCS_USER").ok();
            let github_user = match env_user
                .as_deref()
                .or_else(|| params_map.get("github_user").and_then(|v| v.as_str()))
            {
                Some(user) => user,
                None => return Some(Err(anyhow!("github_user parameter is required or FDL_DEFAULT_DOCS_USER environment variable must be set"))),
            };

            // Validate model parameter - allow any model that starts with "claude-"
            if !model.starts_with("claude-") {
                return Some(Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name (e.g., 'claude-opus-4-20250514')", model)));
            }

            // Build CLI arguments
            let mut args = vec!["task", "docs"];

            // Add required parameters
            args.extend(&["--model", model]);
            args.extend(&["--working-directory", working_directory]);
            args.extend(&["--github-user", github_user]);

            // Debug output removed to satisfy clippy

            // Execute the CLI command
            match run_orchestrator_cli(&args) {
                Ok(output) => Some(Ok(json!({
                    "success": true,
                    "message": "Documentation generation initiated successfully",
                    "output": output,
                    "parameters_used": {
                        "model": model,
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

            let service = match params_map.get("service").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return Some(Err(anyhow!("Missing required parameter: service"))),
            };

            // Extract optional parameters with defaults
            let docs_repository_url = params_map
                .get("docs_repository_url")
                .and_then(|v| v.as_str());

            let docs_project_directory = params_map
                .get("docs_project_directory")
                .and_then(|v| v.as_str());

            let working_directory = params_map.get("working_directory").and_then(|v| v.as_str());

            // Extract parameters with task-specific default
            let model = params_map
                .get("model")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());

            let model = match model {
                Some(m) => m,
                None => return Some(Err(anyhow!("Model parameter is required. Please specify a model like 'claude-opus-4-20250514' or 'claude-sonnet-4-20250514'"))),
            };

            let github_user = params_map.get("github_user").and_then(|v| v.as_str());

            // Get GitHub user from environment variable (takes precedence) or parameter
            let env_code_user = std::env::var("FDL_DEFAULT_CODE_USER").ok();
            let github_user = env_code_user.as_deref().or(github_user);

            let local_tools = params_map.get("local_tools").and_then(|v| v.as_str());

            let remote_tools = params_map.get("remote_tools").and_then(|v| v.as_str());

            let context_version = params_map
                .get("context_version")
                .and_then(serde_json::Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())
                .unwrap_or(1);

            let prompt_modification = params_map
                .get("prompt_modification")
                .and_then(|v| v.as_str());

            let docs_branch = params_map
                .get("docs_branch")
                .and_then(|v| v.as_str())
                .unwrap_or("main");

            let continue_session = params_map
                .get("continue_session")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            let overwrite_memory = params_map
                .get("overwrite_memory")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            let env = params_map.get("env").and_then(|v| v.as_object());

            let env_from_secrets = params_map
                .get("env_from_secrets")
                .and_then(|v| v.as_array());

            // Validate model parameter - allow any model that starts with "claude-"
            if !model.starts_with("claude-") {
                return Some(Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name (e.g., 'claude-sonnet-4-20250514')", model)));
            }

            // Validate service name (must be valid for PVC naming)
            if !service
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Some(Err(anyhow!("Invalid service name '{}'. Must contain only lowercase letters, numbers, and hyphens", service)));
            }

            // Build CLI arguments using the new CLI interface
            let mut args = vec!["task", "code"];

            // Add required parameters (task_id is positional, not a flag)
            let task_id_str = task_id.to_string();
            args.push(&task_id_str);
            args.extend(&["--service", service]);

            // Add model parameter
            args.extend(&["--model", model]);

            // Add docs repository URL if specified
            if let Some(docs_repo) = docs_repository_url {
                args.extend(&["--docs-repository-url", docs_repo]);
            }

            // Add docs project directory if specified
            if let Some(docs_proj_dir) = docs_project_directory {
                args.extend(&["--docs-project-directory", docs_proj_dir]);
            }

            // Add working directory if specified
            if let Some(wd) = working_directory {
                args.extend(&["--working-directory", wd]);
            }

            // Add GitHub user if specified
            if let Some(user) = github_user {
                args.extend(&["--github-user", user]);
            }

            // Add tool configuration parameters
            if let Some(local) = local_tools {
                args.extend(&["--local-tools", local]);
            }

            if let Some(remote) = remote_tools {
                args.extend(&["--remote-tools", remote]);
            }

            // Add context version
            let context_version_str = context_version.to_string();
            args.extend(&["--context-version", &context_version_str]);

            // Add prompt modification if specified
            if let Some(prompt_mod) = prompt_modification {
                args.extend(&["--prompt-modification", prompt_mod]);
            }

            // Add docs branch
            args.extend(&["--docs-branch", docs_branch]);

            // Add session flags
            if continue_session {
                args.push("--continue-session");
            }

            if overwrite_memory {
                args.push("--overwrite-memory");
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
                        "docs_repository_url": docs_repository_url,
                        "docs_project_directory": docs_project_directory,
                        "working_directory": working_directory,
                        "model": model,
                        "github_user": github_user,
                        "local_tools": local_tools,
                        "remote_tools": remote_tools,
                        "context_version": context_version,
                        "prompt_modification": prompt_modification,
                        "docs_branch": docs_branch,
                        "continue_session": continue_session,
                        "overwrite_memory": overwrite_memory,
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

```

### src/cli/analyzer.rs (810 lines)

**Key Definitions:**
```rust
8:pub struct CodebaseAnalysis {
16:pub struct ProjectOverview {
25:pub struct ProjectStatistics {
34:pub struct Component {
45:pub enum ComponentType {
55:pub struct SourceFile {
64:pub struct ApiDefinition {
72:pub struct ApiEndpoint {
80:pub struct DataModel {
88:pub struct ConfigFile {
95:pub struct CodebaseAnalyzer {
100:impl CodebaseAnalyzer {
101:pub fn new(workspace_root: PathBuf, include_source: bool) -> Self {
108:pub fn analyze(&self) -> Result<CodebaseAnalysis> {
614:pub fn generate_modular_markdown(&self, analysis: &CodebaseAnalysis, output_dir: &str) -> Result<()> {
779:pub fn generate_single_markdown(&self, analysis: &CodebaseAnalysis) -> Result<String> {
```

**Full Content:**
```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    pub overview: ProjectOverview,
    pub components: Vec<Component>,
    pub apis: Vec<ApiDefinition>,
    pub configurations: Vec<ConfigFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectOverview {
    pub name: String,
    pub description: String,
    pub architecture: String,
    pub technologies: Vec<String>,
    pub statistics: ProjectStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub rust_crates: usize,
    pub total_rs_files: usize,
    pub total_lines_of_code: usize,
    pub config_files: usize,
    pub components_analyzed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub path: String,
    pub component_type: ComponentType,
    pub source_files: Vec<SourceFile>,
    pub dependencies: Vec<String>,
    pub description: String,
    pub line_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentType {
    RustBinary,
    RustLibrary,
    HelmChart,
    KubernetesConfig,
    Documentation,
    Scripts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub file_type: String,
    pub line_count: usize,
    pub key_definitions: Vec<String>,
    pub content: Option<String>, // Only included if include_source is true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub name: String,
    pub file_path: String,
    pub endpoints: Vec<ApiEndpoint>,
    pub data_models: Vec<DataModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub line_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub model_type: String,
    pub fields: Vec<String>,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub config_type: String,
    pub content: Option<String>,
}

pub struct CodebaseAnalyzer {
    workspace_root: PathBuf,
    include_source: bool,
}

impl CodebaseAnalyzer {
    pub fn new(workspace_root: PathBuf, include_source: bool) -> Self {
        Self {
            workspace_root,
            include_source,
        }
    }

        pub fn analyze(&self) -> Result<CodebaseAnalysis> {
        println!("🔍 Analyzing codebase at: {}", self.workspace_root.display());

        let mut overview = self.analyze_project_overview()?;
        let components = self.analyze_components()?;
        let apis = self.analyze_apis()?;
        let configurations = self.analyze_configurations()?;

        // Update the components analyzed count
        overview.statistics.components_analyzed = components.len();

        Ok(CodebaseAnalysis {
            overview,
            components,
            apis,
            configurations,
        })
    }

    fn analyze_project_overview(&self) -> Result<ProjectOverview> {
        println!("📋 Analyzing project overview...");

        let readme_path = self.workspace_root.join("README.md");
        let mut description = String::new();
        let mut name = "Unknown Project".to_string();

        if readme_path.exists() {
            let content = fs::read_to_string(&readme_path)?;
            if let Some(first_line) = content.lines().find(|line| !line.starts_with('#') && !line.trim().is_empty()) {
                description = first_line.to_string();
            }
            if let Some(header) = content.lines().find(|line| line.starts_with("# ")) {
                name = header.trim_start_matches("# ").to_string();
            }
        }

        // Calculate statistics
        let rust_crates = self.count_cargo_files()?;
        let (total_rs_files, total_lines_of_code) = self.count_rust_files()?;
        let config_files = self.count_config_files()?;

        Ok(ProjectOverview {
            name,
            description,
            architecture: "Kubernetes-based orchestrator with MCP integration".to_string(),
            technologies: vec![
                "Rust".to_string(),
                "Kubernetes".to_string(),
                "Helm".to_string(),
                "MCP".to_string(),
                "Docker".to_string(),
            ],
            statistics: ProjectStatistics {
                rust_crates,
                total_rs_files,
                total_lines_of_code,
                config_files,
                components_analyzed: 0, // Will be set correctly in analyze()
            },
        })
    }

    fn analyze_components(&self) -> Result<Vec<Component>> {
        println!("🔧 Analyzing components...");

        let mut components = Vec::new();

        // Analyze Rust components
        self.analyze_rust_components(&mut components)?;

        // Analyze infrastructure components
        self.analyze_infra_components(&mut components)?;

        println!("✅ Found {} components", components.len());
        Ok(components)
    }

    fn analyze_rust_components(&self, components: &mut Vec<Component>) -> Result<()> {
        let cargo_files = self.find_files_by_name("Cargo.toml")?;

        for cargo_path in cargo_files {
            if cargo_path.to_string_lossy().contains("target/") {
                continue;
            }

            let component_dir = cargo_path.parent().unwrap();
            let rel_path = component_dir.strip_prefix(&self.workspace_root)
                .unwrap_or(component_dir)
                .to_string_lossy()
                .to_string();

            let cargo_content = fs::read_to_string(&cargo_path)?;
            let component_name = self.extract_cargo_name(&cargo_content)
                .unwrap_or_else(|| component_dir.file_name().unwrap().to_string_lossy().to_string());

            let component_type = if component_dir.join("src/main.rs").exists() {
                ComponentType::RustBinary
            } else {
                ComponentType::RustLibrary
            };

            let source_files = self.analyze_rust_source_files(component_dir)?;
            let dependencies = self.extract_dependencies(&cargo_content);
            let line_count = source_files.iter().map(|f| f.line_count).sum();

            components.push(Component {
                name: component_name,
                path: rel_path,
                component_type,
                source_files,
                dependencies,
                description: self.extract_description(&cargo_content),
                line_count,
            });
        }

        Ok(())
    }

    fn analyze_rust_source_files(&self, component_dir: &Path) -> Result<Vec<SourceFile>> {
        let mut source_files = Vec::new();

        self.walk_directory(component_dir, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel_path = path.strip_prefix(component_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    let line_count = content.lines().count();
                    let key_definitions = self.extract_key_definitions(&content);

                    source_files.push(SourceFile {
                        path: rel_path,
                        file_type: "rust".to_string(),
                        line_count,
                        key_definitions,
                        content: if self.include_source { Some(content) } else { None },
                    });
                }
            }
        })?;

        Ok(source_files)
    }

    fn analyze_infra_components(&self, components: &mut Vec<Component>) -> Result<()> {
        let infra_components = vec![
            ("helm-charts", "infra/charts", ComponentType::HelmChart),
            ("kubernetes-config", "infra/cluster-config", ComponentType::KubernetesConfig),
            ("scripts", "infra/scripts", ComponentType::Scripts),
            ("documentation", "docs", ComponentType::Documentation),
        ];

        for (name, path, comp_type) in infra_components {
            let full_path = self.workspace_root.join(path);
            if full_path.exists() {
                let source_files = self.analyze_config_files(&full_path)?;
                let line_count = source_files.iter().map(|f| f.line_count).sum();

                components.push(Component {
                    name: name.to_string(),
                    path: path.to_string(),
                    component_type: comp_type,
                    source_files,
                    dependencies: Vec::new(),
                    description: format!("{} configuration and files", name),
                    line_count,
                });
            }
        }

        Ok(())
    }

    fn analyze_config_files(&self, dir: &Path) -> Result<Vec<SourceFile>> {
        let mut source_files = Vec::new();

        self.walk_directory(dir, &mut |path| {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if matches!(ext, "yaml" | "yml" | "toml" | "json" | "md" | "sh") {
                    if let Ok(content) = fs::read_to_string(path) {
                        let rel_path = path.strip_prefix(dir)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();

                        let line_count = content.lines().count();

                        source_files.push(SourceFile {
                            path: rel_path,
                            file_type: ext.to_string(),
                            line_count,
                            key_definitions: Vec::new(),
                            content: if self.include_source { Some(content) } else { None },
                        });
                    }
                }
            }
        })?;

        Ok(source_files)
    }

        fn analyze_apis(&self) -> Result<Vec<ApiDefinition>> {
        println!("🌐 Analyzing API surface...");

        let mut apis = Vec::new();

        // Look for route definitions in main.rs files and handler files
        let mut api_files = Vec::new();

        // Find main.rs files that define routes
        let main_files = self.find_files_by_name("main.rs")?;
        for file in main_files {
            if file.to_string_lossy().contains("orchestrator") && !file.to_string_lossy().contains("target") {
                api_files.push(file);
            }
        }

        // Find handler files
        let handler_files = self.find_files_in_path("handlers")?;
        for file in handler_files {
            if file.extension().and_then(|s| s.to_str()) == Some("rs") {
                api_files.push(file);
            }
        }

        for file in api_files {
            let content = fs::read_to_string(&file)?;
            let endpoints = self.extract_api_endpoints(&content);

            if !endpoints.is_empty() {
                let file_name = if file.file_name().unwrap().to_string_lossy() == "main.rs" {
                    "routes".to_string()
                } else {
                    file.file_stem().unwrap().to_string_lossy().to_string()
                };

                apis.push(ApiDefinition {
                    name: file_name,
                    file_path: file.strip_prefix(&self.workspace_root)
                        .unwrap_or(&file)
                        .to_string_lossy()
                        .to_string(),
                    endpoints,
                    data_models: Vec::new(), // Could be enhanced
                });
            }
        }

        Ok(apis)
    }

    fn analyze_configurations(&self) -> Result<Vec<ConfigFile>> {
        println!("⚙️  Analyzing configurations...");

        let mut configs = Vec::new();

        let extensions = vec!["yaml", "yml", "toml", "json"];
        for ext in extensions {
            let files = self.find_files_by_extension(ext)?;
            for file in files {
                let rel_path = file.strip_prefix(&self.workspace_root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .to_string();

                if rel_path.contains("target/") || rel_path.contains(".git/") {
                    continue;
                }

                let content = if self.include_source {
                    fs::read_to_string(&file).ok()
                } else {
                    None
                };

                configs.push(ConfigFile {
                    name: file.file_name().unwrap().to_string_lossy().to_string(),
                    path: rel_path,
                    config_type: ext.to_string(),
                    content,
                });
            }
        }

        Ok(configs)
    }

    // Helper methods
    fn count_cargo_files(&self) -> Result<usize> {
        Ok(self.find_files_by_name("Cargo.toml")?
            .into_iter()
            .filter(|p| !p.to_string_lossy().contains("target/"))
            .count())
    }

    fn count_rust_files(&self) -> Result<(usize, usize)> {
        let rust_files = self.find_files_by_extension("rs")?;
        let file_count = rust_files.len();
        let mut total_lines = 0;

        for file in rust_files {
            if let Ok(content) = fs::read_to_string(&file) {
                total_lines += content.lines().count();
            }
        }

        Ok((file_count, total_lines))
    }

    fn count_config_files(&self) -> Result<usize> {
        let mut count = 0;
        for ext in &["yaml", "yml", "toml", "json"] {
            count += self.find_files_by_extension(ext)?.len();
        }
        Ok(count)
    }

    fn find_files_by_name(&self, name: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.file_name().and_then(|s| s.to_str()) == Some(name) {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn find_files_by_extension(&self, ext: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn find_files_in_path(&self, subpath: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_directory(&self.workspace_root, &mut |path| {
            if path.to_string_lossy().contains(subpath) && path.is_file() {
                files.push(path.to_path_buf());
            }
        })?;
        Ok(files)
    }

    fn walk_directory<F>(&self, dir: &Path, callback: &mut F) -> Result<()>
    where
        F: FnMut(&Path),
    {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name == "target" || name == ".git" || name.starts_with('.') {
                        continue;
                    }
                }

                callback(&path);

                if path.is_dir() {
                    self.walk_directory(&path, callback)?;
                }
            }
        }
        Ok(())
    }

    fn extract_cargo_name(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            if line.starts_with("name =") {
                return line.split('=').nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string());
            }
        }
        None
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = Vec::new();
        let mut in_deps_section = false;

        for line in content.lines() {
            if line.starts_with("[dependencies]") {
                in_deps_section = true;
                continue;
            }
            if line.starts_with('[') && in_deps_section {
                break;
            }
            if in_deps_section && line.contains('=') && !line.starts_with('#') {
                if let Some(dep_name) = line.split('=').next() {
                    deps.push(dep_name.trim().to_string());
                }
            }
        }

        deps
    }

    fn extract_description(&self, content: &str) -> String {
        for line in content.lines() {
            if line.starts_with("description =") {
                return line.split('=').nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }
        "No description available".to_string()
    }

    fn extract_key_definitions(&self, content: &str) -> Vec<String> {
        let mut definitions = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("pub struct")
                || line.trim().starts_with("pub enum")
                || line.trim().starts_with("pub fn")
                || line.trim().starts_with("impl ")
                || line.trim().starts_with("pub trait") {
                definitions.push(format!("{}:{}", line_num + 1, line.trim()));
            }
        }

        definitions
    }

        fn extract_api_endpoints(&self, content: &str) -> Vec<ApiEndpoint> {
        let mut endpoints = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Look for axum route definitions
            if trimmed.contains(".route(") {
                // Handle .route("/path", method(handler)) syntax
                if let Some(path) = self.extract_route_path(trimmed) {
                    let method = if trimmed.contains("post(") { "POST" }
                    else if trimmed.contains("get(") { "GET" }
                    else if trimmed.contains("put(") { "PUT" }
                    else if trimmed.contains("delete(") { "DELETE" }
                    else { "ANY" };

                    endpoints.push(ApiEndpoint {
                        method: method.to_string(),
                        path,
                        handler: trimmed.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
            // Also look for direct method calls like .get("/path", handler)
            else if trimmed.contains(".get(") || trimmed.contains(".post(")
                || trimmed.contains(".put(") || trimmed.contains(".delete(") {
                if let Some(method) = self.extract_http_method(trimmed) {
                    if let Some(path) = self.extract_route_path(trimmed) {
                        endpoints.push(ApiEndpoint {
                            method,
                            path,
                            handler: trimmed.to_string(),
                            line_number: line_num + 1,
                        });
                    }
                }
            }
        }

        endpoints
    }

    fn extract_http_method(&self, line: &str) -> Option<String> {
        if line.contains(".get(") { Some("GET".to_string()) }
        else if line.contains(".post(") { Some("POST".to_string()) }
        else if line.contains(".put(") { Some("PUT".to_string()) }
        else if line.contains(".delete(") { Some("DELETE".to_string()) }
        else { Some("ROUTE".to_string()) }
    }

    fn extract_route_path(&self, line: &str) -> Option<String> {
        // Look for quoted strings that look like routes (start with /)
        let mut start_pos = 0;
        while let Some(start) = line[start_pos..].find('"') {
            let actual_start = start_pos + start;
            if let Some(end) = line[actual_start + 1..].find('"') {
                let path = &line[actual_start + 1..actual_start + 1 + end];
                if path.starts_with('/') {
                    return Some(path.to_string());
                }
                start_pos = actual_start + 1 + end + 1;
            } else {
                break;
            }
        }
        None
    }

    pub fn generate_modular_markdown(&self, analysis: &CodebaseAnalysis, output_dir: &str) -> Result<()> {
        println!("📝 Generating modular markdown documentation...");

        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path)?;

        // Generate master index
        self.generate_index_file(analysis, output_path)?;

        // Generate component files
        for component in &analysis.components {
            self.generate_component_file(component, output_path)?;
        }

        // Generate API documentation
        if !analysis.apis.is_empty() {
            self.generate_api_file(&analysis.apis, output_path)?;
        }

        // Generate configuration summary
        self.generate_config_file(&analysis.configurations, output_path)?;

        println!("✅ Modular documentation generated in: {}", output_dir);
        Ok(())
    }

    fn generate_index_file(&self, analysis: &CodebaseAnalysis, output_path: &Path) -> Result<()> {
        let index_file = output_path.join("README.md");
        let mut content = String::new();

        content.push_str(&format!("# {} - Codebase Analysis\n\n", analysis.overview.name));
        content.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        content.push_str("## Overview\n\n");
        content.push_str(&format!("**Description:** {}\n\n", analysis.overview.description));
        content.push_str(&format!("**Architecture:** {}\n\n", analysis.overview.architecture));
        content.push_str(&format!("**Technologies:** {}\n\n", analysis.overview.technologies.join(", ")));

        content.push_str("## Statistics\n\n");
        let stats = &analysis.overview.statistics;
        content.push_str(&format!("- **Rust Crates:** {}\n", stats.rust_crates));
        content.push_str(&format!("- **Rust Files:** {}\n", stats.total_rs_files));
        content.push_str(&format!("- **Lines of Code:** {}\n", stats.total_lines_of_code));
        content.push_str(&format!("- **Config Files:** {}\n", stats.config_files));
        content.push_str(&format!("- **Components:** {}\n\n", analysis.components.len()));

        content.push_str("## Components\n\n");
        for component in &analysis.components {
            content.push_str(&format!("- [{}](./{}.md) - `{}` ({} lines)\n",
                component.name,
                component.name.replace(' ', "-").to_lowercase(),
                component.path,
                component.line_count
            ));
        }

        if !analysis.apis.is_empty() {
            content.push_str("\n- [API Surface](./api-surface.md) - REST endpoints and data models\n");
        }
        content.push_str("- [Configurations](./configurations.md) - All configuration files\n");

        fs::write(index_file, content)?;
        Ok(())
    }

    fn generate_component_file(&self, component: &Component, output_path: &Path) -> Result<()> {
        let filename = format!("{}.md", component.name.replace(' ', "-").to_lowercase());
        let file_path = output_path.join(filename);
        let mut content = String::new();

        content.push_str(&format!("# {} Analysis\n\n", component.name));
        content.push_str(&format!("**Path:** `{}`\n", component.path));
        content.push_str(&format!("**Type:** {:?}\n", component.component_type));
        content.push_str(&format!("**Lines of Code:** {}\n", component.line_count));
        content.push_str(&format!("**Description:** {}\n\n", component.description));

        if !component.dependencies.is_empty() {
            content.push_str("## Dependencies\n\n");
            for dep in &component.dependencies {
                content.push_str(&format!("- {}\n", dep));
            }
            content.push_str("\n");
        }

        content.push_str("## Source Files\n\n");
        for source_file in &component.source_files {
            content.push_str(&format!("### {} ({} lines)\n\n", source_file.path, source_file.line_count));

            if !source_file.key_definitions.is_empty() {
                content.push_str("**Key Definitions:**\n```rust\n");
                for def in &source_file.key_definitions {
                    content.push_str(&format!("{}\n", def));
                }
                content.push_str("```\n\n");
            }

            if let Some(file_content) = &source_file.content {
                content.push_str("**Full Content:**\n```");
                content.push_str(&source_file.file_type);
                content.push_str("\n");
                content.push_str(file_content);
                content.push_str("\n```\n\n");
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_api_file(&self, apis: &[ApiDefinition], output_path: &Path) -> Result<()> {
        let file_path = output_path.join("api-surface.md");
        let mut content = String::new();

        content.push_str("# API Surface Analysis\n\n");

        for api in apis {
            content.push_str(&format!("## {} ({})\n\n", api.name, api.file_path));

            if !api.endpoints.is_empty() {
                content.push_str("### Endpoints\n\n");
                for endpoint in &api.endpoints {
                    content.push_str(&format!("- **{}** `{}` - Line {}\n",
                        endpoint.method, endpoint.path, endpoint.line_number));
                    content.push_str(&format!("  ```rust\n  {}\n  ```\n\n", endpoint.handler));
                }
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_config_file(&self, configs: &[ConfigFile], output_path: &Path) -> Result<()> {
        let file_path = output_path.join("configurations.md");
        let mut content = String::new();

        content.push_str("# Configuration Files\n\n");

        let mut configs_by_type: HashMap<String, Vec<&ConfigFile>> = HashMap::new();
        for config in configs {
            configs_by_type.entry(config.config_type.clone())
                .or_insert_with(Vec::new)
                .push(config);
        }

        for (config_type, type_configs) in configs_by_type {
            content.push_str(&format!("## {} Files\n\n", config_type.to_uppercase()));

            for config in type_configs {
                content.push_str(&format!("### {} ({})\n\n", config.name, config.path));

                if let Some(file_content) = &config.content {
                    content.push_str("```");
                    content.push_str(&config.config_type);
                    content.push_str("\n");
                    content.push_str(file_content);
                    content.push_str("\n```\n\n");
                }
            }
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn generate_single_markdown(&self, analysis: &CodebaseAnalysis) -> Result<String> {
        let mut content = String::new();

        content.push_str(&format!("# {} - Complete Codebase Analysis\n\n", analysis.overview.name));
        content.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        content.push_str("## Project Overview\n\n");
        content.push_str(&format!("**Description:** {}\n\n", analysis.overview.description));
        content.push_str(&format!("**Architecture:** {}\n\n", analysis.overview.architecture));
        content.push_str(&format!("**Technologies:** {}\n\n", analysis.overview.technologies.join(", ")));

        content.push_str("## Components\n\n");
        for component in &analysis.components {
            content.push_str(&format!("### {} ({})\n\n", component.name, component.path));
            content.push_str(&format!("**Type:** {:?} | **Lines:** {}\n\n", component.component_type, component.line_count));
            content.push_str(&format!("{}\n\n", component.description));

            for source_file in &component.source_files {
                if let Some(file_content) = &source_file.content {
                    content.push_str(&format!("#### {}\n\n", source_file.path));
                    content.push_str("```");
                    content.push_str(&source_file.file_type);
                    content.push_str("\n");
                    content.push_str(file_content);
                    content.push_str("\n```\n\n");
                }
            }
        }

        Ok(content)
    }
}
```

### src/cli/docs_generator.rs (207 lines)

**Key Definitions:**
```rust
11:pub struct DocsGenerator;
13:impl DocsGenerator {
15:pub fn prepare_for_submission(
```

**Full Content:**
```rust
//! Minimal documentation generator for preparing local files before submission

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Simple docs generator that handles local file preparation
pub struct DocsGenerator;

impl DocsGenerator {
    /// Prepare documentation files and return git info for submission
    pub fn prepare_for_submission(
        working_directory: Option<&str>,
    ) -> Result<(String, String, String, String)> {
        info!("Preparing documentation files for submission...");

        // Auto-detect git repository URL
        let repo_url = Self::get_git_remote_url()?;

        // Auto-detect working directory (relative path from repo root to current dir)
        let working_dir = Self::get_working_directory(working_directory)?;

        // Auto-detect source branch
        let source_branch = Self::get_current_branch()?;

        // Generate unique target branch name with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let target_branch = format!("docs-generation-{timestamp}");

        info!("Repository: {}", repo_url);
        info!("Working directory: {}", working_dir);
        info!("Source branch: {}", source_branch);
        info!("Target branch: {}", target_branch);

        // Check and commit .taskmaster changes if needed
        Self::check_and_commit_taskmaster_changes(&working_dir, &source_branch)?;

        // Create documentation directory structure and copy task files
        Self::create_docs_structure(&working_dir)?;

        Ok((repo_url, working_dir, source_branch, target_branch))
    }

    fn get_git_remote_url() -> Result<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .context("Failed to get git remote URL")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to detect git repository URL. Please specify with --repository-url"
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_working_directory(working_directory: Option<&str>) -> Result<String> {
        if let Some(wd) = working_directory {
            return Ok(wd.to_string());
        }

        let current_dir = std::env::current_dir()?;
        let repo_root = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get git repo root")?
            .stdout;
        let repo_root_string = String::from_utf8(repo_root)?;
        let repo_root = repo_root_string.trim();

        let rel_path = current_dir
            .strip_prefix(repo_root)
            .context("Current directory is not in repo")?
            .to_string_lossy()
            .to_string();

        Ok(if rel_path.is_empty() {
            ".".to_string()
        } else {
            rel_path
        })
    }

    fn get_current_branch() -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get current git branch")?;

        if !output.status.success() {
            return Ok("main".to_string());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn check_and_commit_taskmaster_changes(working_dir: &str, source_branch: &str) -> Result<()> {
        let taskmaster_path = format!("{working_dir}/.taskmaster");

        if !Path::new(&taskmaster_path).exists() {
            anyhow::bail!("No .taskmaster directory found in {}", working_dir);
        }

        info!("Checking for uncommitted .taskmaster changes...");

        let status_output = Command::new("git")
            .args(["status", "--porcelain", &taskmaster_path])
            .output()
            .context("Failed to check git status")?;

        if status_output.stdout.is_empty() {
            info!("No uncommitted changes in .taskmaster directory");
        } else {
            info!("Found uncommitted changes in .taskmaster directory");

            Command::new("git")
                .args(["add", &taskmaster_path])
                .status()
                .context("Failed to add .taskmaster files")?;

            Command::new("git")
                .args([
                    "commit",
                    "-m",
                    "chore: auto-commit .taskmaster directory for documentation generation",
                ])
                .status()
                .context("Failed to commit .taskmaster files")?;

            info!("Pushing commit to remote...");
            let push_result = Command::new("git")
                .args(["push", "origin", source_branch])
                .status()
                .context("Failed to push commits")?;

            if !push_result.success() {
                anyhow::bail!("Failed to push .taskmaster commit");
            }

            info!("✓ Auto-committed and pushed .taskmaster directory");
        }

        Ok(())
    }

    fn create_docs_structure(working_dir: &str) -> Result<()> {
        info!("Creating documentation directory structure...");

        let taskmaster_path = format!("{working_dir}/.taskmaster");
        let tasks_json_path = format!("{taskmaster_path}/tasks/tasks.json");

        if !Path::new(&tasks_json_path).exists() {
            anyhow::bail!("No tasks.json found at {}", tasks_json_path);
        }

        let content = fs::read_to_string(&tasks_json_path).context("Failed to read tasks.json")?;

        let json: Value = serde_json::from_str(&content).context("Failed to parse tasks.json")?;

        let tasks = json
            .get("master")
            .and_then(|m| m.get("tasks"))
            .and_then(|t| t.as_array())
            .context("No tasks found in tasks.json")?;

        let docs_dir = format!("{taskmaster_path}/docs");
        fs::create_dir_all(&docs_dir).context("Failed to create docs directory")?;

        let mut created_count = 0;
        for task in tasks {
            if let Some(task_id) = task.get("id").and_then(serde_json::Value::as_u64) {
                if let Some(title) = task.get("title").and_then(|t| t.as_str()) {
                    let task_dir = format!("{docs_dir}/task-{task_id}");
                    fs::create_dir_all(&task_dir)
                        .context(format!("Failed to create directory for task {task_id}"))?;

                    let source_file = format!("{taskmaster_path}/tasks/task_{task_id:03}.txt");
                    let dest_file = format!("{task_dir}/task.txt");

                    if Path::new(&source_file).exists() {
                        fs::copy(&source_file, &dest_file)
                            .context(format!("Failed to copy task file for task {task_id}"))?;
                        info!("✓ Copied task file for task {}: {}", task_id, title);
                    } else {
                        info!(
                            "⚠ No task file found for task {} (expected: {})",
                            task_id, source_file
                        );
                    }

                    created_count += 1;
                }
            }
        }

        info!(
            "✓ Created documentation structure for {} tasks",
            created_count
        );
        Ok(())
    }
}

```

### src/cli/commands.rs (388 lines)

**Key Definitions:**
```rust
355:pub fn handle_analyze_command(
```

**Full Content:**
```rust
use anyhow::Result;
use common::models::{CodeRequest, DocsRequest};
use std::path::PathBuf;

use crate::api::ApiClient;
use crate::docs_generator::DocsGenerator;
use crate::output::OutputManager;

/// Handle task command routing
pub async fn handle_task_command(
    command: crate::TaskCommands,
    api_url: &str,
    _output_format: &str,
) -> Result<()> {
    let api_client = ApiClient::new(api_url.to_string());
    let output = OutputManager::new();

    match command {
        crate::TaskCommands::Docs {
            working_directory,
            model,
            repository_url,
            source_branch,
            github_user,
        } => {
            handle_docs_command(
                &api_client,
                &output,
                working_directory.as_deref(),
                model.as_deref(),
                repository_url.as_deref(),
                source_branch.as_deref(),
                &github_user,
            )
            .await
        }
        crate::TaskCommands::Code {
            task_id,
            service,
            repository_url,
            docs_repository_url,
            docs_project_directory,
            github_user,
            working_directory,
            model,
            local_tools,
            remote_tools,
            context_version,
            prompt_modification,
            docs_branch,
            continue_session,
            overwrite_memory,
            env,
            env_from_secrets,
        } => {
            handle_code_command(
                &api_client,
                &output,
                task_id,
                &service,
                repository_url.as_deref(),
                docs_repository_url.as_deref(),
                docs_project_directory.as_deref(),
                &github_user,
                working_directory.as_deref(),
                model.as_deref(),
                local_tools.as_deref(),
                remote_tools.as_deref(),
                context_version,
                prompt_modification.as_deref(),
                &docs_branch,
                continue_session,
                overwrite_memory,
                env.as_deref(),
                env_from_secrets.as_deref(),
            )
            .await
        }
    }
}

/// Handle docs command - does local file prep then submits docs generation job
async fn handle_docs_command(
    api_client: &ApiClient,
    output: &OutputManager,
    working_directory: Option<&str>,
    model: Option<&str>,
    repository_url: Option<&str>,
    source_branch: Option<&str>,
    github_user: &str,
) -> Result<()> {
    output.info("Initializing documentation generator...");

    // Do local file preparation and get git info (used as fallbacks)
    let (detected_repo_url, detected_working_dir, detected_source_branch, _generated_docs_branch) =
        DocsGenerator::prepare_for_submission(working_directory)?;

    // Use provided parameters or fall back to auto-detected values
    let final_repo_url = repository_url.unwrap_or(&detected_repo_url);
    let final_working_dir = working_directory.unwrap_or(&detected_working_dir);
    let final_source_branch = source_branch.unwrap_or(&detected_source_branch);

    // Create documentation generation request
    let request = DocsRequest {
        repository_url: final_repo_url.to_string(),
        working_directory: final_working_dir.to_string(),
        source_branch: final_source_branch.to_string(),
        model: model.map(|s| s.to_string()),
        github_user: github_user.to_string(),
    };

    output.info("Submitting documentation generation job...");

    match api_client.submit_docs_generation(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message);

                if let Some(data) = response.data {
                    if let Some(taskrun_name) = data.get("taskrun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("TaskRun name: {taskrun_name}"));
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"));
                        output.info("You can monitor the job with:");
                        output.info(&format!("  kubectl -n {namespace} get taskrun"));
                    }
                }
            } else {
                output.error(&response.message);
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!(
                "Failed to submit documentation generation job: {e}"
            ));
            return Err(e);
        }
    }

    Ok(())
}

/// Handle code command - submits code task directly
#[allow(clippy::too_many_arguments)]
async fn handle_code_command(
    api_client: &ApiClient,
    output: &OutputManager,
    task_id: u32,
    service: &str,
    repository_url: Option<&str>,
    docs_repository_url: Option<&str>,
    docs_project_directory: Option<&str>,
    github_user: &str,
    working_directory: Option<&str>,
    model: Option<&str>,
    local_tools: Option<&str>,
    remote_tools: Option<&str>,
    context_version: u32,
    prompt_modification: Option<&str>,
    docs_branch: &str,
    continue_session: bool,
    overwrite_memory: bool,
    env: Option<&str>,
    env_from_secrets: Option<&str>,
) -> Result<()> {
    output.info(&format!(
        "Submitting code task {task_id} for service '{service}'..."
    ));

    // Auto-detect target repository URL if not provided
    let repo_url = match repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?,
    };

    // Auto-detect docs repository URL if not provided
    let docs_repo_url = match docs_repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?, // TODO: This should be configurable
    };

    // Use provided GitHub user (now required)
    let github_user_name = github_user.to_string();

    // Auto-detect working directory if not provided
    let working_dir = match working_directory {
        Some(wd) => wd.to_string(),
        None => get_working_directory()?,
    };

    // Parse environment variables
    let env_map = parse_env_vars(env)?;
    let env_from_secrets_vec = parse_env_from_secrets(env_from_secrets)?;

    // Create code task request
    let request = CodeRequest {
        task_id,
        service: service.to_string(),
        repository_url: repo_url.clone(),
        docs_repository_url: docs_repo_url.clone(),
        docs_project_directory: docs_project_directory.map(std::string::ToString::to_string),
        working_directory: Some(working_dir.clone()),
        model: model.map(|s| s.to_string()),
        github_user: github_user_name.clone(),
        local_tools: local_tools.map(std::string::ToString::to_string),
        remote_tools: remote_tools.map(std::string::ToString::to_string),
        context_version,
        prompt_modification: prompt_modification.map(std::string::ToString::to_string),
        docs_branch: docs_branch.to_string(),
        continue_session,
        overwrite_memory,
        env: env_map,
        env_from_secrets: env_from_secrets_vec,
    };

    output.info(&format!("Target repository: {repo_url}"));
    output.info(&format!("Docs repository: {docs_repo_url}"));
    output.info(&format!("Docs branch: {docs_branch}"));
    output.info(&format!("Working directory: {working_dir}"));
    output.info(&format!("Context version: {context_version}"));
    output.info(&format!("GitHub user: {github_user_name}"));

    match api_client.submit_code_task(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message);

                if let Some(data) = response.data {
                    if let Some(coderun_name) = data.get("coderun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("CodeRun name: {coderun_name}"));
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"));
                        output.info("You can monitor the job with:");
                        output.info(&format!("  kubectl -n {namespace} get coderun"));
                    }
                }
            } else {
                output.error(&response.message);
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!("Failed to submit code task: {e}"));
            return Err(e);
        }
    }

    Ok(())
}

/// Helper functions for git operations
fn get_git_remote_url() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git remote URL");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn get_working_directory() -> Result<String> {
    use std::process::Command;

    let current_dir = std::env::current_dir()?;
    let repo_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?
        .stdout;
    let repo_root_string = String::from_utf8(repo_root)?;
    let repo_root = repo_root_string.trim();

    let rel_path = current_dir
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .to_string();

    Ok(if rel_path.is_empty() {
        ".".to_string()
    } else {
        rel_path
    })
}

/// Parse environment variables from comma-separated key=value string
fn parse_env_vars(env_str: Option<&str>) -> Result<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;

    let mut env_map = HashMap::new();

    if let Some(env_str) = env_str {
        for pair in env_str.split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let mut parts = pair.splitn(2, '=');
            let key = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid env format: {}", pair))?;
            let value = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid env format: {}", pair))?;

            env_map.insert(key.to_string(), value.to_string());
        }
    }

    Ok(env_map)
}

/// Parse environment variables from secrets in format: name:secretName:secretKey,...
fn parse_env_from_secrets(
    env_secrets_str: Option<&str>,
) -> Result<Vec<common::models::code_request::SecretEnvVar>> {
    use common::models::code_request::SecretEnvVar;

    let mut secrets = Vec::new();

    if let Some(secrets_str) = env_secrets_str {
        for secret_spec in secrets_str.split(',') {
            let secret_spec = secret_spec.trim();
            if secret_spec.is_empty() {
                continue;
            }

            let parts: Vec<&str> = secret_spec.split(':').collect();
            if parts.len() != 3 {
                anyhow::bail!(
                    "Invalid secret env format: {}. Expected name:secretName:secretKey",
                    secret_spec
                );
            }

            secrets.push(SecretEnvVar {
                name: parts[0].to_string(),
                secret_name: parts[1].to_string(),
                secret_key: parts[2].to_string(),
            });
        }
    }

    Ok(secrets)
}

/// Handle analyze command
pub fn handle_analyze_command(
    output: String,
    format: String,
    working_directory: Option<String>,
    include_source: bool,
) -> Result<()> {
    use crate::analyzer::CodebaseAnalyzer;

    let work_dir = working_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let analyzer = CodebaseAnalyzer::new(work_dir, include_source);
    let analysis = analyzer.analyze()?;

    match format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&analysis)?;
            std::fs::write(&output, json_output)?;
            println!("✅ Codebase analysis written to: {} (JSON format)", output);
        }
        "single" => {
            let markdown_output = analyzer.generate_single_markdown(&analysis)?;
            std::fs::write(&output, markdown_output)?;
            println!("✅ Codebase analysis written to: {} (Single Markdown)", output);
        }
        "modular" | _ => {
            analyzer.generate_modular_markdown(&analysis, &output)?;
            println!("✅ Modular codebase analysis written to: {}/", output);
        }
    }

    Ok(())
}

```

### src/cli/output.rs (26 lines)

**Key Definitions:**
```rust
8:pub struct OutputManager;
10:impl OutputManager {
11:pub fn new() -> Self {
15:pub fn info(&self, message: &str) {
19:pub fn success(&self, message: &str) {
23:pub fn error(&self, message: &str) {
```

**Full Content:**
```rust
//! Simple output formatting for the CLI

#![allow(clippy::disallowed_macros)]

use colored::Colorize;

/// Simple output manager for consistent formatting
pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, message: &str) {
        println!("{} {}", "INFO:".blue().bold(), message);
    }

    pub fn success(&self, message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }
}

```

### src/cli/main.rs (191 lines)

**Key Definitions:**
```rust
79:pub enum TaskCommands {
```

**Full Content:**
```rust
/*
 * 5D Labs Agent Platform - CLI Tools for AI Coding Agents
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

//! Orchestrator CLI - Simplified with just docs and code task submission

mod analyzer;
mod api;
mod commands;
mod docs_generator;
mod output;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orchestrator")]
#[command(about = "CLI for Orchestrator Service", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API endpoint URL
    #[arg(
        long,
        env = "ORCHESTRATOR_API_URL",
        default_value = "http://orchestrator.orchestrator.svc.cluster.local/api/v1"
    )]
    api_url: String,

    /// Output format (table, json, yaml)
    #[arg(long, short, default_value = "table")]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Task operations
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Analyze codebase and generate documentation for Task Master PRD
    Analyze {
        /// Output directory (default: docs/codebase-analysis)
        #[arg(short, long, default_value = "docs/codebase-analysis")]
        output: String,

        /// Output format: modular, single, json (default: modular)
        #[arg(short, long, default_value = "modular")]
        format: String,

        /// Working directory to analyze (default: current directory)
        #[arg(short, long)]
        working_directory: Option<String>,

        /// Include full source code (default: true)
        #[arg(long, default_value = "true")]
        include_source: bool,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum TaskCommands {
    /// Generate documentation for Task Master tasks
    Docs {
        /// Working directory containing .taskmaster folder
        #[arg(long, short = 'w')]
        working_directory: Option<String>,

        /// Claude model to use (required) - e.g., 'claude-opus-4-20250514' or 'claude-sonnet-4-20250514'
        #[arg(long)]
        model: Option<String>,

        /// Documentation repository URL
        #[arg(long)]
        repository_url: Option<String>,

        /// Source branch to use
        #[arg(long)]
        source_branch: Option<String>,

        /// GitHub username for authentication (required)
        #[arg(long)]
        github_user: String,
    },

    /// Submit implementation task to orchestrator
    Code {
        /// Task ID to implement
        task_id: u32,

        /// Target service name
        #[arg(long, short = 's')]
        service: String,

        /// Target project repository URL (where implementation work happens)
        #[arg(long)]
        repository_url: Option<String>,

        /// Documentation repository URL (where Task Master definitions come from)
        #[arg(long)]
        docs_repository_url: Option<String>,

        /// Project directory within docs repository (e.g. "_projects/simple-api")
        #[arg(long)]
        docs_project_directory: Option<String>,

        /// GitHub username for authentication (required)
        #[arg(long)]
        github_user: String,

        /// Working directory within target repository
        #[arg(long, short = 'w')]
        working_directory: Option<String>,

        /// Claude model to use (required) - e.g., 'claude-opus-4-20250514' or 'claude-sonnet-4-20250514'
        #[arg(long)]
        model: Option<String>,

        /// Local MCP tools to enable (comma-separated)
        #[arg(long)]
        local_tools: Option<String>,

        /// Remote MCP tools to enable (comma-separated)
        #[arg(long)]
        remote_tools: Option<String>,

        /// Context version for retry attempts (incremented on each retry)
        #[arg(long, default_value = "1")]
        context_version: u32,

        /// Additional context for retry attempts
        #[arg(long)]
        prompt_modification: Option<String>,

        /// Docs branch to use (e.g., "main", "feature/branch")
        #[arg(long, default_value = "main")]
        docs_branch: String,

        /// Whether to continue a previous session
        #[arg(long)]
        continue_session: bool,

        /// Whether to overwrite memory before starting
        #[arg(long)]
        overwrite_memory: bool,

        /// Environment variables (format: KEY=value,KEY2=value2)
        #[arg(long)]
        env: Option<String>,

        /// Environment variables from secrets (format: name:secretName:secretKey,...)
        #[arg(long)]
        env_from_secrets: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Task { command } => {
            commands::handle_task_command(command, &cli.api_url, &cli.output).await?;
        }
        Commands::Analyze { output, format, working_directory, include_source } => {
            commands::handle_analyze_command(output, format, working_directory, include_source)?;
        }
    }

    Ok(())
}

```

### src/cli/api.rs (100 lines)

**Key Definitions:**
```rust
12:pub struct ApiResponse {
21:pub struct ApiClient {
26:impl ApiClient {
28:pub fn new(base_url: String) -> Self {
```

**Full Content:**
```rust
//! HTTP API client for orchestrator `TaskRun` submissions

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

```

