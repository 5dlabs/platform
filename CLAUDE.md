# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Rust Projects (Orchestrator & Toolman)

**Build all components:**
```bash
# Build orchestrator components
cd orchestrator && cargo build --release

# Build toolman
cd toolman && cargo build --release
```

**Run tests:**
```bash
# Run orchestrator tests
cd orchestrator && cargo test

# Run toolman tests (including integration tests)
cd toolman && cargo test
cd toolman && cargo test --test integration
```

**Linting and quality checks:**
```bash
# ALWAYS run clippy before pushing code (required)
cd orchestrator && cargo clippy --all-targets --all-features -- -D warnings
cd toolman && cargo clippy --all-targets --all-features -- -D warnings
```

**Run specific services locally:**
```bash
# Start toolman HTTP server for local testing
cd toolman && cargo run --bin toolman-server -- --project-dir .

# Build CLI tools
cd orchestrator && cargo build --release --bin fdl --bin fdl-mcp

# Run single test
cd orchestrator && cargo test test_name
cd toolman && cargo test test_name
```

### Kubernetes and Infrastructure

**Deploy the platform:**
```bash
# Install CRDs first
kubectl apply -f infra/charts/orchestrator/crds/platform-crds.yaml

# Deploy orchestrator
helm install orchestrator infra/charts/orchestrator --namespace orchestrator --create-namespace

# Setup agent secrets (interactive)
./infra/scripts/setup-agent-secrets.sh
```

**Testing infrastructure:**
```bash
# Comprehensive testing suite
./infra/scripts/comprehensive-test.sh

# Test telemetry pipeline
./infra/scripts/test-telemetry-pipeline-v2.sh
```

## Architecture Overview

This is an **AI-powered development platform** with three main components:

### 1. **Orchestrator** (Rust)
- **Location**: `orchestrator/`
- **Purpose**: Kubernetes REST API service that creates CodeRun/DocsRun CRDs
- **Components**:
  - `core/`: Main orchestrator service with Kubernetes controllers
  - `tools/`: CLI tools (`fdl`, `fdl-mcp`) for direct API calls and MCP integration
  - `common/`: Shared models and utilities

### 2. **Toolman** (Rust) 
- **Location**: `toolman/`
- **Purpose**: MCP (Model Context Protocol) proxy and tool management
- **Key Features**: Session-based configuration, HTTP server, stdio wrapper
- **Binaries**: `toolman-client`, `toolman-server`

### 3. **Infrastructure** (Kubernetes/Helm)
- **Location**: `infra/`
- **Purpose**: Kubernetes deployment manifests, Helm charts, and operational scripts
- **Key Components**:
  - `charts/orchestrator/`: Main Helm chart with Claude agent templates
  - `claude-templates/`: Handlebars templates for agent configuration
  - `scripts/`: Deployment and testing automation

## Key Architectural Patterns

### Data Flow
1. **Cursor/Claude** calls `docs()` or `task()` via MCP protocol
2. **MCP server** (`fdl-mcp`) receives call and executes CLI (`fdl`)
3. **CLI** makes HTTP requests to orchestrator REST API (`/pm/tasks`)
4. **Orchestrator** creates CodeRun/DocsRun custom resources in Kubernetes
5. **Kubernetes controller** deploys Claude agents as Jobs with workspace isolation
6. **Agents** complete work and submit GitHub PRs

### Template System
- **Handlebars templates** in `infra/charts/orchestrator/claude-templates/`
- **Docs tasks**: Use `docs/` templates for documentation generation
- **Code tasks**: Use `code/` templates for task implementation
- **Templates control**: Agent settings, prompts, container scripts, hooks

### Workspace Management
- **Persistent volumes** for agent workspaces (`workspace-{service}`)
- **Dual-repo workflow**: Docs repository + target implementation repository
- **Session continuity**: Support for retrying failed tasks with context
- **Container isolation**: Each agent runs in isolated Kubernetes Jobs with dedicated resources

