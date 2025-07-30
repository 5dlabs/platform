# Platform Configuration Analysis: Defaults and Auto-Detection Matrix

## Component Responsibility Matrix

| Field | Component | File Location | Type |
|-------|-----------|---------------|------|
| `working_directory` | CLI | `tools/src/cli/docs_generator.rs:67-87` | Auto-detected |
| `model` | MCP Server | `tools/src/mcp/main.rs:167` | Hard-coded default |
| `repository_url` | CLI | `tools/src/cli/docs_generator.rs:47-59` | Auto-detected |
| `source_branch` | CLI | `tools/src/cli/docs_generator.rs:89-100` | Auto-detected |
| `github_user` | MCP Server | `tools/src/mcp/main.rs:170` | Environment variable |
| `task_id` | N/A | N/A | Required field |
| `service` | N/A | N/A | Required field |
| `docs_repository_url` | N/A | N/A | Optional |
| `docs_project_directory` | N/A | N/A | Optional |
| `local_tools` | N/A | N/A | Optional |
| `remote_tools` | N/A | N/A | Optional |
| `context_version` | MCP Server | `tools/src/mcp/main.rs:263` | Hard-coded default |
| `prompt_modification` | N/A | N/A | Optional |
| `docs_branch` | Backend | `common/src/models/code_request.rs:89` | Hard-coded default |
| `continue_session` | MCP Server | `tools/src/mcp/main.rs:277` | Hard-coded default |
| `overwrite_memory` | MCP Server | `tools/src/mcp/main.rs:282` | Hard-coded default |
| `env` | N/A | N/A | Optional |
| `env_from_secrets` | N/A | N/A | Optional |
| `api_url` | CLI | `tools/src/cli/main.rs:41` | Hard-coded default |
| `output_format` | CLI | `tools/src/cli/main.rs:46` | Hard-coded default |

**Helm Configurable Defaults:**
- `models.defaultDocsModel` → Backend Orchestrator (`infra/charts/orchestrator/values.yaml:129`)
- `models.defaultCodeModel` → Backend Orchestrator (`infra/charts/orchestrator/values.yaml:131`) 
- Storage, networking, resource limits → Backend Orchestrator (`infra/charts/orchestrator/values.yaml`)

---

## Executive Summary

This analysis documents all default values and auto-detection mechanisms across the AI-powered development platform, covering configuration sources from administrator level (Helm) down to runtime auto-detection.

## Configuration Hierarchy

The platform uses a multi-layer configuration hierarchy:
1. **Administrator Level** - Helm chart values and environment variables
2. **CLI Level** - Command-line arguments and flags  
3. **MCP Server Level** - MCP tool parameter processing
4. **Runtime Auto-Detection** - Git commands and filesystem introspection

---

## Field-by-Field Analysis

### Documentation Generation (`docs` command)

| Field | Required | Default Value | Source Level | Implementation Details |
|-------|----------|---------------|--------------|----------------------|
| `working_directory` | ❌ | Auto-detected | Runtime | Uses `git rev-parse --show-toplevel` + relative path calculation in `docs_generator.rs:get_working_directory()` |
| `model` | ❌ | `"claude-opus-4-20250514"` | Administrator/CLI | Hard-coded default in MCP server `tools/src/mcp/main.rs:167` |
| `repository_url` | ❌ | Auto-detected | Runtime | Uses `git remote get-url origin` in `docs_generator.rs:get_git_remote_url()` |
| `source_branch` | ❌ | Auto-detected (fallback: `"main"`) | Runtime | Uses `git rev-parse --abbrev-ref HEAD` in `docs_generator.rs:get_current_branch()` |
| `github_user` | ✅ | Environment variable | Administrator | `FDL_DEFAULT_DOCS_USER` env var takes precedence over CLI parameter |

#### Detailed Implementation Notes:
- **Working Directory Detection**: Platform calculates relative path from git repository root to current directory
- **Branch Detection**: If git command fails, defaults to `"main"`
- **Repository URL**: Must be accessible git remote, typically GitHub
- **Model Selection**: Administrator can override via Helm values `models.defaultDocsModel`

### Code Task Submission (`task` command)

