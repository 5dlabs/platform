# ***CRITICAL PATH: PM → Orchestrator → Agent Deployment***
## ***DEADLINE: 1.5 DAYS TO WORKING SYSTEM***
## ***FOCUS: Helm Deployment Flow Only***

## 🚨 CRITICAL PATH IMPLEMENTATION 🚨

### PRIMARY FOCUS:
1. **READ HELM DEPLOYMENT DESIGN FIRST**: @docs/helm-deployment-flow-design.md
2. **IGNORE OTHER FEATURES**: Focus ONLY on PM task submission → Helm deployment
3. **MVP GOAL**: Submit task via CLI → Deploy Claude agent via Helm → Agent works on service

### Critical Path Components:
- **PM Endpoint**: `POST /api/v1/pm/tasks` with Markdown payloads
- **Helm Binary Wrapper**: Using `Command::new("helm")` 
- **Init Container Pattern**: Single job with init container for workspace prep
- **ConfigMap Storage**: Store Markdown files for agent consumption

### What to IGNORE for Now:
- GitHub webhooks (existing code)
- Slack/Discord integrations
- Complex monitoring
- Multi-agent patterns
- Everything not in critical path

## Implementation Priorities

### Phase 0: Helm Chart Updates (PREREQUISITE - Do First!)
1. **Update `infra/claude-code` chart**: Convert prepare job to init container
2. **Test init container**: Verify file copying from ConfigMap works
3. **Remove prepareJob**: Clean up values.yaml and templates

### Phase 1: Core Implementation (1.5 Day Target)
1. **PM Endpoint**: `/api/v1/pm/tasks` accepting task + Markdown files
2. **Helm Wrapper**: Simple wrapper around helm binary
3. **ConfigMap Creation**: Store task files via Kubernetes API
4. **Basic CLI**: Submit tasks with design specs

### Current Architecture (from docs)
- **Storage**: One PVC per microservice (local-path-provisioner, RWO)
- **Deployment**: Job-based (not Deployment/StatefulSet)
- **Pattern**: Init container prepares workspace, main container runs Claude
- **Node Affinity**: All jobs on same worker node for PVC access

### Kubernetes Access Requirements
- **ServiceAccount**: Orchestrator needs SA with ConfigMap + Job permissions
- **In-cluster config**: Default for orchestrator pod
- **Helm binary**: Must be available in orchestrator container
- **Namespace**: All operations scoped to `orchestrator` namespace

## Critical Path Files

**Orchestrator Core:**
- `orchestrator/src/handlers/pm.rs` - PM endpoint handler (TO BE CREATED)
- `orchestrator/src/services/helm_client.rs` - Helm wrapper (TO BE CREATED)
- `orchestrator/src/models/pm_task.rs` - Task request model (TO BE CREATED)

**Helm Chart:**
- `infra/claude-code/templates/job.yaml` - NEEDS UPDATE to init container
- `infra/claude-code/values.yaml` - NEEDS UPDATE to remove prepareJob

**Documentation:**
- `docs/helm-deployment-flow-design.md` - PRIMARY REFERENCE
- `CLAUDE.md` - This file with critical path focus

## File Locations & Workspace Structure

### Agent Workspace (from design):
```
/workspace/{service_name}/
├── .task/{task_id}/run-{attempt}/
│   ├── task.md                    # Task details in Markdown
│   ├── design-spec.md             # Comprehensive design
│   ├── prompt.md                  # Autonomous instructions
│   ├── acceptance-criteria.md     # Extracted criteria
│   └── metadata.yaml              # Attempt tracking
├── src/                           # Service source code
└── CLAUDE.md                      # Lean with @imports
```

### ConfigMap Structure:
- Created by orchestrator with all Markdown files
- Mounted at `/config/` in init container
- Copied to workspace by init container

## Critical Path Development Flow

### For THIS Orchestrator Implementation:

1. **Start with Helm Chart** - Update to init container pattern FIRST
2. **Create PM Models** - `PmTaskRequest` with markdown_files Vec
3. **Implement PM Handler** - Parse request, create ConfigMap, deploy Helm
4. **Test with Mock Task** - Submit test task, verify agent deployment
5. **Build CLI** - Basic submission tool for testing

### Validation Requirements:
- Run `cargo clippy --all-targets --all-features -- -D warnings` after EVERY change
- Run `cargo test` before moving to next component
- Test Helm deployment with actual chart
- Verify ConfigMap creation in Kubernetes

## Testing the Critical Path

### Manual Testing Flow:
```bash
# 1. Update Helm chart first
cd infra/claude-code
# Edit templates/job.yaml to use init container
# Test with: helm template . --values test-values.yaml

# 2. Test orchestrator endpoint
cargo run --bin orchestrator
# In another terminal:
curl -X POST http://localhost:8080/api/v1/pm/tasks \
  -H "Content-Type: application/json" \
  -d @test-task.json

# 3. Verify Kubernetes resources
kubectl get configmaps -n orchestrator
kubectl get jobs -n orchestrator
kubectl logs -n orchestrator job/[job-name]
```

### CLI Focus (MVP Scope):
- **Task submission**: Primary command for submitting tasks
- **Retry support**: Re-run failed tasks with incremented attempt number
- **OUT OF SCOPE**: Interactive mode, status checking, log viewing (use dashboards)

### Success Criteria:
1. ✅ Helm chart deploys with init container
2. ✅ ConfigMap created with Markdown files
3. ✅ Init container copies files successfully
4. ✅ Claude agent starts with prepared workspace
5. ✅ Agent can read task files via @imports

## Task Master Context (Secondary - Not Critical Path)

### Essential Commands (for reference only):
```bash
# Show current tasks
task-master list

# Get next task
task-master next

# Mark task complete
task-master set-status --id=<id> --status=done
```

### Current Tasks (check with `task-master list`):
Focus on tasks related to:
- PM endpoint implementation
- Helm integration
- ConfigMap generation
- CLI tool for task submission

## Remember: 1.5 Days to Working System

**Focus ONLY on:**
1. Helm chart with init container
2. PM endpoint that accepts tasks
3. ConfigMap creation for task files  
4. Helm deployment of Claude agents
5. Basic CLI for testing

**Everything else can wait!**

## GitHub Actions Workflow

**ALWAYS use the wait script when pushing changes:**
1. After pushing changes that trigger GitHub Actions builds
2. **IMPORTANT**: Use the PAT token: `export GITHUB_TOKEN=$pat`
3. Get the latest run ID: `gh run list --workflow=build-orchestrator.yml --limit=1 --json databaseId --jq '.[0].databaseId'`
4. Run the wait script: `./scripts/wait-for-github-action.sh -w build-orchestrator.yml`
5. The script will monitor the build and notify when complete
6. Only proceed with deployment after successful build

**Remember: Always use PAT ($pat) for GitHub authentication!**

---

_Critical Path: PM submits task → Orchestrator creates ConfigMap → Helm deploys agent → Agent works autonomously_