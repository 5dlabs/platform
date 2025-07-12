# Orchestrator Flow Diagram

This document provides a visual representation of the complete flow from MCP server to job execution, including all configuration details and data transformations.

## Overview Flow

```mermaid
graph TB
    subgraph "User Environment"
        MCP["MCP Server<br/>init_docs tool"]
        CLI["Orchestrator CLI<br/>task init-docs"]
    end

    subgraph "API Layer"
        API["Orchestrator API<br/>/pm/docs/generate"]
        HANDLER["PM Handler<br/>generate_docs()"]
    end

    subgraph "Kubernetes Control Plane"
        TASKRUN_CRD["TaskRun CRD<br/>docs-gen-{timestamp}"]
        CONTROLLER["TaskRun Controller<br/>reconcile()"]
    end

    subgraph "Generated Resources"
        CONFIGMAP["ConfigMap<br/>settings + files"]
        PVC["PersistentVolumeClaim<br/>workspace-{service}"]
        PREP_JOB["Prep Job<br/>Repository setup"]
        CLAUDE_JOB["Claude Job<br/>Documentation generation"]
    end

    subgraph "Job Execution"
        PREP_POD["Prep Container<br/>alpine/git"]
        CLAUDE_POD["Claude Container<br/>claude-code:latest"]
        HOOK["Post-completion Hook<br/>.stop-hook-docs-pr.sh"]
    end

    MCP -->|"init_docs args"| CLI
    CLI -->|"DocsGenerationRequest"| API
    API -->|"validate & transform"| HANDLER
    HANDLER -->|"create TaskRun"| TASKRUN_CRD
    TASKRUN_CRD -->|"watch event"| CONTROLLER

    CONTROLLER -->|"build resources"| CONFIGMAP
    CONTROLLER -->|"ensure exists"| PVC
    CONTROLLER -->|"create"| PREP_JOB
    PREP_JOB -->|"on success"| CLAUDE_JOB

    PREP_JOB -->|"mount volumes"| PREP_POD
    CLAUDE_JOB -->|"mount volumes"| CLAUDE_POD
    CLAUDE_POD -->|"on completion"| HOOK

    classDef userEnv fill:#e1f5fe
    classDef apiLayer fill:#f3e5f5
    classDef k8sControl fill:#e8f5e8
    classDef resources fill:#fff3e0
    classDef execution fill:#fce4ec

    class MCP,CLI userEnv
    class API,HANDLER apiLayer
    class TASKRUN_CRD,CONTROLLER k8sControl
    class CONFIGMAP,PVC,PREP_JOB,CLAUDE_JOB resources
    class PREP_POD,CLAUDE_POD,HOOK execution
```

## Detailed Configuration Flow

