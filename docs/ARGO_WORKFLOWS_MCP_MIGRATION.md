# Argo Workflows MCP Server Migration Plan

## Executive Summary

This document outlines the migration strategy from the current Rust API server architecture to an Argo Workflows-based system for the 5D Labs AI Agent Platform. The approach creates a new MCP server (`fdl-mcp-argo`) alongside the existing one (`fdl-mcp`) to enable parallel operation, thorough validation, and zero-risk migration.

**Note**: `fdl-mcp-argo` is a temporary name during migration. Once the migration is complete and the old server is removed, it will be renamed back to `fdl-mcp`.

## Architecture Overview

### Current Architecture
```
Cursor/Claude → MCP Server → CLI Tool → Rust API Server → Kubernetes Jobs → Agent Pods
```

### Target Architecture  
```
Cursor/Claude → MCP Server → Argo Workflows API → Workflow Templates → Agent Pods
```

## Migration Strategy: Parallel Implementation

### Phase 1: New MCP Server Creation
- **Objective**: Create `fdl-mcp-argo` alongside existing `fdl-mcp`
- **Duration**: 1-2 weeks
- **Risk Level**: None (no changes to existing system)

### Phase 2: Function Migration
- **Objective**: Migrate `docs()` and `task()` functions one by one
- **Duration**: 2-3 weeks  
- **Risk Level**: Low (existing system remains operational)

### Phase 3: Validation & Traffic Shifting
- **Objective**: Compare systems side-by-side, gradually shift traffic
- **Duration**: 2-4 weeks
- **Risk Level**: Low (instant rollback capability)

### Phase 4: Cleanup
- **Objective**: Remove old components after extensive validation
- **Duration**: 1 week
- **Risk Level**: None (new system proven)

## Argo Workflows API Integration

### Key Endpoints (from API Spec Analysis)

#### Workflow Submission
```http
POST /api/v1/workflows/{namespace}/submit
Content-Type: application/json

{
  "namespace": "orchestrator",
  "resourceKind": "WorkflowTemplate", 
  "resourceName": "coderun-template",
  "submitOptions": {
    "parameters": [
      {"name": "task-id", "value": "1.2"},
      {"name": "service-id", "value": "unique-id"},
      {"name": "target-repo", "value": "5dlabs/platform"}
    ]
  }
}
```

#### Workflow Status Monitoring
```http
GET /api/v1/workflows/{namespace}/{name}
```

#### Workflow Logs
```http
GET /api/v1/workflows/{namespace}/{name}/log
```

### Workflow Templates

#### CodeRun Template (`infra/workflow-templates/coderun.yaml`)
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: coderun-template
  namespace: orchestrator
spec:
  entrypoint: code-agent
  arguments:
    parameters:
    - name: task-id
    - name: service-id  
    - name: target-repo
    - name: docs-repo
    - name: docs-project-dir
    - name: working-dir
    - name: github-user
    - name: model
      value: "claude-sonnet-4-20250514"
    - name: continue-session
      value: "false"
  templates:
  - name: code-agent
    container:
      image: ghcr.io/5dlabs/platform/claude-code:latest
      env:
        - name: TASK_ID
          value: "{{workflow.parameters.task-id}}"
        - name: SERVICE_ID
          value: "{{workflow.parameters.service-id}}"
        - name: TARGET_REPO
          value: "{{workflow.parameters.target-repo}}"
        - name: DOCS_REPO
          value: "{{workflow.parameters.docs-repo}}"
        - name: DOCS_PROJECT_DIR
          value: "{{workflow.parameters.docs-project-dir}}"
        - name: WORKING_DIR
          value: "{{workflow.parameters.working-dir}}"
        - name: GITHUB_USER
          value: "{{workflow.parameters.github-user}}"
        - name: MODEL
          value: "{{workflow.parameters.model}}"
        - name: CONTINUE_SESSION
          value: "{{workflow.parameters.continue-session}}"
      volumeMounts:
        - name: workspace
          mountPath: /workspace
        - name: claude-config
          mountPath: /.claude
        - name: git-config
          mountPath: /root/.ssh
      resources:
        requests:
          memory: "2Gi"
          cpu: "1000m"
        limits:
          memory: "8Gi"
          cpu: "4000m"
  volumes:
    - name: workspace
      persistentVolumeClaim:
        claimName: "workspace-{{workflow.parameters.service-id}}"
    - name: claude-config
      configMap:
        name: claude-config-{{workflow.parameters.service-id}}
    - name: git-config
      secret:
        secretName: git-credentials
