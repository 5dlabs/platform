# MCP Proxy Server Design

## Overview

The MCP (Model Context Protocol) proxy server is an integrated component of the orchestrator platform that manages tool availability for AI agents. It acts as an intelligent gateway between agents and MCP servers, providing selective tool exposure based on task requirements.

## Goals

1. **Prevent Tool Overwhelm**: Expose only necessary tools for each task
2. **Dynamic Management**: Enable/disable tools during task execution
3. **Integration**: Seamless integration with TaskRun workflow
4. **Flexibility**: Support both internal Claude tools and external MCP servers
5. **Performance**: Efficient context management for better agent performance

## Architecture

### Components

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Claude Agent  │────▶│   MCP Proxy      │────▶│  MCP Servers    │
│  (in Job Pod)   │     │   (Sidecar)      │     │  (External)     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
         │                       │                          │
         │                       ▼                          │
         │              ┌──────────────────┐               │
         │              │ Tool Registry    │               │
         │              │ & Filter Engine  │               │
         │              └──────────────────┘               │
         │                       │                          │
         └───────────────────────┴──────────────────────────┘
                          TaskRun Config
```

### Integration with TaskRun

The TaskRun CRD will be extended with tool configuration:

```yaml
apiVersion: orchestrator.io/v1
kind: TaskRun
metadata:
  name: task-1001
spec:
  taskId: 1001
  serviceName: todo-api
  toolConfig:
    # Internal Claude tools configuration
    internalTools:
      permissions:
        allow: ["*"]  # Default: all tools
        deny: ["WebFetch"]  # Optionally restrict
    
    # External MCP servers
    mcpServers:
      - name: github
        enabled: true
        tools:
          # Expose only specific tools
          allow: ["create_issue", "list_issues", "create_pr"]
          
      - name: filesystem
        enabled: true
        tools:
          allow: ["read_file", "write_file", "list_files"]
          deny: ["delete_file"]  # Explicitly deny dangerous operations
          
      - name: task-master
        enabled: true
        tools:
          allow: ["*"]  # All task management tools
```

### Deployment Models

#### Option 1: Sidecar Container (Recommended)
- Deploy MCP proxy as a sidecar in the agent Job pod
- Direct localhost communication
- Isolated per task
- No network latency

```yaml
containers:
  - name: claude-agent
    image: claude-code:latest
    env:
      - name: MCP_PROXY_URL
        value: "http://localhost:8080"
  
  - name: mcp-proxy
    image: orchestrator-mcp-proxy:latest
    env:
      - name: TOOL_CONFIG
        valueFrom:
          configMapKeyRef:
            name: task-1001-config
            key: tools.yaml
```

#### Option 2: Shared Service
- Central MCP proxy deployment
- Multi-tenant with context isolation
- Shared resource efficiency
- Network overhead

### Tool Registry

Built-in catalog of MCP servers and their tools:

```yaml
servers:
  github:
    description: "GitHub API operations"
    command: ["npx", "-y", "@modelcontextprotocol/server-github"]
    tools:
      - name: create_issue
        description: "Create a new GitHub issue"
        category: "issues"
        
      - name: create_pr
        description: "Create a pull request"
        category: "pull-requests"
        
  filesystem:
    description: "Local filesystem operations"
    command: ["npx", "-y", "@modelcontextprotocol/server-filesystem"]
    tools:
      - name: read_file
        description: "Read file contents"
        category: "read"
        
      - name: write_file
        description: "Write content to file"
        category: "write"
```

### Dynamic Tool Management

Support for runtime tool changes via the add-context API:

```bash
# Enable additional tools during task execution
orchestrator-cli task add-context \
  --task-id 1001 \
  --enable-tools "github:merge_pr,github:close_issue"
  
# Disable tools if needed
orchestrator-cli task add-context \
  --task-id 1001 \
  --disable-tools "filesystem:delete_file"
```

## Implementation Plan

### Phase 1: Core Proxy
1. MCP protocol implementation
2. Tool filtering engine
3. Request routing to upstream servers

### Phase 2: TaskRun Integration
1. Extend TaskRun CRD schema
2. ConfigMap generation for tool config
3. Sidecar container deployment

### Phase 3: Tool Registry
1. Built-in server catalog
2. Tool discovery API
3. Documentation generation

### Phase 4: Dynamic Management
1. Runtime configuration updates
2. State persistence
3. Context versioning

## Security Considerations

1. **Tool Permissions**: Granular control over tool access
2. **Isolation**: Each task has isolated tool configuration
3. **Audit**: Log all tool invocations for security analysis
4. **Validation**: Validate tool parameters before forwarding

## Performance Optimizations

1. **Lazy Loading**: Only initialize requested MCP servers
2. **Connection Pooling**: Reuse connections to upstream servers
3. **Caching**: Cache tool metadata to reduce discovery overhead
4. **Minimal Context**: Only include enabled tools in agent context

## Example Workflow

1. **Task Submission**:
   ```bash
   orchestrator-cli task submit \
     --service todo-api \
     --task-file task.md \
     --tools "github:*,filesystem:read_file,filesystem:write_file"
   ```

2. **Proxy Configuration Generated**:
   - ConfigMap created with tool allowlist
   - Sidecar configured with upstream servers

3. **Agent Execution**:
   - Agent connects to local proxy
   - Only sees allowed tools
   - All requests filtered through proxy

4. **Dynamic Updates**:
   - PM realizes agent needs database tools
   - Adds context with additional tools
   - Proxy updates configuration without restart

## Benefits

1. **Reduced Context**: Agents only see relevant tools
2. **Security**: Fine-grained tool access control
3. **Flexibility**: Easy to add new MCP servers
4. **Debugging**: Central point for tool usage monitoring
5. **Performance**: Optimized tool discovery and loading