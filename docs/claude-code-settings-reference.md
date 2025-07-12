# Claude Code Settings Reference

This document provides an accurate reference for how Claude Code settings are configured in our Orchestrator, based on the actual Rust implementation in `orchestrator/orchestrator-core/src/controllers/taskrun.rs`.

## Overview

Claude Code uses a hierarchical settings system with `settings.json` files. Our orchestrator generates these settings dynamically based on job type and API request parameters.

## Settings File Location

- **Container path**: `/workspace/{working_dir}/.claude/settings.local.json`
- **Reason for local**: Container-specific settings that don't get committed to git
- **Auto-generated**: Created by the orchestrator controller, not manually edited

## Job Type Detection

The orchestrator determines job type based on `task_id`:
- **Docs Generation**: `task_id == DOCS_GENERATION_TASK_ID` (999999)
- **Implementation**: All other `task_id` values

---

## Hard-Coded Settings by Job Type

### 📝 **Docs Generation Jobs** (task_id == 999999)

These settings are **completely hard-coded** and never change:

#### **Permissions (Hard-coded)**
```json
{
  "permissions": {
    "allow": [
      "Bash(git:*, gh:*, echo:*, ls:*, mkdir:*, cp:*, mv:*, rm:*, find:*, grep:*, cat:*, head:*, tail:*, curl:*, wget:*)",
      "Edit(*)",
      "Read(*)",
      "Write(*)",
      "MultiEdit(*)",
      "Glob(*)",
      "Grep(*)",
      "LS(*)",
      "WebSearch(*)",
      "WebFetch(*)"
    ],
    "deny": [
      "Bash(npm:install*, yarn:install*, cargo:install*, docker:*, kubectl:*, rm:-rf*)"
    ],
    "defaultMode": "acceptEdits"
  }
}
```

#### **Environment Variables (Dynamic based on telemetry config)**
```json
{
  "env": {
    "NODE_ENV": "production",
    "DISABLE_AUTOUPDATER": "1",
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",  // or "0" if telemetry disabled
    "OTEL_METRICS_EXPORTER": "otlp",     // or "none" if telemetry disabled
    "OTEL_LOGS_EXPORTER": "otlp",        // or "none" if telemetry disabled
    "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT": "http://{config.telemetry.otlp_endpoint}/v1/logs",
    "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL": "grpc",  // or "http/protobuf" based on config
    "DISABLE_COST_WARNINGS": "1",
    "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "1",
    "CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR": "true"
  }
}
```

#### **Model & Other Settings (Hard-coded)**
```json
{
  "model": "claude-opus-4-20250514",
  "cleanupPeriodDays": 3,
  "includeCoAuthoredBy": true,
  "hooks": {
    "onStop": "./.stop-hook-docs-pr.sh"
  }
}
```

### 🔧 **Implementation Jobs** (all other task_id values)

#### **Default Permissions (When agent_tools is empty)**
```json
{
  "permissions": {
    "allow": [
      "Bash(*)",
      "Edit(*)",
      "Read(*)",
      "Write(*)",
      "MultiEdit(*)",
      "Glob(*)",
      "Grep(*)",
      "LS(*)"
    ],
    "deny": [],
    "defaultMode": "promptUser"
  }
}
```

#### **Environment Variables (Dynamic based on config and retries)**
```json
{
  "env": {
    "NODE_ENV": "production",
    "DISABLE_AUTOUPDATER": "1",
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",  // or "0" if telemetry disabled
    "OTEL_METRICS_EXPORTER": "otlp",     // or "none" if telemetry disabled
    "OTEL_LOGS_EXPORTER": "otlp",        // or "none" if telemetry disabled
    "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT": "http://{config.telemetry.otlp_endpoint}/v1/logs",
    "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL": "grpc",  // or "http/protobuf" based on config
    "DISABLE_COST_WARNINGS": "0",
    "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "0",
    "CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR": "true",
    // On retries (attempts > 1):
    "BASH_DEFAULT_TIMEOUT_MS": "30000",  // 30 seconds
    "BASH_MAX_TIMEOUT_MS": "300000"     // 5 minutes
  }
}
```

#### **Model & Other Settings**
```json
{
  "model": "{tr.spec.model}",  // From API request, defaults to "sonnet"
  "cleanupPeriodDays": 7,
  "includeCoAuthoredBy": true
  // No hooks for implementation jobs
}
```

---

## Container Environment Variables (Separate from Settings)

These are set at the Kubernetes container level, **not in settings.json**:

### **Essential Container Env Vars (All Jobs)**
```yaml
env:
  - name: ANTHROPIC_API_KEY
    valueFrom:
      secretKeyRef:
        name: claude-api-key
        key: api-key
  - name: TASK_ID
    value: "{tr.spec.task_id}"
  - name: SERVICE_NAME
    value: "{tr.spec.service_name}"
  - name: AGENT_NAME
    value: "{tr.spec.agent_name}"
  - name: HOME
    value: "/workspace"
  - name: WORKDIR
    value: "/workspace"
```

