use crate::config::controller_config::ControllerConfig;
#[cfg(test)]
use crate::crds::{MarkdownFile, MarkdownFileType};
use crate::crds::{TaskRun, TaskRunPhase};
use chrono::Utc;
use futures::StreamExt;
use handlebars::Handlebars;
use k8s_openapi::{
    api::{
        batch::v1::Job,
        core::v1::{
            ConfigMap, PersistentVolumeClaim, PersistentVolumeClaimSpec, VolumeResourceRequirements,
        },
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams},
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
        watcher::Config,
    },
    Client, ResourceExt,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info};

// Error type for the controller
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Missing object key")]
    MissingObjectKey,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task configuration error: {0}")]
    ConfigError(String),
}

type Result<T, E = Error> = std::result::Result<T, E>;

// Context for the controller
struct Context {
    client: Client,
    namespace: String,
    config: Arc<ControllerConfig>,
}

// Finalizer name for cleanup
const FINALIZER_NAME: &str = "taskruns.orchestrator.io/finalizer";

/// Ensure PVC exists for a service
async fn ensure_pvc_exists(
    pvcs: &Api<PersistentVolumeClaim>,
    pvc_name: &str,
    service_name: &str,
) -> Result<()> {
    // Check if PVC already exists
    match pvcs.get(pvc_name).await {
        Ok(pvc) => {
            info!(
                "PVC {} already exists, status: {:?}",
                pvc_name,
                pvc.status.as_ref().map(|s| &s.phase)
            );
            Ok(())
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // PVC doesn't exist, create it
            info!("Creating PVC {} for service {}", pvc_name, service_name);

            let pvc = PersistentVolumeClaim {
                metadata: ObjectMeta {
                    name: Some(pvc_name.to_string()),
                    labels: Some(BTreeMap::from([
                        (
                            "app.kubernetes.io/managed-by".to_string(),
                            "taskrun-controller".to_string(),
                        ),
                        (
                            "orchestrator.io/service".to_string(),
                            service_name.to_string(),
                        ),
                    ])),
                    ..Default::default()
                },
                spec: Some(PersistentVolumeClaimSpec {
                    access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                    resources: Some(VolumeResourceRequirements {
                        requests: Some(BTreeMap::from([(
                            "storage".to_string(),
                            Quantity("10Gi".to_string()),
                        )])),
                        ..Default::default()
                    }),
                    storage_class_name: Some("local-path".to_string()),
                    volume_mode: Some("Filesystem".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            pvcs.create(&PostParams::default(), &pvc).await?;
            info!("Created PVC {} successfully", pvc_name);

            // Wait a moment for PVC to be bound
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Run the TaskRun controller
pub async fn run_taskrun_controller(client: Client, namespace: String) -> Result<()> {
    info!("Starting TaskRun controller in namespace: {}", namespace);

    // Load controller configuration from ConfigMap
    let config =
        match ControllerConfig::from_configmap(&client, &namespace, "taskrun-controller-config")
            .await
        {
            Ok(cfg) => {
                info!("Loaded controller configuration from ConfigMap");
                cfg
            }
            Err(e) => {
                info!(
                    "Failed to load configuration from ConfigMap, using defaults: {}",
                    e
                );
                ControllerConfig::default()
            }
        };

    let taskruns = Api::<TaskRun>::namespaced(client.clone(), &namespace);
    let context = Arc::new(Context {
        client,
        namespace,
        config: Arc::new(config),
    });

    Controller::new(taskruns, Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, context)
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

/// Main reconciliation logic
async fn reconcile(tr: Arc<TaskRun>, ctx: Arc<Context>) -> Result<Action> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;

    // Create APIs
    let taskruns: Api<TaskRun> = Api::namespaced(client.clone(), namespace);
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);

    let name = tr.name_any();
    info!("Reconciling TaskRun: {}", name);

    // Handle finalizers for cleanup
    let _result = finalizer(&taskruns, FINALIZER_NAME, tr.clone(), |event| async {
        match event {
            FinalizerEvent::Apply(tr) => {
                // Create or update resources
                reconcile_create_or_update(tr, &jobs, &configmaps, &pvcs, &taskruns, &ctx.config)
                    .await
            }
            FinalizerEvent::Cleanup(tr) => {
                // Cleanup resources when TaskRun is deleted (don't delete PVCs - they're shared)
                cleanup_resources(tr, &jobs, &configmaps).await
            }
        }
    })
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    // If we have a running job, check its status
    if let Some(status) = &tr.status {
        if status.phase == Some(TaskRunPhase::Running) {
            monitor_job_status(&tr, &jobs, &taskruns).await?;
        }
    }

    // Requeue after 30 seconds to check status
    Ok(Action::requeue(Duration::from_secs(30)))
}

/// Reconciliation logic for create/update
async fn reconcile_create_or_update(
    tr: Arc<TaskRun>,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
    pvcs: &Api<PersistentVolumeClaim>,
    taskruns: &Api<TaskRun>,
    config: &ControllerConfig,
) -> Result<Action> {
    let name = tr.name_any();

    // Update status to Pending if not set
    if tr.status.is_none() {
        update_status(taskruns, &name, TaskRunPhase::Pending, "TaskRun created").await?;
    }

    // Ensure PVC exists for the service
    let pvc_name = format!("workspace-{}", tr.spec.service_name);
    ensure_pvc_exists(pvcs, &pvc_name, &tr.spec.service_name).await?;

    // Create ConfigMap first (needed by both prep and main jobs)
    let cm_name = format!(
        "{}-{}-task{}-v{}-files",
        tr.spec.agent_name.replace('_', "-"),
        tr.spec.service_name.replace('_', "-"),
        tr.spec.task_id,
        tr.spec.context_version
    );

    let cm = build_configmap(&tr, &cm_name, config)?;
    match configmaps.create(&PostParams::default(), &cm).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Check for and delete older job versions
    let job_list = jobs
        .list(&ListParams::default().labels(&format!("task-id={}", tr.spec.task_id)))
        .await?;

    for job in job_list.items {
        if let Some(version) = job
            .metadata
            .labels
            .as_ref()
            .and_then(|l| l.get("context-version"))
            .and_then(|v| v.parse::<u32>().ok())
        {
            if version < tr.spec.context_version {
                if let Some(job_name) = &job.metadata.name {
                    jobs.delete(job_name, &DeleteParams::background()).await?;
                    info!("Deleted older job version: {}", job_name);
                }
            }
        }
    }

    // Special handling for documentation generation tasks (task_id = 999999)
    if tr.spec.task_id == 999999 {
        info!("Documentation generation task detected, using minimal prep job");
        // Use a special prep job for docs generation
        let prep_job_name = format!(
            "prep-{}-{}-task{}-attempt{}",
            tr.spec.agent_name.replace('_', "-"),
            tr.spec.service_name.replace('_', "-"),
            tr.spec.task_id,
            tr.spec.context_version
        );
        
        // Check if prep job exists
        match jobs.get(&prep_job_name).await {
            Ok(prep_job) => {
                // Check prep job status
                if let Some(job_status) = &prep_job.status {
                    if job_status.succeeded.unwrap_or(0) > 0 {
                        info!("Docs prep job succeeded, creating Claude job");
                        create_claude_job(tr, jobs, taskruns, &cm_name, config).await?;
                    } else if job_status.failed.unwrap_or(0) > 0 {
                        update_status(
                            taskruns,
                            &name,
                            TaskRunPhase::Failed,
                            "Documentation prep failed",
                        )
                        .await?;
                    }
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // Create minimal prep job for docs
                info!("Creating docs prep job: {}", prep_job_name);
                let prep_job = build_docs_prep_job(&tr, &prep_job_name, config)?;
                jobs.create(&PostParams::default(), &prep_job).await?;
                update_status(
                    taskruns,
                    &name,
                    TaskRunPhase::Preparing,
                    "Preparing documentation workspace",
                )
                .await?;
            }
            Err(e) => return Err(e.into()),
        }
        return Ok(Action::requeue(Duration::from_secs(10)));
    }

    // Check prep job status for normal tasks
    let prep_job_name = format!(
        "prep-{}-{}-task{}-attempt{}",
        tr.spec.agent_name.replace('_', "-"),
        tr.spec.service_name.replace('_', "-"),
        tr.spec.task_id,
        tr.spec.context_version
    );

    // Try to get prep job
    match jobs.get(&prep_job_name).await {
        Ok(prep_job) => {
            // Prep job exists, check its status
            if let Some(job_status) = &prep_job.status {
                if job_status.succeeded.unwrap_or(0) > 0 {
                    // Prep job succeeded, create main Claude job
                    info!("Prep job succeeded, creating Claude job");
                    create_claude_job(tr, jobs, taskruns, &cm_name, config).await?;
                } else if job_status.failed.unwrap_or(0) > 0 {
                    // Prep job failed
                    update_status(
                        taskruns,
                        &name,
                        TaskRunPhase::Failed,
                        "Workspace preparation failed",
                    )
                    .await?;
                } else {
                    // Prep job still running
                    if tr.status.as_ref().and_then(|s| s.phase.as_ref())
                        != Some(&TaskRunPhase::Preparing)
                    {
                        update_status(
                            taskruns,
                            &name,
                            TaskRunPhase::Preparing,
                            "Preparing workspace",
                        )
                        .await?;
                    }
                }
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Prep job doesn't exist, create it
            info!("Creating prep job: {}", prep_job_name);
            let prep_job = build_prep_job(&tr, &prep_job_name, &cm_name, config)?;
            match jobs.create(&PostParams::default(), &prep_job).await {
                Ok(_) => {
                    info!("Created prep job: {}", prep_job_name);
                    update_status(
                        taskruns,
                        &name,
                        TaskRunPhase::Preparing,
                        "Workspace preparation started",
                    )
                    .await?;
                }
                Err(e) => {
                    update_status(
                        taskruns,
                        &name,
                        TaskRunPhase::Failed,
                        &format!("Failed to create prep job: {e}"),
                    )
                    .await?;
                    return Err(e.into());
                }
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(Action::requeue(Duration::from_secs(10))) // Check more frequently during preparation
}

/// Create the main Claude job after prep job succeeds
async fn create_claude_job(
    tr: Arc<TaskRun>,
    jobs: &Api<Job>,
    taskruns: &Api<TaskRun>,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<()> {
    let name = tr.name_any();
    let job_name = format!(
        "{}-{}-task{}-attempt{}",
        tr.spec.agent_name.replace('_', "-"),
        tr.spec.service_name.replace('_', "-"),
        tr.spec.task_id,
        tr.spec.context_version
    );

    // Check if Claude job already exists
    match jobs.get(&job_name).await {
        Ok(_) => {
            info!("Claude job already exists: {}", job_name);
            if tr.status.as_ref().and_then(|s| s.phase.as_ref()) != Some(&TaskRunPhase::Running) {
                update_status(taskruns, &name, TaskRunPhase::Running, "Job already exists").await?;
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Create Claude job
            let job = build_claude_job(&tr, &job_name, cm_name, config)?;
            match jobs.create(&PostParams::default(), &job).await {
                Ok(_) => {
                    info!("Created Claude job: {}", job_name);
                    update_status_with_details(
                        taskruns,
                        &name,
                        TaskRunPhase::Running,
                        "Claude agent started",
                        Some(job_name),
                        Some(cm_name.to_string()),
                    )
                    .await?;
                }
                Err(e) => {
                    update_status(
                        taskruns,
                        &name,
                        TaskRunPhase::Failed,
                        &format!("Failed to create Claude job: {e}"),
                    )
                    .await?;
                    return Err(e.into());
                }
            }
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

/// Monitor Job status and update TaskRun
async fn monitor_job_status(tr: &TaskRun, jobs: &Api<Job>, taskruns: &Api<TaskRun>) -> Result<()> {
    if let Some(job_name) = tr.status.as_ref().and_then(|s| s.job_name.as_ref()) {
        if let Ok(job) = jobs.get(job_name).await {
            if let Some(job_status) = &job.status {
                let (phase, message) = if job_status.succeeded.unwrap_or(0) > 0 {
                    (TaskRunPhase::Succeeded, "Job completed successfully")
                } else if job_status.failed.unwrap_or(0) > 0 {
                    (TaskRunPhase::Failed, "Job failed")
                } else {
                    (TaskRunPhase::Running, "Job is running")
                };

                // Only update if phase changed
                if tr.status.as_ref().and_then(|s| s.phase.as_ref()) != Some(&phase) {
                    update_status(taskruns, &tr.name_any(), phase, message).await?;
                }
            }
        }
    }
    Ok(())
}

/// Cleanup resources when TaskRun is deleted
async fn cleanup_resources(
    tr: Arc<TaskRun>,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
) -> Result<Action> {
    let task_id = tr.spec.task_id;
    info!("Cleaning up resources for task: {}", task_id);

    // Delete all jobs for this task
    let job_list = jobs
        .list(&ListParams::default().labels(&format!("task-id={task_id}")))
        .await?;

    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }

    // Delete all configmaps for this task
    let cm_list = configmaps
        .list(&ListParams::default().labels(&format!("task-id={task_id}")))
        .await?;

    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }

    Ok(Action::await_change())
}

/// Error policy for the controller - exponential backoff
fn error_policy(_tr: Arc<TaskRun>, error: &Error, ctx: Arc<Context>) -> Action {
    error!("Reconciliation error: {:?}", error);
    // Exponential backoff: 5s, 10s, 20s, 40s...
    Action::requeue(Duration::from_secs(
        5_u64.pow(ctx.client.default_namespace().len() as u32 % 4 + 1),
    ))
}

/// Update TaskRun status
async fn update_status(
    api: &Api<TaskRun>,
    name: &str,
    phase: TaskRunPhase,
    message: &str,
) -> Result<()> {
    // Get current TaskRun to preserve attempt count
    let current_tr = api.get(name).await.map_err(Error::KubeError)?;
    let attempts = current_tr.status.as_ref().map(|s| s.attempts).unwrap_or(0);

    let status = json!({
        "status": {
            "phase": phase,
            "message": message,
            "lastUpdated": Utc::now().to_rfc3339(),
            "attempts": attempts,
        }
    });

    api.patch_status(name, &PatchParams::default(), &Patch::Merge(status))
        .await
        .map_err(Error::KubeError)?;

    Ok(())
}

/// Update TaskRun status with additional details
async fn update_status_with_details(
    api: &Api<TaskRun>,
    name: &str,
    phase: TaskRunPhase,
    message: &str,
    job_name: Option<String>,
    configmap_name: Option<String>,
) -> Result<()> {
    // Get current TaskRun to access attempt count
    let current_tr = api.get(name).await.map_err(Error::KubeError)?;
    let current_attempts = current_tr.status.as_ref().map(|s| s.attempts).unwrap_or(0);

    // Increment attempts when creating a new job (Running phase)
    let new_attempts = if phase == TaskRunPhase::Running {
        current_attempts + 1
    } else {
        current_attempts
    };

    let mut status = serde_json::json!({
        "phase": phase,
        "message": message,
        "lastUpdated": Utc::now().to_rfc3339(),
        "attempts": new_attempts,
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
        &Patch::Merge(json!({"status": status})),
    )
    .await
    .map_err(Error::KubeError)?;

    Ok(())
}

/// Build ConfigMap from TaskRun
fn build_configmap(tr: &TaskRun, name: &str, config: &ControllerConfig) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Add all markdown files
    for file in &tr.spec.markdown_files {
        data.insert(file.filename.clone(), file.content.clone());
    }

    // Generate Claude Code configuration file for tool permissions (using correct filename)
    let settings_json = generate_claude_settings(tr, config)?;
    // Use claude-settings-local-json for ConfigMap (/ not allowed in keys)
    data.insert("claude-settings-local-json".to_string(), settings_json);


    // CLAUDE.md should always be provided by the client
    // This allows task-specific instructions and Git workflows

    Ok(ConfigMap {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(tr.namespace().unwrap_or_default()),
            labels: Some(BTreeMap::from([
                ("task-id".to_string(), tr.spec.task_id.to_string()),
                ("service-name".to_string(), tr.spec.service_name.clone()),
                (
                    "context-version".to_string(),
                    tr.spec.context_version.to_string(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    })
}

/// Build Claude Job from TaskRun
fn build_claude_job(
    tr: &TaskRun,
    job_name: &str,
    _cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    // API key will be injected from secret
    let service_name = &tr.spec.service_name;

    // Build volumes list - use dedicated PVC for this service
    let pvc_name = format!("workspace-{service_name}");
    let volumes = vec![json!({
        "name": "workspace",
        "persistentVolumeClaim": {
            "claimName": pvc_name
        }
    })];

    let mut telemetry_env = vec![];

    if config.telemetry.enabled {
        telemetry_env.extend(vec![
            json!({
                "name": "CLAUDE_CODE_ENABLE_TELEMETRY",
                "value": "1"
            }),
            // OpenTelemetry exporters
            json!({
                "name": "OTEL_METRICS_EXPORTER",
                "value": "otlp"
            }),
            json!({
                "name": "OTEL_LOGS_EXPORTER",
                "value": "otlp"
            }),
            // OTLP endpoints based on protocol
            json!({
                "name": "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT",
                "value": if config.telemetry.otlp_protocol == "grpc" {
                    format!("http://{}:4317/v1/logs", config.telemetry.otlp_endpoint.trim_end_matches(":4317"))
                } else {
                    format!("http://{}:4318/v1/logs", config.telemetry.otlp_endpoint.trim_end_matches(":4318"))
                }
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL",
                "value": if config.telemetry.otlp_protocol == "grpc" { "grpc" } else { "http/protobuf" }
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
                "value": if config.telemetry.otlp_protocol == "grpc" {
                    format!("http://{}:4317/v1/metrics", config.telemetry.otlp_endpoint.trim_end_matches(":4317"))
                } else {
                    format!("http://{}:4318/v1/metrics", config.telemetry.otlp_endpoint.trim_end_matches(":4318"))
                }
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL",
                "value": if config.telemetry.otlp_protocol == "grpc" { "grpc" } else { "http/protobuf" }
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_ENDPOINT",
                "value": format!("http://{}", config.telemetry.otlp_endpoint)
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_PROTOCOL",
                "value": if config.telemetry.otlp_protocol == "grpc" { "grpc" } else { "http/protobuf" }
            }),
            json!({
                "name": "OTEL_EXPORTER_OTLP_INSECURE",
                "value": config.telemetry.otlp_insecure.to_string()
            }),
            // Service identification
            json!({
                "name": "OTEL_SERVICE_NAME",
                "value": format!("{}-{}", config.telemetry.service_name, tr.spec.service_name)
            }),
            json!({
                "name": "OTEL_SERVICE_VERSION",
                "value": config.telemetry.service_version.clone()
            }),
            // Resource attributes
            json!({
                "name": "OTEL_RESOURCE_ATTRIBUTES",
                "value": build_resource_attributes(tr, config)
            }),
            // Export intervals
            json!({
                "name": "OTEL_METRIC_EXPORT_INTERVAL",
                "value": config.telemetry.metrics_export_interval.clone()
            }),
            json!({
                "name": "OTEL_METRIC_EXPORT_TIMEOUT",
                "value": config.telemetry.metrics_export_timeout.clone()
            }),
            // Logging configuration
            json!({
                "name": "OTEL_LOG_LEVEL",
                "value": config.telemetry.log_level.clone()
            }),
            // Claude Code specific settings
            json!({
                "name": "NODE_ENV",
                "value": "production"
            }),
            json!({
                "name": "DISABLE_AUTOUPDATER",
                "value": "1"
            })
        ]);

        if config.telemetry.log_user_prompts {
            telemetry_env.push(json!({
                "name": "OTEL_LOG_USER_PROMPTS",
                "value": "1"
            }));
        }
    }

    let job_json = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "namespace": tr.namespace().unwrap_or_default(),
            "labels": {
                "task-id": tr.spec.task_id.to_string(),
                "service-name": tr.spec.service_name.clone(),
                "context-version": tr.spec.context_version.to_string(),
                "managed-by": "taskrun-controller",
            }
        },
        "spec": {
            "backoffLimit": config.job.backoff_limit,
            "activeDeadlineSeconds": config.job.active_deadline_seconds,
            "ttlSecondsAfterFinished": config.job.ttl_seconds_after_finished,
            "template": {
                "spec": {
                    "restartPolicy": config.job.restart_policy.clone(),
                    "imagePullSecrets": [{"name": "ghcr-secret"}],
                    "containers": [{
                        "name": "claude-agent",
                        "image": format!("{}:{}", config.agent.image.repository, config.agent.image.tag),
                        "command": ["/bin/sh", "-c"],
                        "args": [build_agent_startup_script(tr, config)?],
                        "env": build_env_vars(tr, telemetry_env, config),
                        "volumeMounts": [{
                            "name": "workspace",
                            "mountPath": "/workspace"
                        }],
                        "workingDir": "/workspace",
                        "securityContext": {
                            "runAsUser": 0,
                            "runAsGroup": 0,
                            "runAsNonRoot": false
                        }
                    }],
                    "volumes": volumes
                }
            }
        }
    });

    serde_json::from_value(job_json).map_err(Error::SerializationError)
}

/// Build Prep Job for workspace preparation
fn build_prep_job(
    tr: &TaskRun,
    job_name: &str,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    // Build volumes for prep job
    let service_name = &tr.spec.service_name;
    let pvc_name = format!("workspace-{service_name}");

    let mut volumes = vec![
        json!({
            "name": "task-files",
            "configMap": {
                "name": cm_name
            }
        }),
        json!({
            "name": "workspace",
            "persistentVolumeClaim": {
                "claimName": pvc_name
            }
        }),
    ];

    let mut volume_mounts = vec![
        json!({
            "name": "task-files",
            "mountPath": "/config"
        }),
        json!({
            "name": "workspace",
            "mountPath": "/workspace"
            // No subPath - needs full PVC access to create directories
        }),
    ];

    // Add secret volume if repository is configured
    if let Some(repo) = &tr.spec.repository {
        let secret_name = format!("github-pat-{}", repo.github_user);
        let secret_volume_name = format!("{secret_name}-secret");

        volume_mounts.push(json!({
            "name": secret_volume_name.clone(),
            "mountPath": format!("/secrets/{}", secret_name),
            "readOnly": true
        }));

        volumes.push(json!({
            "name": secret_volume_name,
            "secret": {
                "secretName": secret_name
            }
        }));
    }

    let job_json = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "namespace": tr.namespace().unwrap_or_default(),
            "labels": {
                "task-id": tr.spec.task_id.to_string(),
                "service-name": tr.spec.service_name.clone(),
                "context-version": tr.spec.context_version.to_string(),
                "managed-by": "taskrun-controller",
                "job-type": "prep",
            }
        },
        "spec": {
            "backoffLimit": 2,  // Less retries for prep
            "activeDeadlineSeconds": 300,  // 5 minutes should be enough
            "ttlSecondsAfterFinished": config.job.ttl_seconds_after_finished,
            "template": {
                "spec": {
                    "restartPolicy": "Never",
                    "imagePullSecrets": [{"name": "ghcr-secret"}],
                    "containers": [{
                        "name": "prep-workspace",
                        "image": "alpine/git:latest",  // Alpine with git for cloning
                        "command": ["/bin/sh", "-c"],
                        "args": [build_prep_script(tr, config)?],
                        "volumeMounts": volume_mounts,
                        "securityContext": {
                            "runAsUser": 0,
                            "runAsGroup": 0,
                            "runAsNonRoot": false
                        },
                        "resources": {
                            "requests": {
                                "memory": "128Mi",
                                "cpu": "100m"
                            },
                            "limits": {
                                "memory": "512Mi",
                                "cpu": "500m"
                            }
                        }
                    }],
                    "volumes": volumes,
                    // Force onto same node as PVC for local-path provisioner
                    "nodeSelector": {
                        "kubernetes.io/hostname": "talos-a43-ee1"
                    }
                }
            }
        }
    });

    serde_json::from_value(job_json).map_err(Error::SerializationError)
}

/// Build minimal prep job for documentation generation
fn build_docs_prep_job(
    tr: &TaskRun,
    job_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    // Extract repository info from TaskRun spec and markdown files
    let repo_url = tr.spec.repository.as_ref()
        .map(|r| r.url.clone())
        .unwrap_or_default();
    let branch = tr.spec.repository.as_ref()
        .map(|r| r.branch.clone())
        .unwrap_or_else(|| "main".to_string());
    let mut working_dir = String::new();
    
    // Parse CLAUDE.md to get working directory (repo and branch come from spec)
    if let Some(claude_md) = tr.spec.markdown_files.iter().find(|f| f.filename == "CLAUDE.md") {
        // Extract working directory from content
        for line in claude_md.content.lines() {
            if line.starts_with("- **Working Directory**: ") {
                working_dir = line.trim_start_matches("- **Working Directory**: ").to_string();
            }
        }
    }
    
    // Debug output
    info!("Docs prep job - repo_url: {}, working_dir: {}, branch: {}", repo_url, working_dir, branch);
    
    let service_name = &tr.spec.service_name;
    let pvc_name = format!("workspace-{service_name}");
    
    let prep_script = format!(
        r#"#!/bin/sh
set -e

echo "=== DOCS PREP JOB STARTING ==="
echo "Repository: {repo_url}"
echo "Working directory: {working_dir}"
echo "Source branch: {branch}"

# Check if repository already exists
if [ -d "/workspace/.git" ]; then
    echo "Repository already exists, updating..."
    cd /workspace
    
    # Reset any local changes and fetch latest
    git reset --hard
    git clean -fd
    git fetch origin
    
    # Checkout the source branch
    SOURCE_BRANCH="{branch}"
    if [ -n "$SOURCE_BRANCH" ]; then
        echo "Checking out source branch: $SOURCE_BRANCH"
        git checkout "$SOURCE_BRANCH" || git checkout -b "$SOURCE_BRANCH" "origin/$SOURCE_BRANCH"
        git pull origin "$SOURCE_BRANCH"
    else
        echo "ERROR: Source branch not specified"
        exit 1
    fi
else
    # Clone repository if it doesn't exist
    echo "Cloning repository into workspace..."
    SOURCE_BRANCH="{branch}"
    if [ -n "$SOURCE_BRANCH" ]; then
        echo "Cloning source branch: $SOURCE_BRANCH"
        git clone --branch "$SOURCE_BRANCH" {repo_url} /workspace
    else
        echo "ERROR: Source branch not specified"
        exit 1
    fi
    cd /workspace
fi

# Create a new branch for documentation changes
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
DOC_BRANCH="docs/task-master-docs-$TIMESTAMP"
echo "Creating documentation branch: $DOC_BRANCH"
git checkout -b "$DOC_BRANCH"

# Configure git for commits
git config user.email "claude@5dlabs.com"
git config user.name "Claude (5D Labs)"

echo "✓ Prepared documentation branch: $DOC_BRANCH from source: $SOURCE_BRANCH"

# Navigate to working directory if specified
if [ -n "{working_dir}" ] && [ "{working_dir}" != "." ]; then
    echo "Working directory specified: {working_dir}"
    TASKMASTER_PATH="/workspace/{working_dir}/.taskmaster"
else
    echo "Using repository root"
    TASKMASTER_PATH="/workspace/.taskmaster"
fi

# Verify .taskmaster exists
if [ -d "$TASKMASTER_PATH" ]; then
    echo "✓ Found .taskmaster directory at $TASKMASTER_PATH"
else
    echo "ERROR: .taskmaster directory not found at $TASKMASTER_PATH"
    exit 1
fi

# Copy ConfigMap files to where Claude will be working
echo "Copying ConfigMap files..."
if [ -n "{working_dir}" ] && [ "{working_dir}" != "." ]; then
    echo "Copying files to /workspace/{working_dir}/.taskmaster/"
    cp -v /config/* /workspace/{working_dir}/.taskmaster/
else
    echo "Copying files to /workspace/.taskmaster/"
    cp -v /config/* /workspace/.taskmaster/
fi

echo "✓ Documentation workspace prepared"
echo "Claude will use --cwd flag to restrict access to the appropriate directory"
"#
    );
    
    let job_json = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "namespace": tr.namespace().unwrap_or_default(),
            "labels": {
                "task-id": tr.spec.task_id.to_string(),
                "service-name": tr.spec.service_name.clone(),
                "context-version": tr.spec.context_version.to_string(),
                "managed-by": "taskrun-controller",
                "job-type": "docs-prep",
            }
        },
        "spec": {
            "backoffLimit": 2,
            "activeDeadlineSeconds": 300,
            "ttlSecondsAfterFinished": config.job.ttl_seconds_after_finished,
            "template": {
                "spec": {
                    "restartPolicy": "Never",
                    "imagePullSecrets": [{"name": "ghcr-secret"}],
                    "containers": [{
                        "name": "prep-docs",
                        "image": "alpine/git:latest",
                        "command": ["/bin/sh", "-c"],
                        "args": [prep_script],
                        "volumeMounts": [
                            {
                                "name": "workspace",
                                "mountPath": "/workspace"
                            },
                            {
                                "name": "task-files",
                                "mountPath": "/config"
                            }
                        ],
                        "securityContext": {
                            "runAsUser": 0,
                            "runAsGroup": 0,
                            "runAsNonRoot": false
                        }
                    }],
                    "volumes": [
                        {
                            "name": "workspace",
                            "persistentVolumeClaim": {
                                "claimName": pvc_name
                            }
                        },
                        {
                            "name": "task-files",
                            "configMap": {
                                "name": format!(
                                    "{}-{}-task{}-v{}-files",
                                    tr.spec.agent_name.replace('_', "-"),
                                    tr.spec.service_name.replace('_', "-"),
                                    tr.spec.task_id,
                                    tr.spec.context_version
                                )
                            }
                        }
                    ]
                }
            }
        }
    });
    
    serde_json::from_value(job_json).map_err(Error::SerializationError)
}

// Template constants
const PREP_JOB_TEMPLATE: &str = include_str!("../../templates/prep-job.sh.hbs");
const MAIN_CONTAINER_TEMPLATE: &str = include_str!("../../templates/main-container.sh.hbs");
const DOCS_GENERATION_CONTAINER_TEMPLATE: &str = include_str!("../../templates/docs-generation-container.sh.hbs");


/// Build prep job script for workspace preparation
fn build_prep_script(tr: &TaskRun, _config: &ControllerConfig) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("prep", PREP_JOB_TEMPLATE)
        .map_err(|e| Error::ConfigError(format!("Failed to register template: {e}")))?;

    // Extract working directory for docs generation tasks (same logic as main container)
    let mut working_dir = String::new();
    if tr.spec.task_id == 999999 {
        // For docs generation, parse working directory from markdown
        if let Some(claude_md) = tr.spec.markdown_files.iter().find(|f| f.filename == "CLAUDE.md") {
            for line in claude_md.content.lines() {
                if line.starts_with("- **Working Directory**: ") {
                    working_dir = line.trim_start_matches("- **Working Directory**: ").to_string();
                    break;
                }
            }
        }
    }

    let data = json!({
        "task_id": tr.spec.task_id,
        "service_name": tr.spec.service_name,
        "repository": tr.spec.repository.as_ref(),
        "attempts": tr.status.as_ref().map_or(1, |s| s.attempts),
        "is_docs_generation": tr.spec.task_id == 999999,
        "working_dir": working_dir,
    });

    handlebars
        .render("prep", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render template: {e}")))
}

/// Build startup script for the agent container
fn build_agent_startup_script(tr: &TaskRun, config: &ControllerConfig) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    // Export script is now created by prep job, no need to render it here

    // Choose template based on task type
    let is_docs_generation = tr.spec.task_id == 999999;
    let template = if is_docs_generation {
        DOCS_GENERATION_CONTAINER_TEMPLATE
    } else {
        MAIN_CONTAINER_TEMPLATE
    };

    // Register the appropriate container template
    handlebars
        .register_template_string("main", template)
        .map_err(|e| Error::ConfigError(format!("Failed to register main template: {e}")))?;

    // Build the Claude command
    let command = config.agent.command.join(" ");

    // Extract working directory for docs generation tasks
    let mut working_dir = String::new();
    if tr.spec.task_id == 999999 {
        // For docs generation, parse working directory from markdown
        if let Some(claude_md) = tr.spec.markdown_files.iter().find(|f| f.filename == "CLAUDE.md") {
            for line in claude_md.content.lines() {
                if line.starts_with("- **Working Directory**: ") {
                    working_dir = line.trim_start_matches("- **Working Directory**: ").to_string();
                    break;
                }
            }
        }
    }

    // Prepare template data
    let data = json!({
        "command": command,
        "model_override": tr.spec.model != "sonnet", // Non-default model
        "model": tr.spec.model.clone(),
        "is_retry": tr.status.as_ref().is_some_and(|s| s.attempts > 1),
        "attempts": tr.status.as_ref().map_or(1, |s| s.attempts),
        "task_id": tr.spec.task_id,
        "is_docs_generation": tr.spec.task_id == 999999, // Special docs generation task
        "working_dir": working_dir,
    });

    handlebars
        .render("main", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render template: {e}")))
}

/// Build environment variables for the container
fn build_env_vars(
    tr: &TaskRun,
    telemetry_env: Vec<serde_json::Value>,
    config: &ControllerConfig,
) -> Vec<serde_json::Value> {
    let mut env_vars = vec![
        json!({
            "name": "ANTHROPIC_API_KEY",
            "valueFrom": {
                "secretKeyRef": {
                    "name": "claude-api-key",
                    "key": "api-key"
                }
            }
        }),
        json!({
            "name": "TASK_ID",
            "value": tr.spec.task_id.to_string()
        }),
        json!({
            "name": "SERVICE_NAME",
            "value": tr.spec.service_name.clone()
        }),
        json!({
            "name": "AGENT_NAME",
            "value": tr.spec.agent_name.clone()
        }),
        json!({
            "name": "HOME",
            "value": "/workspace"  // Set HOME to working directory for Claude settings
        }),
        json!({
            "name": "WORKDIR",
            "value": "/workspace"
        }),
    ];

    // Add telemetry environment variables from config
    env_vars.extend(telemetry_env);

    // Add GitHub token if repository is configured
    if let Some(repo) = &tr.spec.repository {
        // Auto-resolve secret name from GitHub user
        let secret_name = format!("github-pat-{}", repo.github_user);
        env_vars.push(json!({
            "name": "GITHUB_TOKEN",
            "valueFrom": {
                "secretKeyRef": {
                    "name": secret_name,
                    "key": "token" // Standard convention
                }
            }
        }));
    }

    // Note: Tool configuration is now handled via settings.json file creation
    // Environment variables CLAUDE_ENABLED_TOOLS and CLAUDE_TOOLS_CONFIG are not valid
    // per Claude Code documentation - tools must be configured via settings.json

    // Add any additional env vars from config
    for env_var in &config.agent.env {
        env_vars.push(json!({
            "name": env_var.name.clone(),
            "value": env_var.value.clone()
        }));
    }

    env_vars
}

/// Build resource attributes for OTEL
fn build_resource_attributes(tr: &TaskRun, config: &ControllerConfig) -> String {
    let mut attributes = vec![
        format!(
            "service.name={}-{}",
            config.telemetry.service_name, tr.spec.service_name
        ),
        format!("service.version={}", config.telemetry.service_version),
        format!(
            "service.namespace={}",
            tr.namespace().unwrap_or_else(|| "orchestrator".to_string())
        ),
        format!("task.id={}", tr.spec.task_id),
        format!("agent.name={}", tr.spec.agent_name),
        format!("team={}", config.telemetry.team_name),
        format!("department={}", config.telemetry.department),
        format!("environment={}", config.telemetry.environment),
        format!("cluster.name={}", config.telemetry.cluster_name),
    ];

    if !config.telemetry.cost_center.is_empty() {
        attributes.push(format!("cost_center={}", config.telemetry.cost_center));
    }

    if !config.telemetry.custom_attributes.is_empty() {
        attributes.push(config.telemetry.custom_attributes.clone());
    }

    attributes.join(",")
}

// CLAUDE.md generation removed - should be provided by client
// This allows task-specific instructions and Git workflows

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(tr: &TaskRun, _config: &ControllerConfig) -> Result<String> {
    let mut allow_rules = Vec::new();
    let mut deny_rules = Vec::new();

    if tr.spec.agent_tools.is_empty() {
        // Default tool permissions for standard development tasks
        allow_rules.extend(vec![
            "Bash(*)".to_string(),
            "Edit(*)".to_string(),
            "Read(*)".to_string(),
            "Write(*)".to_string(),
            "MultiEdit(*)".to_string(),
            "Glob(*)".to_string(),
            "Grep(*)".to_string(),
            "LS(*)".to_string(),
            "TodoRead(*)".to_string(),
            "TodoWrite(*)".to_string(),
            "WebFetch(*)".to_string(),
            "WebSearch(*)".to_string(),
        ]);

        // No deny rules by default - trust Claude to be responsible
    } else {
        // Build permissions based on agent_tools specification
        for tool in &tr.spec.agent_tools {
            if tool.enabled {
                let tool_rule = match tool.name.as_str() {
                    "bash" => {
                        if tool.restrictions.is_empty() {
                            "Bash(*)".to_string()
                        } else {
                            // Apply restrictions as deny rules
                            for restriction in &tool.restrictions {
                                deny_rules.push(format!("Bash({restriction})"));
                            }
                            "Bash(*)".to_string()
                        }
                    }
                    "edit" => "Edit(*)".to_string(),
                    "read" => "Read(*)".to_string(),
                    "write" => "Write(*)".to_string(),
                    "multiedit" => "MultiEdit(*)".to_string(),
                    "glob" => "Glob(*)".to_string(),
                    "grep" => "Grep(*)".to_string(),
                    "webfetch" => "WebFetch(*)".to_string(),
                    "websearch" => "WebSearch(*)".to_string(),
                    _ => continue, // Skip unknown tools
                };
                allow_rules.push(tool_rule);
            }
        }
    }

    // Use the correct Claude Code configuration format as per claudelog.com documentation
    // Claude's working directory is set to /workspace/{service_name}
    let settings = json!({
        "projects": {
            "/workspace": {
                "allowedTools": allow_rules.iter().map(|rule| {
                    // Convert "Bash(*)" format to "Bash" format
                    rule.replace("(*)", "")
                }).collect::<Vec<String>>(),
                "model": tr.spec.model
            }
        }
    });

    serde_json::to_string_pretty(&settings).map_err(Error::SerializationError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::TaskRunSpec;

    #[test]
    fn test_build_configmap() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                markdown_files: vec![
                    MarkdownFile {
                        filename: "task.md".to_string(),
                        content: "Task content".to_string(),
                        file_type: Some(MarkdownFileType::Task),
                    },
                    MarkdownFile {
                        filename: "design-spec.md".to_string(),
                        content: "Design spec".to_string(),
                        file_type: Some(MarkdownFileType::DesignSpec),
                    },
                ],
                agent_tools: vec![],
                repository: None,
            },
            status: None,
        };

        let config = ControllerConfig::default();
        let cm = build_configmap(&tr, "test-cm", &config).unwrap();
        let data = cm.data.unwrap();
        assert!(data.contains_key("task.md"));
        assert!(data.contains_key("design-spec.md"));
        assert_eq!(data.get("task.md").unwrap(), "Task content");
        assert_eq!(data.get("design-spec.md").unwrap(), "Design spec");
    }
}
