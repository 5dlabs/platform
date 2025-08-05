use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;

// Global agents configuration loaded once at startup
static AGENTS_CONFIG: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Load agent configuration from environment variables
/// Looks for AGENT_* environment variables (e.g., AGENT_MORGAN=5DLabs-Morgan)
fn load_agents_from_env() -> Result<HashMap<String, String>> {
    let mut agents = HashMap::new();
    
    for (key, value) in std::env::vars() {
        if let Some(agent_name) = key.strip_prefix("AGENT_") {
            let agent_name = agent_name.to_lowercase();
            agents.insert(agent_name, value);
        }
    }
    
    if agents.is_empty() {
        return Err(anyhow!("No AGENT_* environment variables found. Required format: AGENT_MORGAN=5DLabs-Morgan"));
    }
    
    Ok(agents)
}

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
            Some(Ok(tools::get_tool_schemas()))
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

/// Get the remote URL for the current git repository
fn get_git_remote_url() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {}", stderr))
    }
}

/// Get the current git branch
fn get_git_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to execute git command")?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?.trim().to_string();
        if branch.is_empty() {
            Ok("main".to_string()) // fallback to main if no branch (detached HEAD)
        } else {
            Ok(branch)
        }
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {}", stderr))
    }
}

/// Validate repository URL format
fn validate_repository_url(repo_url: &str) -> Result<()> {
    if !repo_url.starts_with("https://github.com/") {
        return Err(anyhow!(
            "Repository URL must be a GitHub HTTPS URL (e.g., 'https://github.com/org/repo')"
        ));
    }
    
    // Basic validation - should have org/repo structure
    let path = repo_url.trim_start_matches("https://github.com/");
    let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(anyhow!(
            "Repository URL must be in format 'https://github.com/org/repo'"
        ));
    }
    
    Ok(())
}

fn handle_docs_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: working_directory"))?;
    
    let agents_config = AGENTS_CONFIG.get().unwrap();
    
    // Auto-detect repository URL (fail if not available)
    let repository_url = get_git_remote_url()
        .context("Failed to auto-detect repository URL. Ensure you're in a git repository with origin remote.")?;
    validate_repository_url(&repository_url)?;
    
    // Auto-detect source branch (fail if not available)
    let source_branch = get_git_current_branch()
        .context("Failed to auto-detect git branch. Ensure you're in a git repository.")?;
    
    // Handle agent name resolution with validation
    let agent_name = arguments.get("agent").and_then(|v| v.as_str());
    let github_app = if let Some(agent) = agent_name {
        // Validate agent name exists in config
        if !agents_config.contains_key(agent) {
            let available_agents: Vec<&String> = agents_config.keys().collect();
            return Err(anyhow!(
                "Unknown agent '{}'. Available agents: {:?}", 
                agent, available_agents
            ));
        }
        agents_config[agent].clone()
    } else {
        return Err(anyhow!("No agent specified. Please provide an 'agent' parameter (e.g., 'morgan', 'rex', 'blaze', 'cipher')"));
    };
    
    // Handle model (use Helm defaults if not provided)  
    let model = arguments.get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No model specified. Please provide a 'model' parameter (e.g., 'claude-opus-4-20250514')"))?;
    
    // Validate model name
    if !model.starts_with("claude-") {
        return Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name", model));
    }
    
    // Handle include_codebase parameter
    let include_codebase = arguments.get("include_codebase")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    let mut params = vec![
        format!("working-directory={working_directory}"),
        format!("repository-url={repository_url}"),
        format!("source-branch={source_branch}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
    ];
    
    // Add include_codebase parameter if enabled
    if include_codebase {
        params.push("include-codebase=true".to_string());
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
            "repository_url": repository_url,
            "source_branch": source_branch,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
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
        
    let docs_project_directory = arguments
        .get("docs_project_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: docs_project_directory"))?;
    
    // Validate repository URL
    validate_repository_url(repository)?;
    
    // Validate service name (must be valid for PVC naming)
    if !service.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(anyhow!("Invalid service name '{}'. Must contain only lowercase letters, numbers, and hyphens", service));
    }
    
    let agents_config = AGENTS_CONFIG.get().unwrap();
    
    // Handle docs repository (require explicit value or rely on Helm defaults)
    let docs_repository = arguments.get("docs_repository")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No docs_repository specified. Please provide a 'docs_repository' parameter"))?;
    
    validate_repository_url(docs_repository)?;
    
    // Handle working directory (require explicit value or rely on Helm defaults)
    let working_directory = arguments.get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No working_directory specified. Please provide a 'working_directory' parameter"))?;
        
    // Handle agent name resolution with validation
    let agent_name = arguments.get("agent").and_then(|v| v.as_str());
    let github_app = if let Some(agent) = agent_name {
        // Validate agent name exists in config
        if !agents_config.contains_key(agent) {
            let available_agents: Vec<&String> = agents_config.keys().collect();
            return Err(anyhow!(
                "Unknown agent '{}'. Available agents: {:?}", 
                agent, available_agents
            ));
        }
        agents_config[agent].clone()
    } else {
        return Err(anyhow!("No agent specified. Please provide an 'agent' parameter (e.g., 'rex', 'blaze', 'cipher')"));
    };
    
    // Handle model (require explicit value or rely on Helm defaults)
    let model = arguments.get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No model specified. Please provide a 'model' parameter (e.g., 'claude-3-5-sonnet-20241022')"))?;
    
    if !model.starts_with("claude-") {
        return Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name", model));
    }
    
    // Auto-detect docs branch (fail if not available)
    let docs_branch = get_git_current_branch()
        .context("Failed to auto-detect git branch. Ensure you're in a git repository.")?;
    
    // Handle continue session (require explicit value or rely on Helm defaults)
    let continue_session = arguments.get("continue_session")
        .and_then(|v| v.as_bool())
        .ok_or(anyhow!("No continue_session specified. Please provide a 'continue_session' parameter (true/false)"))?;
    
    // Handle overwrite memory (require explicit value or rely on Helm defaults)
    let overwrite_memory = arguments.get("overwrite_memory")
        .and_then(|v| v.as_bool())
        .ok_or(anyhow!("No overwrite_memory specified. Please provide an 'overwrite_memory' parameter (true/false)"))?;
    
    let mut params = vec![
        format!("task-id={task_id}"),
        format!("service-id={service}"),
        format!("repository-url={repository}"),
        format!("docs-repository-url={docs_repository}"),
        format!("docs-project-directory={docs_project_directory}"),
        format!("working-directory={working_directory}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
        format!("continue-session={continue_session}"),
        format!("overwrite-memory={overwrite_memory}"),
        format!("docs-branch={docs_branch}"),
        format!("context-version=0"), // Auto-assign by controller
    ];
    
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
            "working_directory": working_directory,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
            "continue_session": continue_session,
            "overwrite_memory": overwrite_memory,
            "docs_branch": docs_branch,
            "context_version": 0,
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
                Ok("export") => Some(handle_export_workflow().map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": result
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

/// Handle export workflow - convert current directory's Rust code to markdown
fn handle_export_workflow() -> Result<String> {
    // Use WORKSPACE_FOLDER_PATHS to get the actual workspace directory
    let project_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            // WORKSPACE_FOLDER_PATHS might contain multiple paths separated by some delimiter
            // For now, take the first one (or the only one)
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            first_path.to_string()
        })
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    eprintln!("🔍 Using workspace directory: {}", project_dir.display());
    
    // Create .taskmaster/docs directory if it doesn't exist
    let taskmaster_dir = project_dir.join(".taskmaster");
    let docs_dir = taskmaster_dir.join("docs");
    
    eprintln!("📁 Creating directory: {}", docs_dir.display());
    eprintln!("📁 Project dir exists: {}", project_dir.exists());
    eprintln!("📁 Project dir is_dir: {}", project_dir.is_dir());
    
    std::fs::create_dir_all(&docs_dir)
        .with_context(|| format!("Failed to create .taskmaster/docs directory at: {}", docs_dir.display()))?;
    
    let output_file = docs_dir.join("codebase.md");
    
    // Generate markdown content
    let markdown_content = generate_codebase_markdown(&project_dir)
        .context("Failed to generate codebase markdown")?;
    
    // Write to file
    std::fs::write(&output_file, &markdown_content)
        .context("Failed to write codebase.md")?;
    
    Ok(format!("✅ Exported codebase to: {}", output_file.display()))
}

