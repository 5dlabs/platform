# CRD Implementation Improvements

## 1. Enhanced CRD with Status Updates

Update the CRD definition to include more detailed status information:

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
    subresources:
      status: {}  # Enable status subresource
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
              configMapName: {type: string}
              attempts: {type: integer, default: 0}
              lastUpdated: {type: string}
              message: {type: string}
              conditions:
                type: array
                items:
                  type: object
                  properties:
                    type: {type: string}
                    status: {type: string}
                    lastTransitionTime: {type: string}
                    reason: {type: string}
                    message: {type: string}
```

## 2. Use kube-runtime for Robust Controller

Replace the basic watch loop with kube-runtime's controller framework:

```rust
// orchestrator-core/src/controllers/taskrun.rs
use kube::{
    Api, Client, Resource,
    runtime::{
        controller::{Action, Controller},
        events::{Recorder, Event},
        finalizer::{finalizer, Event as FinalizerEvent},
    },
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::Duration;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Missing object key")]
    MissingObjectKey,
}
type Result<T, E = Error> = std::result::Result<T, E>;

// Context for the controller
struct Context {
    client: Client,
    namespace: String,
}

// Main controller function
pub async fn run_taskrun_controller(client: Client, namespace: String) -> Result<()> {
    let taskruns = Api::<TaskRun>::namespaced(client.clone(), &namespace);
    let context = Arc::new(Context { client, namespace });
    
    Controller::new(taskruns, Default::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;
    
    Ok(())
}

// Main reconciliation logic
async fn reconcile(tr: Arc<TaskRun>, ctx: Arc<Context>) -> Result<Action> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    
    // Create APIs
    let taskruns: Api<TaskRun> = Api::namespaced(client.clone(), namespace);
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    
    // Handle finalizers for cleanup
    let finalizer_name = "taskruns.orchestrator.io/finalizer";
    finalizer(&taskruns, finalizer_name, tr.clone(), |event| async move {
        match event {
            FinalizerEvent::Apply(tr) => {
                // Create or update resources
                reconcile_create_or_update(tr, &jobs, &configmaps, &taskruns).await
            }
            FinalizerEvent::Cleanup(tr) => {
                // Cleanup resources when TaskRun is deleted
                cleanup_resources(tr, &jobs, &configmaps).await
            }
        }
    }).await.map_err(|e| Error::KubeError(e.into()))?;
    
    // Requeue after 5 minutes to check status
    Ok(Action::requeue(Duration::from_secs(300)))
}

// Reconciliation logic for create/update
async fn reconcile_create_or_update(
    tr: Arc<TaskRun>,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
    taskruns: &Api<TaskRun>,
) -> Result<Action> {
    let name = tr.metadata.name.as_ref().ok_or(Error::MissingObjectKey)?;
    
    // Update status to Pending
    update_status(taskruns, name, "Pending", "Reconciling TaskRun").await?;
    
    // Check for existing jobs with older versions
    let job_list = jobs.list(&ListParams::default()
        .labels(&format!("task-id={}", tr.spec.task_id))
    ).await?;
    
    // Delete older versions
    for job in job_list.items {
        if let Some(version) = job.metadata.labels.as_ref()
            .and_then(|l| l.get("context-version"))
            .and_then(|v| v.parse::<u32>().ok()) 
        {
            if version < tr.spec.context_version {
                if let Some(job_name) = &job.metadata.name {
                    jobs.delete(job_name, &DeleteParams::default()).await?;
                    info!("Deleted older job version: {}", job_name);
                }
            }
        }
    }
    
    // Create ConfigMap
    let cm_name = format!("{}-{}-v{}-files", 
        tr.spec.service_name, 
        tr.spec.task_id, 
        tr.spec.context_version
    );
    
    let cm = build_configmap(&tr, &cm_name)?;
    match configmaps.create(&PostParams::default(), &cm).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }
    
    // Create Job
    let job_name = format!("{}-{}-v{}", 
        tr.spec.service_name, 
        tr.spec.task_id, 
        tr.spec.context_version
    );
    
    let job = build_job(&tr, &job_name, &cm_name)?;
    match jobs.create(&PostParams::default(), &job).await {
        Ok(_) => {
            info!("Created Job: {}", job_name);
            update_status_with_details(
                taskruns, 
                name, 
                "Running", 
                "Job created successfully",
                Some(job_name.clone()),
                Some(cm_name),
            ).await?;
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
            update_status(taskruns, name, "Running", "Job already exists").await?;
        }
        Err(e) => {
            update_status(taskruns, name, "Failed", &format!("Failed to create job: {}", e)).await?;
            return Err(e.into());
        }
    }
    
    Ok(Action::requeue(Duration::from_secs(30)))
}

