# Code Job Configuration Analysis

This document analyzes the complete flow of required fields, defaults, and auto-detection for Code Job implementation from MCP server to Kubernetes CRD.

## Component Flow Overview

```
MCP Server → CLI → Backend Handler → Kubernetes CRD → Controller
```

## Field Analysis by Component

### 1. Kubernetes CRD (CodeRunSpec)

**Location**: `orchestrator/core/src/crds/coderun.rs`

| Field | Required | Type | Default | Notes |
|-------|----------|------|---------|-------|
| `task_id` | ✅ | `u32` | - | Must be provided |
| `service` | ✅ | `String` | - | Must be provided |
| `repository_url` | ✅ | `String` | - | Must be provided |
| `docs_repository_url` | ✅ | `String` | - | Must be provided |
| `docs_project_directory` | ❌ | `Option<String>` | `None` | Optional |
| `working_directory` | ❌ | `Option<String>` | `None` | Optional |
| `model` | ✅ | `String` | - | Must be provided |
| `github_user` | ✅ | `String` | - | Must be provided |
| `local_tools` | ❌ | `Option<String>` | `None` | Optional |
| `remote_tools` | ❌ | `Option<String>` | `None` | Optional |
| `context_version` | ❌ | `u32` | `1` | Has CRD default function |
| `prompt_modification` | ❌ | `Option<String>` | `None` | Optional |
| `docs_branch` | ❌ | `String` | `"main"` | Has CRD default function |
| `continue_session` | ❌ | `bool` | `false` | Has CRD default function |
| `overwrite_memory` | ❌ | `bool` | `false` | Has CRD default function |
| `env` | ❌ | `HashMap<String, String>` | `{}` | Optional |
| `env_from_secrets` | ❌ | `Vec<SecretEnvVar>` | `[]` | Optional |

### 2. Backend Handler (submit_code_task)

**Location**: `orchestrator/core/src/handlers/code_handler.rs`

**Defaults/Fallbacks Applied**:
- `model`: Falls back to `DEFAULT_CODE_MODEL` environment variable, then hard-coded `"claude-sonnet-4-20250514"`

**Processing**:
- Transforms `CodeRequest` into `CodeRunSpec`
- Maps `env_from_secrets` from common model to CRD format
- Creates CodeRun with generated name: `code-{task_id}-{timestamp}`
- Sets initial status to "Pending"

### 3. Common Models (CodeRequest)

**Location**: `orchestrator/common/src/models/code_request.rs`

| Field | Required | Type | Default | Serde Default Function |
|-------|----------|------|---------|----------------------|
| `task_id` | ✅ | `u32` | - | - |
| `service` | ✅ | `String` | - | - |
| `repository_url` | ✅ | `String` | - | - |
| `docs_repository_url` | ✅ | `String` | - | - |
| `docs_project_directory` | ❌ | `Option<String>` | `None` | - |
| `working_directory` | ❌ | `Option<String>` | `None` | - |
| `model` | ❌ | `Option<String>` | `None` | - |
| `github_user` | ✅ | `String` | - | - |
| `local_tools` | ❌ | `Option<String>` | `None` | `#[serde(default)]` |
| `remote_tools` | ❌ | `Option<String>` | `None` | `#[serde(default)]` |
| `context_version` | ❌ | `u32` | `1` | `#[serde(default = "default_context_version")]` |
| `prompt_modification` | ❌ | `Option<String>` | `None` | `#[serde(default)]` |
| `docs_branch` | ❌ | `String` | `"main"` | `#[serde(default = "default_docs_branch")]` |
| `continue_session` | ❌ | `bool` | `false` | `#[serde(default)]` |
| `overwrite_memory` | ❌ | `bool` | `false` | `#[serde(default)]` |
| `env` | ❌ | `HashMap<String, String>` | `{}` | `#[serde(default)]` |
| `env_from_secrets` | ❌ | `Vec<SecretEnvVar>` | `[]` | `#[serde(default)]` |

### 4. CLI (handle_code_command)

**Location**: `orchestrator/tools/src/cli/commands.rs`

**Auto-Detection Performed**:
- `repository_url`: Auto-detected via `git remote get-url origin` if not provided
- `docs_repository_url`: Auto-detected via `git remote get-url origin` if not provided (TODO: should be configurable)
- `working_directory`: Auto-detected via `git rev-parse --show-toplevel` and relative path calculation if not provided