```

#### DocsRun Template (`infra/workflow-templates/docsrun.yaml`)
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: docsrun-template
  namespace: orchestrator
spec:
  entrypoint: docs-agent
  arguments:
    parameters:
    - name: working-directory
    - name: github-user
    - name: model
      value: "claude-sonnet-4-20250514"
  templates:
  - name: docs-agent
    container:
      image: ghcr.io/5dlabs/platform/claude-docs:latest
      env:
        - name: WORKING_DIRECTORY
          value: "{{workflow.parameters.working-directory}}"
        - name: GITHUB_USER
          value: "{{workflow.parameters.github-user}}"
        - name: MODEL
          value: "{{workflow.parameters.model}}"
        - name: JOB_TYPE
          value: "docs"
      volumeMounts:
        - name: workspace
          mountPath: /workspace
        - name: claude-config
          mountPath: /.claude
        - name: git-config
          mountPath: /root/.ssh
      resources:
        requests:
          memory: "2Gi" 
          cpu: "1000m"
        limits:
          memory: "6Gi"
          cpu: "2000m"
  volumes:
    - name: workspace
      persistentVolumeClaim:
        claimName: "workspace-docs"
    - name: claude-config
      configMap:
        name: claude-config-docs
    - name: git-config
      secret:
        secretName: git-credentials
```

## CLI Functionality Migration to MCP Server

### Critical CLI Features to Migrate

The current CLI (`fdl`) performs significant auto-detection and file generation that must be migrated to the new MCP server. This functionality cannot be lost during the migration:

#### 1. **Git Auto-Detection Functions** (`commands.rs:250-282`)
```rust
// These functions must be moved to MCP server
fn get_git_remote_url() -> Result<String>        // Auto-detect repo URL
fn get_git_current_branch() -> Result<String>    // Auto-detect current branch
```

#### 2. **Parameter Auto-Detection** (`commands.rs:154-187`)
- **Repository URL Detection**: Auto-detect from `git remote get-url origin`
- **Docs Repository URL**: Auto-detect from current git context
- **Working Directory**: Auto-detect relative path from repo root
- **Docs Branch**: Auto-detect current git branch
- **Context Version**: Auto-increment based on existing CodeRuns (currently hardcoded to 1)

#### 3. **Environment Variable Parsing** (`commands.rs:284-344`)
```rust
// Complex parsing logic that must be preserved
fn parse_env_vars(env_str: Option<&str>) -> Result<HashMap<String, String>>
fn parse_env_from_secrets(env_secrets_str: Option<&str>) -> Result<Vec<SecretEnvVar>>
```

#### 4. **Documentation File Preparation** (`docs_generator.rs`)
This is the most complex functionality that **must** be preserved:

- **Auto-commit .taskmaster changes** (`check_and_commit_taskmaster_changes`)
  - Detect uncommitted changes in `.taskmaster` directory
  - Auto-commit and push to remote before docs generation
  - Generate meaningful commit messages

- **Documentation Structure Creation** (`create_docs_structure`)
  - Parse `tasks.json` to extract all tasks
  - Create organized directory structure in `.taskmaster/docs/`
  - Copy individual task files to organized locations
  - Handle missing task files gracefully

- **Git Repository Analysis** (`prepare_for_submission`)
  - Detect repository URL, working directory, source branch
  - Generate unique target branch names with timestamps
  - Validate git repository state

### Migration Strategy for CLI Functionality

#### Option 1: Embed Git Operations in MCP Server ⭐ **Recommended**
```rust
// New module: mcp-argo/git_operations.rs
pub struct GitOperations {
    working_dir: PathBuf,
}

impl GitOperations {
    pub async fn detect_repository_info(&self) -> Result<RepositoryInfo> {
        // Migrate git detection logic
    }
    
    pub async fn prepare_taskmaster_files(&self) -> Result<()> {
        // Migrate docs preparation logic
    }
    
    pub async fn auto_commit_changes(&self, message: &str) -> Result<()> {
        // Migrate auto-commit functionality
    }
}
```

#### Option 2: Shell Command Execution from MCP Server
```rust
// Execute git commands via tokio::process::Command
let output = tokio::process::Command::new("git")
    .args(["remote", "get-url", "origin"])
    .current_dir(&working_dir)
    .output()
    .await?;
```

