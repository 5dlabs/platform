# Toolman Guide for Task 1: Setup Project Structure and Environment

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 1, which focuses on setting up a monorepo project structure with React frontend and Node.js backend, including containerization and orchestration configurations.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the project directory structure including frontend, backend, and kubernetes folders

**When to Use**: 
- At the beginning of the task to establish the monorepo structure
- When creating nested configuration directories

**How to Use**:
```
# Create main project structure
create_directory /chat-application
create_directory /chat-application/frontend
create_directory /chat-application/frontend/src
create_directory /chat-application/backend
create_directory /chat-application/backend/src
create_directory /chat-application/kubernetes
create_directory /chat-application/kubernetes/deployment-configs
```

**Parameters**:
- `path`: The absolute or relative path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create all configuration files, source files, and documentation

**When to Use**: 
- After research is complete to create package.json files
- To write Docker, docker-compose, and Kubernetes configurations
- To create TypeScript configurations and documentation

**How to Use**:
```
# Create package.json for frontend
write_file /chat-application/frontend/package.json <content>

# Create Docker configuration
write_file /chat-application/Dockerfile <docker-content>

# Create environment configuration
write_file /chat-application/.env.development <env-vars>
```

**Parameters**:
- `path`: File path to write to
- `content`: The complete file content

### 3. **brave_web_search** (Remote)
**Purpose**: Research modern best practices for React + Node.js project structures

**When to Use**: 
- Beginning of task for architecture research
- When investigating monorepo patterns
- For TypeScript configuration best practices

**How to Use**:
```
# Search for modern practices
brave_web_search "React Node.js monorepo TypeScript 2024 best practices"
brave_web_search "Docker compose React Node.js development setup"
brave_web_search "ESLint Prettier configuration monorepo"
```

**Parameters**:
- `query`: Search query string
- `count`: Number of results (max 20)
- `offset`: For pagination
- `freshness`: Filter by recency (e.g., "month", "year")

### 4. **query_rust_docs** (Remote)
**Purpose**: Research containerization patterns from Rust ecosystem

**When to Use**: 
- When researching container optimization techniques
- For multi-stage Docker build patterns
- To understand efficient deployment practices

**How to Use**:
```
# Query containerization patterns
query_rust_docs {
  "crate": "tokio",
  "query": "containerization deployment Docker patterns",
  "max_results": 10
}
```

**Parameters**:
- `crate`: The Rust crate to search
- `query`: Semantic search query
- `max_results`: Number of results to return

### 5. **searchModules** & **moduleDetails** (Remote - terraform)
**Purpose**: Research Kubernetes deployment configurations using Terraform modules

**When to Use**: 
- When designing Kubernetes deployment configurations
- For understanding container orchestration patterns
- To find production-ready deployment templates

**How to Use**:
```
# First search for relevant modules
searchModules "kubernetes deployment react node"

# Then get details using the moduleID from search results
moduleDetails <moduleID>
```

**Parameters**:
- `query`: Search term for modules
- `moduleID`: Specific module identifier (from search results)

## Supporting Tools

### **read_file** (Local - filesystem)
**Purpose**: Verify created files and review configurations

**When to Use**: After writing files to verify content

### **list_directory** (Local - filesystem)
**Purpose**: Confirm directory structure creation

**When to Use**: After creating directories to verify structure

### **edit_file** (Local - filesystem)
**Purpose**: Make line-based edits to configuration files

**When to Use**: When updating existing configurations based on research

## Implementation Flow

1. **Research Phase** (Use remote tools first)
   - Start with `brave_web_search` for React/Node.js best practices
   - Use `query_rust_docs` for containerization patterns
   - Use `searchModules` for Kubernetes configurations

2. **Structure Creation Phase**
   - Use `create_directory` to build the monorepo structure
   - Create all necessary subdirectories

3. **Configuration Phase**
   - Use `write_file` to create all configuration files
   - Create package.json files with researched dependencies
   - Write Docker and docker-compose configurations
   - Create TypeScript configurations

4. **Verification Phase**
   - Use `list_directory` to verify structure
   - Use `read_file` to review configurations
   - Use `edit_file` for any necessary adjustments

## Best Practices

1. **Research First**: Always complete research using remote tools before creating files
2. **Incremental Creation**: Build directory structure before writing files
3. **Configuration Templates**: Use research findings to inform configuration file content
4. **Environment Variables**: Create separate .env files for different environments
5. **Documentation**: Write comprehensive README.md based on setup decisions

## Troubleshooting

- **Directory Already Exists**: The `create_directory` tool will succeed silently if directory exists
- **File Overwrites**: The `write_file` tool will overwrite existing files - use `read_file` first to check
- **Search Limitations**: Web search returns max 20 results - use pagination with offset
- **Module Search**: Always use `searchModules` before `moduleDetails` to get valid moduleID

## Task-Specific Tips

1. Focus on modern practices - use freshness filter in web searches
2. Consider both development and production configurations
3. Ensure Docker setup supports hot reloading
4. Design with Kubernetes deployment in mind from the start
5. Include comprehensive .gitignore for both frontend and backend