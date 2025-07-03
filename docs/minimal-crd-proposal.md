# Minimal CRD Implementation for Task Management

## Why Skip Hooks?

After reviewing the feedback, implementing all the safeguards for hooks (cleanup, retry logic, monitoring) would take MORE effort than a simple CRD. Let's do it right the first time.

## Implementation (1.5 Days)

### 1. CRD Definition (30 minutes)

```yaml
# taskrun-crd.yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: taskruns.orchestrator.io
spec:
  group: orchestrator.io
  scope: Namespaced
  versions:
  - name: v1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              taskId: {type: integer}
              serviceName: {type: string}
              agentName: {type: string}
              contextVersion: {type: integer}
              markdownFiles:
                type: array
                items:
                  type: object
                  properties:
                    filename: {type: string}
                    content: {type: string}
          status:
            type: object
            properties:
              phase: {type: string, enum: [Pending, Running, Succeeded, Failed]}
              jobName: {type: string}
              attempts: {type: integer}
              lastUpdated: {type: string}
```

### 2. Minimal Controller in Orchestrator (4 hours)

```rust
// orchestrator-core/src/controllers/taskrun.rs
use kube::{Api, Client, CustomResource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.io", version = "v1", kind = "TaskRun")]
#[kube(namespaced)]
pub struct TaskRunSpec {
    pub task_id: u32,
    pub service_name: String,
    pub agent_name: String,
    pub context_version: u32,
    pub markdown_files: Vec<MarkdownFile>,
}

pub async fn taskrun_controller(client: Client) -> Result<()> {
    let taskruns: Api<TaskRun> = Api::namespaced(client.clone(), "orchestrator");
    let jobs: Api<Job> = Api::namespaced(client.clone(), "orchestrator");
    
    // Watch for TaskRun changes
    let mut stream = taskruns.watch(&Default::default(), "0").await?.boxed();
    
    while let Some(event) = stream.try_next().await? {
        match event {
            WatchEvent::Added(tr) | WatchEvent::Modified(tr) => {
                reconcile_taskrun(&tr, &jobs).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn reconcile_taskrun(tr: &TaskRun, jobs: &Api<Job>) -> Result<()> {
    let job_name = format!("{}-{}-v{}", 
        tr.spec.service_name, 
        tr.spec.task_id, 
        tr.spec.context_version
    );
    
    // Check if Job exists with older version
    let existing_jobs = jobs.list(&ListParams::default()
        .labels(&format!("task-id={}", tr.spec.task_id))
    ).await?;
    
    // Delete older versions
    for job in existing_jobs.items {
        if let Some(version) = job.metadata.labels.get("context-version") {
            if version.parse::<u32>().unwrap_or(0) < tr.spec.context_version {
                jobs.delete(&job.metadata.name.unwrap(), &Default::default()).await?;
            }
        }
    }
    
    // Create ConfigMap with markdown files
    let cm = create_configmap_from_taskrun(tr)?;
    configmaps.create(&PostParams::default(), &cm).await?;
    
    // Create Job
    let job = build_job_from_taskrun(tr, &cm.metadata.name.unwrap())?;
    jobs.create(&PostParams::default(), &job).await?;
    
    Ok(())
}
```

### 3. Update PM Handler (2 hours)

```rust
// Instead of Helm, create TaskRun
pub async fn submit_task(
    State(state): State<AppState>,
    Json(request): Json<PmTaskRequest>,
) -> Result<Json<ApiResponse>, AppError> {
    let taskrun = TaskRun {
        metadata: ObjectMeta {
            name: Some(format!("task-{}", request.id)),
            namespace: Some("orchestrator".to_string()),
            ..Default::default()
        },
        spec: TaskRunSpec {
            task_id: request.id,
            service_name: request.service_name,
            agent_name: request.agent_name,
            context_version: 1,
            markdown_files: request.markdown_files,
        },
        status: None,
    };
    
    let api: Api<TaskRun> = Api::namespaced(state.k8s_client, "orchestrator");
    api.create(&PostParams::default(), &taskrun).await?;
    
    Ok(Json(ApiResponse::success("Task submitted")))
}

// Adding context is now trivial
pub async fn add_context(
    State(state): State<AppState>,
    Path(task_id): Path<u32>,
    Json(context): Json<AddContextRequest>,
) -> Result<Json<ApiResponse>, AppError> {
    let api: Api<TaskRun> = Api::namespaced(state.k8s_client, "orchestrator");
    let mut tr = api.get(&format!("task-{}", task_id)).await?;
    
    // Simply increment version and add context
    tr.spec.context_version += 1;
    tr.spec.markdown_files.push(MarkdownFile {
        filename: format!("context-v{}.md", tr.spec.context_version),
        content: context.additional_context,
    });
    
    // Update triggers reconciliation
    api.replace(&tr.metadata.name.unwrap(), &PostParams::default(), &tr).await?;
    
    Ok(Json(ApiResponse::success("Context added")))
}
```

### 4. Benefits Over Hooks

1. **Standard Kubernetes Pattern** - Anyone familiar with k8s understands this
2. **Automatic History** - CRD stores all versions
3. **No Cleanup Needed** - Controller manages Job lifecycle
4. **Easy Status Checks** - `kubectl get taskruns`
5. **Natural Upgrades** - Just update the CRD
6. **Proper Resource Tracking** - Everything is visible

### 5. Migration Path

```bash
# Week 1: Deploy CRD alongside existing system
kubectl apply -f taskrun-crd.yaml

# Week 2: Update orchestrator to use CRD for new tasks
# Existing Helm releases continue working

# Week 3: Migrate active tasks to CRD
# Delete old Helm releases

# Week 4: Remove Helm Job logic entirely
```

## Effort Summary

| Task | Hooks + Safeguards | CRD |
|------|-------------------|-----|
| Base implementation | 4h | 6h |
| Cleanup/lifecycle | 6h | 0h (built-in) |
| Retry/race conditions | 3h | 0h (built-in) |
| Testing edge cases | 6h | 3h |
| Documentation | 2h | 1h |
| **Total** | **21h** | **10h** |

## Recommendation

Skip the hooks entirely. The CRD approach is:
- ✅ Less total work
- ✅ Follows best practices
- ✅ Easier to debug
- ✅ No technical debt
- ✅ Natural context updates

The "temporary" hooks solution would take MORE time than doing it right!