### Implementation Requirements

#### 1. **Preserve All Auto-Detection Logic**
- Repository URL detection from git remote
- Working directory calculation from git repo root
- Current branch detection with fallbacks
- Environment variable parsing (both direct and from secrets)

#### 2. **Preserve Documentation Preparation**
- Taskmaster file auto-commit and push functionality
- Documentation directory structure creation
- Task file organization and copying
- Error handling for missing files

#### 3. **Add New Workflow-Specific Logic**
- Unique workflow name generation (instead of unique branch names)
- Workflow parameter validation
- Service ID generation and validation
- Context version management for Argo Workflows

#### 4. **Enhanced Error Handling**
- Async-compatible error handling throughout
- Proper logging for workflow submission context
- Graceful degradation when git operations fail

### Updated MCP Server Structure
```
orchestrator/tools/src/mcp-argo/
├── main.rs                 # Main MCP server loop
├── workflow_client.rs      # Argo Workflows API client
├── git_operations.rs       # Git auto-detection and file ops
├── parameter_parsing.rs    # Environment variable parsing
├── docs_preparation.rs     # Documentation file preparation
└── tools.rs               # MCP tool schemas
```

### Testing Strategy for CLI Migration

#### 1. **Functional Parity Tests**
```bash
# Test that both systems produce identical results
./test_cli_parity.sh --cli-command "fdl task code 1 --service test" \
                     --mcp-call "task" --mcp-params '{"task_id": 1, "service": "test"}'
```

#### 2. **Git Operations Tests**
- Test auto-detection in various git repository states
- Test auto-commit functionality with different change scenarios
- Test working directory detection from various locations
- Test branch detection in detached HEAD state

#### 3. **Documentation Preparation Tests**
- Test with various taskmaster directory structures
- Test with missing task files
- Test with uncommitted changes
- Test with different git repository configurations

### Risk Mitigation for CLI Migration

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Lost auto-detection functionality** | High | Comprehensive test suite comparing CLI vs MCP behavior |
| **Git operation failures** | Medium | Fallback to user-provided parameters when auto-detection fails |
| **Documentation prep complexity** | Medium | Migrate exact logic, extensive testing with real taskmaster projects |
| **Environment parsing breakage** | Low | Unit tests for all environment parsing scenarios |

### Migration Checklist

- [ ] **Git Operations Module**: Migrate all git detection functions
- [ ] **Parameter Parsing Module**: Migrate environment variable parsing
- [ ] **Documentation Preparation Module**: Migrate entire docs_generator.rs logic
- [ ] **Async Conversion**: Convert all blocking operations to async
- [ ] **Error Handling**: Implement proper async error handling
- [ ] **Testing**: Create comprehensive functional parity tests
- [ ] **Validation**: Test with real taskmaster projects
- [ ] **Fallback Logic**: Implement graceful degradation for failed auto-detection

---

## Implementation Plan

### Step 1: Project Structure Setup

```bash
# Create new MCP server directory
mkdir orchestrator/tools/src/mcp-argo
cp -r orchestrator/tools/src/mcp/* orchestrator/tools/src/mcp-argo/

# Update Cargo.toml
[[bin]]
name = "fdl-mcp-argo"
path = "tools/src/mcp-argo/main.rs"
```

### Step 2: Argo Workflows Client Implementation

#### Dependencies (Add to `Cargo.toml`)
```toml
[dependencies]
# Existing dependencies...
argo-workflows = "0.1"  # Argo Workflows Rust client
k8s-openapi = "0.21"    # Kubernetes API types
kube = "0.88"           # Kubernetes client
uuid = "1.0"            # For generating unique workflow names
```

