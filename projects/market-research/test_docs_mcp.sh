#!/bin/bash

# Test script for MCP server docs functionality on market-research project
# Run from market-research directory

echo "Testing MCP server docs functionality for market-research project..."

# Path to cto-mcp binary
CTO_MCP="./controller/target/release/cto-mcp"

if [[ ! -f "$CTO_MCP" ]]; then
    echo "Error: cto-mcp binary not found at $CTO_MCP"
    exit 1
fi

# Create a test script that sends JSON-RPC messages for docs generation
cat << 'EOF' | "$CTO_MCP"
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}}, "clientInfo": {"name": "test", "version": "1.0.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "docs", "arguments": {"working_directory": "projects/market-research", "model": "claude-3-5-sonnet-20241022"}}}
EOF

echo "MCP server docs test completed for market-research project"