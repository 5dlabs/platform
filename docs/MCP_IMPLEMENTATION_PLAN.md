# MCP Implementation Plan

## Overview

This document outlines the plan to fix our MCP server implementation by restoring functionality from the old CLI implementation and aligning with the new workflow template structure.

## Current Issues

1. Missing auto-detection functionality (git branch, repo URL, working directory)
2. Simplified agent resolution instead of using agents config
3. Missing validation (repository format, model validation)
4. Still uses deprecated `github-user` parameter (should be removed)
5. Parameter mismatch with new workflow templates

## Tool Schemas and Parameters

### 1. `docs` Tool - Documentation Generation

#### Mandatory Parameters
- `working_directory` (string) - Working directory containing .taskmaster folder

#### Optional Parameters with Defaults
- `model` (string) - Claude model to use
  - **Default**: From `values.yaml` → `defaults.workflow.docs.model` → `"claude-opus-4-20250514"`
- `agent` (string) - Agent name for task assignment (e.g., "morgan", "rex", "blaze")
  - **Default**: From `values.yaml` → `defaults.docsAgent` → `"morgan"`
  - **Resolves to**: GitHub App name via `agents[agent].githubApp` → `"5DLabs-Morgan"`

#### Auto-Detected Parameters
- `repository-url` (string) - Repository URL
  - **Auto-detected**: From `git remote get-url origin`
  - **Fallback**: From `values.yaml` → `defaults.workflow.docs.repositoryUrl` → `"https://github.com/5dlabs/projects"`
- `source-branch` (string) - Source branch to work from
  - **Auto-detected**: From `git branch --show-current`
  - **Fallback**: From `values.yaml` → `defaults.workflow.docs.sourceBranch` → `"main"`

#### Removed Parameters
- ❌ `github_user` - No longer used (GitHub Apps only)

---

### 2. `task` Tool - Code Implementation

#### Mandatory Parameters
- `task_id` (integer) - Task ID to implement (minimum: 1)
- `service` (string) - Service identifier for workspace PVC (pattern: `^[a-z0-9-]+$`)
- `repository` (string) - Target repository URL (e.g., 'https://github.com/5dlabs/cto')
- `docs_project_directory` (string) - Project directory within docs repository (e.g., 'projects/market-research')

#### Optional Parameters with Defaults
- `docs_repository` (string) - Documentation repository URL
  - **Default**: From `values.yaml` → `defaults.workflow.code.docsRepositoryUrl` → `"https://github.com/5dlabs/projects"`
- `working_directory` (string) - Working directory within target repository
  - **Default**: From `values.yaml` → `defaults.workflow.code.workingDirectory` → `"."`
- `model` (string) - Claude model to use
  - **Default**: From `values.yaml` → `defaults.workflow.code.model` → `"claude-3-5-sonnet-20241022"`
- `agent` (string) - Agent name for task assignment (e.g., "morgan", "rex", "blaze", "cipher")
  - **Default**: From `values.yaml` → `defaults.codeAgent` → `"rex"`
  - **Resolves to**: GitHub App name via `agents[agent].githubApp` → `"5DLabs-Rex"`
- `continue_session` (boolean) - Whether to continue previous session
  - **Default**: From `values.yaml` → `defaults.workflow.code.continueSession` → `false`
- `overwrite_memory` (boolean) - Whether to overwrite CLAUDE.md memory file
  - **Default**: From `values.yaml` → `defaults.workflow.code.overwriteMemory` → `false`
- `env` (object) - Environment variables to set in container
  - **Default**: `{}` (empty object)
- `env_from_secrets` (array) - Environment variables from secrets
  - **Default**: `[]` (empty array)

#### Auto-Detected Parameters
- `docs_branch` (string) - Documentation branch to work from
  - **Auto-detected**: From `git branch --show-current`
  - **Fallback**: From `values.yaml` → `defaults.workflow.code.docsBranch` → `"main"`
- `context_version` (integer) - Version for this task execution
  - **MCP submits**: `0` (auto-assign mode)
  - **Controller calculates**: Queries existing CodeRuns for same task_id + service, increments max version

