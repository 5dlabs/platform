#!/bin/bash

echo "=== MCP Process Monitor ==="
echo "Press Ctrl+C to stop monitoring"
echo

while true; do
    clear
    echo "$(date): MCP Process Status"
    echo "=================================="
    
    # Show running MCP processes with details
    MCP_PROCESSES=$(ps aux | grep cto-mcp | grep -v grep | grep -v monitor)
    if [ -n "$MCP_PROCESSES" ]; then
        echo "RUNNING MCP PROCESSES:"
        echo "PID    STATUS  %CPU %MEM  START_TIME  COMMAND"
        echo "$MCP_PROCESSES" | awk '{printf "%-6s %-7s %-4s %-4s  %-10s  %s\n", $2, $8, $3, $4, $9, $11}'
        echo
        
        # Show file descriptors for each process
        echo "FILE DESCRIPTORS:"
        echo "$MCP_PROCESSES" | while read line; do
            PID=$(echo $line | awk '{print $2}')
            echo "PID $PID:"
            sudo lsof -p $PID 2>/dev/null | head -5 || echo "  (no file descriptors or access denied)"
            echo
        done
    else
        echo "No MCP processes currently running"
    fi
    
    echo "=================================="
    echo "Press Ctrl+C to stop monitoring"
    sleep 3
done