```mermaid
graph TB
    subgraph "MCP Server Input"
        MCP_ARGS["MCP init_docs Arguments:<br/>• working_directory: string<br/>• model: 'opus' (default)<br/>• force: bool<br/>• task_id: Optional[u32]<br/>• dry_run: bool"]
    end

    subgraph "CLI Processing"
        CLI_DETECT["Auto-detection:<br/>• repository_url (git remote)<br/>• source_branch (git branch)<br/>• github_user (from config)<br/>• working_directory (relative to repo)"]
        CLI_REQ["DocsGenerationRequest:<br/>• repository_url: string<br/>• working_directory: string<br/>• source_branch: string<br/>• target_branch: docs-gen-{timestamp}<br/>• service_name: 'docs-generator'<br/>• agent_name: 'claude-docs-{model}'<br/>• model: string<br/>• github_user: string<br/>• task_id: Option[u32]<br/>• force: bool<br/>• dry_run: bool"]
    end

    subgraph "API Handler Transformation"
        TASKRUN_SPEC["TaskRunSpec:<br/>• task_id: 999999 (DOCS_GENERATION_TASK_ID)<br/>• service_name: 'docs-generator'<br/>• agent_name: 'claude-docs-{model}'<br/>• model: '{model}'<br/>• context_version: 1<br/>• repository: RepositorySpec<br/>• markdown_files: [CLAUDE.md]"]

        REPO_SPEC["RepositorySpec:<br/>• url: repository_url<br/>• branch: source_branch<br/>• github_user: github_user<br/>• token: None"]

        CLAUDE_MD["CLAUDE.md Content:<br/>• Repository info<br/>• Working directory<br/>• Branch details<br/>• Step-by-step instructions<br/>• Git workflow commands"]
    end

    subgraph "Controller Resource Generation"
        SETTINGS_GEN["Settings Generation:<br/>is_docs_generation(tr) = true<br/>↓<br/>Hard-coded permissions<br/>Research + Git tools<br/>Web search enabled<br/>Model: claude-opus-4-20250514<br/>Hooks: .stop-hook-docs-pr.sh<br/>defaultMode: acceptEdits"]

        CONFIGMAP_DATA["ConfigMap Data:<br/>• settings-local.json<br/>• CLAUDE.md<br/>• .stop-hook-docs-pr.sh"]

        JOB_CONFIG["Job Configuration:<br/>• Prep Job: alpine/git<br/>• Claude Job: claude-code:latest<br/>• PVC: workspace-{service}<br/>• Secrets: github-pat-{user}"]
    end

    subgraph "Runtime Configuration"
        PREP_ENV["Prep Container Env:<br/>• GITHUB_TOKEN (from secret)<br/>• Repository cloning<br/>• Branch creation<br/>• File copying"]

        CLAUDE_ENV["Claude Container Env:<br/>• ANTHROPIC_API_KEY (secret)<br/>• GITHUB_TOKEN (secret)<br/>• TASK_ID: 999999<br/>• SERVICE_NAME: docs-generator<br/>• HOME: /workspace<br/>• Working dir: /workspace/{working_dir}"]

        CLAUDE_SETTINGS["Claude Settings (.claude/settings.local.json):<br/>• permissions.allow: [research tools]<br/>• permissions.deny: [install tools]<br/>• permissions.defaultMode: acceptEdits<br/>• model: claude-opus-4-20250514<br/>• env: telemetry + optimization<br/>• hooks.onStop: ./.stop-hook-docs-pr.sh<br/>• cleanupPeriodDays: 3"]
    end

    MCP_ARGS --> CLI_DETECT
    CLI_DETECT --> CLI_REQ
    CLI_REQ --> TASKRUN_SPEC
    TASKRUN_SPEC --> REPO_SPEC
    TASKRUN_SPEC --> CLAUDE_MD

    TASKRUN_SPEC --> SETTINGS_GEN
    SETTINGS_GEN --> CONFIGMAP_DATA
    CONFIGMAP_DATA --> JOB_CONFIG

    JOB_CONFIG --> PREP_ENV
    JOB_CONFIG --> CLAUDE_ENV
    CONFIGMAP_DATA --> CLAUDE_SETTINGS

    classDef input fill:#e3f2fd
    classDef processing fill:#f1f8e9
    classDef generation fill:#fff3e0
    classDef runtime fill:#fce4ec

    class MCP_ARGS input
    class CLI_DETECT,CLI_REQ processing
    class TASKRUN_SPEC,REPO_SPEC,CLAUDE_MD,SETTINGS_GEN,CONFIGMAP_DATA,JOB_CONFIG generation
    class PREP_ENV,CLAUDE_ENV,CLAUDE_SETTINGS runtime
```

## Settings Generation Detail

