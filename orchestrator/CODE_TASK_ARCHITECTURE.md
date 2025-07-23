# Code Task Architecture Documentation

## Overview

Code tasks are implementation tasks that execute actual code development for specific Task Master tasks. They utilize a **focused, lightweight approach** where the code agent receives only the minimal context needed for the specific task at hand, avoiding large files and unnecessary complexity.

## Architecture Components

### Dual Repository Pattern

Code tasks operate across two distinct repositories:

1. **Platform Repository** (Source of Truth)
   - Contains task-specific documentation in `.taskmaster/docs/task-N/`:
     - `task.md` - Task requirements and description
     - `acceptance-criteria.md` - Success criteria and validation
     - `architecture.md` - Technical approach and guidance
   - Example: `git@github.com:5dlabs/platform.git`
   - **Note**: The large `tasks.json` file is NOT copied to avoid token consumption

2. **Destination Repository** (Implementation Target)
   - Where actual code implementation occurs
   - Receives **only specific task files** copied to `task/` directory
   - **No `.taskmaster/` directory** - keeps agent focused on implementation
   - **No `tasks.json`** - avoids token consumption from large task lists
   - Agent works in specified working directory (e.g., `services/trader/`)
   - Creates feature branch: `feature/task-N-implementation`
   - Example: `git@github.com:5dlabs/trader.git`

### Task Documentation Structure

**In Platform Repository:**
```
.taskmaster/docs/task-N/
├── task.md                    # Task requirements and description
├── acceptance-criteria.md     # Success criteria and validation
└── architecture.md           # Technical approach and guidance
```

**Copied to Destination Repository:**
```
target-repo/
├── task/                      # Copied task documentation
│   ├── task.md
│   ├── acceptance-criteria.md
│   └── architecture.md
├── docs/                      # Created on first run (if needed)
├── [working-directory]/       # Agent works here (e.g., services/trader/)
├── coding-guidelines.md       # From templates
├── github-guidelines.md       # From templates
├── CLAUDE.md                  # Agent memory file
└── .claude/
    └── settings.local.json    # Claude Code settings
```

## CodeRun CRD Structure

The `CodeRun` Custom Resource Definition orchestrates code task execution:

```yaml
apiVersion: orchestrator.platform/v1
kind: CodeRun
metadata:
  name: task-N-service-attempt-M
  namespace: orchestrator
spec:
  # Required fields
  taskId: N                                    # Task identifier
  service: "service-name"                      # Target service name
  repository: "git@github.com:org/repo.git"   # Destination repository
  branch: "main"                              # Base branch for feature branch
  githubUser: "bot-user"                      # GitHub user for commits

  # Optional fields
  workingDirectory: "services/trader"         # Where agent works in repo (defaults to service name)

  # Platform repository (for task docs)
  platformRepository: "git@github.com:5dlabs/platform.git"
  platformBranch: "main"                      # Branch containing task docs

  # Optional retry/continuation fields
  contextVersion: 1                           # For tracking retries
  promptModification: "additional context"    # Extra instructions for subsequent runs
  promptMode: "append"                        # append|replace - how to handle prompt updates
  resumeSession: false                        # Whether to use --resume flag when starting Claude
  overwriteMemory: false                      # Preserve CLAUDE.md by default
```

### Field Behavior Notes

**Working Directory Defaults**:
- If `workingDirectory` is not specified, it defaults to the `service` name
- Example: `service: "trader"` → `workingDirectory: "trader"`
- This covers the common case where service directories match service names
- Override only when the service code is in a different directory structure

**Prompt Management on Subsequent Runs**:
- `promptMode: "append"`: Add `promptModification` content to existing prompt
- `promptMode: "replace"`: Replace entire prompt with `promptModification` content
- **Use Case**: Providing additional context, corrections, or new requirements

**Claude Session Control**:
- `resumeSession: false` (default): Start fresh Claude session
- `resumeSession: true`: Use `--resume` flag to continue previous Claude session
- **Experimental**: Both approaches need testing to determine effectiveness

## Template Rendering System

### Server-Side Processing

The orchestrator controller renders Handlebars templates with task-specific data:

```
📂 ConfigMap: Raw Templates (/claude-templates/code/)
├── CLAUDE.md.hbs                 # Agent memory and task pointer
├── container.sh.hbs              # Container initialization script
├── coding-guidelines.md.hbs      # Project coding standards
├── github-guidelines.md.hbs      # Git workflow standards
├── settings.json.hbs             # Claude Code settings
├── mcp.json.hbs                  # MCP server configuration
├── client-config.json.hbs        # MCP client configuration
├── prompt.hbs                    # Main agent prompt
└── hooks/
    ├── early-test.sh.hbs         # Pre-implementation testing
    └── stop-commit.sh.hbs        # Post-implementation workflow
```

### Template Data Context