#### Removed Parameters
- ❌ `github_user` - No longer used (GitHub Apps only)

---

## Implementation Phases

### Phase 1: Core MCP Server Updates

#### 1.1 Add Git Auto-Detection Utilities
```rust
fn get_git_remote_url() -> Result<String>
fn get_git_current_branch() -> Result<String>
fn validate_repository_url(repo: &str) -> Result<()>
```

#### 1.2 Update Parameter Handling
- Remove all `github-user` parameters
- Add `repository-url` parameter for docs workflow
- Add missing parameters: `overwrite-memory`, `docs-branch`
- Set `context-version=0` for auto-assignment by controller
- Implement defaults from `values.yaml` structure

#### 1.3 Enhance Agent Resolution
- Replace `AgentsConfig` with environment variable approach (AGENT_*)
- Implement friendly name resolution: `agent_name` → `env["AGENT_" + agent_name.upper()]`
- Dynamic agent validation at runtime (no hardcoded enum in schema)
- Provide helpful error messages with available agent list
- Fallback logic: `provided_agent.or(default_agent)` (hardcoded defaults)

#### 1.4 Add Validation
- Repository URL validation (must be valid GitHub HTTPS URL)
- Model name validation (must start with "claude-")
- Service name validation (lowercase, numbers, hyphens only)
- Agent name validation against `AgentsConfig` with helpful error messages

### Phase 2: Update Tool Schemas

**Agent Validation Strategy**: Remove hardcoded enums from schemas and validate dynamically at runtime. This provides flexibility while giving users helpful error messages when they use invalid agent names.

**Implementation Example**:
```rust
// In MCP handler
fn validate_agent_name(agent_name: &str, agents_config: &AgentsConfig) -> Result<()> {
    if !agents_config.agents.contains_key(agent_name) {
        let available_agents: Vec<&String> = agents_config.agents.keys().collect();
        return Err(anyhow!(
            "Unknown agent '{}'. Available agents: {:?}", 
            agent_name, available_agents
        ));
    }
    Ok(())
}

// Usage in tool handler
if let Some(agent) = arguments.get("agent").and_then(|v| v.as_str()) {
    validate_agent_name(agent, agents_config)?;
    let github_app = agents_config.agents[agent].github_app.clone();
    // ... continue with resolved agent
}
```

#### 2.1 Update `docs` Tool Schema
```json
{
  "name": "docs",
  "description": "Initialize documentation for Task Master tasks using Claude",
  "inputSchema": {
    "properties": {
      "working_directory": {"type": "string", "description": "Working directory containing .taskmaster folder"},
      "agent": {"type": "string", "description": "Agent name (e.g., morgan, rex, blaze, cipher)"},
      "model": {"type": "string", "description": "Claude model to use"}
    },
    "required": ["working_directory"]
  }
}
```

#### 2.2 Update `task` Tool Schema  
```json
{
  "name": "task", 
  "description": "Submit a Task Master task for implementation using Claude",
  "inputSchema": {
    "properties": {
      "task_id": {"type": "integer", "minimum": 1},
      "service": {"type": "string", "pattern": "^[a-z0-9-]+$"},
      "repository": {"type": "string", "description": "Target repository URL (e.g., https://github.com/5dlabs/cto)"},
      "docs_project_directory": {"type": "string", "description": "Project directory in docs repo"},
      "agent": {"type": "string", "description": "Agent name (e.g., morgan, rex, blaze, cipher)"},
      "model": {"type": "string", "description": "Claude model to use"}
    },
    "required": ["task_id", "service", "repository", "docs_project_directory"]
  }
}
```

### Phase 3: Update Test Scripts

#### 3.1 Remove Deprecated Parameters
- Remove `-p github-user=""` from both test scripts

#### 3.2 Update Parameter Structure
- Use new parameter names matching workflow templates
- Use defaults from `values.yaml` structure

### Phase 4: Controller Updates

#### 4.1 Context Version Management
**Location**: `controller/core/src/tasks/code/controller.rs`

