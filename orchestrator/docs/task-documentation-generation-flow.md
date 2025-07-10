# Task Documentation Generation Flow

This document traces the complete flow from CLI command to job execution for the Task Master documentation generation feature.

## Overview

The documentation generation flow consists of several stages:
1. **CLI Command** - User invokes `task init-docs`
2. **API Request** - CLI sends POST request to orchestrator service
3. **TaskRun Creation** - API handler creates TaskRun CRD in Kubernetes
4. **Controller Processing** - TaskRun controller manages job lifecycle
5. **Job Execution** - Kubernetes runs prep job and Claude agent job

## Detailed Flow

### 1. CLI Command (orchestrator-cli)

**File**: `orchestrator-cli/src/main.rs:154-183` and `commands.rs:680-982`

```bash
orchestrator task init-docs --dry-run
```

**Key Changes Made**:
- Removed `--taskmaster-dir` parameter (always uses `.taskmaster` within current directory)
- Changed default model from `sonnet` to `opus`
- Removed `--target-branch` parameter (auto-generates timestamp-based branch names)
- Auto-detects working directory relative to git repository root

**CLI Processing**:
1. **Directory Detection**: Auto-detects working directory relative to git repo root
2. **Branch Generation**: Creates unique target branch: `docs-generation-YYYYMMDD-HHMMSS`
3. **Git Operations**: Auto-commits and pushes `tasks.json` if needed
4. **Request Building**: Creates `DocsGenerationRequest` with:
   - Repository URL (auto-detected from git remote)
   - Working directory (current dir relative to repo root)
   - Source branch (current git branch)
   - Generated target branch name
   - Model selection (default: opus)

### 2. API Request (orchestrator-cli/src/api.rs)

**Endpoint**: `POST {base_url}/pm/docs/generate`

**API Client Code** (`api.rs:104-120`):
```rust
pub async fn submit_docs_generation(&self, request: &DocsGenerationRequest) -> Result<SimpleApiResponse> {
    let response = self
        .client
        .post(format!("{}/pm/docs/generate", self.base_url))
        .json(request)
        .send()
        .await?;
    
    self.handle_simple_response(response).await
}
```

**Request Structure** (`DocsGenerationRequest`):
```json
{
  "repository_url": "https://github.com/5dlabs/agent-platform.git",
  "working_directory": "example",
  "source_branch": "feature/example-project-and-cli",
  "target_branch": "docs-generation-20250710-201352",
  "service_name": "docs-generator",
  "agent_name": "claude-docs-opus",
  "model": "opus",
  "github_user": "swe-1-5dlabs",
  "task_id": null,
  "force": false,
  "dry_run": false
}
```

### 3. API Handler (orchestrator-core/src/handlers/pm_taskrun.rs)

**Handler Function**: `generate_docs()` (lines 766-936)

**Processing Steps**:
1. **TaskRun Generation**: Creates unique TaskRun name with timestamp
2. **Repository Spec**: Maps request to TaskRun repository specification
3. **Task Instructions**: Generates comprehensive CLAUDE.md with:
   - Repository and branch information
   - Working directory context
   - Step-by-step documentation generation instructions
   - Git workflow commands (add, commit, push, PR creation)
4. **Tool Configuration**: Configures available tools (bash, edit, read, write, glob)
5. **TaskRun Creation**: Creates TaskRun CRD in Kubernetes

**Special Task ID**: Uses `task_id: 999999` to indicate documentation generation task

### 4. TaskRun Controller (orchestrator-core/src/controllers/taskrun.rs)

**Main Controller**: `reconcile()` function (lines 162-210)

**Documentation Generation Logic** (lines 272-319):
```rust
// Special handling for documentation generation tasks (task_id = 999999)
if tr.spec.task_id == 999999 {
    info!("Documentation generation task detected, using minimal prep job");
    // Create docs-specific prep job
    let prep_job = build_docs_prep_job(&tr, &prep_job_name, config)?;
    // After prep succeeds, create Claude job
    create_claude_job(tr, jobs, taskruns, &cm_name, config).await?;
}
```

**Controller Flow**:
1. **PVC Creation**: Ensures workspace PVC exists for service
2. **ConfigMap Creation**: Creates ConfigMap with:
   - All markdown files (including CLAUDE.md instructions)
   - Claude Code settings.json for tool permissions
   - Export session script
3. **Prep Job**: Creates documentation-specific prep job
4. **Claude Job**: After prep succeeds, creates main Claude agent job
5. **Status Monitoring**: Tracks job status and updates TaskRun

### 5. Job Execution

#### Prep Job (Documentation-Specific)

**Function**: `build_docs_prep_job()` (lines 917-1103)