#### New Workflow Client (`mcp-argo/workflow_client.rs`)
```rust
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::ObjectReference;
use kube::{Api, Client, ResourceExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ArgoWorkflowsClient {
    client: Client,
    namespace: String,
}

impl ArgoWorkflowsClient {
    pub async fn new(namespace: String) -> Result<Self> {
        let client = Client::try_default().await?;
        Ok(Self { client, namespace })
    }

    pub async fn submit_coderun_workflow(
        &self,
        task_id: u64,
        service: &str,
        repository: &str,
        docs_repository: &str,
        docs_project_directory: &str,
        working_directory: &str,
        github_user: &str,
        model: Option<&str>,
        continue_session: bool,
        env: Option<&Value>,
        env_from_secrets: Option<&Value>,
    ) -> Result<String> {
        let workflow_name = format!("coderun-{}-{}", service, Uuid::new_v4());
        
        let mut parameters = vec![
            json!({"name": "task-id", "value": task_id.to_string()}),
            json!({"name": "service-id", "value": service}),
            json!({"name": "target-repo", "value": repository}),
            json!({"name": "docs-repo", "value": docs_repository}),
            json!({"name": "docs-project-dir", "value": docs_project_directory}),
            json!({"name": "working-dir", "value": working_directory}),
            json!({"name": "github-user", "value": github_user}),
            json!({"name": "continue-session", "value": continue_session.to_string()}),
        ];

        if let Some(m) = model {
            parameters.push(json!({"name": "model", "value": m}));
        }

        let submit_request = json!({
            "namespace": self.namespace,
            "resourceKind": "WorkflowTemplate",
            "resourceName": "coderun-template",
            "submitOptions": {
                "parameters": parameters,
                "name": workflow_name
            }
        });

        // Make HTTP request to Argo Workflows API
        let response = self.submit_workflow_request(submit_request).await?;
        
        Ok(workflow_name)
    }

    pub async fn submit_docsrun_workflow(
        &self,
        working_directory: &str,
        github_user: &str,
        model: Option<&str>,
    ) -> Result<String> {
        let workflow_name = format!("docsrun-{}", Uuid::new_v4());
        
        let mut parameters = vec![
            json!({"name": "working-directory", "value": working_directory}),
            json!({"name": "github-user", "value": github_user}),
        ];

        if let Some(m) = model {
            parameters.push(json!({"name": "model", "value": m}));
        }

        let submit_request = json!({
            "namespace": self.namespace,
            "resourceKind": "WorkflowTemplate", 
            "resourceName": "docsrun-template",
            "submitOptions": {
                "parameters": parameters,
                "name": workflow_name
            }
        });

        let response = self.submit_workflow_request(submit_request).await?;
        
        Ok(workflow_name)
    }

    pub async fn get_workflow_status(&self, workflow_name: &str) -> Result<Value> {
        // Implementation for workflow status monitoring
        self.get_workflow_request(workflow_name).await
    }

    pub async fn get_workflow_logs(&self, workflow_name: &str) -> Result<String> {
        // Implementation for workflow log retrieval
        self.get_workflow_logs_request(workflow_name).await
    }

    async fn submit_workflow_request(&self, submit_request: Value) -> Result<Value> {
        // HTTP client implementation to call Argo Workflows API
        let client = reqwest::Client::new();
        let url = format!("http://argo-workflows-server:2746/api/v1/workflows/{}/submit", self.namespace);
        
        let response = client
            .post(&url)
            .json(&submit_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Workflow submission failed: {}", error_text));
        }

        let workflow: Value = response.json().await?;
        Ok(workflow)
    }

    async fn get_workflow_request(&self, workflow_name: &str) -> Result<Value> {
        let client = reqwest::Client::new();
        let url = format!("http://argo-workflows-server:2746/api/v1/workflows/{}/{}", self.namespace, workflow_name);
        
        let response = client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get workflow status: {}", error_text));
        }

        let workflow: Value = response.json().await?;
        Ok(workflow)
    }

    async fn get_workflow_logs_request(&self, workflow_name: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("http://argo-workflows-server:2746/api/v1/workflows/{}/{}/log", self.namespace, workflow_name);
        
        let response = client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to get workflow logs: {}", error_text));
        }

        let logs = response.text().await?;
        Ok(logs)
    }
}
```

### Step 3: Modified MCP Server Implementation