**Implementation**:
```rust
async fn reconcile_coderun(&self, coderun: &CodeRun) -> Result<()> {
    // Auto-assign context version if set to 0
    if coderun.spec.context_version == 0 {
        let next_version = self.calculate_next_context_version(
            coderun.spec.task_id,
            &coderun.spec.service
        ).await?;
        
        self.patch_context_version(coderun, next_version).await?;
    }
    
    // Continue with normal reconciliation...
}

async fn calculate_next_context_version(&self, task_id: u32, service: &str) -> Result<u32> {
    let existing_runs = self.list_coderuns_for_task_service(task_id, service).await?;
    let max_version = existing_runs.iter()
        .map(|cr| cr.spec.context_version)
        .max()
        .unwrap_or(0);
    Ok(max_version + 1)
}
```

**Benefits**:
- ✅ Kubernetes-native resource state management
- ✅ Handles race conditions via reconciliation loops
- ✅ Atomic operations with optimistic locking
- ✅ Clean separation: MCP handles UI, Controller handles resource logic

### Phase 5: Testing and Validation

#### 5.1 Test Auto-Detection
- Verify git branch detection works
- Verify repository URL detection works
- Test fallback to defaults when git commands fail

#### 5.2 Test Context Version Management
- Test sequential task submissions (versions 1, 2, 3...)
- Test concurrent submissions (race condition handling)
- Verify context version patching in controller

#### 5.3 Test Workflow Integration
- Test docs workflow with Morgan agent
- Test code workflow with Rex/Blaze agents
- Verify cross-repo functionality

#### 5.4 Test MCP Integration
- Test with Cursor IDE integration
- Verify parameter validation and error handling
- Test agent resolution logic
- Test agent validation with invalid names (should get helpful error messages)

---

## Values.yaml Default Structure Reference

```yaml
defaults:
  docsAgent: "morgan"
  codeAgent: "rex"
  
  workflow:
    code:
      workingDirectory: "."
      serviceId: ""
      docsRepositoryUrl: "https://github.com/5dlabs/projects"
      docsProjectDirectory: "."
      docsBranch: "main"
      continueSession: false
      overwriteMemory: false
      timeout: 3600
      model: "claude-3-5-sonnet-20241022"
      
    docs:
      workingDirectory: "."
      repositoryUrl: "https://github.com/5dlabs/projects"
      sourceBranch: "main"
      timeout: 1800
      model: "claude-opus-4-20250514"

agents:
  morgan:
    name: "Morgan"
    githubApp: "5DLabs-Morgan"
    # ... other agent config
  rex:
    name: "Rex"
    githubApp: "5DLabs-Rex"
    # ... other agent config
```

---

## Migration Notes

### Breaking Changes
- ❌ `github_user` parameter removed completely
- ✅ `repository-url` parameter added to docs workflow
- ✅ Auto-detection for git repository URL and branch (fails fast if not available)
- ❌ **No fallback values** - all parameters must be explicitly provided or auto-detected

### Fail-Fast Philosophy
- **No default/fallback values** to prevent masking configuration issues
- **Clear error messages** when required parameters are missing
- **Environment variables required** - fails at startup if AGENT_* vars not found
- **Git repository required** - fails if not in a valid git repo with origin remote

### Dependencies
- Git commands used for auto-detection (requires git in PATH)
- Agent configuration via environment variables (AGENT_* pattern)
- All other configuration handled via Helm values.yaml

### Environment Variables
The MCP server requires agent configuration via environment variables:

```bash
AGENT_MORGAN="5DLabs-Morgan"
AGENT_REX="5DLabs-Rex" 
AGENT_BLAZE="5DLabs-Blaze"
AGENT_CIPHER="5DLabs-Cipher"
```

These are automatically set by Helm deployment from values.yaml:
```yaml
env:
  AGENT_MORGAN: "{{ .Values.agents.morgan.githubApp }}"
  AGENT_REX: "{{ .Values.agents.rex.githubApp }}"
  AGENT_BLAZE: "{{ .Values.agents.blaze.githubApp }}"
  AGENT_CIPHER: "{{ .Values.agents.cipher.githubApp }}"
```