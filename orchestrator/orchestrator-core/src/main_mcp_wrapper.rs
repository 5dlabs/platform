//! MCP Wrapper - Lightweight HTTP forwarder for MCP protocol
//!
//! This binary acts as a bridge between agents that expect stdin/stdout MCP
//! and the toolman HTTP server. It reads MCP messages from stdin, forwards
//! them to the toolman server via HTTP, and writes responses back to stdout.

use anyhow::{Context, Result};
use serde_json::Value;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufWriter};
use tracing::{debug, error, info};

/// Configuration for the MCP wrapper
#[derive(Debug)]
struct WrapperConfig {
    /// URL of the toolman server
    toolman_url: String,
    /// Timeout for HTTP requests (seconds)
    timeout_seconds: u64,
    /// Enable debug logging
    debug: bool,
}

impl WrapperConfig {
    fn from_env() -> Self {
        Self {
            toolman_url: env::var("MCP_TOOLMAN_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:3000/mcp".to_string()),
            timeout_seconds: env::var("MCP_WRAPPER_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            debug: env::var("MCP_WRAPPER_DEBUG")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}

/// MCP Wrapper that forwards messages between stdin/stdout and HTTP
struct McpWrapper {
    config: WrapperConfig,
    client: reqwest::Client,
}

impl McpWrapper {
    fn new(config: WrapperConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Run the wrapper main loop
    async fn run(&self) -> Result<()> {
        info!("Starting MCP wrapper, forwarding to: {}", self.config.toolman_url);

        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);
        let stdout = tokio::io::stdout();
        let mut writer = BufWriter::new(stdout);

        let mut line = String::new();
        loop {
            line.clear();
            
            // Read line from stdin
            let bytes_read = reader.read_line(&mut line).await
                .context("Failed to read from stdin")?;
                
            if bytes_read == 0 {
                // EOF reached
                debug!("EOF reached, shutting down wrapper");
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if self.config.debug {
                debug!("Received MCP message: {}", line);
            }

            // Parse JSON to validate it's a proper MCP message
            let message: Value = match serde_json::from_str(line) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse JSON message: {} - Raw: {}", e, line);
                    continue;
                }
            };

            // Forward to toolman server
            match self.forward_message(message).await {
                Ok(response) => {
                    // Write response to stdout
                    let response_str = serde_json::to_string(&response)
                        .context("Failed to serialize response")?;
                    
                    if self.config.debug {
                        debug!("Sending response: {}", response_str);
                    }
                    
                    writer.write_all(response_str.as_bytes()).await
                        .context("Failed to write to stdout")?;
                    writer.write_all(b"\n").await
                        .context("Failed to write newline")?;
                    writer.flush().await
                        .context("Failed to flush stdout")?;
                }
                Err(e) => {
                    error!("Failed to forward message: {}", e);
                    
                    // Send error response back to agent
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": message.get("id"),
                        "error": {
                            "code": -32603,
                            "message": format!("Toolman communication error: {}", e)
                        }
                    });
                    
                    let error_str = serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#.to_string());
                    
                    writer.write_all(error_str.as_bytes()).await.ok();
                    writer.write_all(b"\n").await.ok();
                    writer.flush().await.ok();
                }
            }
        }

        info!("MCP wrapper shutting down");
        Ok(())
    }

    /// Forward a message to the toolman server via HTTP
    async fn forward_message(&self, message: Value) -> Result<Value> {
        debug!("Forwarding message to toolman: {}", self.config.toolman_url);

        let response = self.client
            .post(&self.config.toolman_url)
            .json(&message)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", self.config.toolman_url))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("HTTP {} from toolman: {}", status, body));
        }

        let response_json: Value = response.json().await
            .context("Failed to parse response JSON from toolman")?;

        debug!("Received response from toolman");
        Ok(response_json)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    info!("Starting MCP wrapper");

    // Load configuration
    let config = WrapperConfig::from_env();
    
    if config.debug {
        debug!("Configuration: {:?}", config);
    }

    // Create and run wrapper
    let wrapper = McpWrapper::new(config);
    wrapper.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_from_env() {
        std::env::set_var("MCP_TOOLMAN_SERVER_URL", "http://test:8080/mcp");
        std::env::set_var("MCP_WRAPPER_TIMEOUT", "60");
        std::env::set_var("MCP_WRAPPER_DEBUG", "true");

        let config = WrapperConfig::from_env();
        assert_eq!(config.toolman_url, "http://test:8080/mcp");
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.debug);

        // Cleanup
        std::env::remove_var("MCP_TOOLMAN_SERVER_URL");
        std::env::remove_var("MCP_WRAPPER_TIMEOUT");
        std::env::remove_var("MCP_WRAPPER_DEBUG");
    }

    #[test]
    fn test_config_defaults() {
        // Ensure clean environment
        std::env::remove_var("MCP_TOOLMAN_SERVER_URL");
        std::env::remove_var("MCP_WRAPPER_TIMEOUT");
        std::env::remove_var("MCP_WRAPPER_DEBUG");

        let config = WrapperConfig::from_env();
        assert_eq!(config.toolman_url, "http://localhost:3000/mcp");
        assert_eq!(config.timeout_seconds, 30);
        assert!(!config.debug);
    }

    #[tokio::test]
    async fn test_wrapper_creation() {
        let config = WrapperConfig {
            toolman_url: "http://localhost:3000/mcp".to_string(),
            timeout_seconds: 30,
            debug: false,
        };

        let wrapper = McpWrapper::new(config);
        assert_eq!(wrapper.config.toolman_url, "http://localhost:3000/mcp");
    }
}