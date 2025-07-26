# orchestrator Analysis

**Path:** `orchestrator`
**Type:** RustLibrary
**Lines of Code:** 7687
**Description:** No description available

## Source Files

### tools/src/mcp/tools.rs (143 lines)

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

### tools/src/mcp/main.rs (548 lines)

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

### tools/src/cli/analyzer.rs (810 lines)

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

### tools/src/cli/docs_generator.rs (207 lines)

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

### tools/src/cli/commands.rs (388 lines)

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

### tools/src/cli/output.rs (26 lines)

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

### tools/src/cli/main.rs (191 lines)

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

### tools/src/cli/api.rs (100 lines)

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

### core/src/crds/mod.rs (5 lines)

**Full Content:**
```rust
pub mod coderun;
pub mod docsrun;

pub use coderun::*;
pub use docsrun::*;

```

### core/src/crds/coderun.rs (181 lines)

**Key Definitions:**
```rust
10:pub struct SecretEnvVar {
51:pub struct CodeRunSpec {
121:pub struct CodeRunStatus {
162:pub struct CodeRunCondition {
```

**Full Content:**
```rust
//! `CodeRun` Custom Resource Definition for code implementation tasks

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reference to a secret for environment variable
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecretEnvVar {
    /// Name of the environment variable
    pub name: String,
    /// Name of the secret
    #[serde(rename = "secretName")]
    pub secret_name: String,
    /// Key within the secret
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

/// Default function for `context_version` field
fn default_context_version() -> u32 {
    1
}

/// Default function for `docs_branch` field
fn default_docs_branch() -> String {
    "main".to_string()
}

/// Default function for `continue_session` field
fn default_continue_session() -> bool {
    false
}

/// Default function for `overwrite_memory` field
fn default_overwrite_memory() -> bool {
    false
}

/// `CodeRun` CRD for code implementation tasks
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.platform", version = "v1", kind = "CodeRun")]
#[kube(namespaced)]
#[kube(status = "CodeRunStatus")]
#[kube(printcolumn = r#"{"name":"Task","type":"integer","jsonPath":".spec.taskId"}"#)]
#[kube(printcolumn = r#"{"name":"Service","type":"string","jsonPath":".spec.service"}"#)]
#[kube(printcolumn = r#"{"name":"Model","type":"string","jsonPath":".spec.model"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct CodeRunSpec {
    /// Task ID to implement
    #[serde(rename = "taskId")]
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    #[serde(rename = "docsRepositoryUrl")]
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    #[serde(default, rename = "docsProjectDirectory")]
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    #[serde(default, rename = "workingDirectory")]
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// GitHub username for authentication and commits
    #[serde(rename = "githubUser")]
    pub github_user: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default, rename = "localTools")]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default, rename = "remoteTools")]
    pub remote_tools: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version", rename = "contextVersion")]
    pub context_version: u32,

    /// Additional context for retry attempts
    #[serde(default, rename = "promptModification")]
    pub prompt_modification: Option<String>,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch", rename = "docsBranch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default = "default_continue_session", rename = "continueSession")]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default = "default_overwrite_memory", rename = "overwriteMemory")]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default, rename = "envFromSecrets")]
    pub env_from_secrets: Vec<SecretEnvVar>,
}

/// Status of the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeRunStatus {
    /// Current phase of the code implementation
    pub phase: String,

    /// Human-readable message about the current state
    pub message: Option<String>,

    /// Timestamp when this phase was reached
    pub last_update: Option<String>,

    /// Associated Kubernetes Job name
    pub job_name: Option<String>,

    /// Pull request URL if created
    pub pull_request_url: Option<String>,

    /// Current retry attempt (if applicable)
    pub retry_count: Option<u32>,

    /// Conditions for the `CodeRun`
    pub conditions: Option<Vec<CodeRunCondition>>,

    /// Name of the `ConfigMap` containing the prompt and context
    pub configmap_name: Option<String>,

    /// Version of the context and prompt used
    pub context_version: Option<u32>,

    /// Modification to the prompt if any
    pub prompt_modification: Option<String>,

    /// Mode of prompt (e.g., "direct", "indirect")
    pub prompt_mode: Option<String>,

    /// Session ID for tracking
    pub session_id: Option<String>,
}

/// Condition for the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

```

### core/src/crds/docsrun.rs (73 lines)

**Key Definitions:**
```rust
13:pub struct DocsRunSpec {
26:pub struct DocsRunStatus {
39:pub struct DocsRunCondition {
62:pub enum DocsRunPhase {
```

**Full Content:**
```rust
//! `DocsRun` Custom Resource Definition for documentation generation

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.platform", version = "v1", kind = "DocsRun")]
#[kube(namespaced)]
#[kube(status = "DocsRunStatus")]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct DocsRunSpec {
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,
    #[serde(rename = "sourceBranch")]
    pub source_branch: String,
    pub model: String,
    #[serde(rename = "githubUser")]
    pub github_user: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DocsRunStatus {
    pub phase: String,
    pub message: Option<String>,
    pub last_update: Option<String>,
    pub job_name: Option<String>,
    pub pull_request_url: Option<String>,
    pub conditions: Option<Vec<DocsRunCondition>>,
    pub configmap_name: Option<String>,
}

/// Condition for the `DocsRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocsRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Phase of `DocsRun` execution
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum DocsRunPhase {
    /// `DocsRun` has been created but not yet processed
    Pending,
    /// Documentation generation is in progress
    Running,
    /// Documentation generation completed successfully
    Succeeded,
    /// Documentation generation failed
    Failed,
    /// `DocsRun` was manually cancelled
    Cancelled,
}

```

### core/src/bin/test_templates.rs (148 lines)

**Full Content:**
```rust
#!/usr/bin/env cargo
//! Template testing utility for local handlebars template validation
//!
//! Usage: cargo run --bin `test_templates`

#![allow(clippy::disallowed_macros)]

use handlebars::Handlebars;
use serde_json::json;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Handlebars Templates...\n");

    // Initialize handlebars engine
    let mut handlebars = Handlebars::new();

    // Template directory
    let template_dir = Path::new("orchestrator-core/templates");

    // Test docs templates
    test_docs_templates(&mut handlebars, template_dir)?;

    // Test code templates
    test_code_templates(&mut handlebars, template_dir)?;

    println!("✅ All templates rendered successfully!");
    Ok(())
}

fn test_docs_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Testing Docs Templates:");

    // Mock DocsRunSpec data
    let docs_data = json!({
        "repository_url": "https://github.com/5dlabs/platform",
        "working_directory": "_projects/simple-api",
        "source_branch": "feature/example-project-and-cli",
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs"
    });

    // Test docs templates
    let docs_templates = [
        "docs/claude.md.hbs",
        "docs/settings.json.hbs",
        "docs/container.sh.hbs",
    ];

    for template_name in &docs_templates {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");

            // Register template
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;

            // Render template
            let result = handlebars.render(template_name, &docs_data)?;

            println!("    ✅ Rendered successfully ({} chars)", result.len());

            // Show first few lines of output for verification
            let lines: Vec<&str> = result.lines().take(3).collect();
            for line in lines {
                println!("    │ {line}");
            }

            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    Ok(())
}

fn test_code_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("💻 Testing Code Templates:");

    // Mock CodeRunSpec data
    let code_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/platform",
        "platform_repository_url": "https://github.com/5dlabs/platform",
        "branch": "feature/example-project-and-cli",
        "working_directory": "_projects/simple-api",
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs",
        "local_tools": "bash,edit,read",
        "remote_tools": "github_create_issue",
        "tool_config": "default",
        "context_version": 1,
        "prompt_modification": null,
        "prompt_mode": "append"
    });

    // Test code templates
    let code_templates = [
        "code/claude.md.hbs",
        "code/settings.json.hbs",
        "code/container.sh.hbs",
    ];

    for template_name in &code_templates {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");

            // Register template
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;

            // Render template
            let result = handlebars.render(template_name, &code_data)?;

            println!("    ✅ Rendered successfully ({} chars)", result.len());

            // Show first few lines of output for verification
            let lines: Vec<&str> = result.lines().take(3).collect();
            for line in lines {
                println!("    │ {line}");
            }

            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    Ok(())
}