## Important Development Notes

### Pre-commit Requirements
- **MUST run clippy** with `-D warnings` flag before pushing
- **No warnings allowed** in CI pipeline
- Use the exact command: `cargo clippy --all-targets --all-features -- -D warnings`

## Kubernetes Controller Development Patterns

### Idempotent Reconciliation (CRITICAL)

**Problem**: Controllers can enter infinite reconciliation loops, creating duplicate resources and causing massive costs.

**Root Cause**: Status updates trigger reconciliation, which creates more resources, which trigger more status updates, creating an endless loop.

**Solution**: Implement proper state machine pattern with idempotent resource creation:

```rust
// ✅ CORRECT: Idempotent reconciliation pattern
async fn reconcile_create_or_update(resource: Arc<MyResource>, ctx: &Context) -> Result<Action> {
    let job_name = generate_deterministic_job_name(&resource);
    
    // Step 1: Check current job state BEFORE taking action
    let job_state = check_job_state(&jobs, &job_name).await?;
    
    match job_state {
        JobState::NotFound => {
            // Only create if no job exists
            create_job_and_resources(&resource).await?;
            update_status_if_changed(&resource, "Running", "Job started").await?;
            Ok(Action::requeue(Duration::from_secs(30)))
        }
        JobState::Running => {
            // Just monitor, don't create anything
            update_status_if_changed(&resource, "Running", "Job in progress").await?;
            Ok(Action::requeue(Duration::from_secs(30)))
        }
        JobState::Completed => {
            // CRITICAL: Use await_change() to STOP reconciliation
            update_status_if_changed(&resource, "Succeeded", "Job completed").await?;
            Ok(Action::await_change())  // This stops the loop!
        }
        JobState::Failed => {
            update_status_if_changed(&resource, "Failed", "Job failed").await?;
            Ok(Action::await_change())  // This stops the loop!
        }
    }
}

// ✅ CORRECT: Only update status when it actually changes
async fn update_status_if_changed(resource: &MyResource, new_phase: &str, message: &str) -> Result<()> {
    let current_phase = resource.status.as_ref().map(|s| s.phase.as_str()).unwrap_or("");
    
    if current_phase == new_phase {
        return Ok(()); // Skip update to prevent reconciliation trigger
    }
    
    // Use status subresource to avoid triggering spec reconciliation
    api.patch_status(&resource.name_any(), &PatchParams::default(), &patch).await?;
    Ok(())
}
```

**JobState Implementation**:
```rust
#[derive(Debug, Clone)]
pub enum JobState {
    NotFound,  // No job exists - safe to create
    Running,   // Job exists and active - monitor only
    Completed, // Job succeeded - final state, stop reconciling
    Failed,    // Job failed - final state, stop reconciling
}

async fn check_job_state(jobs: &Api<Job>, job_name: &str) -> Result<JobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                // Check Kubernetes job conditions
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        if condition.type_ == "Complete" && condition.status == "True" {
                            return Ok(JobState::Completed);
                        }
                        if condition.type_ == "Failed" && condition.status == "True" {
                            return Ok(JobState::Failed);
                        }
                    }
                }
            }
            Ok(JobState::Running)
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(JobState::NotFound),
        Err(e) => Err(e.into()),
    }
}
```

### Resource Cleanup Best Practices

**Automatic Pod Cleanup**: Use TTL controllers for completed workloads:
```rust
// In job spec - standard production practice
"spec": {
    "ttlSecondsAfterFinished": 600,  // Delete pods after 10 minutes
    "backoffLimit": 0,
    "template": { /* ... */ }
}
```

**Owner References**: Ensure proper parent-child relationships:
```rust
let owner_ref = OwnerReference {
    api_version: "orchestrator.platform/v1".to_string(),
    kind: "DocsRun".to_string(),
    name: docs_run.name_any(),
    uid: docs_run.metadata.uid.clone().unwrap_or_default(),
    controller: Some(true),
    block_owner_deletion: Some(true),
};
```