| Field | Required | Default Value | Source Level | Implementation Details |
|-------|----------|---------------|--------------|----------------------|
| `task_id` | ✅ | N/A | CLI | Positional argument, must be valid task ID |
| `service` | ✅ | N/A | CLI | Must match Kubernetes naming conventions (lowercase, hyphens) |
| `model` | ✅ | No default | CLI | Must be specified, validated to start with `"claude-"` |
| `repository_url` | ❌ | Auto-detected | Runtime | Same as docs: `git remote get-url origin` |
| `docs_repository_url` | ❌ | `null` | CLI | Optional, for dual-repository workflows |
| `docs_project_directory` | ❌ | `null` | CLI | Optional, path within docs repository |
| `working_directory` | ❌ | Service name | CLI | Defaults to service parameter value if not specified |
| `github_user` | ❌ | Environment variable | Administrator | `FDL_DEFAULT_CODE_USER` env var, no CLI fallback required |
| `local_tools` | ❌ | `null` | CLI | Comma-separated MCP tool names |
| `remote_tools` | ❌ | `null` | CLI | Comma-separated MCP tool names |
| `context_version` | ❌ | `1` | CLI | Auto-incremented for retry attempts |
| `prompt_modification` | ❌ | `null` | CLI | Additional context for retries |
| `docs_branch` | ❌ | `"main"` | CLI | Hard-coded default in `common/src/models/code_request.rs:89` |
| `continue_session` | ❌ | `false` | CLI | Boolean flag for session continuation |
| `overwrite_memory` | ❌ | `false` | CLI | Boolean flag for memory management |
| `env` | ❌ | `{}` | CLI | Key-value pairs for container environment |
| `env_from_secrets` | ❌ | `[]` | CLI | Kubernetes secret references |

#### Detailed Implementation Notes:
- **Service Validation**: Must contain only lowercase letters, numbers, and hyphens (Kubernetes PVC naming)
- **Model Validation**: Must start with `"claude-"` prefix
- **Working Directory**: When not specified, uses service name as working directory
- **Context Version**: Automatically incremented for retry scenarios

---

## Administrator-Level Configuration (Helm)

### Location: `infra/charts/orchestrator/values.yaml`

| Setting | Default Value | Purpose | Override Method |
|---------|---------------|---------|-----------------|
| `models.defaultDocsModel` | `"claude-opus-4-20250514"` | Documentation generation model | Helm values override |
| `models.defaultCodeModel` | `"claude-sonnet-4-20250514"` | Code implementation model | Helm values override |
| `config.kubernetesNamespace` | `"orchestrator"` | Target namespace | Helm values override |
| `config.serverHost` | `"0.0.0.0"` | API server bind address | Helm values override |
| `config.serverPort` | `"8080"` | API server port | Helm values override |
| `config.rustLog` | `"debug"` | Logging level | Helm values override |
| `storage.storageClassName` | `"local-path"` | PVC storage class | Helm values override |
| `storage.workspaceSize` | `"10Gi"` | Workspace PVC size | Helm values override |
| `agent.image.repository` | `ghcr.io/5dlabs/platform/claude-code` | Agent container image | Helm values override |
| `agent.image.tag` | `"1.0.56"` | Agent image version | Helm values override |

### Environment Variables (Administrator-Set)

| Variable | Purpose | Default Behavior | Required |
|----------|---------|------------------|----------|
| `FDL_DEFAULT_DOCS_USER` | GitHub user for docs generation | MCP server requires github_user parameter if not set | ❌ |
| `FDL_DEFAULT_CODE_USER` | GitHub user for code tasks | Uses parameter if available, optional otherwise | ❌ |
| `DEFAULT_DOCS_MODEL` | Runtime model override | Falls back to hard-coded default | ❌ |
| `ORCHESTRATOR_API_URL` | CLI API endpoint | `http://orchestrator.orchestrator.svc.cluster.local/api/v1` | ❌ |

---

## CLI-Level Configuration

### Location: `orchestrator/tools/src/cli/main.rs`

| Argument | Default Value | Environment Variable | Auto-Detection |
|----------|---------------|----------------------|----------------|
| `--api-url` | `http://orchestrator.orchestrator.svc.cluster.local/api/v1` | `ORCHESTRATOR_API_URL` | ❌ |
| `--output` | `"table"` | None | ❌ |
| `--working-directory` | Auto-detected | None | ✅ |
| `--model` | See command-specific defaults | None | ❌ |
| `--repository-url` | Auto-detected | None | ✅ |
| `--source-branch` | Auto-detected | None | ✅ |

---

## MCP Server Configuration

