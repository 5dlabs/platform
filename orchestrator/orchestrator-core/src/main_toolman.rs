//! Toolman MCP Server
//!
//! A tool management proxy server that filters and controls access to MCP tools
//! for agents. This server acts as a middleware layer between agents and actual
//! MCP tool servers, providing fine-grained control over tool availability.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Configuration for the toolman server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolmanConfig {
    /// Map of server names to their configurations
    pub servers: HashMap<String, ServerConfig>,
    /// Default tool access policy
    pub default_access: AccessPolicy,
    /// Agent-specific tool access policies
    pub agent_policies: HashMap<String, AccessPolicy>,
}

/// Configuration for a backend MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to run the server
    pub command: String,
    /// Arguments to pass to the server
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Whether this server is enabled
    pub enabled: bool,
}

/// Tool access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// Set of allowed tools (if empty, all tools are allowed)
    pub allowed_tools: HashSet<String>,
    /// Set of blocked tools
    pub blocked_tools: HashSet<String>,
    /// Whether to allow unknown tools
    pub allow_unknown: bool,
}

/// MCP message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpMessage {
    #[serde(rename = "initialize")]
    Initialize { 
        id: Value, 
        params: Value 
    },
    #[serde(rename = "tools/list")]
    ListTools { 
        id: Value, 
        params: Option<Value> 
    },
    #[serde(rename = "tools/call")]
    CallTool { 
        id: Value, 
        params: Value 
    },
    #[serde(rename = "resources/list")]
    ListResources { 
        id: Value, 
        params: Option<Value> 
    },
    #[serde(rename = "resources/read")]
    ReadResource { 
        id: Value, 
        params: Value 
    },
    #[serde(rename = "prompts/list")]
    ListPrompts { 
        id: Value, 
        params: Option<Value> 
    },
    #[serde(rename = "prompts/get")]
    GetPrompt { 
        id: Value, 
        params: Value 
    },
    #[serde(other)]
    Other,
}

/// Toolman server state
#[derive(Debug)]
pub struct ToolmanServer {
    config: Arc<RwLock<ToolmanConfig>>,
    available_tools: Arc<RwLock<HashMap<String, Value>>>,
    server_processes: Arc<RwLock<HashMap<String, std::process::Child>>>,
}

impl ToolmanServer {
    /// Create a new toolman server
    pub fn new(config: ToolmanConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            available_tools: Arc::new(RwLock::new(HashMap::new())),
            server_processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the server and connect to backend MCP servers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing toolman server");
        
        let config = self.config.read().await;
        
        // Start all enabled backend servers
        for (name, server_config) in &config.servers {
            if server_config.enabled {
                self.start_backend_server(name, server_config).await?;
            }
        }
        
        // Discover available tools from all backend servers
        self.discover_tools().await?;
        
        Ok(())
    }

    /// Start a backend MCP server
    async fn start_backend_server(&self, name: &str, config: &ServerConfig) -> Result<()> {
        info!("Starting backend server: {}", name);
        
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        
        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let child = cmd.spawn()
            .with_context(|| format!("Failed to start backend server: {}", name))?;
        
        let mut processes = self.server_processes.write().await;
        processes.insert(name.to_string(), child);
        
        Ok(())
    }

