# Code Job Refactor Implementation Plan

## Overview

This document outlines a comprehensive refactor of the code job flow to simplify the MCP interface, improve clarity, and remove unused/complex features while maintaining necessary functionality.

## Current Usage Analysis

### Context Version Functionality
**Current Usage**: 
- ConfigMap naming: `{service_name}-task{task_id}-v{context_version}-files`
- Job naming: `code-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}`
- Resource labeling: `"context-version": code_run.spec.context_version.to_string()`
- Template context variable

**Recommendation**: Handle automatically in backend - increment based on existing resources for the same task.

### Overwrite Memory Functionality
**Current Usage**:
- Template variable: `"overwrite_memory": code_run.spec.overwrite_memory`
- Container script logic: Controls CLAUDE.md memory persistence behavior

**Recommendation**: Set to `false` by default in CLI, don't expose to MCP server (reduces complexity).

## Proposed Changes

### 1. MCP Server Interface Simplification

#### Remove Parameters:
- `local_tools` - Moving to Toolman approach in future PR
- `remote_tools` - Moving to Toolman approach in future PR  
- `context_version` - Handle automatically in backend
- `prompt_modification` - Changes will be in prompt files instead
- `overwrite_memory` - Set to false by default, not user-configurable

#### Make Required:
- `docs_project_directory` - No longer optional
- `working_directory` - No longer optional (remove auto-detection)

#### Repository Format Change:
- `repository_url` → `repository` (format: `org/repo` or `user/repo`)
- `docs_repository_url` → `docs_repository` (format: `org/repo` or `user/repo`)

#### Default Handling:
- `docs_branch` - Set default to "main" in CLI, required on backend
- `model` - Use Helm configuration default (remove hard-coded fallbacks)

### 2. Updated MCP Tool Schema

```json
{
  "name": "task",
  "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task_id": {
        "type": "integer",
        "description": "REQUIRED: Task ID to implement from task files",
        "minimum": 1
      },
      "service": {
        "type": "string", 
        "description": "REQUIRED: Target service name (creates workspace-{service} PVC)",
        "pattern": "^[a-z0-9-]+$"
      },
      "repository": {
        "type": "string",
        "description": "REQUIRED: Target repository in format 'org/repo' or 'user/repo' (e.g., '5dlabs/platform')"
      },
      "docs_repository": {
        "type": "string", 
        "description": "REQUIRED: Documentation repository in format 'org/repo' or 'user/repo' where Task Master definitions are stored"
      },
      "docs_project_directory": {
        "type": "string",
        "description": "REQUIRED: Project directory within docs repository (e.g., '_projects/simple-api')"
      },
      "working_directory": {
        "type": "string",
        "description": "REQUIRED: Working directory within target repository"
      },
      "model": {
        "type": "string",
        "description": "Claude model to use (optional, defaults to Helm configuration value)"
      },
      "github_user": {
        "type": "string",
        "description": "GitHub username for authentication (optional if FDL_DEFAULT_CODE_USER environment variable is set)"
      },
      "docs_branch": {
        "type": "string", 
        "description": "Docs branch to use (optional, defaults to 'main')"
      },
      "continue_session": {
        "type": "boolean",
        "description": "Whether to continue a previous session (optional, defaults to false)"
      },
      "env": {
        "type": "object",
        "description": "Environment variables to set in the container (optional)",
        "additionalProperties": {"type": "string"}
      },
      "env_from_secrets": {
        "type": "array", 
        "description": "Environment variables from secrets (optional)",
        "items": {
          "type": "object",
          "properties": {
            "name": {"type": "string"},
            "secretName": {"type": "string"}, 
            "secretKey": {"type": "string"}
          },
          "required": ["name", "secretName", "secretKey"]
        }
      }
    },
    "required": ["task_id", "service", "repository", "docs_repository", "docs_project_directory", "working_directory"]
  }
}
```

### 3. Component Changes Required

#### A. MCP Server (`orchestrator/tools/src/mcp/main.rs`)
- Remove `local_tools`, `remote_tools`, `context_version`, `prompt_modification`, `overwrite_memory` parameters
- Change `repository_url`/`docs_repository_url` to `repository`/`docs_repository`
- Make `docs_project_directory` and `working_directory` required
- Remove model validation and hard-coded default
- Add validation for `org/repo` format