**Prep Job Script**:
```bash
#!/bin/sh
set -e

# Clone or update repository
if [ -d "/workspace/.git" ]; then
    # Update existing repo
    git reset --hard && git clean -fd
    git fetch origin && git checkout "$SOURCE_BRANCH"
    git pull origin "$SOURCE_BRANCH"
else
    # Clone repository
    git clone --branch "$SOURCE_BRANCH" $REPO_URL /workspace
fi

# Create documentation branch
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
DOC_BRANCH="docs/task-master-docs-$TIMESTAMP"
git checkout -b "$DOC_BRANCH"

# Configure git
git config user.email "claude@5dlabs.com"
git config user.name "Claude (5D Labs)"

# Copy ConfigMap files to appropriate location
if [ "$WORKING_DIR" != "." ]; then
    cp /config/* /workspace/$WORKING_DIR/.taskmaster/
else
    cp /config/* /workspace/.taskmaster/
fi
```

**Key Features**:
- Handles both new clones and existing repositories
- Creates unique documentation branch with timestamp
- Configures git for automated commits
- Places files in correct working directory

#### Claude Agent Job

**Function**: `build_claude_job()` (lines 636-800)

**Agent Configuration**:
- **Image**: Uses configured Claude Code image
- **Working Directory**: `/workspace`
- **Environment**:
  - `ANTHROPIC_API_KEY`: From Kubernetes secret
  - `GITHUB_TOKEN`: From user-specific GitHub PAT secret
  - Telemetry configuration (OTEL)
  - Task metadata (task_id, service_name, agent_name)

**Agent Tools**:
- `bash`: File operations and git commands
- `edit`: Modify existing files
- `read`: Read Task Master files
- `write`: Create new documentation files
- `glob`: Find files

**Startup Script** (`build_agent_startup_script()` lines 1151-1194):
```bash
# For docs generation tasks, use --cwd to restrict to working directory
if [ "$WORKING_DIR" != "." ]; then
    claude --cwd "/workspace/$WORKING_DIR" $CLAUDE_ARGS
else
    claude --cwd "/workspace" $CLAUDE_ARGS
fi
```

### 6. Documentation Generation Process

**Claude's Instructions** (from generated CLAUDE.md):

1. **Read Tasks**: Parse `.taskmaster/tasks/tasks.json`
2. **Generate Documentation**: For each task, create:
   - `task.md`: Comprehensive task overview
   - `prompt.md`: Autonomous AI agent prompt
   - `acceptance-criteria.md`: Test cases and criteria
3. **Git Workflow**:
   - Stage changes: `git add .`
   - Commit: `git commit -m "docs: auto-generate Task Master documentation"`
   - Push: `git push origin HEAD`
   - Create PR: `gh pr create --base SOURCE_BRANCH --title "..."`

### 7. Status Tracking

**TaskRun Status Updates**:
- `Pending`: TaskRun created
- `Preparing`: Prep job running
- `Running`: Claude agent job started
- `Succeeded`: Documentation generated successfully
- `Failed`: Error occurred

**Monitoring**: Controller checks job status every 30 seconds and updates TaskRun accordingly

## Architecture Benefits

1. **Separation of Concerns**: Prep job handles repository setup, Claude job handles documentation
2. **Workspace Isolation**: Each service gets dedicated PVC for persistence
3. **Security**: GitHub tokens auto-resolved by username, API keys from secrets
4. **Flexibility**: Working directory auto-detection supports multi-project repositories
5. **Observability**: Full telemetry integration with OpenTelemetry
6. **Git Integration**: Automated branch creation, commits, and PR generation

## File Locations Summary

| Component | File Path |
|-----------|-----------|
| CLI Commands | `orchestrator-cli/src/main.rs:154-183` |
| CLI Implementation | `orchestrator-cli/src/commands.rs:680-982` |
| API Client | `orchestrator-cli/src/api.rs:104-120` |
| API Handler | `orchestrator-core/src/handlers/pm_taskrun.rs:766-936` |
| Controller Logic | `orchestrator-core/src/controllers/taskrun.rs:272-319` |
| Prep Job Builder | `orchestrator-core/src/controllers/taskrun.rs:917-1103` |
| Claude Job Builder | `orchestrator-core/src/controllers/taskrun.rs:636-800` |
| TaskRun CRD | `orchestrator-core/src/crds/taskrun.rs` |

## Recent Improvements

1. **Simplified CLI**: Removed redundant parameters, auto-detection of directories
2. **Better Defaults**: Opus model for documentation, timestamp-based branches
3. **Working Directory Support**: Handles projects with `.taskmaster` in subdirectories
4. **Git Automation**: Auto-commits tasks.json changes before processing
5. **Unique Branch Names**: Prevents conflicts with timestamp-based naming

This flow provides a robust, automated system for generating comprehensive Task Master documentation with minimal user input and maximum flexibility.