```

### core/src/lib.rs (31 lines)

**Full Content:**
```rust
/*
 * 5D Labs Agent Platform - Kubernetes Orchestrator for AI Coding Agents
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

//! Orchestrator core library
//!
//! This crate provides the core functionality for the unified orchestration service,
//! including Kubernetes client wrapper, job orchestration, and request handling.

pub mod controllers;
pub mod crds;
pub mod handlers;

// Re-export commonly used types
pub use controllers::task_controller::ControllerConfig;
pub use crds::{CodeRun, CodeRunSpec, CodeRunStatus, DocsRun, DocsRunSpec, DocsRunStatus};
pub use handlers::*;

```

### core/src/main.rs (159 lines)

**Full Content:**
```rust
/*
 * 5D Labs Agent Platform - Kubernetes Orchestrator for AI Coding Agents
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

//! Main entry point for the Orchestrator service

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use core::{
    controllers::run_task_controller,
    handlers::{code_handler::submit_code_task, common::AppState, docs_handler::generate_docs},
};
use kube::Client;
use serde_json::{json, Value};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn create_app_state() -> Result<AppState> {
    // Initialize Kubernetes client
    let k8s_client = Client::try_default()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create K8s client: {}", e))?;

    // Get namespace from environment or use default
    let namespace =
        std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "orchestrator".to_string());

    info!("Initialized orchestrator for namespace: {}", namespace);

    Ok(AppState {
        k8s_client,
        namespace,
    })
}

/// Health check endpoint
async fn health_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Create API routes
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/pm/tasks", post(submit_code_task))
        .route("/pm/docs/generate", post(generate_docs))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with OpenTelemetry support
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting Orchestrator service v{} with TaskRun CRD support",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize application state
    let app_state = create_app_state().await?;

    // Start task controller
    let client = app_state.k8s_client.clone();
    let namespace = app_state.namespace.clone();

    info!("Starting task controller in namespace: {}", namespace);

    // Spawn the controller in the background
    tokio::spawn(async move {
        if let Err(e) = run_task_controller(client, namespace).await {
            error!("Task controller error: {}", e);
        }
    });

    // Build the application with middleware layers
    let app = Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(health_check)) // Root health check for load balancers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()), // Simplified for now
        )
        .with_state(app_state);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}", listener.local_addr()?);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}

```

### core/src/controllers/mod.rs (8 lines)

**Full Content:**
```rust
// TODO: Remove this old controller once new one is complete
// pub mod taskrun_old;
// pub use taskrun_old::run_taskrun_controller;

pub mod task_controller;

// Re-export the main controller function for easy access
pub use task_controller::run_task_controller;

```

### core/src/controllers/task_controller/types.rs (212 lines)

**Key Definitions:**
```rust
9:pub enum Error {
37:pub enum TaskType {
42:impl TaskType {
43:pub fn name(&self) -> String {
50:pub fn is_docs(&self) -> bool {
54:pub fn service_name(&self) -> &str {
61:pub fn model(&self) -> &str {
68:pub fn github_user(&self) -> &str {
75:pub fn repository_url(&self) -> &str {
82:pub fn source_branch(&self) -> Option<&str> {
90:pub fn working_directory(&self) -> &str {
103:pub fn task_id(&self) -> Option<u32> {
111:pub fn context_version(&self) -> u32 {
118:pub fn retry_count(&self) -> u32 {
125:pub fn session_id(&self) -> Option<&str> {
132:pub fn prompt_modification(&self) -> Option<&str> {
140:pub fn local_tools(&self) -> Option<&str> {
147:pub fn remote_tools(&self) -> Option<&str> {
155:pub fn docs_repository_url(&self) -> Option<&str> {
163:pub fn uses_ssh() -> bool {
168:pub fn ssh_secret_name(&self) -> String {
173:pub fn github_token_secret_name(&self) -> String {
178:pub fn docs_branch(&self) -> &str {
187:pub fn continue_session(&self) -> bool {
198:pub fn overwrite_memory(&self) -> bool {
206:pub fn docs_project_directory(&self) -> Option<&str> {
```

**Full Content:**
```rust
use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use kube::{Client, ResourceExt};
use std::sync::Arc;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Missing object key")]
    MissingObjectKey,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Context shared across controller operations
pub(crate) struct Context {
    pub client: Client,
    pub namespace: String,
    pub config: Arc<ControllerConfig>,
}

// Finalizer names for cleanup
pub(crate) const DOCS_FINALIZER_NAME: &str = "docsruns.orchestrator.io/finalizer";
pub(crate) const CODE_FINALIZER_NAME: &str = "coderuns.orchestrator.io/finalizer";

// Enum to represent either task type for shared functionality
pub enum TaskType {
    Docs(Arc<DocsRun>),
    Code(Arc<CodeRun>),
}

impl TaskType {
    pub fn name(&self) -> String {
        match self {
            TaskType::Docs(dr) => dr.name_any(),
            TaskType::Code(cr) => cr.name_any(),
        }
    }

    pub fn is_docs(&self) -> bool {
        matches!(self, TaskType::Docs(_))
    }

    pub fn service_name(&self) -> &str {
        match self {
            TaskType::Docs(_) => "docs-generator", // Fixed service name for docs
            TaskType::Code(cr) => &cr.spec.service,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.model,
            TaskType::Code(cr) => &cr.spec.model,
        }
    }

    pub fn github_user(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.github_user,
            TaskType::Code(cr) => &cr.spec.github_user,
        }
    }

    pub fn repository_url(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.repository_url,
            TaskType::Code(cr) => &cr.spec.repository_url,
        }
    }

    pub fn source_branch(&self) -> Option<&str> {
        match self {
            TaskType::Docs(dr) => Some(&dr.spec.source_branch),
            TaskType::Code(_) => None, // CodeRun uses platform_branch instead
        }
    }

    /// Get working directory (defaults to service name if not specified)
    pub fn working_directory(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.working_directory,
            TaskType::Code(cr) => {
                // Default to service name if working_directory is None or empty
                match &cr.spec.working_directory {
                    Some(wd) if !wd.is_empty() => wd,
                    _ => &cr.spec.service,
                }
            }
        }
    }

    pub fn task_id(&self) -> Option<u32> {
        match self {
            TaskType::Docs(_) => None, // Docs generation doesn't have a specific task ID
            TaskType::Code(cr) => Some(cr.spec.task_id),
        }
    }

    /// Get retry/versioning information for `CodeRun` (docs don't have retries)
    pub fn context_version(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 1, // Docs don't have context versions
            TaskType::Code(cr) => cr.spec.context_version,
        }
    }

    pub fn retry_count(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 0, // Docs don't retry
            TaskType::Code(cr) => cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.session_id.as_deref()),
        }
    }

    pub fn prompt_modification(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.prompt_modification.as_deref(),
        }
    }

    /// Get tool configuration for the task
    pub fn local_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.local_tools.as_deref(),
        }
    }

    pub fn remote_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.remote_tools.as_deref(),
        }
    }

    /// Get docs repository info (only for `CodeRun`)
    pub fn docs_repository_url(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => Some(&cr.spec.docs_repository_url),
        }
    }

    /// Always use SSH authentication (we're SSH-only now)
    pub fn uses_ssh() -> bool {
        true
    }

    /// Get SSH secret name for this GitHub user
    pub fn ssh_secret_name(&self) -> String {
        format!("github-ssh-{}", self.github_user())
    }

    /// Get GitHub token secret name for this GitHub user
    pub fn github_token_secret_name(&self) -> String {
        format!("github-token-{}", self.github_user())
    }

    /// Get docs branch (only for `CodeRun`)
    pub fn docs_branch(&self) -> &str {
        match self {
            TaskType::Docs(_) => "main", // Docs use default branch
            TaskType::Code(cr) => &cr.spec.docs_branch,
        }
    }

    /// Get continue session flag - true for retries or user-requested continuation
    #[allow(dead_code)]
    pub fn continue_session(&self) -> bool {
        match self {
            TaskType::Docs(_) => false, // Docs don't continue sessions
            TaskType::Code(cr) => {
                // Continue if it's a retry attempt OR user explicitly requested it
                self.retry_count() > 0 || cr.spec.continue_session
            }
        }
    }

    /// Get overwrite memory flag (only for `CodeRun`)
    pub fn overwrite_memory(&self) -> bool {
        match self {
            TaskType::Docs(_) => true, // Docs always overwrite memory
            TaskType::Code(cr) => cr.spec.overwrite_memory,
        }
    }

    /// Get docs project directory (only for `CodeRun`)
    pub fn docs_project_directory(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.docs_project_directory.as_deref(),
        }
    }
}

```

### core/src/controllers/task_controller/config.rs (348 lines)

**Key Definitions:**
```rust
12:pub struct ControllerConfig {
38:pub struct JobConfig {
46:pub struct AgentConfig {
57:pub struct ImageConfig {
67:pub struct SecretsConfig {
79:pub struct PermissionsConfig {
93:pub struct TelemetryConfig {
116:pub struct StorageConfig {
132:pub struct CleanupConfig {
169:impl Default for CleanupConfig {
180:impl ControllerConfig {
182:pub fn validate(&self) -> Result<(), anyhow::Error> {
195:pub fn from_mounted_file(config_path: &str) -> Result<Self, anyhow::Error> {
226:impl Default for ControllerConfig {
```

**Full Content:**
```rust
//! Task Controller Configuration
//!
//! Simplified configuration structure for the new DocsRun/CodeRun controller.
//! Contains only the essential configuration needed for our current implementation.

use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::Api, Client};
use serde::{Deserialize, Serialize};

/// Main controller configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControllerConfig {
    /// Job configuration
    pub job: JobConfig,

    /// Agent configuration
    pub agent: AgentConfig,

    /// Secrets configuration
    pub secrets: SecretsConfig,

    /// Tool permissions configuration
    pub permissions: PermissionsConfig,

    /// Telemetry configuration
    pub telemetry: TelemetryConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Cleanup configuration
    #[serde(default)]
    pub cleanup: CleanupConfig,
}

/// Job configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobConfig {
    /// Job timeout in seconds
    #[serde(rename = "activeDeadlineSeconds")]
    pub active_deadline_seconds: i64,
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    /// Container image configuration
    pub image: ImageConfig,

    /// Image pull secrets for private registries
    #[serde(default, rename = "imagePullSecrets")]
    pub image_pull_secrets: Vec<String>,
}

/// Image configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageConfig {
    /// Image repository (e.g., "ghcr.io/5dlabs/claude")
    pub repository: String,

    /// Image tag (e.g., "latest", "v2.1.0")
    pub tag: String,
}

/// Secrets configuration - only what we actually use
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretsConfig {
    /// Anthropic API key secret name (for rotation)
    #[serde(rename = "apiKeySecretName")]
    pub api_key_secret_name: String,

    /// Anthropic API key secret key
    #[serde(rename = "apiKeySecretKey")]
    pub api_key_secret_key: String,
}

/// Tool permissions configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionsConfig {
    /// Whether to override default tool permissions
    #[serde(rename = "agentToolsOverride")]
    pub agent_tools_override: bool,

    /// Allowed tool patterns
    pub allow: Vec<String>,

    /// Denied tool patterns
    pub deny: Vec<String>,
}

/// Telemetry configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,

    /// OTLP endpoint URL
    #[serde(rename = "otlpEndpoint")]
    pub otlp_endpoint: String,

    /// OTLP protocol (grpc/http)
    #[serde(rename = "otlpProtocol")]
    pub otlp_protocol: String,

    /// Logs endpoint (for code tasks)
    #[serde(rename = "logsEndpoint")]
    pub logs_endpoint: String,

    /// Logs protocol (for code tasks)
    #[serde(rename = "logsProtocol")]
    pub logs_protocol: String,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Storage class name for PVCs (e.g., "local-path" for local development)
    #[serde(rename = "storageClassName")]
    pub storage_class_name: Option<String>,

    /// Storage size for workspace PVCs
    #[serde(rename = "workspaceSize", default = "default_workspace_size")]
    pub workspace_size: String,
}

fn default_workspace_size() -> String {
    "10Gi".to_string()
}

/// Cleanup configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    /// Whether automatic cleanup is enabled
    #[serde(default = "default_cleanup_enabled")]
    pub enabled: bool,

    /// Minutes to wait before cleaning up completed (successful) jobs
    #[serde(
        rename = "completedJobDelayMinutes",
        default = "default_completed_delay"
    )]
    pub completed_job_delay_minutes: u64,

    /// Minutes to wait before cleaning up failed jobs
    #[serde(rename = "failedJobDelayMinutes", default = "default_failed_delay")]
    pub failed_job_delay_minutes: u64,

    /// Whether to delete the ConfigMap when cleaning up the job
    #[serde(rename = "deleteConfigMap", default = "default_delete_configmap")]
    pub delete_configmap: bool,
}

fn default_cleanup_enabled() -> bool {
    true
}

fn default_completed_delay() -> u64 {
    5 // 5 minutes
}

fn default_failed_delay() -> u64 {
    60 // 60 minutes (1 hour)
}

fn default_delete_configmap() -> bool {
    true
}

impl Default for CleanupConfig {
    fn default() -> Self {
        CleanupConfig {
            enabled: default_cleanup_enabled(),
            completed_job_delay_minutes: default_completed_delay(),
            failed_job_delay_minutes: default_failed_delay(),
            delete_configmap: default_delete_configmap(),
        }
    }
}

impl ControllerConfig {
    /// Validate that configuration has required fields
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.agent.image.repository == "MISSING_IMAGE_CONFIG"
            || self.agent.image.tag == "MISSING_IMAGE_CONFIG"
        {
            return Err(anyhow::anyhow!(
                "Agent image configuration is missing! This indicates the controller ConfigMap was not loaded properly. \
                Please ensure the 'agent.image.repository' and 'agent.image.tag' are set in the Helm values."
            ));
        }
        Ok(())
    }

    /// Load configuration from mounted ConfigMap file
    pub fn from_mounted_file(config_path: &str) -> Result<Self, anyhow::Error> {
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;

        let config: ControllerConfig = serde_yaml::from_str(&config_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse config YAML: {}", e))?;

        Ok(config)
    }

    /// Load configuration from a `ConfigMap` (legacy API-based method)
    pub async fn from_configmap(
        client: &Client,
        namespace: &str,
        name: &str,
    ) -> Result<Self, anyhow::Error> {
        let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
        let cm = api.get(name).await?;

        let data = cm
            .data
            .ok_or_else(|| anyhow::anyhow!("ConfigMap has no data"))?;
        let config_str = data
            .get("config.yaml")
            .ok_or_else(|| anyhow::anyhow!("ConfigMap missing config.yaml"))?;

        let config: ControllerConfig = serde_yaml::from_str(config_str)?;
        Ok(config)
    }
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            job: JobConfig {
                active_deadline_seconds: 7200, // 2 hours
            },
            agent: AgentConfig {
                image: ImageConfig {
                    repository: "MISSING_IMAGE_CONFIG".to_string(),
                    tag: "MISSING_IMAGE_CONFIG".to_string(),
                },
                image_pull_secrets: vec!["ghcr-secret".to_string()],
            },
            secrets: SecretsConfig {
                api_key_secret_name: "anthropic-api-key".to_string(),
                api_key_secret_key: "api-key".to_string(),
            },
            permissions: PermissionsConfig {
                agent_tools_override: false,
                allow: vec![
                    "Bash(*)".to_string(),
                    "Edit(*)".to_string(),
                    "Read(*)".to_string(),
                    "Write(*)".to_string(),
                    "MultiEdit(*)".to_string(),
                    "Glob(*)".to_string(),
                    "Grep(*)".to_string(),
                    "LS(*)".to_string(),
                ],
                deny: vec![
                    "Bash(npm:install*, yarn:install*, cargo:install*, docker:*, kubectl:*, rm:-rf*, git:*)".to_string(),
                ],
            },
            // Telemetry configuration with environment variable overrides:
            // - OTLP_ENDPOINT: OTLP traces endpoint (default: http://localhost:4317)
            // - LOGS_ENDPOINT: Logs endpoint (default: http://localhost:4318)
            // - LOGS_PROTOCOL: Logs protocol (default: http)
            telemetry: TelemetryConfig {
                enabled: false,
                otlp_endpoint: std::env::var("OTLP_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4317".to_string()),
                otlp_protocol: "grpc".to_string(),
                logs_endpoint: std::env::var("LOGS_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4318".to_string()),
                logs_protocol: std::env::var("LOGS_PROTOCOL")
                    .unwrap_or_else(|_| "http".to_string()),
            },
            storage: StorageConfig {
                storage_class_name: None, // Let K8s use default storage class
                workspace_size: "10Gi".to_string(),
            },
            cleanup: CleanupConfig {
                enabled: true,
                completed_job_delay_minutes: 5,
                failed_job_delay_minutes: 60,
                delete_configmap: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
job:
  activeDeadlineSeconds: 3600

agent:
  image:
    repository: "test/image"
    tag: "latest"

secrets:
  apiKeySecretName: "test-secret"
  apiKeySecretKey: "key"

permissions:
  agentToolsOverride: true
  allow: ["*"]
  deny: []

telemetry:
  enabled: true
  otlpEndpoint: "localhost:4317"
  otlpProtocol: "grpc"
  logsEndpoint: "localhost:4318"
  logsProtocol: "http"

storage:
  storageClassName: "local-path"
  workspaceSize: "5Gi"

cleanup:
  enabled: true
  completedJobDelayMinutes: 5
  failedJobDelayMinutes: 60
  deleteConfigMap: true
"#;

        let config: ControllerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.job.active_deadline_seconds, 3600);
        assert_eq!(config.agent.image.repository, "test/image");
        assert!(config.telemetry.enabled);
        assert_eq!(config.permissions.allow, vec!["*"]);
        assert!(config.cleanup.enabled);
        assert_eq!(config.cleanup.completed_job_delay_minutes, 5);
        assert_eq!(config.cleanup.failed_job_delay_minutes, 60);
    }

    #[test]
    fn test_default_config() {
        let config = ControllerConfig::default();
        assert_eq!(config.job.active_deadline_seconds, 7200);
        assert_eq!(config.agent.image.repository, "MISSING_IMAGE_CONFIG");
        assert_eq!(config.secrets.api_key_secret_name, "anthropic-api-key");
        assert!(!config.telemetry.enabled);
        assert!(!config.permissions.agent_tools_override);
    }
}

```

### core/src/controllers/task_controller/auth.rs (31 lines)

**Key Definitions:**
```rust
5:pub fn generate_ssh_volumes(task: &TaskType) -> Vec<serde_json::Value> {
```

**Full Content:**
```rust
use super::types::TaskType;
use serde_json::json;

/// Generate SSH key volume configuration if needed
pub fn generate_ssh_volumes(task: &TaskType) -> Vec<serde_json::Value> {
    if !TaskType::uses_ssh() {
        return vec![];
    }

    let ssh_secret_name = task.ssh_secret_name();

    vec![json!({
        "name": "ssh-key",
        "secret": {
            "secretName": ssh_secret_name,
            "defaultMode": 0o600,
            "items": [
                {
                    "key": "ssh-privatekey",
                    "path": "id_ed25519",
                    "mode": 0o600
                },
                {
                    "key": "ssh-publickey",
                    "path": "id_ed25519.pub",
                    "mode": 0o644
                }
            ]
        }
    })]
}

```

### core/src/controllers/task_controller/mod.rs (19 lines)

**Full Content:**
```rust
//! Task Controller
//!
//! Unified controller for both `DocsRun` and `CodeRun` resources.
//! Handles job orchestration, resource management, and status tracking.

// Public API - re-export the main controller function
pub use reconcile::run_task_controller;

// Public types - re-export config for external use
pub use config::ControllerConfig;

// Internal modules
pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod reconcile;
pub(crate) mod resources;
pub(crate) mod status;
pub(crate) mod templates;
pub(crate) mod types;

```

### core/src/controllers/task_controller/reconcile.rs (266 lines)

**Full Content:**
```rust
use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use futures::StreamExt;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::{
    api::Api,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
        watcher::Config,
    },
    Client,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::error;

use super::resources::{cleanup_resources, reconcile_create_or_update};
use super::status::monitor_job_status;
use super::types::{Context, Error, Result, TaskType, CODE_FINALIZER_NAME, DOCS_FINALIZER_NAME};

/// Run the task controller for both `DocsRun` and `CodeRun` resources
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    error!(
        "🚀 AGGRESSIVE DEBUG: Starting task controller in namespace: {}",
        namespace
    );

    error!("🔧 AGGRESSIVE DEBUG: About to load controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            error!("✅ AGGRESSIVE DEBUG: Successfully loaded controller configuration from mounted file");
            error!(
                "🔧 AGGRESSIVE DEBUG: Configuration cleanup enabled = {}",
                cfg.cleanup.enabled
            );

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!(
                    "❌ AGGRESSIVE DEBUG: Configuration validation failed: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ AGGRESSIVE DEBUG: Configuration validation passed");
            cfg
        }
        Err(e) => {
            error!(
                "❌ AGGRESSIVE DEBUG: Failed to load configuration from mounted file, using defaults: {}",
                e
            );
            error!("🔧 AGGRESSIVE DEBUG: About to create default configuration...");
            let default_config = ControllerConfig::default();

            // Validate default configuration - this should fail if image config is missing
            if let Err(validation_error) = default_config.validate() {
                error!(
                    "❌ AGGRESSIVE DEBUG: Default configuration is invalid: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ AGGRESSIVE DEBUG: Default configuration validation passed");
            default_config
        }
    };

    error!("🏗️ AGGRESSIVE DEBUG: Creating controller context...");
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    error!("✅ AGGRESSIVE DEBUG: Controller context created successfully");

    // Start controllers for both DocsRun and CodeRun
    error!("🔗 AGGRESSIVE DEBUG: Creating API clients for DocsRun and CodeRun...");
    let docs_runs = Api::<DocsRun>::namespaced(client.clone(), &namespace);
    let code_runs = Api::<CodeRun>::namespaced(client.clone(), &namespace);

    error!("✅ AGGRESSIVE DEBUG: API clients created, starting controllers...");

    let docs_controller = Controller::new(docs_runs, Config::default())
        .shutdown_on_signal()
        .run(reconcile_docs, error_policy_docs, context.clone())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()));

    let code_controller = Controller::new(code_runs, Config::default())
        .shutdown_on_signal()
        .run(reconcile_code, error_policy_code, context.clone())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()));

    error!("🚀 AGGRESSIVE DEBUG: Both controllers started, entering main loop...");

    // Run both controllers concurrently
    tokio::select! {
        () = docs_controller => error!("DocsRun controller finished"),
        () = code_controller => error!("CodeRun controller finished"),
    }

    Ok(())
}

/// Reconciliation logic for `DocsRun` resources
async fn reconcile_docs(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    error!(
        "📝 AGGRESSIVE DEBUG: Starting reconcile_docs for: {}",
        docs_run
            .metadata
            .name
            .as_ref()
            .unwrap_or(&"unnamed".to_string())
    );

    let task = TaskType::Docs(docs_run.clone());
    error!("🔍 AGGRESSIVE DEBUG: Created task type, calling reconcile_common...");

    let result = reconcile_common(task, ctx, DOCS_FINALIZER_NAME).await;
    error!(
        "🏁 AGGRESSIVE DEBUG: reconcile_common completed with result: {:?}",
        result.is_ok()
    );

    result
}

/// Reconcile function for `CodeRun` resources
async fn reconcile_code(cr: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    let task = TaskType::Code(cr.clone());
    reconcile_common(task, ctx, CODE_FINALIZER_NAME).await
}

/// Common reconciliation logic for both `DocsRun` and `CodeRun`
async fn reconcile_common(
    task: TaskType,
    ctx: Arc<Context>,
    finalizer_name: &str,
) -> Result<Action> {
    error!(
        "🎯 AGGRESSIVE DEBUG: Starting reconcile_common for: {}",
        task.name()
    );

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    error!(
        "🔄 AGGRESSIVE DEBUG: Reconciling {}: {}",
        if task.is_docs() { "DocsRun" } else { "CodeRun" },
        name
    );

    // Create APIs
    error!("🔗 AGGRESSIVE DEBUG: Creating Kubernetes API clients...");
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    error!("✅ AGGRESSIVE DEBUG: API clients created successfully");

    // Handle finalizers for cleanup based on task type
    let _result = match &task {
        TaskType::Docs(dr) => {
            let docsruns: Api<DocsRun> = Api::namespaced(client.clone(), namespace);
            finalizer(&docsruns, finalizer_name, dr.clone(), |event| async {
                match event {
                    FinalizerEvent::Apply(dr) => {
                        let task = TaskType::Docs(dr);
                        reconcile_create_or_update(
                            task,
                            &jobs,
                            &configmaps,
                            &pvcs,
                            &ctx.config,
                            &ctx,
                        )
                        .await
                    }
                    FinalizerEvent::Cleanup(dr) => {
                        let task = TaskType::Docs(dr);
                        cleanup_resources(task, &jobs, &configmaps).await
                    }
                }
            })
            .await
        }
        TaskType::Code(cr) => {
            let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
            finalizer(&coderuns, finalizer_name, cr.clone(), |event| async {
                match event {
                    FinalizerEvent::Apply(cr) => {
                        let task = TaskType::Code(cr);
                        reconcile_create_or_update(
                            task,
                            &jobs,
                            &configmaps,
                            &pvcs,
                            &ctx.config,
                            &ctx,
                        )
                        .await
                    }
                    FinalizerEvent::Cleanup(cr) => {
                        let task = TaskType::Code(cr);
                        cleanup_resources(task, &jobs, &configmaps).await
                    }
                }
            })
            .await
        }
    };

    // Handle finalizer errors
    let _result = _result.map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    // Monitor running jobs
    monitor_running_job(&task, &jobs, &ctx).await?;

    // Requeue after 30 seconds to check status
    Ok(Action::requeue(Duration::from_secs(30)))
}

/// Monitor running job status for both task types
async fn monitor_running_job(task: &TaskType, jobs: &Api<Job>, ctx: &Arc<Context>) -> Result<()> {
    let is_running = match task {
        TaskType::Docs(dr) => dr.status.as_ref().is_some_and(|s| s.phase == "Running"),
        TaskType::Code(cr) => cr.status.as_ref().is_some_and(|s| s.phase == "Running"),
    };

    if is_running {
        monitor_job_status(task, jobs, ctx).await?;
    }

    Ok(())
}

/// Error policy for `DocsRun` controller
fn error_policy_docs(_dr: Arc<DocsRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("DocsRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}

/// Error policy for `CodeRun` controller
fn error_policy_code(_cr: Arc<CodeRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("CodeRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}

```

### core/src/controllers/task_controller/status.rs (347 lines)

**Full Content:**
```rust
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::types::{Context, Result, TaskType};
use crate::crds::{CodeRun, CodeRunCondition, DocsRun, DocsRunCondition};

/// Monitor Job status and update CRD accordingly
pub async fn monitor_job_status(
    task: &TaskType,
    jobs: &Api<Job>,
    ctx: &Arc<Context>,
) -> Result<()> {
    let job_name = get_current_job_name(task);

    if let Some(job_name) = job_name {
        // Get the current job
        match jobs.get(&job_name).await {
            Ok(job) => {
                let (phase, message, pull_request_url) = analyze_job_status(&job);
                update_task_status(task, ctx, &phase, &message, pull_request_url).await?;

                // Schedule cleanup if job is complete and cleanup is enabled
                if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                    schedule_job_cleanup(task, ctx, &job_name, &phase).await?;
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // Job doesn't exist yet, which is fine for newly created tasks
                info!("Job {} not found yet for task {}", job_name, task.name());
            }
            Err(e) => {
                warn!(
                    "Failed to get job {} for task {}: {}",
                    job_name,
                    task.name(),
                    e
                );
            }
        }
    }

    Ok(())
}

/// Get the current job name for a task
fn get_current_job_name(task: &TaskType) -> Option<String> {
    match task {
        TaskType::Docs(dr) => dr.status.as_ref().and_then(|s| s.job_name.clone()),
        TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.job_name.clone()),
    }
}

/// Analyze job status and return (phase, message, `pull_request_url`)
fn analyze_job_status(job: &Job) -> (String, String, Option<String>) {
    if let Some(status) = &job.status {
        // Check completion time first
        if status.completion_time.is_some() {
            if let Some(conditions) = &status.conditions {
                for condition in conditions {
                    if condition.type_ == "Complete" && condition.status == "True" {
                        return (
                            "Succeeded".to_string(),
                            "Job completed successfully".to_string(),
                            None,
                        );
                    } else if condition.type_ == "Failed" && condition.status == "True" {
                        let message = condition.message.as_deref().unwrap_or("Job failed");
                        return ("Failed".to_string(), message.to_string(), None);
                    }
                }
            }
        }

        // Check if job is running
        if let Some(active) = status.active {
            if active > 0 {
                return ("Running".to_string(), "Job is running".to_string(), None);
            }
        }

        // Check for failure conditions
        if let Some(failed) = status.failed {
            if failed > 0 {
                return ("Failed".to_string(), "Job failed".to_string(), None);
            }
        }
    }

    // Default to pending if we can't determine status
    (
        "Pending".to_string(),
        "Job status unknown".to_string(),
        None,
    )
}

/// Update the task CRD status
async fn update_task_status(
    task: &TaskType,
    ctx: &Arc<Context>,
    phase: &str,
    message: &str,
    pull_request_url: Option<String>,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_dr) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "conditions": build_docs_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match docs_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated DocsRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update DocsRun status for {}: {}", name, e);
                }
            }
        }
        TaskType::Code(cr) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "retryCount": cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
                    "conditions": build_code_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match code_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated CodeRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update CodeRun status for {}: {}", name, e);
                }
            }
        }
    }

    Ok(())
}

/// Build conditions for `DocsRun` status
fn build_docs_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<DocsRunCondition> {
    vec![DocsRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Build conditions for `CodeRun` status
fn build_code_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<CodeRunCondition> {
    vec![CodeRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Update task status when job starts (called from reconcile logic)
pub async fn update_job_started(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    configmap_name: &str,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();
    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_docs_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            docs_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
        TaskType::Code(_) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_code_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            code_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
    }

    info!("Updated {} status to Running with job: {}", name, job_name);
    Ok(())
}

/// Schedule cleanup of completed job after configured delay
async fn schedule_job_cleanup(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    phase: &str,
) -> Result<()> {
    let delay_minutes = if phase == "Succeeded" {
        ctx.config.cleanup.completed_job_delay_minutes
    } else {
        ctx.config.cleanup.failed_job_delay_minutes
    };

    let job_name = job_name.to_string();
    let task_name = task.name();
    let namespace = ctx.namespace.clone();
    let client = ctx.client.clone();
    let delete_configmap = ctx.config.cleanup.delete_configmap;

    info!(
        "Scheduling cleanup for job {} in {} minutes (phase: {})",
        job_name, delay_minutes, phase
    );

    // Spawn background task to handle cleanup after delay
    tokio::spawn(async move {
        // Wait for the configured delay
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_minutes * 60)).await;

        info!("Starting scheduled cleanup for job: {}", job_name);

        // Delete the job
        let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
        match jobs_api
            .delete(&job_name, &kube::api::DeleteParams::background())
            .await
        {
            Ok(_) => info!("Successfully deleted job: {}", job_name),
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Job {} already deleted", job_name);
            }
            Err(e) => {
                error!("Failed to delete job {}: {}", job_name, e);
            }
        }

        // Delete associated ConfigMap if enabled
        if delete_configmap {
            let configmaps_api: Api<k8s_openapi::api::core::v1::ConfigMap> =
                Api::namespaced(client.clone(), &namespace);

            // Find ConfigMap associated with this job
            let labels_selector = "app=orchestrator".to_string();
            let list_params = kube::api::ListParams::default().labels(&labels_selector);

            match configmaps_api.list(&list_params).await {
                Ok(cms) => {
                    for cm in cms.items {
                        if let Some(cm_name) = &cm.metadata.name {
                            // Check if ConfigMap is associated with this job
                            if cm_name.starts_with(&task_name.replace('_', "-")) {
                                match configmaps_api
                                    .delete(cm_name, &kube::api::DeleteParams::default())
                                    .await
                                {
                                    Ok(_) => info!("Successfully deleted ConfigMap: {}", cm_name),
                                    Err(kube::Error::Api(ae)) if ae.code == 404 => {
                                        info!("ConfigMap {} already deleted", cm_name);
                                    }
                                    Err(e) => {
                                        error!("Failed to delete ConfigMap {}: {}", cm_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to list ConfigMaps for cleanup: {}", e);
                }
            }
        }

        info!("Completed cleanup for job: {}", job_name);
    });

    Ok(())
}

```

### core/src/controllers/task_controller/resources.rs (612 lines)

**Full Content:**
```rust
use super::config::ControllerConfig;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use super::auth::generate_ssh_volumes;
use super::status::update_job_started;
use super::templates::generate_templates;
use super::types::{Result, TaskType};

/// Reconciliation logic for create/update operations
pub async fn reconcile_create_or_update(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
    pvcs: &Api<PersistentVolumeClaim>,
    config: &Arc<ControllerConfig>,
    ctx: &Arc<super::types::Context>,
) -> Result<Action> {
    let name = task.name();
    info!("Creating/updating resources for task: {}", name);

    // Ensure PVC exists for code tasks (docs use emptyDir)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");
        ensure_pvc_exists(pvcs, &pvc_name, service_name, config).await?;
    }

    // Clean up older versions for retries
    cleanup_old_jobs(&task, jobs).await?;
    cleanup_old_configmaps(&task, configmaps).await?;

    // Create ConfigMap FIRST (without owner reference) so Job can mount it
    let cm_name = generate_configmap_name(&task);
    let configmap = create_configmap(&task, &cm_name, config, None)?;

    match configmaps.create(&PostParams::default(), &configmap).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Create Job SECOND (now it can successfully mount the existing ConfigMap)
    let job_ref = create_job(&task, jobs, &cm_name, config, ctx).await?;

    // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
    if let Some(owner_ref) = job_ref {
        update_configmap_owner(&task, configmaps, &cm_name, owner_ref).await?;
    }

    Ok(Action::await_change())
}

/// Generate a unique `ConfigMap` name for the task
fn generate_configmap_name(task: &TaskType) -> String {
    let task_id = task.task_id().unwrap_or(0); // Fallback for docs
    let service_name = task.service_name().replace('_', "-");
    let context_version = task.context_version();

    if task.is_docs() {
        format!("{service_name}-docs-v{context_version}-files")
    } else {
        format!("{service_name}-task{task_id}-v{context_version}-files")
    }
}

/// Create `ConfigMap` with all template files
fn create_configmap(
    task: &TaskType,
    name: &str,
    config: &ControllerConfig,
    owner_ref: Option<OwnerReference>,
) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Generate all templates for this task
    let templates = generate_templates(task, config)?;
    for (filename, content) in templates {
        data.insert(filename, content);
    }

    let labels = create_task_labels(task);
    let mut metadata = ObjectMeta {
        name: Some(name.to_string()),
        labels: Some(labels),
        ..Default::default()
    };

    // Set owner reference if provided (for automatic cleanup)
    if let Some(owner) = owner_ref {
        metadata.owner_references = Some(vec![owner]);
    }

    Ok(ConfigMap {
        metadata,
        data: Some(data),
        ..Default::default()
    })
}

/// Create the main job for the task
async fn create_job(
    task: &TaskType,
    jobs: &Api<Job>,
    cm_name: &str,
    config: &ControllerConfig,
    ctx: &Arc<super::types::Context>,
) -> Result<Option<OwnerReference>> {
    let job_name = generate_job_name(task);
    let job = build_job_spec(task, &job_name, cm_name, config)?;

    match jobs.create(&PostParams::default(), &job).await {
        Ok(created_job) => {
            info!("Created job: {}", job_name);
            update_job_started(task, ctx, &job_name, cm_name).await?;

            // Return owner reference for the created job
            if let (Some(uid), Some(name)) = (created_job.metadata.uid, created_job.metadata.name) {
                Ok(Some(OwnerReference {
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    name,
                    uid,
                    controller: Some(true),
                    block_owner_deletion: Some(true),
                }))
            } else {
                Ok(None)
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
            // Try to get existing job for owner reference
            match jobs.get(&job_name).await {
                Ok(existing_job) => {
                    if let (Some(uid), Some(name)) =
                        (existing_job.metadata.uid, existing_job.metadata.name)
                    {
                        Ok(Some(OwnerReference {
                            api_version: "batch/v1".to_string(),
                            kind: "Job".to_string(),
                            name,
                            uid,
                            controller: Some(true),
                            block_owner_deletion: Some(true),
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// Generate a deterministic job name for the task (based on resource name, not timestamp)
fn generate_job_name(task: &TaskType) -> String {
    let resource_name = task.name().replace(['_', '.'], "-");
    match task {
        TaskType::Docs(_) => {
            format!("docs-gen-{resource_name}")
        }
        TaskType::Code(_) => {
            let task_id = task.task_id().unwrap_or(0);
            let context_version = task.context_version();
            format!("code-impl-{resource_name}-task{task_id}-v{context_version}")
        }
    }
}

/// Build the complete Job specification
fn build_job_spec(
    task: &TaskType,
    job_name: &str,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    let labels = create_task_labels(task);

    // Build volumes based on task type
    let mut volumes = vec![];
    let mut volume_mounts = vec![];

    // ConfigMap volume (always needed)
    volumes.push(json!({
        "name": "task-files",
        "configMap": {
            "name": cm_name
        }
    }));
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/config"
    }));

    // Workspace volume (only for code tasks)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");

        volumes.push(json!({
            "name": "workspace",
            "persistentVolumeClaim": {
                "claimName": pvc_name
            }
        }));
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));
    }

    // SSH volumes if needed
    if TaskType::uses_ssh() {
        let ssh_volumes = generate_ssh_volumes(task);
        volumes.extend(ssh_volumes);

        volume_mounts.push(json!({
            "name": "ssh-key",
            "mountPath": "/workspace/.ssh",
            "readOnly": true
        }));
    }

    // Mount settings.json directly to /etc/claude-code/managed-settings.json
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/etc/claude-code/managed-settings.json",
        "subPath": "settings.json",
        "readOnly": true
    }));

    // Guidelines files will be copied from ConfigMap to working directory by container.sh
    // No need to mount them separately since they need to be in the working directory

    // Environment variables
    let mut env_vars = vec![
        json!({"name": "ANTHROPIC_API_KEY", "valueFrom": {"secretKeyRef": {"name": config.secrets.api_key_secret_name, "key": config.secrets.api_key_secret_key}}}),
        json!({"name": "TASK_TYPE", "value": if task.is_docs() { "docs" } else { "code" }}),
        json!({"name": "MODEL", "value": task.model()}),
        json!({"name": "GITHUB_USER", "value": task.github_user()}),
        json!({"name": "REPOSITORY_URL", "value": task.repository_url()}),
        json!({"name": "WORKING_DIRECTORY", "value": task.working_directory()}),
    ];

    // Add GitHub token from secret for API operations (PR creation, etc.)
    env_vars.push(json!({
        "name": "GH_TOKEN",
        "valueFrom": {
            "secretKeyRef": {
                "name": task.github_token_secret_name(),
                "key": "token"
            }
        }
    }));

    // Add task-specific environment variables
    match task {
        TaskType::Docs(dr) => {
            env_vars.push(json!({"name": "SOURCE_BRANCH", "value": dr.spec.source_branch}));
        }
        TaskType::Code(cr) => {
            env_vars.push(json!({"name": "TASK_ID", "value": cr.spec.task_id.to_string()}));
            env_vars.push(json!({"name": "SERVICE_NAME", "value": cr.spec.service}));
            env_vars
                .push(json!({"name": "DOCS_REPOSITORY_URL", "value": cr.spec.docs_repository_url}));
            env_vars
                .push(json!({"name": "MCP_CLIENT_CONFIG", "value": "/.claude/client-config.json"}));

            if let Some(local_tools) = &cr.spec.local_tools {
                env_vars.push(json!({"name": "LOCAL_TOOLS", "value": local_tools}));
            }
            if let Some(remote_tools) = &cr.spec.remote_tools {
                env_vars.push(json!({"name": "REMOTE_TOOLS", "value": remote_tools}));
            }

            // Add toolman server URL for MCP integration
            // Environment variable: TOOLMAN_SERVER_URL (default: http://toolman.mcp.svc.cluster.local:3000/mcp)
            let toolman_url = std::env::var("TOOLMAN_SERVER_URL")
                .unwrap_or_else(|_| "http://toolman.mcp.svc.cluster.local:3000/mcp".to_string());
            env_vars.push(json!({"name": "TOOLMAN_SERVER_URL", "value": toolman_url}));

            // Add custom environment variables
            for (name, value) in &cr.spec.env {
                env_vars.push(json!({"name": name, "value": value}));
            }

            // Add environment variables from secrets
            for secret_env in &cr.spec.env_from_secrets {
                env_vars.push(json!({
                    "name": secret_env.name,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_env.secret_name,
                            "key": secret_env.secret_key
                        }
                    }
                }));
            }
        }
    }

    // Job deadline from config
    let job_deadline = config.job.active_deadline_seconds;

    // Agent image from config
    let agent_image = format!(
        "{}:{}",
        config.agent.image.repository, config.agent.image.tag
    );

    let job_spec = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "labels": labels
        },
        "spec": {
            "activeDeadlineSeconds": job_deadline,
            "backoffLimit": 0,
            "template": {
                "metadata": {
                    "labels": labels
                },
                "spec": {
                    "restartPolicy": "Never",
                    "securityContext": {
                        "fsGroup": 1000,
                        "runAsUser": 1000,
                        "runAsGroup": 1000
                    },
                    "imagePullSecrets": config.agent.image_pull_secrets.iter().map(|name| {
                        json!({"name": name})
                    }).collect::<Vec<_>>(),
                    "containers": [{
                        "name": "claude",
                        "image": agent_image,
                        "command": ["/bin/bash", "/config/container.sh"],
                        "env": env_vars,
                        "volumeMounts": volume_mounts,
                        "resources": {
                            "requests": {
                                "cpu": "100m",
                                "memory": "256Mi"
                            },
                            "limits": {
                                "cpu": "2",
                                "memory": "4Gi"
                            }
                        }
                    }],
                    "volumes": volumes
                }
            }
        }
    });

    Ok(serde_json::from_value(job_spec)?)
}

/// Create standard labels for task resources
fn create_task_labels(task: &TaskType) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();

    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert(
        "component".to_string(),
        if task.is_docs() {
            "docs-generator"
        } else {
            "code-runner"
        }
        .to_string(),
    );
    labels.insert(
        "github-user".to_string(),
        sanitize_label_value(task.github_user()),
    );
    labels.insert(
        "context-version".to_string(),
        task.context_version().to_string(),
    );

    match task {
        TaskType::Docs(_) => {
            labels.insert("task-type".to_string(), "docs".to_string());
        }
        TaskType::Code(_) => {
            labels.insert("task-type".to_string(), "code".to_string());
            if let Some(task_id) = task.task_id() {
                labels.insert("task-id".to_string(), task_id.to_string());
            }
            labels.insert("service-name".to_string(), task.service_name().to_string());
        }
    }

    labels
}

/// Sanitize a string value for use as a Kubernetes label value
/// Kubernetes labels must be an empty string or consist of alphanumeric characters, '-', '_' or '.',
/// and must start and end with an alphanumeric character
fn sanitize_label_value(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Replace spaces with hyphens, convert to lowercase
    let mut sanitized = input.to_lowercase().replace([' ', '_'], "-"); // Normalize spaces and underscores to hyphens

    // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
    sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    // Ensure it starts with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().next().unwrap().is_alphanumeric() {
        sanitized.remove(0);
    }

    // Ensure it ends with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().last().unwrap().is_alphanumeric() {
        sanitized.pop();
    }

    // If we ended up with an empty string, provide a fallback
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

/// Ensure PVC exists for the given service
async fn ensure_pvc_exists(
    pvcs: &Api<PersistentVolumeClaim>,
    pvc_name: &str,
    service_name: &str,
    config: &ControllerConfig,
) -> Result<()> {
    match pvcs.get(pvc_name).await {
        Ok(_) => {
            info!("PVC already exists: {}", pvc_name);
            return Ok(());
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // PVC doesn't exist, create it
        }
        Err(e) => return Err(e.into()),
    }

    let mut pvc_spec = json!({
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": pvc_name,
            "labels": {
                "app": "orchestrator",
                "service": service_name
            }
        },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": config.storage.workspace_size
                }
            }
        }
    });

    // Add storage class if specified
    if let Some(storage_class) = &config.storage.storage_class_name {
        pvc_spec["spec"]["storageClassName"] = json!(storage_class);
    }

    let pvc: PersistentVolumeClaim = serde_json::from_value(pvc_spec)?;
    pvcs.create(&PostParams::default(), &pvc).await?;
    info!("Created PVC: {}", pvc_name);

    Ok(())
}

/// Clean up older job versions for retry attempts
async fn cleanup_old_jobs(task: &TaskType, jobs: &Api<Job>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let job_list = jobs
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for job in job_list.items {
            if let Some(version) = job
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(job_name) = &job.metadata.name {
                        jobs.delete(job_name, &DeleteParams::background()).await?;
                        info!("Deleted older job version: {}", job_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Clean up older configmap versions for retry attempts
async fn cleanup_old_configmaps(task: &TaskType, configmaps: &Api<ConfigMap>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let cm_list = configmaps
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for cm in cm_list.items {
            if let Some(version) = cm
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(cm_name) = &cm.metadata.name {
                        configmaps.delete(cm_name, &DeleteParams::default()).await?;
                        info!("Deleted older configmap version: {}", cm_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Update the owner reference of an existing `ConfigMap`
async fn update_configmap_owner(
    _task: &TaskType,
    configmaps: &Api<ConfigMap>,
    cm_name: &str,
    owner_ref: OwnerReference,
) -> Result<()> {
    let mut configmap = configmaps.get(cm_name).await?;
    configmap.metadata.owner_references = Some(vec![owner_ref]);
    configmaps
        .replace(cm_name, &PostParams::default(), &configmap)
        .await?;
    info!("Updated ConfigMap owner reference for: {}", cm_name);
    Ok(())
}

/// Cleanup resources when task is deleted
pub async fn cleanup_resources(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
) -> Result<Action> {
    let task_label = if let Some(task_id) = task.task_id() {
        format!("task-id={task_id}")
    } else {
        format!(
            "task-type=docs,github-user={}",
            sanitize_label_value(task.github_user())
        )
    };

    info!("Cleaning up resources for task: {}", task.name());

    // Delete all jobs for this task
    let job_list = jobs
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }

    // Delete all configmaps for this task
    let cm_list = configmaps
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }

    Ok(Action::await_change())
}

```

### core/src/controllers/task_controller/templates.rs (541 lines)

**Key Definitions:**
```rust
32:pub fn generate_templates(
```

**Full Content:**
```rust
use super::config::ControllerConfig;
use super::types::{Result, TaskType};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

/// Load a template file from the mounted `ConfigMap`
fn load_template(relative_path: &str) -> Result<String> {
    // Convert path separators to underscores for ConfigMap key lookup
    let configmap_key = relative_path.replace('/', "_");
    let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
    debug!(
        "Loading template from: {} (key: {})",
        full_path.display(),
        configmap_key
    );

    fs::read_to_string(&full_path).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to load template {relative_path} (key: {configmap_key}): {e}"
        ))
    })
}

/// Generate all template files for a task
pub fn generate_templates(
    task: &TaskType,
    config: &ControllerConfig,
) -> Result<BTreeMap<String, String>> {
    let mut templates = BTreeMap::new();

    // Generate container startup script
    templates.insert("container.sh".to_string(), generate_container_script(task)?);

    // Generate Claude memory
    templates.insert("CLAUDE.md".to_string(), generate_claude_memory(task)?);

    // Generate Claude settings
    templates.insert(
        "settings.json".to_string(),
        generate_claude_settings(task, config)?,
    );

    // Generate task-specific templates
    if task.is_docs() {
        // Generate docs prompt
        templates.insert("prompt.md".to_string(), generate_docs_prompt(task)?);
    } else {
        // Generate code-specific templates
        templates.insert("mcp.json".to_string(), generate_mcp_config(task, config)?);
        templates.insert(
            "client-config.json".to_string(),
            generate_client_config(task, config)?,
        );
        templates.insert(
            "coding-guidelines.md".to_string(),
            generate_coding_guidelines(task)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            generate_github_guidelines(task)?,
        );
        templates.insert("mcp-tools.md".to_string(), generate_mcp_tools_doc(task)?);
    }

    // Generate hook scripts
    let hook_scripts = generate_hook_scripts(task)?;
    for (filename, content) in hook_scripts {
        // Use hooks- prefix to comply with ConfigMap key constraints
        templates.insert(format!("hooks-{filename}"), content);
    }

    Ok(templates)
}

/// Generate CLAUDE.md content from memory template
fn generate_claude_memory(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/claude.md.hbs"
    } else {
        "code/claude.md.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("claude_memory", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register CLAUDE.md template: {e}"
            ))
        })?;

    let data = json!({
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "task_id": task.task_id(),
        "docs_repository_url": task.docs_repository_url()
    });

    handlebars.render("claude_memory", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render CLAUDE.md template: {e}"
        ))
    })
}

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/settings.json.hbs"
    } else {
        "code/settings.json.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("settings", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register settings template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("settings", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render settings template: {e}"
        ))
    })
}

/// Generate container startup script from template
fn generate_container_script(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/container.sh.hbs"
    } else {
        "code/container.sh.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("container_script", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register container script template: {e}"
            ))
        })?;

    // Prompt content is now embedded inline in container script - no template needed

    let data = json!({
        "repository_url": task.repository_url(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "model": task.model(),
        "service_name": task.service_name(),
        "task_id": task.task_id(),
        "source_branch": task.source_branch(),
        "docs_repository_url": task.docs_repository_url(),
        "docs_branch": task.docs_branch(),
        "docs_project_directory": task.docs_project_directory(),
        "overwrite_memory": task.overwrite_memory(),
        "continue_session": task.continue_session(),
        "user_requested": match task {
            crate::controllers::task_controller::types::TaskType::Code(cr) => cr.spec.continue_session,
            _ => false
        }
    });

    handlebars.render("container_script", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render container script template: {e}"
        ))
    })
}

/// Generate MCP configuration for implementation tasks
fn generate_mcp_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp.json.hbs")?;

    handlebars
        .register_template_string("mcp", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("mcp", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP template: {e}"
        ))
    })
}

/// Generate MCP tools documentation based on task configuration
fn generate_mcp_tools_doc(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp-tools.md.hbs")?;

    handlebars
        .register_template_string("mcp_tools", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP tools template: {e}"
            ))
        })?;

    // Parse comma-separated tool strings into arrays
    let local_tools: Vec<String> = task
        .local_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let remote_tools: Vec<String> = task
        .remote_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let data = json!({
        "localTools": local_tools,
        "remoteTools": remote_tools,
        "service": task.service_name(),
        "task_id": task.task_id()
    });

    handlebars.render("mcp_tools", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP tools template: {e}"
        ))
    })
}

/// Generate client configuration for dynamic tool selection
fn generate_client_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/client-config.json.hbs")?;

    handlebars
        .register_template_string("client_config", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register client config template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("client_config", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render client config template: {e}"
        ))
    })
}

/// Generate coding guidelines for implementation tasks
fn generate_coding_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/coding-guidelines.md.hbs")?;

    handlebars
        .register_template_string("coding_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register coding guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name()
    });

    handlebars.render("coding_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render coding guidelines template: {e}"
        ))
    })
}

/// Generate GitHub guidelines for implementation tasks
fn generate_github_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/github-guidelines.md.hbs")?;

    handlebars
        .register_template_string("github_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register GitHub guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user()
    });

    handlebars.render("github_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render GitHub guidelines template: {e}"
        ))
    })
}

/// Generate docs prompt for documentation generation tasks
fn generate_docs_prompt(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("docs/prompt.md.hbs")?;

    handlebars
        .register_template_string("docs_prompt", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register docs prompt template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "repository_url": task.repository_url()
    });

    handlebars.render("docs_prompt", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render docs prompt template: {e}"
        ))
    })
}

/// Build template data for settings/MCP/client config templates
fn build_settings_template_data(task: &TaskType, config: &ControllerConfig) -> serde_json::Value {
    let mut data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "model": task.model(),
        "github_user": task.github_user(),
        "repository": {
            "url": task.repository_url(),
            "githubUser": task.github_user()
        },
        "working_directory": task.working_directory(),
        "agent_tools_override": config.permissions.agent_tools_override,
        "permissions": {
            "allow": config.permissions.allow,
            "deny": config.permissions.deny
        },
        "telemetry": {
            "enabled": config.telemetry.enabled,
            "otlpEndpoint": config.telemetry.otlp_endpoint,
            "otlpProtocol": config.telemetry.otlp_protocol,
            "logs_endpoint": config.telemetry.logs_endpoint,
            "logs_protocol": config.telemetry.logs_protocol
        }
    });

    // Add retry information for code tasks
    if !task.is_docs() {
        let retry_data = json!({
            "context_version": task.context_version(),
            "prompt_modification": task.prompt_modification(),
            "session_id": task.session_id()
        });
        data["retry"] = retry_data;

        // Add tool configuration
        let (local_tools, remote_tools) = parse_tool_configuration(task);
        data["tools"] = json!({
            "local": local_tools,
            "remote": remote_tools
        });

        // Add docs repository info
        if let Some(docs_url) = task.docs_repository_url() {
            data["docs_repository_url"] = json!(docs_url);
        }
    }

    data
}

/// Parse tool configuration into local and remote tool lists
fn parse_tool_configuration(task: &TaskType) -> (Vec<String>, Vec<String>) {
    let local_tools = task
        .local_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let remote_tools = task
        .remote_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    (local_tools, remote_tools)
}

/// Generate hook scripts from the hooks directory
fn generate_hook_scripts(task: &TaskType) -> Result<BTreeMap<String, String>> {
    let mut hook_scripts = BTreeMap::new();
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    // Get hook templates based on task type
    let hook_templates = get_hook_templates(task)?;

    // Prepare template data
    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "attempts": task.retry_count() + 1, // retry_count + 1 = attempt number
        "is_docs_generation": task.is_docs(),
        "docs_repository_url": task.docs_repository_url()
    });

    // Process each hook template
    for (hook_name, template_content) in hook_templates {
        handlebars
            .register_template_string(&hook_name, &template_content)
            .map_err(|e| {
                crate::controllers::task_controller::types::Error::ConfigError(format!(
                    "Failed to register hook template {hook_name}: {e}"
                ))
            })?;

        let rendered = handlebars.render(&hook_name, &data).map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to render hook template {hook_name}: {e}"
            ))
        })?;

        // Remove .hbs extension for the final filename
        let filename = hook_name.strip_suffix(".hbs").unwrap_or(&hook_name);
        hook_scripts.insert(filename.to_string(), rendered);
    }

    Ok(hook_scripts)
}

/// Get all hook templates for a specific task type by scanning the filesystem
fn get_hook_templates(task: &TaskType) -> Result<Vec<(String, String)>> {
    let hooks_prefix = match task {
        TaskType::Docs(_) => "docs_hooks_",
        TaskType::Code(_) => "code_hooks_",
    };

    debug!("Scanning for hook templates with prefix: {}", hooks_prefix);

    let mut templates = Vec::new();

    // Read the ConfigMap directory and find files with the hook prefix
    match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if this is a hook template for our task type
                        if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                            // Extract just the hook filename (remove prefix and convert back)
                            let hook_name = filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    debug!(
                                        "Loaded hook template: {} (from {})",
                                        hook_name, filename
                                    );
                                    templates.push((hook_name.to_string(), content));
                                }
                                Err(e) => {
                                    debug!("Failed to load hook template {}: {}", filename, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!(
                "Templates directory {} not found or not accessible: {}",
                CLAUDE_TEMPLATES_PATH, e
            );
            // Don't fail - hooks are optional
        }
    }

    Ok(templates)
}

```

### core/src/handlers/code_handler.rs (126 lines)

**Full Content:**
```rust
//! Code task submission handler

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use kube::Api;
use std::collections::HashMap;
use tracing::{error, info};

use crate::crds::{CodeRun, CodeRunSpec, CodeRunStatus};
use crate::handlers::common::{ApiResponse, AppState};
use common::models::CodeRequest;

pub async fn submit_code_task(
    State(state): State<AppState>,
    Json(request): Json<CodeRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!(
        "Received code task request: task_id={}, service={}",
        request.task_id, request.service
    );

    let spec = CodeRunSpec {
        task_id: request.task_id,
        service: request.service.clone(),
        repository_url: request.repository_url,
        docs_repository_url: request.docs_repository_url,
        docs_project_directory: request.docs_project_directory,
        working_directory: request.working_directory,
        model: request.model.unwrap_or_else(|| {
            std::env::var("DEFAULT_CODE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string())
        }),
        github_user: request.github_user,
        local_tools: request.local_tools,
        remote_tools: request.remote_tools,
        context_version: request.context_version,
        prompt_modification: request.prompt_modification,
        docs_branch: request.docs_branch,
        continue_session: request.continue_session,
        overwrite_memory: request.overwrite_memory,
        env: request.env,
        env_from_secrets: request
            .env_from_secrets
            .into_iter()
            .map(|s| crate::crds::coderun::SecretEnvVar {
                name: s.name,
                secret_name: s.secret_name,
                secret_key: s.secret_key,
            })
            .collect(),
    };

    let coderun = CodeRun {
        metadata: kube::api::ObjectMeta {
            name: Some(format!(
                "code-{}-{}",
                request.task_id,
                Utc::now().timestamp()
            )),
            namespace: Some(state.namespace.clone()),
            ..Default::default()
        },
        spec,
        status: Some(CodeRunStatus {
            phase: "Pending".to_string(),
            message: Some("CodeRun created successfully".to_string()),
            last_update: Some(Utc::now().to_rfc3339()),
            job_name: None,
            pull_request_url: None,
            retry_count: Some(0),
            conditions: None,
            configmap_name: None,
            context_version: Some(1),
            prompt_modification: None,
            prompt_mode: Some("direct".to_string()),
            session_id: None,
        }),
    };

    let api: Api<CodeRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);

    // Check if a CodeRun already exists for this task
    let existing_name = format!("code-{}", request.task_id);
    if let Ok(_existing) = api.get(&existing_name).await {
        error!("CodeRun already exists for task {}", request.task_id);
        return Ok(Json(ApiResponse {
            success: false,
            message: format!("CodeRun already exists for task {}", request.task_id),
            data: None,
        }));
    }

    match api.create(&Default::default(), &coderun).await {
        Ok(created) => {
            info!("CodeRun created successfully: {:?}", created.metadata.name);

            let mut response_data = HashMap::new();
            if let Some(name) = &created.metadata.name {
                response_data.insert(
                    "coderun_name".to_string(),
                    serde_json::Value::String(name.clone()),
                );
            }
            response_data.insert(
                "namespace".to_string(),
                serde_json::Value::String(state.namespace.clone()),
            );

            Ok(Json(ApiResponse {
                success: true,
                message: "Code task submitted successfully".to_string(),
                data: Some(serde_json::Value::Object(
                    response_data.into_iter().collect(),
                )),
            }))
        }
        Err(e) => {
            error!("Failed to create CodeRun: {}", e);
            Ok(Json(ApiResponse {
                success: false,
                message: format!("Failed to create CodeRun: {e}"),
                data: None,
            }))
        }
    }
}

```

### core/src/handlers/mod.rs (9 lines)

**Full Content:**
```rust
//! Request handlers for the orchestrator service

pub mod code_handler;
pub mod common;
pub mod docs_handler;

pub use code_handler::submit_code_task;
pub use common::{ApiResponse, AppError, AppState};
pub use docs_handler::generate_docs;

```

### core/src/handlers/docs_handler.rs (102 lines)

**Full Content:**
```rust
//! Documentation generation handler

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, PostParams};
use serde_json::json;
use std::collections::BTreeMap;
use tracing::{error, info};

use crate::crds::{DocsRun, DocsRunSpec, DocsRunStatus};
use crate::handlers::common::{ApiResponse, AppError, AppState};
use common::models::DocsRequest;

/// Generate documentation for Task Master tasks
pub async fn generate_docs(
    State(state): State<AppState>,
    Json(request): Json<DocsRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), (StatusCode, Json<ApiResponse>)> {
    info!(
        "Generate documentation request received for repository: {}",
        request.repository_url
    );

    // Generate a unique DocsRun name using timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let docsrun_name = format!("docs-gen-{timestamp}");

    // Create DocsRun spec for documentation generation
    let spec = DocsRunSpec {
        repository_url: request.repository_url.clone(),
        working_directory: request.working_directory.clone(),
        source_branch: request.source_branch.clone(),
        model: request.model.unwrap_or_else(|| {
            std::env::var("DEFAULT_DOCS_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-20250514".to_string())
        }),
        github_user: request.github_user.clone(),
    };

    // Create DocsRun
    let docsrun = DocsRun {
        metadata: ObjectMeta {
            name: Some(docsrun_name.clone()),
            namespace: Some(state.namespace.clone()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app".to_string(), "orchestrator".to_string());
                labels.insert("type".to_string(), "docs".to_string());
                labels
            }),
            ..Default::default()
        },
        spec,
        status: Some(DocsRunStatus {
            phase: "Pending".to_string(),
            message: Some("DocsRun created successfully".to_string()),
            last_update: Some(chrono::Utc::now().to_rfc3339()),
            job_name: None,
            pull_request_url: None,
            conditions: None,
            configmap_name: None,
        }),
    };

    // Create DocsRun in Kubernetes
    let api: Api<DocsRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);

    match api.create(&PostParams::default(), &docsrun).await {
        Ok(created) => {
            info!("Created documentation generation DocsRun: {}", docsrun_name);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse {
                    success: true,
                    message: "Documentation generation job submitted successfully".to_string(),
                    data: Some(json!({
                        "docsrun_name": docsrun_name,
                        "namespace": state.namespace,
                        "repository_url": created.spec.repository_url,
                        "model": created.spec.model,
                    })),
                }),
            ))
        }
        Err(e) => {
            error!("Failed to create documentation generation DocsRun: {}", e);
            let status_code = StatusCode::from(AppError::from(e));
            Err((
                status_code,
                Json(ApiResponse::error(&format!(
                    "Failed to submit documentation generation job: {}",
                    status_code.canonical_reason().unwrap_or("Unknown error")
                ))),
            ))
        }
    }
}

```

### core/src/handlers/common.rs (77 lines)

**Key Definitions:**
```rust
9:pub struct AppState {
16:pub enum AppError {
22:impl std::fmt::Display for AppError {
32:impl std::error::Error for AppError {}
34:impl From<kube::Error> for AppError {
40:impl From<AppError> for StatusCode {
52:pub struct ApiResponse {
59:impl ApiResponse {
61:pub fn success(message: &str) -> Self {
70:pub fn error(message: &str) -> Self {
```

**Full Content:**
```rust
//! Shared types and utilities for API handlers

use axum::http::StatusCode;
use kube::Client;
use serde_json::Value;

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

```

### tests/simple_integration_test.rs (91 lines)

**Full Content:**
```rust
use std::env;

#[tokio::test]
async fn test_basic_functionality() {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        println!("Skipping integration tests due to SKIP_INTEGRATION_TESTS environment variable");
        return;
    }

    println!("🧪 Running basic functionality test...");

    // Test that we can create a test JSON structure
    let test_event = create_test_github_event(123, "Test issue", "Test body");
    assert_eq!(test_event["issue"]["number"], 123);
    assert_eq!(test_event["issue"]["title"], "Test issue");
    assert_eq!(test_event["action"], "opened");

    println!("✅ JSON event creation works");

    // Test that Axum router can be created
    use axum::Router;

    let _app: Router<()> = Router::new().route("/health", axum::routing::get(|| async { "OK" }));

    println!("✅ Axum router creation works");

    // TODO: Fix configuration test when config module is available
    // use core::config::Config;
    // let _config = Config::default();

    println!("✅ Basic tests completed (some functionality commented out)");

    println!("🎉 Basic integration test completed successfully!");
}

fn create_test_github_event(
    issue_number: i64,
    title: &str,
    body: &str,
) -> serde_json::Value {
    use serde_json::json;

    serde_json::from_value(json!({
        "action": "opened",
        "issue": {
            "id": 123456789,
            "number": issue_number,
            "title": title,
            "body": body,
            "html_url": format!("https://github.com/test-org/test-repo/issues/{}", issue_number),
            "created_at": "2024-06-30T10:00:00Z",
            "updated_at": "2024-06-30T10:00:00Z",
            "labels": [
                {
                    "name": "enhancement",
                    "color": "a2eeef",
                    "description": "New feature or request"
                }
            ],
            "user": {
                "login": "testuser",
                "id": 12345,
                "avatar_url": "https://avatars.githubusercontent.com/u/12345?v=4",
                "html_url": "https://github.com/testuser"
            },
            "state": "open"
        },
        "repository": {
            "id": 987654321,
            "name": "test-repo",
            "full_name": "test-org/test-repo",
            "owner": {
                "login": "test-org",
                "id": 54321,
                "avatar_url": "https://avatars.githubusercontent.com/u/54321?v=4",
                "html_url": "https://github.com/test-org"
            },
            "html_url": "https://github.com/test-org/test-repo",
            "description": "A test repository for webhook testing",
            "default_branch": "main",
            "clone_url": "https://github.com/test-org/test-repo.git"
        },
        "sender": {
            "login": "testuser",
            "id": 12345,
            "avatar_url": "https://avatars.githubusercontent.com/u/12345?v=4",
            "html_url": "https://github.com/testuser"
        }
    }))
    .expect("Failed to create test GitHub event")
}

```

### common/src/error.rs (37 lines)

**Key Definitions:**
```rust
6:pub enum Error {
33:impl From<anyhow::Error> for Error {
```

**Full Content:**
```rust
//! Common error types for the orchestrator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes operation failed: {0}")]
    Kubernetes(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Job failed: {0}")]
    JobFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversion from anyhow::Error for easier error handling
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

```

### common/src/lib.rs (28 lines)

**Full Content:**
```rust
/*
 * 5D Labs Agent Platform - Common Types and Utilities
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

//! Shared types and utilities for the Orchestrator project

pub mod error;
pub mod models;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Re-export commonly used types for convenience
pub use models::{AgentType, Job, JobStatus, JobType, Request, RequestSource, Task, TaskStatus};

```

### common/src/models/response.rs (239 lines)

**Key Definitions:**
```rust
10:pub struct ApiResponse<T> {
20:pub enum ResponseStatus {
28:pub struct ErrorDetails {
36:pub struct ResponseMetadata {
45:pub struct TaskResponse {
59:pub struct JobResponse {
74:pub struct JobListResponse {
83:pub struct TaskListResponse {
92:pub struct HealthResponse {
102:pub enum HealthStatus {
110:pub struct ComponentHealth {
118:pub struct WebhookResponse {
127:pub fn success(data: T, request_id: String) -> Self {
143:pub fn error(error: ErrorDetails, request_id: String) -> Self {
158:pub fn with_duration(mut self, duration_ms: u64) -> Self {
164:impl From<super::task::Task> for TaskResponse {
180:impl From<super::job::Job> for JobResponse {
```

**Full Content:**
```rust
//! Response models for API endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: ResponseStatus,
    pub data: Option<T>,
    pub error: Option<ErrorDetails>,
    pub metadata: ResponseMetadata,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Error,
    Partial,
}

/// Error details in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub version: String,
}

/// Task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: super::task::TaskStatus,
    pub priority: super::task::TaskPriority,
    pub microservice: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub job_ids: Vec<String>,
}

/// Job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub task_id: String,
    pub job_type: super::job::JobType,
    pub status: super::job::JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub logs_url: Option<String>,
}

/// Job list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Task list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Webhook processing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub accepted: bool,
    pub task_id: Option<String>,
    pub job_ids: Vec<String>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// Create a success response
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(error: ErrorDetails, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: None,
            error: Some(error),
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Set the duration in milliseconds
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.metadata.duration_ms = Some(duration_ms);
        self
    }
}

impl From<super::task::Task> for TaskResponse {
    fn from(task: super::Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            microservice: task.microservice,
            created_at: task.created_at,
            updated_at: task.updated_at,
            job_ids: Vec::new(), // To be populated by service layer
        }
    }
}

impl From<super::job::Job> for JobResponse {
    fn from(job: super::Job) -> Self {
        Self {
            id: job.id,
            task_id: job.task_id,
            job_type: job.job_type,
            status: job.status,
            k8s_job_name: job.k8s_job_name,
            namespace: job.namespace,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            logs_url: None, // To be populated by service layer
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::{TaskPriority, TaskStatus};

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(
            TaskResponse {
                id: "task-123".to_string(),
                title: "Test Task".to_string(),
                description: "A test task".to_string(),
                status: TaskStatus::Pending,
                priority: TaskPriority::Medium,
                microservice: "auth".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                job_ids: vec![],
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<TaskResponse> = ApiResponse::error(
            ErrorDetails {
                code: "TASK_NOT_FOUND".to_string(),
                message: "Task not found".to_string(),
                details: None,
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Error);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}

```

### common/src/models/request.rs (201 lines)

**Key Definitions:**
```rust
9:pub struct Request {
20:pub enum RequestSource {
31:pub enum RequestAction {
44:pub struct RequestMetadata {
56:pub struct ParsedRequest {
69:pub struct CliRequest {
77:pub struct CreateTaskRequest {
89:pub struct UpdateTaskRequest {
99:pub struct AssistanceRequest {
110:pub enum AssistanceType {
121:pub enum AssistancePriority {
128:impl Request {
131:pub fn new(source: RequestSource, action: RequestAction, payload: Value) -> Self {
149:pub fn with_trace_id(mut self, trace_id: String) -> Self {
156:pub fn with_user(mut self, user: String) -> Self {
```

**Full Content:**
```rust
//! Request models for unified orchestration

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Unified request interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub source: RequestSource,
    pub action: RequestAction,
    pub payload: Value,
    pub metadata: RequestMetadata,
}

/// Source of incoming requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestSource {
    Cli,
    PmAgent,
    GitHub,
    Grafana,
    Discord,
}

/// Action to be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestAction {
    CreateTask,
    UpdateTask,
    GetTaskStatus,
    TriggerAssistance,
    ListJobs,
    GetJobLogs,
    ReviewPR,
    HandleAlert,
}

/// Additional request metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestMetadata {
    pub user: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub channel: Option<String>,
    pub timestamp: String,
    pub trace_id: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Parsed request after normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequest {
    pub action: RequestAction,
    pub task_id: Option<String>,
    pub microservice: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub metadata: Value,
}

/// CLI request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub command: String,
    pub args: Vec<String>,
    pub options: HashMap<String, String>,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub microservice: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub agent_type: Option<super::AgentType>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Task update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<super::TaskStatus>,
    pub priority: Option<super::task::TaskPriority>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Assistance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceRequest {
    pub task_id: String,
    pub reason: String,
    pub assist_type: AssistanceType,
    pub context: Option<Value>,
    pub priority: AssistancePriority,
}

/// Type of assistance needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistanceType {
    ImplementationGuidance,
    ArchitectureReview,
    ErrorDiagnosis,
    TestDebugging,
    PerformanceOptimization,
}

/// Priority of assistance request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistancePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Request {
    /// Create a new request
    #[must_use]
    pub fn new(source: RequestSource, action: RequestAction, payload: Value) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        Self {
            id: Uuid::new_v4().to_string(),
            source,
            action,
            payload,
            metadata: RequestMetadata {
                timestamp: Utc::now().to_rfc3339(),
                ..Default::default()
            },
        }
    }

    /// Add trace ID for distributed tracing
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }

    /// Add user information
    #[must_use]
    pub fn with_user(mut self, user: String) -> Self {
        self.metadata.user = Some(user);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_creation() {
        let request = Request::new(
            RequestSource::Cli,
            RequestAction::CreateTask,
            json!({
                "title": "Test Task",
                "description": "A test task"
            }),
        );

        assert_eq!(request.source, RequestSource::Cli);
        assert_eq!(request.action, RequestAction::CreateTask);
        assert!(!request.id.is_empty());
        assert!(!request.metadata.timestamp.is_empty());
    }

    #[test]
    fn test_create_task_request_serialization() {
        let req = CreateTaskRequest {
            microservice: "auth".to_string(),
            title: "Implement JWT validation".to_string(),
            description: "Add JWT token validation".to_string(),
            acceptance_criteria: vec!["Validate tokens".to_string()],
            priority: Some("high".to_string()),
            agent_type: Some(super::super::AgentType::Claude),
            metadata: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: CreateTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.title, deserialized.title);
        assert_eq!(req.microservice, deserialized.microservice);
    }
}

```

### common/src/models/job.rs (244 lines)

**Key Definitions:**
```rust
10:pub struct Job {
26:pub enum JobType {
40:pub enum JobStatus {
50:pub struct JobSpec {
73:pub struct VolumeSpec {
83:pub enum VolumeType {
94:impl Job {
97:pub fn new(
120:pub fn update_from_k8s_job(&mut self, k8s_job: &K8sJob) {
147:pub fn is_terminal(&self) -> bool {
153:pub fn duration(&self) -> Option<chrono::Duration> {
161:impl Default for JobSpec {
177:impl std::fmt::Display for JobType {
188:impl std::fmt::Display for JobStatus {
```

**Full Content:**
```rust
//! Job-related data models for Kubernetes job orchestration

use chrono::{DateTime, Utc};
use k8s_openapi::api::batch::v1::Job as K8sJob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Kubernetes job for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub task_id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub spec: JobSpec,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Type of job in the orchestration pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Prepare job that sets up workspace and context files
    Prepare,
    /// Execute job that runs the primary agent (Claude)
    Execute,
    /// Assist job that runs helper agent (Gemini)
    Assist,
    /// Review job for code review tasks
    Review,
}

/// Job execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

/// Job specification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// Container image to use
    pub image: String,
    /// Agent type for execution jobs
    pub agent: Option<super::AgentType>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Resource limits and requests
    pub resources: super::ResourceLimits,
    /// Volume mounts
    pub volumes: Vec<VolumeSpec>,
    /// Command to execute
    pub command: Option<Vec<String>>,
    /// Working directory
    pub working_dir: Option<String>,
    /// Job timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Number of retries
    pub retry_limit: Option<u32>,
}

/// Volume specification for job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub mount_path: String,
    pub volume_type: VolumeType,
    pub read_only: bool,
}

/// Types of volumes that can be mounted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeType {
    /// `ConfigMap` volume
    ConfigMap { name: String },
    /// `PersistentVolumeClaim`
    Pvc { claim_name: String },
    /// `EmptyDir` volume
    EmptyDir,
    /// Secret volume
    Secret { name: String },
}

impl Job {
    /// Create a new job
    #[must_use]
    pub fn new(
        id: String,
        task_id: String,
        job_type: JobType,
        k8s_job_name: String,
        namespace: String,
        spec: JobSpec,
    ) -> Self {
        Self {
            id,
            task_id,
            job_type,
            status: JobStatus::Pending,
            k8s_job_name,
            namespace,
            spec,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Update job status based on Kubernetes job status
    pub fn update_from_k8s_job(&mut self, k8s_job: &K8sJob) {
        if let Some(status) = &k8s_job.status {
            if status.succeeded == Some(1) {
                self.status = JobStatus::Succeeded;
                self.completed_at = status.completion_time.as_ref().map(|t| {
                    DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                        .unwrap()
                        .with_timezone(&Utc)
                });
            } else if status.failed.unwrap_or(0) > 0 {
                self.status = JobStatus::Failed;
                self.completed_at = Some(Utc::now());
            } else if status.active == Some(1) {
                self.status = JobStatus::Running;
                if self.started_at.is_none() {
                    self.started_at = status.start_time.as_ref().map(|t| {
                        DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                            .unwrap()
                            .with_timezone(&Utc)
                    });
                }
            }
        }
    }

    /// Check if the job is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Succeeded | JobStatus::Failed)
    }

    /// Get job duration if available
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            image: "busybox:latest".to_string(),
            agent: None,
            env_vars: HashMap::new(),
            resources: super::ResourceLimits::default(),
            volumes: Vec::new(),
            command: None,
            working_dir: None,
            timeout_seconds: Some(1800), // 30 minutes default
            retry_limit: Some(2),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Prepare => write!(f, "Prepare"),
            JobType::Execute => write!(f, "Execute"),
            JobType::Assist => write!(f, "Assist"),
            JobType::Review => write!(f, "Review"),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Succeeded => write!(f, "Succeeded"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let spec = JobSpec::default();
        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(!job.is_terminal());
    }

    #[test]
    fn test_job_serialization() {
        let spec = JobSpec {
            image: "claude:latest".to_string(),
            agent: Some(super::super::AgentType::Claude),
            ..Default::default()
        };

        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job.id, deserialized.id);
        assert_eq!(job.job_type, deserialized.job_type);
    }
}

```

### common/src/models/config.rs (147 lines)

**Key Definitions:**
```rust
9:pub enum AgentType {
17:pub struct AgentConfig {
29:pub struct ResourceLimits {
40:pub struct McpServerConfig {
50:pub struct OrchestratorConfig {
60:impl Default for ResourceLimits {
73:impl AgentType {
76:pub fn display_name(&self) -> &'static str {
85:pub fn default_image(&self) -> &'static str {
94:pub fn can_implement(&self) -> bool {
103:pub fn can_assist(&self) -> bool {
```

**Full Content:**
```rust
//! Configuration-related models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent types that can execute tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    #[default]
    Claude,
    Gemini,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub image: String,
    pub version: String,
    pub env_vars: HashMap<String, String>,
    pub resources: ResourceLimits,
    pub capabilities: Vec<String>,
    pub mcp_servers: Vec<String>,
}

/// Resource limits and requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub ephemeral_storage_request: Option<String>,
    pub ephemeral_storage_limit: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub capabilities: Vec<String>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub namespace: String,
    pub agents: HashMap<AgentType, AgentConfig>,
    pub default_timeout_seconds: u32,
    pub max_retry_attempts: u32,
    pub workspace_pvc_template: String,
    pub prepare_job_image: String,
    pub node_selector: HashMap<String, String>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "2Gi".to_string(),
            ephemeral_storage_request: None,
            ephemeral_storage_limit: None,
        }
    }
}

impl AgentType {
    /// Get display name for the agent
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::Claude => "Claude Code",
            AgentType::Gemini => "Gemini CLI",
        }
    }

    /// Get the default image for the agent
    #[must_use]
    pub fn default_image(&self) -> &'static str {
        match self {
            AgentType::Claude => "anthropic/claude-code:latest",
            AgentType::Gemini => "google/gemini-cli:latest",
        }
    }

    /// Check if this agent can be a primary implementer
    #[must_use]
    pub fn can_implement(&self) -> bool {
        match self {
            AgentType::Claude => true,
            AgentType::Gemini => false, // Gemini is assistance-only in our pattern
        }
    }

    /// Check if this agent can provide assistance
    #[must_use]
    pub fn can_assist(&self) -> bool {
        match self {
            AgentType::Claude => false, // Claude is implementation-only in our pattern
            AgentType::Gemini => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_capabilities() {
        assert!(AgentType::Claude.can_implement());
        assert!(!AgentType::Claude.can_assist());
        assert!(!AgentType::Gemini.can_implement());
        assert!(AgentType::Gemini.can_assist());
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu_request, "100m");
        assert_eq!(limits.memory_limit, "2Gi");
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            agent_type: AgentType::Claude,
            image: "claude:v1".to_string(),
            version: "1.0.0".to_string(),
            env_vars: HashMap::new(),
            resources: ResourceLimits::default(),
            capabilities: vec!["code".to_string(), "test".to_string()],
            mcp_servers: vec!["taskmaster".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.agent_type, deserialized.agent_type);
        assert_eq!(config.capabilities, deserialized.capabilities);
    }
}

```

### common/src/models/webhook.rs (232 lines)

**Key Definitions:**
```rust
9:pub struct WebhookPayload {
17:pub struct GitHubWebhookPayload {
27:pub struct GitHubIssue {
41:pub struct GitHubPullRequest {
56:pub struct GitHubRepository {
67:pub struct GitHubUser {
76:pub struct GitHubLabel {
83:pub struct GitHubRef {
91:pub struct GrafanaAlert {
106:pub struct GrafanaWebhookPayload {
120:pub struct PmAgentPayload {
128:pub struct PmTaskData {
141:pub struct DiscordPayload {
```

**Full Content:**
```rust
//! Webhook payload models for various sources

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub source: super::RequestSource,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

/// GitHub webhook payload for issue events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub pull_request: Option<GitHubPullRequest>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// GitHub issue structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub pull request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub head: GitHubRef,
    pub base: GitHubRef,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub private: bool,
    pub default_branch: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(rename = "type")]
    pub user_type: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

/// GitHub ref (branch/tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    pub label: String,
    pub ref_field: String,
    pub sha: String,
}

/// Grafana alert webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaAlert {
    pub status: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub values: HashMap<String, f64>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
}

/// Grafana webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaWebhookPayload {
    pub receiver: String,
    pub status: String,
    pub alerts: Vec<GrafanaAlert>,
    #[serde(rename = "groupLabels")]
    pub group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    pub common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    pub common_annotations: HashMap<String, String>,
}

/// PM Agent webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmAgentPayload {
    pub action: String,
    pub project_id: String,
    pub task: PmTaskData,
}

/// PM Agent task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Discord webhook payload (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub command: String,
    pub args: Vec<String>,
    pub message_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_webhook_deserialization() {
        let json = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 42,
                "title": "Test Issue",
                "body": "Test body",
                "state": "open",
                "labels": [],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "user": {
                    "login": "testuser",
                    "id": 456,
                    "type": "User"
                }
            },
            "repository": {
                "id": 789,
                "name": "test-repo",
                "full_name": "org/test-repo",
                "owner": {
                    "login": "org",
                    "id": 999,
                    "type": "Organization"
                },
                "private": false,
                "default_branch": "main"
            },
            "sender": {
                "login": "testuser",
                "id": 456,
                "type": "User"
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, "opened");
        assert!(payload.issue.is_some());
        assert_eq!(payload.issue.unwrap().number, 42);
    }

    #[test]
    fn test_grafana_alert_deserialization() {
        let json = r#"{
            "receiver": "webhook",
            "status": "firing",
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "HighErrorRate",
                    "task_id": "123"
                },
                "annotations": {
                    "summary": "High error rate detected"
                },
                "values": {
                    "error_rate": 0.45
                },
                "startsAt": "2024-01-01T00:00:00Z",
                "endsAt": null,
                "generatorURL": "http://grafana/alert"
            }],
            "groupLabels": {},
            "commonLabels": {},
            "commonAnnotations": {}
        }"#;

        let payload: GrafanaWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.status, "firing");
        assert_eq!(payload.alerts.len(), 1);
        assert_eq!(
            payload.alerts[0].labels.get("task_id"),
            Some(&"123".to_string())
        );
    }
}

```

### common/src/models/code_request.rs (91 lines)

**Key Definitions:**
```rust
9:pub struct SecretEnvVar {
21:pub struct CodeRequest {
```

**Full Content:**
```rust
//! Clean code task submission request structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export SecretEnvVar from orchestrator-core crate to avoid duplication
// For now, we'll define it locally until we can reorganize the type sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEnvVar {
    /// Name of the environment variable
    pub name: String,
    /// Name of the secret
    #[serde(rename = "secretName")]
    pub secret_name: String,
    /// Key within the secret
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    /// Task ID to implement
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default)]
    pub remote_tools: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version")]
    pub context_version: u32,

    /// Additional context for retry attempts
    #[serde(default)]
    pub prompt_modification: Option<String>,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default)]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default)]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default)]
    pub env_from_secrets: Vec<SecretEnvVar>,
}

/// Default context version
fn default_context_version() -> u32 {
    1
}

/// Default docs branch
fn default_docs_branch() -> String {
    "main".to_string()
}

```

### common/src/models/task.rs (176 lines)

**Key Definitions:**
```rust
9:pub struct Task {
26:pub enum TaskStatus {
38:pub enum TaskPriority {
48:pub struct TaskMetadata {
62:impl Task {
65:pub fn new(id: String, title: String, description: String, microservice: String) -> Self {
84:pub fn is_terminal(&self) -> bool {
92:pub fn update_status(&mut self, status: TaskStatus) {
98:impl std::fmt::Display for TaskStatus {
111:impl std::fmt::Display for TaskPriority {
```

**Full Content:**
```rust
//! Task-related data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub microservice: String,
    pub agent_type: Option<super::AgentType>,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Additional metadata for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskMetadata {
    /// Source that created this task
    pub source: Option<String>,
    /// GitHub issue number if applicable
    pub github_issue: Option<u64>,
    /// Task Master task ID if applicable
    pub task_master_id: Option<String>,
    /// Custom labels
    pub labels: HashMap<String, String>,
    /// Additional arbitrary data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Task {
    /// Create a new task with default values
    #[must_use]
    pub fn new(id: String, title: String, description: String, microservice: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            acceptance_criteria: Vec::new(),
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            microservice,
            agent_type: None,
            metadata: TaskMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the task is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }

    /// Update the task status and timestamp
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert_eq!(task.id, "test-123");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(!task.is_terminal());
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.status, deserialized.status);
    }

    #[test]
    fn test_terminal_states() {
        let mut task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert!(!task.is_terminal());

        task.update_status(TaskStatus::Completed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::Failed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::InProgress);
        assert!(!task.is_terminal());
    }
}

```

### common/src/models/docs_request.rs (21 lines)

**Key Definitions:**
```rust
6:pub struct DocsRequest {
```

**Full Content:**
```rust
//! Clean documentation generation request structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsRequest {
    /// Git repository URL
    pub repository_url: String,

    /// Working directory within the repository
    pub working_directory: String,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Source branch (auto-detected)
    pub source_branch: String,
}

```

### common/src/models/mod.rs (25 lines)

**Full Content:**
```rust
//! Core data models module

pub mod code_request;
pub mod config;
pub mod docs_request;
pub mod job;
pub mod pm_task;
pub mod request;
pub mod response;
pub mod task;
pub mod webhook;

// Re-export commonly used types
pub use code_request::CodeRequest;
pub use config::{AgentConfig, AgentType, ResourceLimits};
pub use docs_request::DocsRequest;
pub use job::{Job, JobSpec, JobStatus, JobType};
pub use pm_task::{
    DocsGenerationRequest, MarkdownPayload, PmTaskRequest, Subtask, Task as PmTask, TaskMaster,
    TaskMasterFile,
};
pub use request::{ParsedRequest, Request, RequestAction, RequestSource};
pub use response::{ApiResponse, JobResponse, TaskResponse};
pub use task::{Task, TaskMetadata, TaskStatus};
pub use webhook::{GitHubWebhookPayload, GrafanaAlert, WebhookPayload};

```

### common/src/models/pm_task.rs (447 lines)

**Key Definitions:**
```rust
7:pub struct PmTaskRequest {
70:pub struct Subtask {
83:pub struct MarkdownPayload {
91:pub struct AgentToolSpec {
102:pub struct RepositorySpec {
133:pub struct DocsGenerationRequest {
173:pub struct TaskMasterFile {
178:pub struct TaskMaster {
183:pub struct Task {
196:impl PmTaskRequest {
199:pub fn new(
218:pub fn new_with_tools(
254:pub fn new_with_repository(
291:pub fn new_with_full_spec(
329:pub fn new_with_prompt_modification(
369:pub fn new_with_tool_config(
```

**Full Content:**
```rust
//! PM task submission models

use serde::{Deserialize, Serialize};

/// PM task request structure according to design document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskRequest {
    // Task Master schema fields
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,

    // PM-specific fields
    pub service_name: String,
    pub agent_name: String,

    // Claude model selection (sonnet, opus)
    pub model: String,

    // Markdown files as structured payloads
    pub markdown_files: Vec<MarkdownPayload>,

    // Agent tools specification
    #[serde(default)]
    pub agent_tools: Vec<AgentToolSpec>,

    // Repository specification for code access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,

    // Working directory within target repository (defaults to service_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    // Additional prompt instructions for retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_modification: Option<String>,

    // How to apply prompt_modification: 'append' or 'replace'
    #[serde(
        default = "default_prompt_mode",
        skip_serializing_if = "is_default_prompt_mode"
    )]
    pub prompt_mode: String,

    // Local Claude Code tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_tools: Vec<String>,

    // Remote MCP tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_tools: Vec<String>,

    // Tool configuration preset
    #[serde(
        default = "default_tool_config",
        skip_serializing_if = "is_default_tool_config"
    )]
    pub tool_config: String,
}

/// Subtask structure from Task Master
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<u32>,
    pub details: String,
    pub status: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
}

/// Markdown file payload for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownPayload {
    pub content: String,
    pub filename: String,
    pub file_type: String,
}

/// Agent tool specification for PM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolSpec {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

/// Repository specification for cloning source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySpec {
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub github_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>, // Reserved for future use - TODO: Implement direct token submission
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_prompt_mode() -> String {
    "append".to_string()
}

fn is_default_prompt_mode(mode: &str) -> bool {
    mode == "append"
}

fn default_tool_config() -> String {
    "default".to_string()
}

fn is_default_tool_config(config: &str) -> bool {
    config == "default"
}

/// Documentation generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsGenerationRequest {
    /// Repository URL to clone
    pub repository_url: String,

    /// Working directory within the repository (path to .taskmaster)
    pub working_directory: String,

    /// Source branch to checkout and base new branch from
    pub source_branch: String,

    /// Target branch for the PR
    pub target_branch: String,

    /// Service name for the job
    pub service_name: String,

    /// Agent name for the job
    pub agent_name: String,

    /// Claude model selection (sonnet, opus)
    pub model: String,

    /// GitHub user for authentication
    pub github_user: String,

    /// Optional specific task ID to generate docs for (if None, generates all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,

    /// Force overwrite existing documentation
    #[serde(default)]
    pub force: bool,

    /// Dry run mode (preview only)
    #[serde(default)]
    pub dry_run: bool,
}

/// Task Master JSON file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMasterFile {
    pub master: TaskMaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMaster {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,
}

impl PmTaskRequest {
    /// Create a new PM task request from Task Master task and markdown files
    #[must_use]
    pub fn new(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
    ) -> Self {
        Self::new_with_tools(
            task,
            service_name,
            agent_name,
            model,
            markdown_files,
            Vec::new(),
        )
    }

    /// Create a new PM task request with agent tools specification
    #[must_use]
    pub fn new_with_tools(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository: None,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request from Task Master task with repository support
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_repository(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full specification including working directory
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_full_spec(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with prompt modification support for retries
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_prompt_modification(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full tool configuration support
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_tool_config(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
        local_tools: Vec<String>,
        remote_tools: Vec<String>,
        tool_config: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools,
            remote_tools,
            tool_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_task_request_creation() {
        let task = Task {
            id: 1001,
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            details: "Test details".to_string(),
            test_strategy: "Test strategy".to_string(),
            priority: "high".to_string(),
            dependencies: vec![],
            status: "pending".to_string(),
            subtasks: vec![],
        };

        let markdown_files = vec![MarkdownPayload {
            content: "# Task Content".to_string(),
            filename: "task.md".to_string(),
            file_type: "task".to_string(),
        }];

        let request = PmTaskRequest::new(
            task,
            "test-service".to_string(),
            "claude-agent-1".to_string(),
            "sonnet".to_string(),
            markdown_files,
        );

        assert_eq!(request.id, 1001);
        assert_eq!(request.service_name, "test-service");
        assert_eq!(request.model, "sonnet");
        assert_eq!(request.markdown_files.len(), 1);
    }
}

```