**Parameter Processing**:
- All optional parameters passed through as-is
- Model parameter passed as `Option<String>` (no CLI defaults)
- Environment variable parsing from comma-separated strings
- Secret environment variable parsing from `name:secretName:secretKey` format

### 5. MCP Server (task tool)

**Location**: `orchestrator/tools/src/mcp/main.rs` (handle_orchestrator_tools - "task")

**Required by MCP Schema**:
- `task_id` (integer, minimum: 1)
- `service` (string, pattern: `^[a-z0-9-]+$`)
- `working_directory` (string)

**MCP Schema Defaults** (from tools.rs):
- `context_version`: `1`
- `docs_branch`: `"main"`
- `continue_session`: `false`
- `overwrite_memory`: `false`
- `model`: Hard-coded default `"claude-sonnet-4-20250514"` (ISSUE: should be removed)

**Environment Variables**:
- `github_user`: Uses `FDL_DEFAULT_CODE_USER` environment variable if parameter not provided

**Validation**:
- Model must start with "claude-"
- Service name must contain only lowercase letters, numbers, and hyphens

## Configuration Issues Identified

### 1. Hard-coded Model Default in MCP Server
**Current**: MCP server has hard-coded `"claude-sonnet-4-20250514"` default
**Issue**: Anti-pattern - should use Helm configuration like docs command
**Fix Needed**: Remove hard-coded default, let backend handle via `DEFAULT_CODE_MODEL` environment variable

### 2. Inconsistent Model Requirement
**Current**: MCP server requires model parameter, but backend has fallback
**Issue**: MCP tool schema shows model as having a default, but implementation requires it
**Fix Needed**: Make model optional in MCP server, align with docs command pattern

### 3. Hard-coded Fallback in Backend
**Current**: Backend falls back to hard-coded `"claude-sonnet-4-20250514"` if env var not set
**Issue**: Should require `DEFAULT_CODE_MODEL` environment variable like docs handler
**Fix Needed**: Remove hard-coded fallback, require environment variable

## Responsibility Matrix

| Component | Defaults | Auto-Detection | Validation | Environment Variables |
|-----------|----------|----------------|------------|---------------------|
| **MCP Server** | ❌ Should not have | ❌ | ✅ Basic (model format, service name) | ✅ `FDL_DEFAULT_CODE_USER` |
| **CLI** | ❌ Pass-through only | ✅ Git operations | ✅ Parsing | ❌ |
| **Backend Handler** | ⚠️ Model only (should use env) | ❌ | ❌ | ✅ `DEFAULT_CODE_MODEL` |
| **Common Models** | ✅ Serde defaults | ❌ | ❌ | ❌ |
| **CRD** | ✅ Field defaults | ❌ | ❌ | ❌ |
| **Helm Config** | ✅ Env vars | ❌ | ❌ | ✅ Values to env vars |

## Required vs Optional Fields Summary

### Truly Required (No Defaults Possible)
- `task_id`: Specifies which task to implement
- `service`: Creates workspace PVC name
- `repository_url`: Target implementation repository
- `docs_repository_url`: Source of task definitions
- `github_user`: Authentication and commit author

### Auto-Detected by CLI
- `repository_url`: From git remote
- `docs_repository_url`: From git remote (should be configurable)
- `working_directory`: From git working directory

### Has Meaningful Defaults
- `model`: Should default from Helm configuration
- `context_version`: `1` (first attempt)
- `docs_branch`: `"main"` (standard branch)
- `continue_session`: `false` (new session)
- `overwrite_memory`: `false` (preserve memory)

### Optional/Advanced Configuration
- `docs_project_directory`: Project-specific path
- `working_directory`: Can be auto-detected
- `local_tools`/`remote_tools`: MCP tool configuration
- `prompt_modification`: Retry context
- `env`/`env_from_secrets`: Container environment

## Recommendations

1. **Centralize Model Defaults**: Move code model default to Helm configuration
2. **Remove Hard-coded Fallbacks**: Require environment variables for defaults
3. **Consistent Optional Pattern**: Make model optional in MCP server like docs command
4. **Improve Auto-Detection**: Make docs repository URL configurable instead of assuming same as target repo
5. **Schema Alignment**: Ensure MCP tool schema reflects actual implementation behavior