Templates are rendered with task-specific data:

```handlebars
{{task_id}}                    # e.g., "1"
{{repository_url}}             # e.g., "git@github.com:org/service.git"
{{platform_repository_url}}    # e.g., "git@github.com:5dlabs/platform.git"
{{working_directory}}          # e.g., "services/trader"
{{branch}}                     # e.g., "main"
{{github_user}}               # e.g., "swe-bot"
{{service}}                   # e.g., "trader"
```

### Claude Memory Persistence

**CLAUDE.md Memory Management**:
- **First Run**: `CLAUDE.md` rendered from template with initial task context and guidance
- **Subsequent Runs**: `CLAUDE.md` preserved by default to maintain Claude's accumulated memory
- **CRD Field**: `overwriteMemory` (defaults to `false`) controls this behavior
- **Use Cases for Overwriting**:
  - Major task requirement changes
  - Intentional fresh start after failed attempts
  - Task context has fundamentally changed

**Why This Matters**:
- Claude writes its learning, discoveries, and decision rationale to `CLAUDE.md`
- Preserving this file allows Claude to build on previous attempts
- Without persistence, each retry starts from scratch, losing valuable context

### File Mounting Strategy

**Static Files (ConfigMap + Subpath Pattern)**:
Files that don't change during agent execution are mounted via ConfigMap subpaths:

```
📂 ConfigMap: Rendered Static Files → Container Mounts
├── code_container.sh          → /scripts/container.sh
├── code_coding-guidelines.md  → /workspace/coding-guidelines.md
├── code_github-guidelines.md  → /workspace/github-guidelines.md
├── code_settings-local.json   → /.claude/settings.local.json
├── code_mcp.json             → /.claude/mcp.json
├── code_client-config.json   → /.claude/client-config.json
├── code_prompt.txt           → /workspace/prompt.txt
├── code_hooks_early-test.sh  → /hooks/early-test.sh
└── code_hooks_stop-commit.sh → /hooks/stop-commit.sh
```

**Dynamic Files (Special Handling)**:
- `CLAUDE.md`: Requires memory persistence logic (not simple ConfigMap mounting)
- Task files: Copied during container initialization from platform repo

**Environment Variables**:
- `MCP_CLIENT_CONFIG`: Must point to mounted client-config.json location
- Required for toolman to locate tool permissions configuration

## Container Initialization Flow

**⚠️ CRITICAL**: This flow extends the proven DocsRun CRD pattern. Use the working docs implementation as the foundation to avoid re-debugging solved problems.

### 1. Platform Repository Setup
```bash
# Clone platform repository for task docs
git clone $PLATFORM_URL platform-repo
cd platform-repo && git checkout $PLATFORM_BRANCH

# Copy ONLY the specific task files (no tasks.json)
mkdir -p task
cp platform-repo/.taskmaster/docs/task-N/task.md task/
cp platform-repo/.taskmaster/docs/task-N/acceptance-criteria.md task/
cp platform-repo/.taskmaster/docs/task-N/architecture.md task/
```

### 2. Sophisticated Destination Repository Setup
```bash
# Clone destination repository
git clone $REPO_URL target-repo
cd target-repo

# Ensure main branch is current
git checkout main
git pull origin main

# Create or checkout feature branch
FEATURE_BRANCH="feature/task-${TASK_ID}-implementation"
if git show-ref --verify --quiet refs/heads/$FEATURE_BRANCH; then
    # Feature branch exists - checkout and try to merge main
    git checkout $FEATURE_BRANCH
    git merge main

    # Check for merge conflicts
    if [ $? -ne 0 ]; then
        echo "Merge conflicts detected. Performing fresh clone..."
        cd ..
        rm -rf target-repo
        git clone $REPO_URL target-repo
        cd target-repo
        git checkout -b $FEATURE_BRANCH
    fi
else
    # Create new feature branch from main
    git checkout -b $FEATURE_BRANCH
fi

# Ensure we're in the correct working directory for Claude
cd $WORKING_DIRECTORY  # Critical: Claude uses launch directory as CWD
```

### 2. Target Repository Setup
```bash
# Clone or reuse destination repository
git clone $REPO_URL target-repo
cd target-repo

# Create or resume feature branch
FEATURE_BRANCH="feature/task-N-implementation"
if [ "$CURRENT_BRANCH" = "$FEATURE_BRANCH" ]; then
    # Retry scenario - clean up existing work
    git reset --hard HEAD && git clean -fd
else
    # Fresh task - create new feature branch
    git checkout $BASE_BRANCH
    git checkout -b $FEATURE_BRANCH
fi
```