```mermaid
graph TB
    subgraph "Job Type Detection"
        TASK_ID["task_id == 999999<br/>(DOCS_GENERATION_TASK_ID)"]
        IS_DOCS["is_docs_generation(tr) = true"]
    end

    subgraph "Permission Generation"
        DOCS_PERMS["Docs Generation Permissions:<br/>ALLOW:<br/>• Bash(git:*, gh:*, basic commands)<br/>• Edit(*), Read(*), Write(*)<br/>• MultiEdit(*), Glob(*), Grep(*)<br/>• LS(*), WebSearch(*), WebFetch(*)<br/><br/>DENY:<br/>• Bash(npm:install*, docker:*, kubectl:*)"]
    end

    subgraph "Environment Generation"
        BASE_ENV["Base Environment:<br/>• NODE_ENV: production<br/>• DISABLE_AUTOUPDATER: 1"]

        TELEMETRY_ENV["Telemetry (if enabled):<br/>• CLAUDE_CODE_ENABLE_TELEMETRY: 1<br/>• OTEL_METRICS_EXPORTER: otlp<br/>• OTEL_LOGS_EXPORTER: otlp<br/>• OTEL_EXPORTER_OTLP_LOGS_ENDPOINT<br/>• OTEL_EXPORTER_OTLP_LOGS_PROTOCOL"]

        DOCS_ENV["Docs Optimization:<br/>• DISABLE_COST_WARNINGS: 1<br/>• DISABLE_NON_ESSENTIAL_MODEL_CALLS: 1<br/>• CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR: true"]
    end

    subgraph "Model Selection"
        DOCS_MODEL["Hard-coded for docs:<br/>claude-opus-4-20250514"]
    end

    subgraph "Hook Configuration"
        DOCS_HOOKS["Docs Generation Hooks:<br/>hooks.onStop: ./.stop-hook-docs-pr.sh"]
    end

    subgraph "Final Settings JSON"
        FINAL_SETTINGS["settings.local.json:<br/>{<br/>  permissions: {...},<br/>  env: {...},<br/>  model: 'claude-opus-4-20250514',<br/>  cleanupPeriodDays: 3,<br/>  includeCoAuthoredBy: true,<br/>  hooks: {...}<br/>}"]
    end

    TASK_ID --> IS_DOCS
    IS_DOCS --> DOCS_PERMS
    IS_DOCS --> BASE_ENV
    BASE_ENV --> TELEMETRY_ENV
    TELEMETRY_ENV --> DOCS_ENV
    IS_DOCS --> DOCS_MODEL
    IS_DOCS --> DOCS_HOOKS

    DOCS_PERMS --> FINAL_SETTINGS
    DOCS_ENV --> FINAL_SETTINGS
    DOCS_MODEL --> FINAL_SETTINGS
    DOCS_HOOKS --> FINAL_SETTINGS

    classDef detection fill:#e8f5e8
    classDef generation fill:#fff3e0
    classDef final fill:#fce4ec

    class TASK_ID,IS_DOCS detection
    class DOCS_PERMS,BASE_ENV,TELEMETRY_ENV,DOCS_ENV,DOCS_MODEL,DOCS_HOOKS generation
    class FINAL_SETTINGS final
```

## Job Execution Sequence

```mermaid
sequenceDiagram
    participant MCP as MCP Server
    participant CLI as Orchestrator CLI
    participant API as API Handler
    participant Controller as TaskRun Controller
    participant K8s as Kubernetes API
    participant PrepJob as Prep Job
    participant ClaudeJob as Claude Job
    participant Hook as Post Hook

    MCP->>CLI: init_docs(working_directory, model, etc.)
    CLI->>CLI: Auto-detect repo info
    CLI->>API: POST /pm/docs/generate

    API->>API: Validate request
    API->>API: Generate CLAUDE.md content
    API->>K8s: Create TaskRun CRD

    K8s->>Controller: Watch event (TaskRun created)
    Controller->>Controller: is_docs_generation() = true
    Controller->>Controller: Generate settings.json
    Controller->>K8s: Create ConfigMap
    Controller->>K8s: Ensure PVC exists
    Controller->>K8s: Create Prep Job

    K8s->>PrepJob: Start prep container
    PrepJob->>PrepJob: Clone repository
    PrepJob->>PrepJob: Create docs branch
    PrepJob->>PrepJob: Copy ConfigMap files
    PrepJob->>K8s: Job completed successfully

    Controller->>Controller: Prep job success detected
    Controller->>K8s: Create Claude Job

    K8s->>ClaudeJob: Start Claude container
    ClaudeJob->>ClaudeJob: Load .claude/settings.local.json
    ClaudeJob->>ClaudeJob: Execute documentation generation
    ClaudeJob->>Hook: Trigger onStop hook
    Hook->>Hook: Commit and push changes
    Hook->>Hook: Create pull request
    ClaudeJob->>K8s: Job completed successfully

    Controller->>API: Update TaskRun status
    API->>CLI: Return success response
    CLI->>MCP: Return job details
```

## Key Configuration Points

### 1. **MCP → CLI Transformation**
- Auto-detection of git repository information
- Generation of unique target branch names
- Default model selection (opus for docs)

### 2. **CLI → API Transformation**
- DocsGenerationRequest structure
- Repository specification
- Service and agent naming

### 3. **API → TaskRun Transformation**
- Hard-coded task_id (999999) for docs generation
- CLAUDE.md content generation with instructions
- Repository spec mapping

### 4. **Controller Settings Generation**
- Job type detection via is_docs_generation()
- Hard-coded permissions for research and git operations
- Model override to claude-opus-4-20250514
- Hook configuration for automated PR creation

### 5. **Runtime Configuration**
- Container environment variables (secrets, metadata)
- Claude settings file (.claude/settings.local.json)
- Volume mounts (workspace PVC, ConfigMap)
- Hook script execution on completion

This flow ensures consistent, optimized configuration for documentation generation while maintaining flexibility for other job types.