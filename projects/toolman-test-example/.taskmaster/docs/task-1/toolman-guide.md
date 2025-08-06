# Task 1: Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to complete the project structure setup task. The tools are specifically chosen to handle research, file operations, and configuration management needed for initializing the chat application monorepo.

## Core Tools

### 1. brave_web_search
**Purpose**: Research current best practices for React + Node.js development
**When to use**: 
- At the beginning of the task to gather information
- When you need to find specific implementation patterns
- To research Docker and Kubernetes configurations

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "React Node.js monorepo best practices 2024",
  "freshness": "month"
}
```

**Example searches**:
- "React Vite TypeScript project setup 2024"
- "Node.js Express TypeScript Docker configuration"
- "npm workspaces monorepo structure"
- "Docker multi-stage build React Node.js"

### 2. query_rust_docs
**Purpose**: Find containerization patterns and deployment best practices from Rust documentation
**When to use**:
- When researching container optimization techniques
- For finding production-grade deployment patterns
- To understand best practices for multi-stage builds

**How to use**:
```json
{
  "tool": "query_rust_docs",
  "crate": "container",
  "query": "multi-stage docker builds optimization"
}
```

### 3. create_directory
**Purpose**: Create the project directory structure
**When to use**:
- After planning the project structure
- To create nested directories in one operation
- For setting up the initial folder hierarchy

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/frontend/src/components"
}
```

**Directory creation sequence**:
1. `/chat-application` (root)
2. `/chat-application/frontend/src`
3. `/chat-application/backend/src`
4. `/chat-application/kubernetes/deployment-configs`

### 4. write_file
**Purpose**: Create all configuration and code files
**When to use**:
- After creating directories
- To write package.json, tsconfig.json, Docker files
- For creating configuration files

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/package.json",
  "content": "{\n  \"name\": \"chat-application\",\n  \"private\": true,\n  \"workspaces\": [\"frontend\", \"backend\"]\n}"
}
```

## Supporting Tools

### directory_tree
**Purpose**: Verify the created project structure
**When to use**:
- After creating all directories and files
- To validate the project structure matches requirements
- For documentation purposes

**How to use**:
```json
{
  "tool": "directory_tree",
  "path": "/chat-application"
}
```

### list_directory
**Purpose**: Check contents of specific directories
**When to use**:
- To verify files were created correctly
- Before making modifications to existing directories
- To check for existing files

### getAPIResources
**Purpose**: Research Kubernetes deployment requirements
**When to use**:
- When creating Kubernetes configuration files
- To understand available API resources
- For validating deployment configurations

## Implementation Flow

### Phase 1: Research (15 minutes)
1. Use `brave_web_search` to find:
   - React + Node.js monorepo best practices
   - TypeScript configuration patterns
   - Docker multi-stage build examples
   
2. Use `query_rust_docs` to research:
   - Container optimization techniques
   - Production deployment patterns

3. Document findings for reference

### Phase 2: Structure Creation (20 minutes)
1. Use `create_directory` to build folder hierarchy:
   ```
   /chat-application
   ├── frontend/src/...
   ├── backend/src/...
   └── kubernetes/...
   ```

2. Use `write_file` to create configuration files:
   - Root package.json with workspaces
   - Frontend package.json and configs
   - Backend package.json and configs
   - Docker and docker-compose files

### Phase 3: Configuration (15 minutes)
1. Use `write_file` for all config files:
   - ESLint configuration (.eslintrc.js)
   - Prettier configuration (.prettierrc)
   - TypeScript configs (tsconfig.json)
   - Environment templates (.env.example)

2. Use `directory_tree` to verify structure

### Phase 4: Documentation (10 minutes)
1. Use `write_file` to create:
   - Comprehensive README.md
   - Setup instructions
   - API documentation templates

## Best Practices

### Tool Usage Tips
1. **Batch Operations**: Create multiple related files together
2. **Validation**: Always verify with `directory_tree` after creation
3. **Research First**: Use search tools before implementing
4. **Incremental Progress**: Test configurations as you create them

### Common Patterns
```javascript
// 1. Research pattern
const bestPractices = await brave_web_search({
  query: "topic best practices 2024",
  freshness: "month"
});

// 2. Create structure pattern
await create_directory("/chat-application/frontend/src/components");
await write_file("/chat-application/frontend/src/App.tsx", appContent);

// 3. Verify pattern
const structure = await directory_tree("/chat-application");
```

## Troubleshooting

### Issue: Directory already exists
**Solution**: Check with `list_directory` first, skip if exists

### Issue: File write fails
**Solution**: Ensure directory exists with `create_directory` first

### Issue: Research returns outdated info
**Solution**: Add year to search query, use `freshness` parameter

### Issue: Configuration conflicts
**Solution**: Research compatibility with `brave_web_search` before writing

## Task Completion Checklist
- [ ] All research completed and documented
- [ ] Directory structure created and verified
- [ ] All configuration files written
- [ ] Docker files created and tested
- [ ] Documentation completed
- [ ] Final structure validation with `directory_tree`

This systematic approach ensures a properly configured, production-ready project structure that follows current best practices.