### **GitHub Integration (When repository configured)**
```yaml
  - name: GITHUB_TOKEN
    valueFrom:
      secretKeyRef:
        name: "github-pat-{repository.github_user}"
        key: token
```

---

## API-Configurable Fields

### ✅ **Fields You CAN Modify via API**

Based on the `TaskRunSpec` structure, these fields can be set in your API request:

#### **Core Task Fields**
- `task_id` (u32) - Determines job type behavior
- `service_name` (String) - Target service name
- `agent_name` (String) - Agent identifier
- `model` (String) - Claude model (only affects implementation jobs)
- `context_version` (u32) - Version for updates

#### **Content & Context**
- `markdown_files` (Vec<MarkdownFile>) - Task content files
  - `filename` (String)
  - `content` (String)
  - `file_type` (Optional<MarkdownFileType>)

#### **Tool Permissions (Implementation Jobs Only)**
- `agent_tools` (Vec<AgentTool>) - **Completely overrides defaults**
  - `name` (String) - Tool name ("bash", "edit", "read", etc.)
  - `enabled` (bool) - Whether tool is enabled
  - `config` (Optional<Value>) - Tool-specific configuration
  - `restrictions` (Vec<String>) - Tool restrictions

#### **Repository Access**
- `repository` (Optional<RepositorySpec>)
  - `url` (String) - Repository URL
  - `branch` (String) - Branch to checkout (default: "main")
  - `github_user` (String) - GitHub username for auth
  - `token` (Optional<String>) - Reserved for future use

### ❌ **Fields You CANNOT Modify**

These are hard-coded in the controller:

#### **For Docs Generation Jobs (task_id == 999999)**
- **All permissions** - Research capabilities are fixed
- **All environment variables** - Optimized for documentation
- **Model** - Always `claude-opus-4-20250514`
- **Hooks** - Always includes post-completion hook
- **Default mode** - Always `acceptEdits`

#### **For All Jobs**
- **Telemetry configuration** - Controlled by controller config
- **Resource limits** - Set by Kubernetes configuration
- **Container environment** - API keys, workspace paths, etc.
- **Cleanup periods** - 3 days for docs, 7 days for implementation

---

## Agent Tools Override Behavior

### **How agent_tools Works**

```rust
if tr.spec.agent_tools.is_empty() {
    // Use job-type-specific defaults (shown above)
} else {
    // Completely replace defaults with your specification
    for tool in &tr.spec.agent_tools {
        if tool.enabled {
            // Add to allow rules based on tool.name
            // Apply tool.restrictions as deny rules
        }
    }
}
```

### **Example: Custom Tool Configuration**

```json
{
  "agent_tools": [
    {
      "name": "bash",
      "enabled": true,
      "restrictions": ["docker:*", "kubectl:*"]
    },
    {
      "name": "edit",
      "enabled": true
    },
    {
      "name": "websearch",
      "enabled": true
    }
  ]
}
```

**Result**: Only bash (with restrictions), edit, and websearch tools enabled. All other defaults are removed.

### **Supported Tool Names**
- `bash` - Shell command execution
- `edit` - File editing
- `read` - File reading
- `write` - File writing
- `multiedit` - Multi-file editing
- `glob` - File pattern matching
- `grep` - Text searching
- `webfetch` - HTTP requests
- `websearch` - Web search capabilities

---

## Telemetry Configuration

Telemetry settings are **dynamically generated** based on the controller configuration:

### **When Telemetry Enabled** (config.telemetry.enabled = true)
```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "1",
    "OTEL_METRICS_EXPORTER": "otlp",
    "OTEL_LOGS_EXPORTER": "otlp",
    "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT": "http://victoria-logs:4318/v1/logs",
    "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL": "http/protobuf"
  }
}
```

### **When Telemetry Disabled** (config.telemetry.enabled = false)
```json
{
  "env": {
    "CLAUDE_CODE_ENABLE_TELEMETRY": "0",
    "OTEL_METRICS_EXPORTER": "none",
    "OTEL_LOGS_EXPORTER": "none"
  }
}
```

### **Protocol Support**
- **GRPC**: Uses port 4317 and protocol "grpc"
- **HTTP**: Uses port 4318 and protocol "http/protobuf"

---

## Summary

### **Docs Generation** (task_id == 999999)
- ✅ **Hard-coded**: All permissions, environment, model, hooks
- ✅ **API Control**: Repository, working directory, markdown content
- ❌ **No Override**: Tool permissions, model selection, environment

### **Implementation Jobs** (all other task_ids)
- ✅ **API Control**: Model, tool permissions, repository, content
- ✅ **Override Capability**: Complete control via `agent_tools`
- ✅ **Flexible**: Defaults provided, but fully customizable

### **Configuration Sources**
- **settings.json**: Claude Code tool permissions, model, environment variables
- **Container env**: API keys, task metadata, workspace paths
- **Controller config**: Telemetry endpoints, resource limits, node affinity

This design ensures docs generation is consistent and optimized, while implementation jobs remain fully flexible for diverse development needs.