#### B. CLI (`orchestrator/tools/src/cli/commands.rs`)
- Remove auto-detection for `working_directory` 
- Set `docs_branch` default to "main" if not provided
- Set `overwrite_memory` to `false` (hardcoded)
- Auto-increment `context_version` based on existing resources
- Convert `org/repo` format to full URLs for backend compatibility
- Remove `local_tools`/`remote_tools` parameters

#### C. Backend Handler (`orchestrator/core/src/handlers/code_handler.rs`)
- Remove hard-coded model fallback, require `DEFAULT_CODE_MODEL` environment variable
- Update to handle new repository format
- Remove `local_tools`/`remote_tools` processing

#### D. Common Models (`orchestrator/common/src/models/code_request.rs`)
- Remove `local_tools`, `remote_tools`, `prompt_modification` fields
- Update field names and requirements per new schema
- Keep `context_version` and `overwrite_memory` for internal use

#### E. CRD (`orchestrator/core/src/crds/coderun.rs`)  
- Remove `local_tools`, `remote_tools`, `prompt_modification` fields
- Keep `context_version` and `overwrite_memory` for internal functionality

#### F. Helm Configuration
- Add `DEFAULT_CODE_MODEL` environment variable (like docs)
- Update values.yaml with `models.defaultCodeModel`

### 4. Context Version Auto-Management

**Implementation Strategy**:
```rust
// In CLI - auto-detect context version
fn get_next_context_version(task_id: u32, service: &str) -> Result<u32> {
    // Query existing CodeRuns for this task_id + service
    // Find highest context_version + 1
    // Default to 1 if none exist
}
```

### 5. Repository Format Handling

**MCP Server Validation**:
```rust
fn validate_repository_format(repo: &str) -> Result<()> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(anyhow!("Repository must be in format 'org/repo' or 'user/repo'"));
    }
    Ok(())
}
```

**CLI URL Conversion**:
```rust
fn repo_to_url(repo: &str) -> String {
    format!("https://github.com/{}.git", repo)
}

fn repo_to_ssh_url(repo: &str) -> String {
    format!("git@github.com:{}.git", repo) 
}
```

### 6. Template Updates

**Container Script Updates**:
- Use `repository` and `docs_repository` environment variables
- Set appropriate URLs based on authentication method needed
- Remove `local_tools`/`remote_tools` processing

## Migration Impact

### Breaking Changes
1. **MCP Tool Interface**: Significant parameter changes
2. **Repository Format**: URLs → org/repo format  
3. **Required Fields**: `docs_project_directory` and `working_directory` now required
4. **Removed Features**: local/remote tools, manual context versioning

### Backward Compatibility
- Internal CRD structure maintains compatibility
- Templates updated to handle new format
- Existing CodeRuns continue to work

## Additional Recommendations

### 1. Repository Format Benefits
- **Clarity**: Removes confusion between SSH vs HTTPS URLs
- **Template Simplification**: Single format to handle
- **Consistency**: Same pattern for both repositories

### 2. Auto-Detection Removal Benefits  
- **Predictability**: No hidden behavior 
- **Explicitness**: All parameters clearly specified
- **Debugging**: Easier to trace issues

### 3. Context Version Automation Benefits
- **User Experience**: One less parameter to manage
- **Retry Logic**: Automatic versioning for retries
- **Resource Management**: Proper cleanup of old versions

### 4. Model Configuration Consistency
- **Centralization**: All defaults in Helm configuration
- **Flexibility**: Environment-specific model selection
- **No Hard-coding**: Removes configuration anti-patterns

## Implementation Order

1. **Update CRD and Common Models** (remove unused fields)
2. **Update Backend Handler** (remove hard-coded fallbacks, add env var requirement)  
3. **Update CLI** (add auto-detection for context_version, set defaults)
4. **Update MCP Server** (new interface, validation)
5. **Update Templates** (handle new repository format)
6. **Update Helm Configuration** (add DEFAULT_CODE_MODEL)
7. **Update Tool Schema** (reflect new interface)
8. **Testing** (verify all flows work with new interface)

## Testing Strategy

1. **Unit Tests**: Repository format validation, context version logic
2. **Integration Tests**: Full MCP → CRD flow
3. **Template Tests**: Verify repository URL handling
4. **Regression Tests**: Ensure existing functionality preserved