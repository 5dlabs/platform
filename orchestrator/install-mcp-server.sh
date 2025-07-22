#!/bin/bash

# Build and install MCP server and CLI to system location
echo "Building orchestrator binaries..."
cargo build --release --bin orchestrator-mcp --bin orchestrator-cli

echo "Installing to /usr/local/bin..."
sudo cp target/release/orchestrator-mcp /usr/local/bin/orchestrator-mcp
sudo cp target/release/orchestrator-cli /usr/local/bin/orchestrator-cli

echo "✅ Orchestrator binaries installed successfully!"
echo "📍 MCP server: /usr/local/bin/orchestrator-mcp"
echo "📍 CLI: /usr/local/bin/orchestrator-cli"
echo "🔄 Please restart MCP server in Cursor to use the updated versions."