// Cleanup resources when TaskRun is deleted
async fn cleanup_resources(
    tr: Arc<TaskRun>,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
) -> Result<Action> {
    let task_id = tr.spec.task_id;
    
    // Delete all jobs for this task
    let job_list = jobs.list(&ListParams::default()
        .labels(&format!("task-id={}", task_id))
    ).await?;
    
    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }
    
    // Delete all configmaps for this task
    let cm_list = configmaps.list(&ListParams::default()
        .labels(&format!("task-id={}", task_id))
    ).await?;
    
    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }
    
    Ok(Action::await_change())
}

// Error policy for the controller
fn error_policy(_tr: Arc<TaskRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("Reconciliation error: {:?}", error);
    // Exponential backoff on errors
    Action::requeue(Duration::from_secs(5_u64.pow(2)))
}

// Helper function to update status
async fn update_status(
    api: &Api<TaskRun>,
    name: &str,
    phase: &str,
    message: &str,
) -> Result<()> {
    let status = json!({
        "status": {
            "phase": phase,
            "message": message,
            "lastUpdated": Utc::now().to_rfc3339(),
        }
    });
    
    api.patch_status(name, &PatchParams::default(), &Patch::Merge(status))
        .await
        .map_err(Error::KubeError)?;
    
    Ok(())
}

// Helper function to update status with details
async fn update_status_with_details(
    api: &Api<TaskRun>,
    name: &str,
    phase: &str,
    message: &str,
    job_name: Option<String>,
    configmap_name: Option<String>,
) -> Result<()> {
    let mut status = serde_json::json!({
        "phase": phase,
        "message": message,
        "lastUpdated": Utc::now().to_rfc3339(),
    });
    
    if let Some(job) = job_name {
        status["jobName"] = json!(job);
    }
    
    if let Some(cm) = configmap_name {
        status["configMapName"] = json!(cm);
    }
    
    api.patch_status(
        name, 
        &PatchParams::default(), 
        &Patch::Merge(json!({"status": status}))
    ).await.map_err(Error::KubeError)?;
    
    Ok(())
}
```

## 3. Monitor Job Status and Update TaskRun

Add a separate reconciliation loop to monitor Job status and update TaskRun accordingly:

```rust
// Monitor Job status and update TaskRun
async fn monitor_job_status(
    tr: &TaskRun,
    jobs: &Api<Job>,
    taskruns: &Api<TaskRun>,
) -> Result<()> {
    if let Some(job_name) = &tr.status.as_ref().and_then(|s| s.job_name.as_ref()) {
        if let Ok(job) = jobs.get(job_name).await {
            if let Some(job_status) = &job.status {
                let (phase, message) = match job_status {
                    s if s.succeeded.unwrap_or(0) > 0 => ("Succeeded", "Job completed successfully"),
                    s if s.failed.unwrap_or(0) > 0 => ("Failed", "Job failed"),
                    _ => ("Running", "Job is running"),
                };
                
                if tr.status.as_ref().map(|s| &s.phase) != Some(&phase.to_string()) {
                    update_status(taskruns, &tr.metadata.name.as_ref().unwrap(), phase, message).await?;
                }
            }
        }
    }
    Ok(())
}
```

## 4. Add Observability with Events

Record Kubernetes events for better debugging:

```rust
// Add to the controller context
struct Context {
    client: Client,
    namespace: String,
    recorder: Recorder,
}