#### Updated Main (`mcp-argo/main.rs`)
```rust
// ... (existing imports and structures)
mod workflow_client;
use workflow_client::ArgoWorkflowsClient;

// Global workflow client (initialized on first use)
static mut WORKFLOW_CLIENT: Option<ArgoWorkflowsClient> = None;

async fn get_workflow_client() -> Result<&'static ArgoWorkflowsClient> {
    unsafe {
        if WORKFLOW_CLIENT.is_none() {
            WORKFLOW_CLIENT = Some(ArgoWorkflowsClient::new("orchestrator".to_string()).await?);
        }
        Ok(WORKFLOW_CLIENT.as_ref().unwrap())
    }
}

// Updated handler functions
fn handle_orchestrator_tools_async(
    method: &str,
    params_map: &HashMap<String, Value>,
) -> Option<impl std::future::Future<Output = Result<Value>>> {
    match method {
        "docs" => {
            let params_map = params_map.clone();
            Some(async move {
                let working_directory = params_map.get("working_directory")
                    .and_then(|v| v.as_str())
                    .ok_or(anyhow!("working_directory parameter is required"))?;

                let github_user = params_map.get("github_user")
                    .and_then(|v| v.as_str())
                    .ok_or(anyhow!("github_user parameter is required"))?;

                let model = params_map.get("model").and_then(|v| v.as_str());

                let client = get_workflow_client().await?;
                let workflow_name = client.submit_docsrun_workflow(
                    working_directory,
                    github_user,
                    model,
                ).await?;

                Ok(json!({
                    "success": true,
                    "message": "Documentation workflow submitted successfully",
                    "workflow_name": workflow_name,
                    "parameters_used": {
                        "working_directory": working_directory,
                        "github_user": github_user,
                        "model": model.unwrap_or("default")
                    }
                }))
            })
        }
        "task" => {
            let params_map = params_map.clone();
            Some(async move {
                // Extract parameters (same validation as current implementation)
                let task_id = params_map.get("task_id")
                    .and_then(|v| v.as_u64())
                    .ok_or(anyhow!("Missing required parameter: task_id"))?;

                let service = params_map.get("service")
                    .and_then(|v| v.as_str())
                    .ok_or(anyhow!("Missing required parameter: service"))?;

                // ... (all existing parameter extraction and validation)

                let client = get_workflow_client().await?;
                let workflow_name = client.submit_coderun_workflow(
                    task_id,
                    service,
                    repository,
                    docs_repository, 
                    docs_project_directory,
                    working_directory,
                    github_user,
                    model,
                    continue_session,
                    env,
                    env_from_secrets,
                ).await?;

                Ok(json!({
                    "success": true,
                    "message": "Code implementation workflow submitted successfully", 
                    "workflow_name": workflow_name,
                    "parameters_used": {
                        "task_id": task_id,
                        "service": service,
                        // ... (all parameters)
                    }
                }))
            })
        }
        _ => None,
    }
}
```

### Step 4: Deployment Configuration

#### Update Helm Chart Values
```yaml
# Add to infra/charts/orchestrator/values.yaml
argo-workflows:
  enabled: true
  server:
    service:
      type: ClusterIP
      port: 2746
  controller:
    enabled: true
```

#### MCP Server Configuration Update
```json
// .mcp.json - Add new server alongside existing
{
  "mcpServers": {
    "fdl-mcp": {
      "command": "./target/release/fdl-mcp"
    },
    "fdl-mcp-argo": {
      "command": "./target/release/fdl-mcp-argo"
    }
  }
}
```

## Validation & Testing Strategy

### Unit Testing
- Test workflow client API calls with mocked Argo API
- Validate parameter transformation and validation
- Test error handling and retry logic

### Integration Testing  
- Deploy both MCP servers in test environment
- Compare outputs between old and new systems
- Validate workflow execution end-to-end
- Test failure scenarios and cleanup

### Performance Testing
- Measure workflow submission latency vs API calls
- Test concurrent workflow execution
- Monitor resource usage patterns
- Validate cleanup and garbage collection

### Comparison Testing
```bash
# Test script for parallel validation
./test_parallel_mcp.sh --old-server fdl-mcp --new-server fdl-mcp-argo --test-cases ./test_cases.json
```

## Rollback Strategy

### Immediate Rollback (< 5 minutes)
1. Update `.mcp.json` to use only old server
2. Restart Claude Code sessions
3. Monitor for any in-flight workflows (leave them to complete)

### Graceful Rollback (30 minutes)
1. Stop accepting new workflows in new server
2. Wait for all in-flight workflows to complete
3. Switch all traffic back to old server
4. Validate system health

### Emergency Rollback (< 1 minute)
1. Scale down new MCP server deployment
2. Update load balancer to route all traffic to old server
3. Monitor and fix any data consistency issues

## Monitoring & Observability

### Key Metrics
- **Workflow Submission Rate**: workflows/minute
- **Workflow Success Rate**: successful/total workflows
- **Average Execution Time**: end-to-end workflow duration
- **Resource Utilization**: CPU/Memory usage per workflow
- **Error Rates**: by workflow type and failure reason

