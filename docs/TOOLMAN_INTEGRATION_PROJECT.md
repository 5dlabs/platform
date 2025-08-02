# Toolman Integration Project Plan

*Context: Reference [@CLAUDE.md](./CLAUDE.md) for platform architecture and development guidelines.*

## Discovery Findings

### Current State Analysis

#### Toolman Deployment Overview
- **Location**: `orchestrator` namespace alongside the main orchestrator service
- **Image**: `ghcr.io/5dlabs/toolman:main-4f4c6e4` (Helm managed)
- **Architecture**: Multi-container pod with toolman service + Docker-in-Docker sidecar
- **Storage**: Uses persistent volumes (`toolman-data`, `home-mcp-data`)
- **Resource Limits**: 500m CPU, 512Mi memory per container

#### Toolman Tool Catalog Structure

The `toolman-tool-catalog` ConfigMap contains a comprehensive catalog of MCP servers and tools organized into two execution environments:

##### Local Execution Tools
- **Filesystem Server**: Complete file system operations
  - 12 tools for file/directory management (read, write, edit, move, search, etc.)
  - JSON schema-based tool definitions
  - Sandboxed to allowed directories
  - Categories: file-operations, search, version-control

##### Remote Execution Tools

1. **Rust Documentation (rustdocs)**
   - **Endpoint**: `http://rustdocs-mcp-rust-docs-mcp-server.mcp.svc.cluster.local:3000/sse`
   - **Tools**: 6 tools for Rust crate documentation
   - **Use Cases**: Crate management, semantic search, documentation queries

2. **Solana Blockchain Development**
   - **Endpoint**: `https://mcp.solana.com/mcp`  
   - **Tools**: 5 specialized Solana/Anchor framework tools
   - **Use Cases**: Blockchain development, expert consultation, documentation search

3. **Brave Search**
   - **Endpoint**: `stdio` 
   - **Tools**: 2 web search tools (local + web search)
   - **Use Cases**: Web search, local business search
   - **Authentication**: Uses API keys from secrets

4. **Reddit Integration**
   - **Tools**: 4 Reddit API tools
   - **Use Cases**: Community engagement, content posting, subreddit management
   - **Authentication**: OAuth-based with stored credentials

#### Tool Categorization Pattern
- **Categories**: general, search, file-operations, version-control, creating-resources, finding-information
- **Use Cases**: Standardized descriptions for tool discovery and routing
- **Schema Validation**: Full JSON Schema definitions for all tool inputs

#### Integration Architecture
- **Service Discovery**: Kubernetes service-based routing for internal MCP servers
- **External APIs**: Direct HTTPS endpoints for external services
- **Authentication**: Kubernetes secrets-based credential management
- **Execution Models**: 
  - Local: NPX-based Node.js MCP servers
  - Remote: HTTP/SSE-based remote MCP servers

### Key Observations

1. **Mature Tool Ecosystem**: Comprehensive catalog with diverse tool types
2. **Kubernetes-Native**: Deep integration with cluster networking and storage
3. **Multi-Execution Model**: Supports both local and remote MCP server execution
4. **Schema-Driven**: Structured tool definitions with validation
5. **Secret Management**: Proper handling of API credentials
6. **Category System**: Organized tool discovery and routing

### Integration Opportunities

The existing toolman infrastructure provides a solid foundation for extending the orchestrator platform with additional MCP capabilities, potentially enabling more sophisticated agent workflows and tool composition.

## Toolman Architecture Deep Dive

### How Toolman Works

After analyzing the Toolman codebase, here's my understanding of how it operates:

#### Core Architecture Components

1. **HTTP Server (`toolman-http`)** - Main aggregation server
   - **Purpose**: Aggregates all tools from configured MCP servers into a single HTTP endpoint
   - **Location**: `src/bin/http_server.rs`
   - **Functionality**: 
     - Loads `servers-config.json` to discover MCP servers
     - Starts/manages multiple MCP server processes
     - Exposes unified HTTP/SSE endpoint at `/mcp`
     - Updates Kubernetes ConfigMap with tool catalog for service discovery
     - Supports both stdio and HTTP transport protocols

2. **Client/Stdio Wrapper (`toolman`)** - Client-side filtering proxy
   - **Purpose**: Provides per-agent tool filtering via client-side configuration
   - **Location**: `src/bin/client.rs` and `src/client.rs`
   - **Functionality**:
     - Loads `client-config.json` or `toolman-example-config.json` for tool filtering
     - Spawns local MCP servers (filesystem, etc.) in client context
     - Proxies selected remote tools from HTTP server
     - Implements MCP stdio protocol for AI assistant integration

#### Configuration System

**Server Configuration (`servers-config.json`)**:
```json
{
  "servers": {
    "filesystem": {
      "name": "Filesystem MCP Server", 
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "transport": "stdio", // or "http"
      "enabled": true
    }
  }
}
```