### Safe Mode for Debugging

**Problem**: Debugging controller issues can trigger expensive operations.

**Solution**: Implement safe mode in agent templates:
```handlebars
{{!-- In claude templates --}}
{{#unless (eq "false" "false")}}
<!-- Full task execution -->
{{else}}
## Safe Test Mode
What time is it? Please answer this simple question and exit immediately.
{{/unless}}
```

### Critical Anti-Patterns to Avoid

❌ **NEVER DO**: Create resources without checking if they exist  
❌ **NEVER DO**: Return `Action::requeue()` for completed states  
❌ **NEVER DO**: Update status on every reconciliation regardless of changes  
❌ **NEVER DO**: Use aggressive reconciliation intervals for completed work  

✅ **ALWAYS DO**: Check resource state before creation  
✅ **ALWAYS DO**: Use `Action::await_change()` for final states  
✅ **ALWAYS DO**: Conditional status updates only when state changes  
✅ **ALWAYS DO**: Implement deterministic resource naming  

### Debugging Reconciliation Issues

1. **Check controller logs** for reconciliation frequency
2. **Monitor resource creation** - should be 1:1, not 1:many
3. **Verify Action types** - await_change() vs requeue()
4. **Test with safe mode** first to prevent cost explosions
5. **Use resource watches** instead of polling where possible

### Configuration Files
- **MCP configuration**: `.cursor/mcp.json` for Cursor integration
- **Server configs**: `servers-config.json`, `client-config.json` in toolman
- **Agent templates**: All in `infra/charts/orchestrator/claude-templates/`
- **Cursor rules**: Structured rules in `.cursor/rules/` for development guidelines

### Testing Strategy
- **Unit tests**: `cargo test` for both orchestrator and toolman
- **Integration tests**: Specialized tests in `toolman/tests/integration/`
- **Infrastructure tests**: Comprehensive test suite in `infra/scripts/`
- **Local development**: Use `toolman-server` with `--project-dir .` for testing

### Development Workflow
- **Follow Cursor rules**: Use structured MDC files in `.cursor/rules/` for consistent development patterns
- **Taskmaster integration**: This platform is designed to work with Task Master AI for project planning and task management
- **Multi-context support**: Task Master supports tagged task lists for different features/branches

### Security and Licensing
- **AGPL-3.0 licensed**: Network service requires source code access compliance
- **Agent isolation**: Kubernetes-based workspace separation
- **Secret management**: Dedicated scripts for GitHub/API key setup

## Task Master Integration

This platform is designed to work with **Task Master AI** projects:
- **Documentation generation**: Analyzes `.taskmaster/` directories
- **Task implementation**: Implements specific task IDs from `tasks.json`  
- **Auto-PR creation**: All agent work results in GitHub pull requests
- **Context preservation**: Supports continuing failed/partial implementations
- **Tagged task lists**: Support for multi-context task management (features, branches, experiments)
- **Research integration**: Built-in AI research capabilities for up-to-date information beyond training cutoff

## MCP Tools Available

The platform provides the following MCP tools for integration with Cursor/Claude:

### Core Tools
- **`docs()`**: Generate comprehensive documentation for Task Master projects
- **`task()`**: Deploy autonomous Claude agents to implement specific tasks
- **`fdl-mcp`**: Primary MCP server that handles both tools above

### Supporting MCP Integrations
- **`task-master-ai`**: Full Task Master AI CLI integration for project planning
- **`toolman`**: Advanced MCP proxy and tool management capabilities
- **`rust-docs`**: Rust documentation search and reference

### Configuration
Configure MCP integration via `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "fdl-mcp": {
      "command": "fdl-mcp",
      "env": {
        "FDL_DEFAULT_DOCS_USER": "your-github-username"
      }
    },
    "task-master-ai": {
      "command": "npx",
      "args": ["-y", "task-master-ai"]
    }
  }
}
```