### 3. Working Directory Setup
```bash
# Navigate to working directory (e.g., services/trader/)
cd $WORKING_DIRECTORY

# Copy rendered configuration files
cp /config/code_CLAUDE.md ./CLAUDE.md
cp /config/code_coding-guidelines.md ./coding-guidelines.md
cp /config/code_github-guidelines.md ./github-guidelines.md

# Setup Claude Code environment
mkdir -p .claude
cp /config/code_settings-local.json ./.claude/settings.local.json
```

### 4. First Run Setup
```bash
# On first run for a service, create docs/ directory if needed
if [ ! -d "docs/" ]; then
    mkdir -p docs/
    echo "# Service Documentation" > docs/README.md
fi

# CLAUDE.md points to the docs/ directory for project context
```

## MCP Tool Integration

### MCP Architecture Overview

The agent uses an MCP aggregator called **toolman** to access external tools while maintaining clean separation between local and remote capabilities.

### Local vs Remote MCP Servers

**Remote MCP Aggregator (toolman)**:
- Central aggregator service running separately from agent container
- Provides tools like `brave-search_brave_web_search`, `memory_create_entities`, `rustdocs_query_rust_docs`
- Agent communicates with aggregator via network
- Typically only ONE server configured due to aggregator pattern

**Local MCP Servers**:
- MCP servers running within the agent container itself
- Used for edge cases requiring container-local capabilities
- Handled by binary included in Claude image
- Same binary manages both local servers and remote aggregator communication

### Configuration Files

**mcp.json** (Server Configuration):
```json
{
  "mcpServers": {
    "local": {
      // Local MCP servers running in agent container (if needed)
    },
    "remote": {
      "toolman": {
        "command": "toolman",
        "transport": {
          "type": "stdio"
        }
      }
    }
  }
}
```
- Usually minimal due to aggregator pattern (typically just toolman)
- Distinguishes between "local" and "remote" server sections
- Toolman uses standard I/O transport MCP server

**client-config.json** (Tool Permissions & Filtering):
```json
{
  "remoteTools": [
    "brave-search_brave_web_search",
    "memory_create_entities",
    "rustdocs_query_rust_docs"
  ]
}
```
- **Purpose**: Controls which tools from which servers are exposed to agent
- **Benefit**: Prevents context pollution by limiting available tools
- **Function**: Ensures agent only sees relevant capabilities for the task

**Environment Variables**:
- `MCP_CLIENT_CONFIG`: Points to client-config.json location
- Must be set correctly in container templates for toolman to find configuration

## Agent Focus Strategy

### What the Agent Sees
- `task/task.md` - Current task requirements
- `task/acceptance-criteria.md` - Success criteria
- `task/architecture.md` - Technical guidance
- `coding-guidelines.md` - Project coding standards
- `github-guidelines.md` - Git workflow standards
- `CLAUDE.md` - Points to docs/ for project context

### What the Agent DOESN'T See
- ❌ `tasks.json` - Large file avoided for token efficiency
- ❌ `.taskmaster/` structure - Keeps agent focused on implementation
- ❌ Other task documentation - Prevents confusion and scope creep
- ❌ Platform repository details - Only relevant task docs copied

### Memory Management
- `CLAUDE.md` provides persistent memory across retries
- Points to `docs/` directory for project-wide context
- Task-specific memory preserved in working directory
- Previous task context available in memory but not files

## Execution Flow

### 1. Container Startup
1. **PVC Mounting**: Persistent Volume Claims mounted for reliable work persistence
2. **Static File Mounting**: ConfigMaps mounted via subpaths to appropriate locations
3. **Authentication Setup**: SSH configuration for repository access
4. **Platform Repository Clone**: Clone for task documentation extraction
5. **Task-Specific File Copying**: Copy only current task docs (no tasks.json)
6. **Sophisticated Repository Setup**: Advanced branch management with conflict handling
7. **Working Directory Configuration**: Set up service-specific workspace
8. **CLAUDE.md Memory Handling**: Apply persistence logic based on `overwriteMemory` flag
9. **MCP Tool Initialization**: Start MCP servers with mounted configurations

### 2. Agent Execution
1. **Prompt Preparation**:
   - Load base prompt from template
   - If `promptMode: "append"`: Add `promptModification` to existing prompt
   - If `promptMode: "replace"`: Use `promptModification` as entire prompt
   - **Always append**: Strong overconfidence mitigation blurb to combat Claude's tendency to declare success prematurely
2. **Claude Session Start**:
   - **Critical**: Ensure current directory is `$WORKING_DIRECTORY`
   - If `resumeSession: false`: Start fresh Claude session
   - If `resumeSession: true`: Use `--resume` flag to continue previous session
3. **Task Implementation**:
   - Read `task/task.md` for requirements
   - Review `task/acceptance-criteria.md` for success criteria
   - Check `task/architecture.md` for technical approach
   - Use `coding-guidelines.md` for project standards
   - Implement solution in working directory
   - Test against acceptance criteria
   - Commit changes to feature branch (see Git Commit Strategy for monitoring in Future Considerations)