**Client Configuration (`client-config.json`)**:
```json
{
  "remoteTools": ["kubernetes_listResources", "kubernetes_getResource"],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/project"],
      "tools": ["read_file", "write_file", "list_directory"],
      "workingDirectory": "project_root"
    }
  }
}
```

#### Data Flow

1. **Server-side**: HTTP server discovers and starts all configured MCP servers
2. **Tool Discovery**: Server aggregates all available tools into HTTP endpoints
3. **Service Discovery**: Updates Kubernetes ConfigMap with complete tool catalog
4. **Client Connection**: AI assistants connect via stdio wrapper
5. **Tool Filtering**: Client configuration determines which tools are exposed
6. **Hybrid Execution**: Local tools run in client context, remote tools proxied to HTTP server

#### Key Features

- **Session Management**: UUID-based session tracking
- **Process Management**: Robust server lifecycle management with health monitoring
- **Template System**: Variable substitution for paths and configuration
- **Transport Abstraction**: Supports both stdio and HTTP/SSE protocols
- **Atomic Configuration**: Safe config updates with backup/recovery
- **Kubernetes Integration**: Native service discovery via ConfigMap updates

#### Tool Categorization

Tools are categorized by:
- **Category**: `general`, `search`, `file-operations`, `version-control`
- **Use Cases**: Standardized descriptions for AI routing decisions
- **Input Schema**: Full JSON Schema validation for all tool parameters

#### Integration Points

1. **Claude Agent Integration**: Stdio wrapper provides MCP protocol for Claude agents
2. **Kubernetes Service Discovery**: ConfigMap-based tool catalog for cluster services
3. **Multi-Protocol Support**: HTTP/SSE for remote services, stdio for local execution
4. **Configuration Management**: Dynamic server configuration with template support

### Technical Insights

- **Rust Implementation**: High-performance async runtime with Tokio
- **Error Recovery**: Comprehensive error handling with backup/restore mechanisms
- **Security**: Sandboxed execution with configurable working directories
- **Scalability**: Supports 25+ concurrent MCP servers via HTTP aggregation
- **Observability**: Extensive logging and health monitoring capabilities

This architecture enables sophisticated AI agent workflows by providing a unified tool interface while maintaining flexibility in tool selection and execution context.

## Docs Job Workflow Analysis

### Docs Job Template Review

I've analyzed all four template files for the docs generation job:

#### 1. Container Script (`container.sh.hbs`) - The Orchestration Engine

**Purpose**: Manages the complete lifecycle of documentation generation in the Kubernetes Job

**Key Workflow Steps**:
1. **SSH Authentication Setup** - Configures GitHub SSH access using mounted private keys
2. **Repository Management** - Clones or updates the target repository 
3. **Working Directory Setup** - Navigates to specified working directory within repo
4. **ConfigMap Integration** - Copies template files (CLAUDE.md, prompt.md, hooks) from mounted ConfigMap
5. **Workspace Validation** - Verifies `.taskmaster` directory structure and required files
6. **Claude Execution** - Runs Claude Code with the prompt file
7. **Git Workflow** - Creates branch, commits changes, pushes, and creates GitHub PR

**Key Features**:
- SSH-only authentication (no token-based auth)
- Safe mode toggle for testing without token consumption
- Comprehensive error handling and validation
- Automatic PR creation with retry logic
- Template variable substitution for dynamic configuration

#### 2. Prompt Template (`prompt.md.hbs`) - The Agent Instructions

**Purpose**: Provides detailed instructions to Claude Code for documentation generation

**Key Requirements**:
- **Mandatory Completion** - Must process ALL tasks, no partial completion
- **Individual Task File Reading** - Uses `.taskmaster/docs/task-{id}/task.txt` files
- **Smart Skipping** - Skip tasks that already have complete documentation (all 3 files)
- **Progress Reporting** - Must announce task processing and provide periodic updates
- **Exact Output Requirements** - Must create 3 specific files per task:
  - `task.md` - Implementation guide
  - `prompt.md` - Autonomous AI agent prompt
  - `acceptance-criteria.md` - Test cases and completion criteria

**Large Project Handling**:
- Explicit instructions for projects with 10+ tasks
- Progress tracking and updates every 5 tasks
- Emphasizes completing ALL tasks without stopping

#### 3. Settings Template (`settings.json.hbs`) - The Agent Configuration

**Purpose**: Configures Claude Code permissions, environment, and behavior

**Key Configuration**:
- **Permissions**: Full tool access (Bash, Edit, Read, Write, MultiEdit, Glob, Grep, etc.)
- **Model**: Template variable for dynamic model selection
- **Environment**: Production settings with telemetry integration
- **Behavior**: Auto-accept edits, disable cost warnings

**Template Features**:
- Conditional tool override support
- OpenTelemetry integration for observability  
- Production-optimized environment variables

#### 4. CLAUDE.md Template (`claude.md.hbs`) - The Agent Memory

**Purpose**: Provides context and working memory for the Claude Code agent