/// Generate markdown representation of Rust codebase
fn generate_codebase_markdown(project_dir: &std::path::Path) -> Result<String> {
    let mut markdown = String::new();
    
    // Add header
    let project_name = project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project");
    
    markdown.push_str(&format!("# Project: {}\n\n", project_name));
    
    // Read Cargo.toml if it exists
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(cargo_content) = std::fs::read_to_string(&cargo_toml_path) {
            markdown.push_str("## Cargo.toml\n\n```toml\n");
            markdown.push_str(&cargo_content);
            markdown.push_str("\n```\n\n");
        }
    }
    
    // Find and process all relevant source files
    markdown.push_str("## Source Files\n\n");
    
    process_source_files(&mut markdown, project_dir, project_dir)?;
    
    Ok(markdown)
}

/// Recursively process source files
fn process_source_files(
    markdown: &mut String, 
    current_dir: &std::path::Path,
    project_root: &std::path::Path
) -> Result<()> {
    let entries = std::fs::read_dir(current_dir)
        .context("Failed to read directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip target directory and hidden directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "target" || name.starts_with('.') {
                continue;
            }
        }
        
        if path.is_dir() {
            process_source_files(markdown, &path, project_root)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            // Include multiple file types beyond just .rs
            let (language, should_include) = match ext {
                "rs" => ("rust", true),
                "py" => ("python", true),
                "sql" => ("sql", true),
                "toml" => ("toml", true),
                "yml" | "yaml" => ("yaml", true),
                "json" => ("json", true),
                "md" => ("markdown", true),
                "txt" => ("text", true),
                "sh" => ("bash", true),
                "dockerfile" => ("dockerfile", true),
                _ => ("text", false)
            };
            
            // Also include files without extensions but with specific names
            let should_include = should_include || matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("Dockerfile") | Some("README") | Some("LICENSE")
            );
            
            if should_include {
                // Get relative path from project root
                let relative_path = path.strip_prefix(project_root)
                    .context("Failed to get relative path")?;
                
                markdown.push_str(&format!("### {}\n\n", relative_path.display()));
                
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        markdown.push_str(&format!("```{}\n", language));
                        markdown.push_str(&content);
                        markdown.push_str("\n```\n\n");
                    }
                    Err(e) => {
                        markdown.push_str(&format!("*Error reading file: {}*\n\n", e));
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[allow(clippy::disallowed_macros)]
fn main() -> Result<()> {
    eprintln!("🚀 Starting 5D Labs MCP Server...");
    
    // Initialize agents configuration from environment variables
    let agents_config = load_agents_from_env()
        .context("Failed to load agents configuration")?;
    eprintln!("📋 Loaded {} agents from environment: {:?}", 
              agents_config.len(), 
              agents_config.keys().collect::<Vec<_>>());
    
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