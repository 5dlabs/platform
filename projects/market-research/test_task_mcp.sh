#!/bin/bash

# Test script for MCP server task functionality on market-research project
# Run from repo root to test task 1 execution

echo "Testing MCP server task functionality for market-research project - Task 1..."

# Path to cto-mcp binary
CTO_MCP="./controller/target/release/cto-mcp"

if [[ ! -f "$CTO_MCP" ]]; then
    echo "Error: cto-mcp binary not found at $CTO_MCP"
    exit 1
fi

# Test task execution for task 1 (Research AI Development Tools Landscape)
cat << 'EOF' | "$CTO_MCP"
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}}, "clientInfo": {"name": "test", "version": "1.0.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "task", "arguments": {"task_id": 1, "service": "market-research", "repository": "5dlabs/cto", "docs_repository": "5dlabs/cto", "docs_project_directory": "projects/market-research", "docs_branch": "argo", "github_user": "pm0-5dlabs", "working_directory": ".", "model": "claude-3-5-sonnet-20241022", "continue_session": false}}}
EOF

echo "MCP server task test completed for market-research project - Task 1"