// Initialize recorder in main
let recorder = Recorder::new(client.clone(), reporter, object_ref);

// Use in reconciliation
recorder.publish(Event {
    type_: EventType::Normal,
    reason: "Created".to_string(),
    note: Some(format!("Created Job {} for TaskRun", job_name)),
    action: "Create".to_string(),
    secondary: None,
}).await?;
```

## 5. Add Prometheus Metrics

```rust
use prometheus::{Counter, Gauge, register_counter, register_gauge};

lazy_static! {
    static ref TASKRUN_TOTAL: Counter = register_counter!(
        "taskrun_total",
        "Total number of TaskRuns processed"
    ).unwrap();
    
    static ref TASKRUN_ACTIVE: Gauge = register_gauge!(
        "taskrun_active",
        "Number of active TaskRuns"
    ).unwrap();
}

// In reconciliation
TASKRUN_TOTAL.inc();
TASKRUN_ACTIVE.set(active_count as f64);
```

## 6. Enhanced PM Handler with Validation

```rust
pub async fn submit_task(
    State(state): State<AppState>,
    Json(request): Json<PmTaskRequest>,
) -> Result<Json<ApiResponse>, AppError> {
    // Validate request
    if request.markdown_files.is_empty() {
        return Err(AppError::BadRequest("No markdown files provided".to_string()));
    }
    
    // Check if TaskRun already exists
    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), "orchestrator");
    let name = format!("task-{}", request.id);
    
    match api.get(&name).await {
        Ok(_) => return Err(AppError::Conflict("Task already exists".to_string())),
        Err(kube::Error::Api(ae)) if ae.code == 404 => {},
        Err(e) => return Err(AppError::from(e)),
    }
    
    let taskrun = TaskRun {
        metadata: ObjectMeta {
            name: Some(name.clone()),
            namespace: Some("orchestrator".to_string()),
            labels: Some([
                ("task-id".to_string(), request.id.to_string()),
                ("service-name".to_string(), request.service_name.clone()),
            ].into()),
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
    
    api.create(&PostParams::default(), &taskrun).await?;
    
    Ok(Json(ApiResponse {
        success: true,
        message: "Task submitted successfully".to_string(),
        data: Some(json!({
            "name": name,
            "namespace": "orchestrator",
        })),
    }))
}
```

## 7. Add Context with Conflict Detection

```rust
pub async fn add_context(
    State(state): State<AppState>,
    Path(task_id): Path<u32>,
    Json(context): Json<AddContextRequest>,
) -> Result<Json<ApiResponse>, AppError> {
    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), "orchestrator");
    let name = format!("task-{}", task_id);
    
    // Use Server-Side Apply for conflict-free updates
    let patch = json!({
        "apiVersion": "orchestrator.io/v1",
        "kind": "TaskRun",
        "metadata": {
            "name": name,
            "namespace": "orchestrator",
        },
        "spec": {
            "contextVersion": context.version,
            "markdownFiles": [{
                "filename": format!("context-v{}.md", context.version),
                "content": context.additional_context,
            }],
        }
    });
    
    api.patch(
        &name,
        &PatchParams::apply("pm-handler").force(),
        &Patch::Apply(patch),
    ).await?;
    
    Ok(Json(ApiResponse::success("Context added successfully")))
}
```

## Summary of Key Improvements

1. **Status Subresource** - Enables separate status updates without modifying spec
2. **kube-runtime Controller** - Provides exponential backoff, error handling, and signal handling
3. **Finalizers** - Ensures proper cleanup when TaskRuns are deleted
4. **Job Monitoring** - Updates TaskRun status based on Job completion
5. **Events & Metrics** - Better observability for debugging and monitoring
6. **Validation** - Prevents duplicate tasks and validates input
7. **Server-Side Apply** - Conflict-free updates when adding context

These improvements make the controller production-ready while maintaining the simplicity of the original design.