### Location: `orchestrator/tools/src/mcp/main.rs`

The MCP server acts as a parameter processing layer with these behaviors:

| Function | Default Strategy | Error Handling |
|----------|-----------------|----------------|
| Model validation | Must start with `"claude-"` | Returns error for invalid models |
| Service validation | Kubernetes naming rules | Returns error for invalid names |
| GitHub user resolution | Environment variable priority | Fails if neither env var nor parameter provided |
| Parameter extraction | Type-safe extraction with defaults | Detailed error messages for missing required fields |

### MCP Parameter Processing Pipeline:

1. **Extract parameters** from MCP JSON request
2. **Apply environment variable overrides** (if applicable)  
3. **Set command-specific defaults** (model selection, context version, etc.)
4. **Validate required parameters** (task_id, service, etc.)
5. **Build CLI argument array** for orchestrator execution
6. **Execute CLI** and return formatted response

---

## Runtime Auto-Detection Mechanisms

### Git Repository Introspection

| Data Point | Command | Fallback Behavior | Location |
|------------|---------|-------------------|----------|
| Repository URL | `git remote get-url origin` | Error if not found | `docs_generator.rs:get_git_remote_url()` |
| Current Branch | `git rev-parse --abbrev-ref HEAD` | Default to `"main"` | `docs_generator.rs:get_current_branch()` |
| Working Directory | `git rev-parse --show-toplevel` + path calculation | Use provided parameter | `docs_generator.rs:get_working_directory()` |

### Filesystem Detection

| Feature | Method | Purpose |
|---------|--------|---------|
| TaskMaster validation | Check for `.taskmaster` directory | Ensure project structure exists |
| Relative path calculation | Compare current dir to git root | Determine working directory for containers |
| Task file enumeration | Scan `.taskmaster/tasks/` directory | Build documentation context |

---

## Configuration Validation Rules

### Model Names
- **Validation**: Must start with `"claude-"`
- **Examples**: `claude-opus-4-20250514`, `claude-sonnet-4-20250514`
- **Location**: `tools/src/mcp/main.rs:181` and `tools/src/mcp/main.rs:291`

### Service Names  
- **Validation**: Kubernetes PVC naming rules
- **Pattern**: `^[a-z0-9-]+$` (lowercase letters, numbers, hyphens only)
- **Location**: `tools/src/mcp/main.rs:296`

### Task IDs
- **Validation**: Must be positive integer
- **Type**: `u32`
- **Location**: `tools/src/mcp/main.rs:214`

---

## Default Value Summary by Source

### Hard-Coded Defaults
- Model for docs: `"claude-opus-4-20250514"`
- Model fallback for API handler: `"claude-opus-4-20250514"`  
- Docs branch: `"main"`
- Context version: `1`
- Continue session: `false`
- Overwrite memory: `false`
- API URL: `http://orchestrator.orchestrator.svc.cluster.local/api/v1`
- Output format: `"table"`

### Environment Variable Defaults
- `FDL_DEFAULT_DOCS_USER`: GitHub username for documentation
- `FDL_DEFAULT_CODE_USER`: GitHub username for code tasks
- `DEFAULT_DOCS_MODEL`: Runtime model override
- `ORCHESTRATOR_API_URL`: API endpoint override

### Helm Chart Defaults  
- Default docs model: `"claude-opus-4-20250514"`
- Default code model: `"claude-sonnet-4-20250514"`
- Namespace: `"orchestrator"`
- Log level: `"debug"`
- Storage class: `"local-path"`
- Workspace size: `"10Gi"`

### Auto-Detected Values
- Repository URL (via `git remote get-url origin`)
- Current branch (via `git rev-parse --abbrev-ref HEAD`)
- Working directory (via git root + path calculation)

---

## Recommendations

### Current Strengths
1. **Clear hierarchy**: Environment variables → CLI parameters → auto-detection
2. **Intelligent defaults**: Reasonable fallbacks for optional parameters
3. **Validation**: Strong input validation at MCP layer
4. **Auto-detection**: Minimizes required user input

### Areas for Improvement
1. **Consistency**: Some defaults in MCP server, others in Helm charts
2. **Documentation**: Default values scattered across multiple files
3. **Override visibility**: Hard to track which defaults are in effect
4. **Model management**: Hard-coded model names should be configurable

### Proposed Centralization
Consider consolidating default values in a single configuration structure to improve maintainability and visibility of the configuration hierarchy.