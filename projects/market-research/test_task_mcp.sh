#!/bin/bash

# Test script for MCP server task functionality on market-research project
# Run from repo root to test task 1 execution

echo "Testing MCP server task functionality for market-research project - Task 1..."

# Path to fdl-mcp binary
FDL_MCP="./controller/target/release/fdl-mcp"

if [[ ! -f "$FDL_MCP" ]]; then
    echo "Error: fdl-mcp binary not found at $FDL_MCP"
    exit 1
fi

# Test task execution for task 1 (Research AI Development Tools Landscape)
cat << 'EOF' | "$FDL_MCP"
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}}, "clientInfo": {"name": "test", "version": "1.0.0"}}}
{"jsonrpc": "2.0", "method": "notifications/initialized"}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "task", "arguments": {"task_id": 1, "service": "market-research", "repository": "5dlabs/platform", "docs_repository": "5dlabs/platform", "docs_project_directory": "projects/market-research", "docs_branch": "argo", "github_user": "pm0-5dlabs", "working_directory": ".", "model": "claude-3-5-sonnet-20241022", "continue_session": false}}}
EOF

echo "MCP server task test completed for market-research project - Task 1"