    /// Discover tools from all backend servers
    async fn discover_tools(&self) -> Result<()> {
        info!("Discovering tools from backend servers");
        
        let mut all_tools = HashMap::new();
        
        // For each backend server, send a tools/list request
        let processes = self.server_processes.read().await;
        for (server_name, _) in processes.iter() {
            match self.list_tools_from_server(server_name).await {
                Ok(tools) => {
                    for tool in tools {
                        let tool_name = tool.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        
                        all_tools.insert(
                            format!("{}:{}", server_name, tool_name),
                            tool
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to list tools from server {}: {}", server_name, e);
                }
            }
        }
        
        let mut available_tools = self.available_tools.write().await;
        *available_tools = all_tools;
        
        info!("Discovered {} tools", available_tools.len());
        Ok(())
    }

    /// List tools from a specific backend server
    async fn list_tools_from_server(&self, server_name: &str) -> Result<Vec<Value>> {
        // This is a simplified implementation
        // In a real implementation, you'd communicate with the backend server
        // via stdin/stdout using the MCP protocol
        
        debug!("Listing tools from server: {}", server_name);
        
        // For now, return some mock tools
        Ok(vec![
            json!({
                "name": "echo",
                "description": "Echo a message",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"}
                    }
                }
            }),
            json!({
                "name": "get_weather",
                "description": "Get weather information",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }
            })
        ])
    }

    /// Check if a tool is allowed for the current agent
    fn is_tool_allowed(&self, tool_name: &str, agent_id: Option<&str>) -> bool {
        // This would check against the access policies
        // For now, implement a simple allow-all policy
        true
    }

    /// Handle an MCP message
    pub async fn handle_message(&self, message: Value) -> Result<Value> {
        debug!("Handling message: {:?}", message);
        
        // Parse the message
        let mcp_message: McpMessage = serde_json::from_value(message.clone())
            .unwrap_or(McpMessage::Other);
        
        match mcp_message {
            McpMessage::Initialize { id, params } => {
                self.handle_initialize(id, params).await
            }
            McpMessage::ListTools { id, params } => {
                self.handle_list_tools(id, params).await
            }
            McpMessage::CallTool { id, params } => {
                self.handle_call_tool(id, params).await
            }
            McpMessage::ListResources { id, params } => {
                self.handle_list_resources(id, params).await
            }
            McpMessage::ReadResource { id, params } => {
                self.handle_read_resource(id, params).await
            }
            McpMessage::ListPrompts { id, params } => {
                self.handle_list_prompts(id, params).await
            }
            McpMessage::GetPrompt { id, params } => {
                self.handle_get_prompt(id, params).await
            }
            McpMessage::Other => {
                // Pass through unknown messages
                Ok(json!({
                    "jsonrpc": "2.0",
                    "id": message.get("id"),
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                }))
            }
        }
    }

    /// Handle initialize message
    async fn handle_initialize(&self, id: Value, _params: Value) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "toolman",
                    "version": "0.1.0"
                }
            }
        }))
    }

    /// Handle list tools message
    async fn handle_list_tools(&self, id: Value, _params: Option<Value>) -> Result<Value> {
        let available_tools = self.available_tools.read().await;
        let tools: Vec<Value> = available_tools.values().cloned().collect();
        
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        }))
    }

    /// Handle call tool message
    async fn handle_call_tool(&self, id: Value, params: Value) -> Result<Value> {
        let tool_name = params.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if !self.is_tool_allowed(tool_name, None) {
            return Ok(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32603,
                    "message": format!("Tool '{}' is not allowed", tool_name)
                }
            }));
        }
        
        // For now, return a mock response
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": format!("Tool '{}' executed successfully", tool_name)
                    }
                ]
            }
        }))
    }

    /// Handle list resources message
    async fn handle_list_resources(&self, id: Value, _params: Option<Value>) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resources": []
            }
        }))
    }

    /// Handle read resource message
    async fn handle_read_resource(&self, id: Value, _params: Value) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "contents": []
            }
        }))
    }

    /// Handle list prompts message
    async fn handle_list_prompts(&self, id: Value, _params: Option<Value>) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "prompts": []
            }
        }))
    }

    /// Handle get prompt message
    async fn handle_get_prompt(&self, id: Value, _params: Value) -> Result<Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "messages": []
            }
        }))
    }

    /// Run the HTTP server
    pub async fn run(&self) -> Result<()> {
        use axum::{
            extract::State,
            http::StatusCode,
            response::Json,
            routing::post,
            Router,
        };
        
        info!("Starting toolman HTTP server");
        
        let server = Arc::clone(&self);
        
        // Create HTTP handler for MCP messages
        let handle_mcp = |State(server): State<Arc<ToolmanServer>>, Json(message): Json<Value>| async move {
            match server.handle_message(message).await {
                Ok(response) => Ok(Json(response)),
                Err(e) => {
                    error!("Error handling MCP message: {}", e);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", e)))
                }
            }
        };
        
        // Build the router
        let app = Router::new()
            .route("/mcp", post(handle_mcp))
            .route("/health", axum::routing::get(|| async { "OK" }))
            .with_state(server);
        
        // Get port from environment or use default
        let port = std::env::var("TOOLMAN_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);
            
        let addr = format!("0.0.0.0:{}", port);
        info!("Toolman server listening on {}", addr);
        
        // Start the HTTP server
        let listener = tokio::net::TcpListener::bind(&addr).await
            .with_context(|| format!("Failed to bind to {}", addr))?;
            
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .with_context(|| "HTTP server error")?;
        
        Ok(())
    }
    
    /// Graceful shutdown signal handler
    async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        info!("Shutdown signal received, starting graceful shutdown");
    }
}

/// Load configuration from environment or file
fn load_config() -> Result<ToolmanConfig> {
    let config_path = env::var("TOOLMAN_CONFIG_PATH")
        .unwrap_or_else(|_| "toolman.json".to_string());
    
    if std::path::Path::new(&config_path).exists() {
        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path))?;
        
        serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", config_path))
    } else {
        // Return default configuration
        Ok(ToolmanConfig {
            servers: HashMap::new(),
            default_access: AccessPolicy {
                allowed_tools: HashSet::new(),
                blocked_tools: HashSet::new(),
                allow_unknown: true,
            },
            agent_policies: HashMap::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting toolman server");
    
    // Load configuration
    let config = load_config()?;
    
    // Create and initialize the server
    let server = ToolmanServer::new(config);
    server.initialize().await?;
    
    // Run the server
    server.run().await?;
    
    info!("Toolman server shutting down");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toolman_server_creation() {
        let config = ToolmanConfig {
            servers: HashMap::new(),
            default_access: AccessPolicy {
                allowed_tools: HashSet::new(),
                blocked_tools: HashSet::new(),
                allow_unknown: true,
            },
            agent_policies: HashMap::new(),
        };
        
        let server = ToolmanServer::new(config);
        assert!(server.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let config = ToolmanConfig {
            servers: HashMap::new(),
            default_access: AccessPolicy {
                allowed_tools: HashSet::new(),
                blocked_tools: HashSet::new(),
                allow_unknown: true,
            },
            agent_policies: HashMap::new(),
        };
        
        let server = ToolmanServer::new(config);
        let response = server.handle_initialize(json!(1), json!({})).await.unwrap();
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"]["serverInfo"]["name"] == "toolman");
    }
}