**Context Information**:
- Repository details and branch information
- Working directory and GitHub user
- Task-specific vs. all-tasks mode
- Documentation generation instructions

**Key Guidance**:
- Individual task file locations
- Skip logic for existing complete documentation
- File output locations and standards
- Repository context and path handling

### Integration Points for Toolman

**Current Tool Access**: The docs job currently uses Claude Code's built-in tools (filesystem, git, web search, etc.)

**Potential Toolman Integration**:
1. **Task-Specific Tool Selection** - Each docs generation could get a customized tool set
2. **Extended Capabilities** - Access to Kubernetes tools, Reddit integration, external search
3. **Configuration Templates** - Generate client-config.json as part of the template system
4. **Service Discovery** - Leverage toolman's ConfigMap-based tool catalog

**Template Integration Points**:
- `settings.json.hbs` could include MCP server configuration
- `container.sh.hbs` could spawn toolman client alongside Claude
- Additional ConfigMap volumes for toolman configuration
- Environment variables for toolman server endpoints

This workflow demonstrates a highly structured approach to AI agent orchestration with comprehensive error handling, validation, and git workflow automation.

## Feature Specification: Task-Specific Toolman Configuration Generation

### Goal
Generate task-specific Toolman client configurations during documentation generation, enabling precise tool selection for code implementation agents.

### Architecture
**IMPORTANT CLARIFICATION**: The docs agent does NOT receive a separate JSON catalog file. The correct architecture is:

1. **Orchestrator** has toolman-tool-catalog ConfigMap mounted (JSON data)
2. **Orchestrator** renders catalog data into markdown using a Handlebars template  
3. **Orchestrator** embeds this rendered markdown into the docs agent's prompt.md file
4. **Docs agent** receives catalog information as part of its markdown prompt
5. **Docs agent** uses the markdown catalog info to generate toolman-config.json files

### Requirements

#### Core Functionality
1. **Markdown Catalog Embedding**: Render toolman catalog data into markdown format and embed in docs agent prompt
2. **Docs Agent Enhancement**: During docs generation, create a 4th file: `toolman-config.json` in each task directory
3. **Specific Tool Selection**: Generate configs with individual tool names (not wildcards) matching `toolman-example-config.json` format
4. **Implementation Analysis**: Analyze task requirements and map to appropriate tools from the embedded catalog information
5. **Complete Configuration**: Include full server details for local servers (command, args, working_directory, etc.)

#### Configuration Format
```json
{
  "remoteTools": [
    "kubernetes_listResources",
    "kubernetes_getResource",
    "brave_web_search"
  ],
  "localServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/project"],
      "tools": ["read_file", "write_file", "list_directory"],
      "workingDirectory": "project_root"
    }
  }
}
```

### Architecture: Template-Based Approach

#### ConfigMap Handling
- **toolman-tool-catalog ConfigMap**: Already contains complete client-side configuration details
  - ✅ Local servers: `command`, `args`, `working_directory`, and full tool arrays
  - ✅ Remote servers: `endpoint`, `description`, and tool arrays
  - ✅ Tool metadata: categories, use_cases, JSON schemas for intelligent selection

#### Template Integration
1. **New Template**: `toolman-catalog.json.hbs` alongside existing 4 docs templates
2. **ConfigMap Mount**: Mount `toolman-tool-catalog` on orchestrator (not docs agent)
3. **Template Rendering**: Populate template with complete catalog data from ConfigMap
4. **Agent Access**: Docs agent gets pre-processed catalog without raw ConfigMap complexity
5. **Auto-Updates**: When Toolman updates ConfigMap → orchestrator automatically gets latest catalog

#### Docs Agent Workflow
1. **Analyze Task**: Read `task.txt`, understand implementation requirements
2. **Tool Selection**: Map requirements to tools using catalog data (categories, use_cases)
3. **Config Generation**: Create `toolman-config.json` with specific tool names and server details
4. **File Placement**: Save in `.taskmaster/docs/task-{id}/toolman-config.json`
5. **Code Agent Consumption**: Later code agent uses this config for precise tool access

### Benefits
- **Reduced Cognitive Load**: Docs agent doesn't handle raw ConfigMap parsing
- **Always Current**: Automatic catalog updates when Toolman ConfigMap changes
- **Precise Tool Access**: Code agents get exactly the tools they need per task
- **No Fallbacks/Constraints**: Full catalog access, failure visibility
- **Consistent Format**: Matches existing toolman-example-config.json structure

### Integration Points
- **Container Script**: No changes needed - existing git workflow handles the new file
- **Prompt Template**: Add instructions for toolman-config.json generation
- **Settings Template**: Ensure docs agent has necessary permissions
- **New Template**: Create toolman-catalog.json.hbs for catalog data
- **ConfigMap Mount**: Add toolman-tool-catalog to orchestrator ConfigMap mounts

This approach ensures task-specific tool selection while maintaining the existing docs generation workflow and leveraging the comprehensive tool catalog already maintained by Toolman.