### Dashboard Components
- Real-time workflow status (Pending/Running/Succeeded/Failed)
- Resource usage trends and alerts
- Comparison metrics between old and new systems
- Agent workspace utilization

### Alerting Rules
```yaml
# Prometheus alerting rules
groups:
- name: argo-workflows-mcp
  rules:
  - alert: WorkflowSubmissionFailure
    expr: increase(workflow_submission_failures_total[5m]) > 5
    labels:
      severity: critical
    annotations:
      summary: "High workflow submission failure rate"
  
  - alert: WorkflowExecutionTimeout  
    expr: argo_workflow_duration_seconds > 3600
    labels:
      severity: warning
    annotations:
      summary: "Workflow execution exceeding 1 hour"
```

## Risk Mitigation

### Technical Risks
| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Argo Workflows API instability | Low | High | Comprehensive testing, fallback to old system |
| Performance degradation | Medium | Medium | Load testing, resource tuning |
| Data loss during migration | Low | High | Parallel operation, no data migration needed |
| Integration complexity | Medium | Medium | Phased approach, extensive validation |

### Operational Risks  
| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Team unfamiliarity with Argo | Medium | Low | Training, documentation, gradual rollout |
| Debugging complexity | Medium | Medium | Enhanced logging, monitoring dashboards |
| Increased operational overhead | Low | Medium | Automation, runbooks, team training |

## Success Criteria

### Phase 1 Success Criteria
- [ ] New MCP server builds and deploys successfully
- [ ] Basic workflow submission works end-to-end
- [ ] No impact on existing system performance
- [ ] All tests pass in CI/CD pipeline

### Phase 2 Success Criteria  
- [ ] Both `docs()` and `task()` functions work via Argo Workflows
- [ ] Performance meets or exceeds current system (< 30s submission time)
- [ ] Error rates remain below 1% for standard operations
- [ ] Resource usage within acceptable limits (< 2x current usage)

### Phase 3 Success Criteria
- [ ] 99.9% functional parity with old system
- [ ] No critical bugs reported during parallel operation
- [ ] Performance improvement measurable (target: 20% faster)
- [ ] Team confident in new system reliability

### Phase 4 Success Criteria
- [ ] Old system components cleanly removed
- [ ] New system handles 100% of production traffic  
- [ ] Documentation and runbooks complete
- [ ] Team fully trained on new architecture

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **Phase 1: Setup** | 1-2 weeks | New MCP server structure, workflow templates, basic client |
| **Phase 2: Implementation** | 2-3 weeks | Complete workflow client, function migration, testing |
| **Phase 3: Validation** | 2-4 weeks | Parallel operation, comparison testing, gradual traffic shift |
| **Phase 4: Cleanup** | 1 week | Remove old components, final documentation |

**Total Timeline: 6-10 weeks**

## TaskMaster Integration

### Update Required Tasks

This migration plan requires updates to existing TaskMaster tasks to reflect the new implementation approach:

#### Task #2: Configure Argo Workflows Infrastructure
- **Updated Scope**: Expand from basic Argo Workflows configuration to complete MCP server migration
- **New Focus**: Implement `fdl-mcp-argo` server with workflow client, CLI functionality migration, and parallel operation
- **Expanded Subtasks**: Include git operations migration, documentation preparation, and comparison testing

#### Additional Tasks to Consider
- **CLI Functionality Audit**: Document all CLI features to ensure none are lost
- **Testing Framework Setup**: Create comprehensive functional parity tests
- **Migration Validation**: Establish metrics and success criteria for gradual traffic shifting

### TaskMaster Task Updates

The migration plan should be reflected in TaskMaster with updated task descriptions, expanded subtasks, and revised timeline estimates. The focus shifts from infrastructure configuration to comprehensive system migration.

---

## Next Steps

1. **Immediate (This Week)**:
   - Review and approve this migration plan
   - **Update TaskMaster tasks** to reflect new migration scope
   - Set up development environment for new MCP server
   - Create initial workflow templates

2. **Short Term (Next 2 Weeks)**:
   - Implement basic Argo Workflows client
   - Create parallel MCP server structure
   - Begin unit testing

3. **Medium Term (Next Month)**:
   - Complete function migration
   - Deploy to test environment
   - Begin comparison testing

4. **Long Term (2-3 Months)**:
   - Production parallel deployment
   - Gradual traffic shifting
   - Final cleanup and documentation

---

*This document will be updated as implementation progresses and new requirements emerge.*