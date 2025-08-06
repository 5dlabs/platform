#!/bin/bash

# Quick MCP process check
COUNT=$(ps aux | grep cto-mcp | grep -v grep | grep -v check | wc -l)
echo "$(date '+%H:%M:%S'): $COUNT MCP processes running"

if [ $COUNT -gt 1 ]; then
    echo "⚠️  WARNING: Multiple MCP processes detected!"
    ps aux | grep cto-mcp | grep -v grep | grep -v check
elif [ $COUNT -eq 1 ]; then
    echo "✅ Single MCP process running (normal)"
else
    echo "🔴 No MCP processes running"
fi