### 3. Hook-Based Workflow
- **early-test.sh**: Pre-implementation validation
- **stop-commit.sh**: Post-implementation commit and PR workflow

## Agent Behavior Management

### Overconfidence Mitigation
**Problem**: Claude has a strong tendency to declare task completion before proper verification
**Solution**: Mandatory system prompt enhancement applied to all code tasks
**Implementation**: Strongly worded blurb appended to every system prompt
**Language Requirements**: Must use strongest possible language to combat this behavioral pattern
**Application**: Universal - applies to all code task executions
**Purpose**: Ensure thorough testing and verification before claiming success

## Key Architectural Principles

### 1. **Focused Context**
- Agent receives only task-specific documentation
- No large files (tasks.json) to consume tokens unnecessarily
- Working directory isolation for clean implementation

### 2. **Lightweight Destination**
- No `.taskmaster/` tooling in destination repository
- Minimal file copying (only current task docs)
- Standard service repository structure maintained

### 3. **Template-Driven Configuration**
- Server-side rendering of all configuration files
- Handlebars templates with task-specific data
- Consistent agent setup across all tasks

### 4. **MCP Tool Enhancement**
- External tools via toolman aggregator for specialized capabilities
- Local vs remote MCP server architecture for flexibility
- Tool permissions filtering prevents context pollution

### 5. **Branch-Based Workflow**
- Feature branch per task: `feature/task-N-implementation`
- Retry support through branch reuse and cleanup
- Git workflow hooks for automation

### 6. **Proven ConfigMap Pattern**
- Static files mounted via ConfigMap + subpath (successful pattern from docs CRD)
- Reliable, performant file distribution to agent containers
- Clean separation between static configuration and dynamic state (CLAUDE.md)

### 7. **PVC-Based Work Persistence**
- All agent work performed on Persistent Volume Claims (PVC)
- Work is never lost even if agent container stops or fails
- Git commits serve monitoring/visibility purpose, not backup
- Ensures reliable task completion across container restarts

### 8. **Docs CRD Pattern Reuse**
- **CRITICAL**: Leverage working DocsRun CRD as foundation for CodeRun implementation
- File locations, initialization scripts, and container setup patterns are proven and debugged
- Extend the docs pattern rather than recreating from scratch
- Same fundamental container initialization flow with code-specific extensions
- **Do NOT debug the same issues twice** - the docs implementation provides the working blueprint

This architecture ensures the code agent remains focused, efficient, and capable while avoiding the complexity and token overhead of larger task management systems.

---

## Future Implementation Considerations

### Task Lifecycle Management
**⚠️ Approach To Be Determined**

Once we have successful task completions, we'll need to address:

#### Task Content Cleanup Strategy
- **Current State**: Task files persist across runs for debugging and context
- **Future Need**: Clean up completed task content to avoid confusion
- **Implementation Blocker**: Need to establish QA process and completion verification
- **Considerations**:
  - When to safely remove task files
  - How to preserve context for debugging failed tasks
  - Integration with PR/review workflow

#### Documentation Update Strategy
- **Current State**: ConfigMap pattern handles prompt updates via controller
- **Future Challenge**: Updates to task instructions or design specifications
- **Current Capability**: `promptModification` field for additional context
- **Future Need**: Strategy for fundamental task requirement changes

#### Git Commit Strategy for Code Tasks
- **Primary Goal**: Visibility and monitoring of agent progress
- **Use Case**: Track agent behavior during remote cluster execution (most tasks are long-running)
- **Business Problem**: Detect if Claude is "going off the rails and wasting tokens"
- **Why GitHub**: Best visibility mechanism for remote cluster work
- **Data Persistence**: Work is safe on PVC, commits are for monitoring not backup
- **Potential Approaches**:
  - **Turn-based commits**: Commit and push after each significant change
  - **Hook-based commits**: Use hooks to trigger commits at intervals
  - **Prompt-driven strategy**: Instructions in prompt for when to commit
- **Target**: Feature branch only (never direct to main)
- **Implementation Status**: Requires research and experimentation
- **Next Steps**:
  - Test different commit strategies once agent is operational
  - Determine optimal commit frequency for effective monitoring
  - Evaluate balance between visibility and git history noise

### Critical Implementation Details

#### Repository Conflict Resolution
- **Merge Conflicts**: Detected automatically during branch setup
- **Resolution Strategy**: Delete and fresh clone (AI-based resolution too complex)
- **Risk Mitigation**: Ensures clean state but loses any local changes

#### Claude Working Directory Requirement
- **Critical**: Claude must be launched from the working directory
- **Reason**: Claude treats launch directory as current working directory
- **Implementation**: `cd $WORKING_DIRECTORY` before starting Claude session
- **Inherited**: Same pattern successfully used in docs CRD