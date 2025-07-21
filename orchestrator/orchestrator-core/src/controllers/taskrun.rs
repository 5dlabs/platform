use crate::config::controller_config::ControllerConfig;
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

// Constants for docs generation detection
const DOCS_GENERATION_TASK_ID: u32 = 999999;

/// Check if a TaskRun is for documentation generation
fn is_docs_generation(tr: &TaskRun) -> bool {
    tr.spec.task_id == DOCS_GENERATION_TASK_ID
}

/// Get the appropriate job deadline based on task type
fn get_job_deadline_seconds(tr: &TaskRun, config: &ControllerConfig) -> i64 {
    if is_docs_generation(tr) {
        // Docs generation needs longer timeout for large projects (4 hours)
        14400
    } else {
        // Implementation jobs with checkpoint system (2 hours)
        // Use config value but with a minimum of 2 hours for implementation
        std::cmp::max(config.job.active_deadline_seconds, 7200)
    }
}

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

    // Ensure PVC exists for the service (skip for docs generation - uses emptyDir)
    if !is_docs_generation(&tr) {
    let pvc_name = format!("workspace-{}", tr.spec.service_name);
    ensure_pvc_exists(pvcs, &pvc_name, &tr.spec.service_name).await?;
    }

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
    if is_docs_generation(&tr) {
        info!("Documentation generation task detected, creating Claude job directly");
        create_claude_job(tr, jobs, taskruns, &cm_name, config).await?;
        return Ok(Action::requeue(Duration::from_secs(30)));
    }

    // For implementation tasks, create Claude job directly (no prep job needed)
    info!("Creating Claude job directly for implementation task");
    create_claude_job(tr, jobs, taskruns, &cm_name, config).await?;

    Ok(Action::requeue(Duration::from_secs(30)))
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

    // Markdown files are no longer stored in spec - all content is generated from templates

    // Generate CLAUDE.md from appropriate memory template based on task type
    // Only generate initial CLAUDE.md for first attempt - preserve existing memory on retries
    let is_retry = tr.status.as_ref().is_some_and(|s| s.attempts > 1);
    if !is_retry {
        let claude_memory = if is_docs_generation(tr) {
            generate_claude_memory(tr, DOCS_CLAUDE_TEMPLATE)?
        } else {
            generate_claude_memory(tr, IMPLEMENTATION_CLAUDE_TEMPLATE)?
        };
        data.insert("CLAUDE.md".to_string(), claude_memory);
    } else {
        // On retry attempts, we need to preserve the existing CLAUDE.md from the PVC
        // The container will handle copying the existing file from workspace
        info!("Retry attempt - preserving existing CLAUDE.md memory file");
    }

    // Add all hook scripts from the hooks directory
    let hook_scripts = generate_hook_scripts(tr)?;
    for (filename, content) in hook_scripts {
        // Use hooks- prefix instead of hooks/ to comply with ConfigMap key constraints
        // Kubernetes ConfigMap keys can only contain: [-._a-zA-Z0-9]+
        data.insert(format!("hooks-{}", filename), content);
    }

    // Generate Claude Code configuration file for tool permissions
    let settings_json = generate_claude_settings(tr, config)?;
    // For docs generation, insert as claude-settings.json to be copied to enterprise location
    // For other tasks, use settings-local.json to be copied to .claude/settings.local.json
    if is_docs_generation(tr) {
        data.insert("claude-settings.json".to_string(), settings_json);
    } else {
        data.insert("settings-local.json".to_string(), settings_json);

        // Generate MCP configuration for implementation tasks
        let mcp_json = generate_mcp_config(tr)?;
        data.insert("mcp.json".to_string(), mcp_json);

        // Add coding and GitHub guidelines for implementation tasks
        let coding_guidelines = generate_coding_guidelines(tr)?;
        data.insert("coding-guidelines.md".to_string(), coding_guidelines);

        let github_guidelines = generate_github_guidelines(tr)?;
        data.insert("github-guidelines.md".to_string(), github_guidelines);
    }

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

    // Build volumes list - use dedicated PVC for this service, emptyDir for docs generation
    let mut volumes = if is_docs_generation(tr) {
        // No workspace volume for docs generation (use container filesystem)
        vec![]
    } else {
        // Use PVC for implementation tasks (need persistent workspace)
    let pvc_name = format!("workspace-{service_name}");
        vec![json!({
        "name": "workspace",
        "persistentVolumeClaim": {
            "claimName": pvc_name
        }
        })]
    };

    // Configure volume mounts based on task type
    let mut volume_mounts = vec![];

    if !is_docs_generation(tr) {
        // Only mount workspace volume for implementation tasks
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));
    }

    if is_docs_generation(tr) {
        volumes.push(json!({
            "name": "task-files",
            "configMap": {
                "name": _cm_name
            }
        }));
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/config"
        }));
    }

    // Add SSH key volume if repository uses SSH URL
    if let Some(repo) = &tr.spec.repository {
        if repo.url.starts_with("git@") || repo.url.starts_with("ssh://") {
            // Use GitHub user-specific SSH key secret
            let ssh_secret_name = format!("github-ssh-{}", repo.github_user);

            volumes.push(json!({
                "name": "ssh-key",
                "secret": {
                    "secretName": ssh_secret_name,
                    "defaultMode": 0o600
                }
            }));

            volume_mounts.push(json!({
                "name": "ssh-key",
                "mountPath": "/ssh-keys",
                "readOnly": true
            }));
        }
    }

    // Telemetry and environment settings are now handled via settings.json
    // Only keep essential container-level env vars here

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
            "activeDeadlineSeconds": get_job_deadline_seconds(tr, config),
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
                        "env": build_env_vars(tr, config),
                        "volumeMounts": volume_mounts,
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





// Template constants
const IMPLEMENTATION_CONTAINER_TEMPLATE: &str = include_str!("../../templates/implementation/container.sh.hbs");
const DOCS_GENERATION_CONTAINER_TEMPLATE: &str = include_str!("../../templates/docs/container.sh.hbs");
const DOCS_GENERATION_PROMPT_TEMPLATE: &str = include_str!("../../templates/docs/prompt.hbs");
const IMPLEMENTATION_PROMPT_TEMPLATE: &str = include_str!("../../templates/implementation/prompt.hbs");
const DOCS_CLAUDE_TEMPLATE: &str = include_str!("../../templates/docs/claude.md.hbs");
const IMPLEMENTATION_CLAUDE_TEMPLATE: &str = include_str!("../../templates/implementation/claude.md.hbs");
const DOCS_SETTINGS_TEMPLATE: &str = include_str!("../../templates/docs/settings.json.hbs");
const IMPLEMENTATION_SETTINGS_TEMPLATE: &str = include_str!("../../templates/implementation/settings.json.hbs");
const IMPLEMENTATION_MCP_TEMPLATE: &str = include_str!("../../templates/implementation/mcp.json.hbs");
const IMPLEMENTATION_CODING_GUIDELINES_TEMPLATE: &str = include_str!("../../templates/implementation/coding-guidelines.md.hbs");
const IMPLEMENTATION_GITHUB_GUIDELINES_TEMPLATE: &str = include_str!("../../templates/implementation/github-guidelines.md.hbs");





/// Build startup script for the agent container
fn build_agent_startup_script(tr: &TaskRun, config: &ControllerConfig) -> Result<String, Error> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    // Export script is now created by prep job, no need to render it here

    // Choose template based on task type
    let is_docs_generation = is_docs_generation(tr);
    let template = if is_docs_generation {
        DOCS_GENERATION_CONTAINER_TEMPLATE
    } else {
        IMPLEMENTATION_CONTAINER_TEMPLATE
    };

    // Register the appropriate container template
    handlebars
        .register_template_string("main", template)
        .map_err(|e| Error::ConfigError(format!("Failed to register main template: {e}")))?;

    // Build the Claude command
    let command = config.agent.command.join(" ");

    // Extract working directory for docs generation tasks
    let mut working_dir = String::new();
    if is_docs_generation {
        // For docs generation, use working_directory field if present
        if let Some(wd) = &tr.spec.working_directory {
            working_dir = wd.clone();
        }
    }

    // Prepare template data
    let mut data = json!({
        "command": command,
        "model_override": tr.spec.model != "sonnet", // Non-default model
        "model": tr.spec.model.clone(),
        "is_retry": tr.status.as_ref().is_some_and(|s| s.attempts > 1),
        "attempts": tr.status.as_ref().map_or(1, |s| s.attempts),
        "task_id": tr.spec.task_id,
        "is_docs_generation": is_docs_generation, // Special docs generation task
        "working_dir": working_dir,
        "service_name": tr.spec.service_name.clone(),
    });

    // Add repository information for both docs generation and implementation tasks
    if let Some(repo) = &tr.spec.repository {
        data["repository"] = json!({
            "url": repo.url,
            "branch": repo.branch,
            "githubUser": repo.github_user
        });
    }

    // Add platform repository information if present
    if let Some(platform_repo) = &tr.spec.platform_repository {
        data["platform_repository"] = json!({
            "url": platform_repo.url,
            "branch": platform_repo.branch,
            "githubUser": platform_repo.github_user
        });
    }

    // Add working directory information
    if let Some(working_directory) = &tr.spec.working_directory {
        data["working_directory"] = json!(working_directory);
    }

    // Add prompt modification fields for retry attempts
    if let Some(prompt_modification) = &tr.spec.prompt_modification {
        data["prompt_modification"] = json!(prompt_modification);
    }
    data["prompt_mode"] = json!(tr.spec.prompt_mode);

    if is_docs_generation {
        // Generate target branch name for docs generation
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let target_branch = format!("docs-generation-{}", chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(chrono::Utc::now)
            .format("%Y%m%d-%H%M%S"));
        data["targetBranch"] = json!(target_branch);

        // Generate the prompt content for docs generation
        let prompt_content = generate_prompt(tr, DOCS_GENERATION_PROMPT_TEMPLATE)?;
        data["prompt_content"] = json!(prompt_content);
    } else {
        // Generate the prompt content for implementation tasks
        let prompt_content = generate_prompt(tr, IMPLEMENTATION_PROMPT_TEMPLATE)?;
        data["prompt_content"] = json!(prompt_content);
    }

    handlebars
        .render("main", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render template: {e}")))
}

/// Build environment variables for the container
fn build_env_vars(
    tr: &TaskRun,
    config: &ControllerConfig,
) -> Vec<serde_json::Value> {
    // Most configuration is now handled via settings.json
    // Only essential container-level env vars are set here
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

    // Note: Telemetry, tool permissions, and Claude configuration are now handled
    // via settings.json file creation rather than environment variables
    // This consolidates all Claude Code configuration in one place

    // Add any additional env vars from config
    for env_var in &config.agent.env {
        env_vars.push(json!({
            "name": env_var.name.clone(),
            "value": env_var.value.clone()
        }));
    }

    env_vars
}



/// Generate CLAUDE.md content from memory template
fn generate_claude_memory(tr: &TaskRun, template: &str) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("claude_memory", template)
        .map_err(|e| Error::ConfigError(format!("Failed to register CLAUDE.md template: {e}")))?;

    // Extract working directory from the TaskRun
    let working_directory = if tr.spec.repository.is_some() {
        // Use working_directory field from TaskRun spec
        tr.spec.working_directory.as_deref().unwrap_or(".")
    } else {
        "."
    };

    let data = json!({
        "repository": tr.spec.repository.as_ref().map(|r| json!({
            "url": r.url,
            "branch": r.branch,
            "githubUser": r.github_user
        })),
        "working_directory": working_directory,
        "task_id": if tr.spec.task_id == DOCS_GENERATION_TASK_ID { json!(null) } else { json!(tr.spec.task_id) }
    });

    handlebars
        .render("claude_memory", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render CLAUDE.md template: {e}")))
}

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(tr: &TaskRun, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    // Choose template based on job type
    let is_docs_generation = is_docs_generation(tr);
    let template = if is_docs_generation {
        DOCS_SETTINGS_TEMPLATE
    } else {
        IMPLEMENTATION_SETTINGS_TEMPLATE
    };

    handlebars
        .register_template_string("settings", template)
        .map_err(|e| Error::ConfigError(format!("Failed to register settings template: {e}")))?;

    // Build data for template
    let data = build_settings_template_data(tr, config)?;

    handlebars
        .render("settings", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render settings template: {e}")))
}

/// Generate MCP configuration for implementation tasks
fn generate_mcp_config(tr: &TaskRun) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("mcp", IMPLEMENTATION_MCP_TEMPLATE)
        .map_err(|e| Error::ConfigError(format!("Failed to register MCP template: {e}")))?;

    // Build basic template data - MCP template is simple and doesn't need complex data
    let data = json!({
        "service_name": tr.spec.service_name,
        "task_id": tr.spec.task_id
    });

    handlebars
        .render("mcp", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render MCP template: {e}")))
}

/// Generate coding guidelines for implementation tasks
fn generate_coding_guidelines(tr: &TaskRun) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("coding_guidelines", IMPLEMENTATION_CODING_GUIDELINES_TEMPLATE)
        .map_err(|e| Error::ConfigError(format!("Failed to register coding guidelines template: {e}")))?;

    // Build basic template data
    let data = json!({
        "service_name": tr.spec.service_name,
        "task_id": tr.spec.task_id
    });

    handlebars
        .render("coding_guidelines", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render coding guidelines template: {e}")))
}

/// Generate GitHub guidelines for implementation tasks
fn generate_github_guidelines(tr: &TaskRun) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("github_guidelines", IMPLEMENTATION_GITHUB_GUIDELINES_TEMPLATE)
        .map_err(|e| Error::ConfigError(format!("Failed to register GitHub guidelines template: {e}")))?;

    // Build basic template data
    let data = json!({
        "service_name": tr.spec.service_name,
        "task_id": tr.spec.task_id
    });

    handlebars
        .render("github_guidelines", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render GitHub guidelines template: {e}")))
}

/// Build template data for Claude settings generation
fn build_settings_template_data(tr: &TaskRun, config: &ControllerConfig) -> Result<serde_json::Value> {
    // Build telemetry configuration
    let telemetry_data = build_telemetry_data(config);

    // Build retry configuration
    let retry_data = build_retry_data(tr);

    // Model is now handled entirely in the templates
    // Docs template has hard-coded opus, implementation template uses user-specified model

    // Handle agent_tools override if specified
    let mut template_data = json!({
        "telemetry": telemetry_data,
        "retry": retry_data,
        "model": tr.spec.model.clone(), // Pass user-specified model to template
        "agent_tools_override": !tr.spec.agent_tools.is_empty()
    });

    // Only add permission arrays if agent_tools are specified (override case)
    if !tr.spec.agent_tools.is_empty() {
        let (allow_rules, deny_rules) = build_agent_tools_permissions(&tr.spec.agent_tools);
        template_data["permissions"] = json!({
            "allow": allow_rules,
            "deny": deny_rules
        });
    }

    Ok(template_data)
}

/// Translate agent_tools API format to Claude permission format (override case only)
/// This only handles the translation - templates define what the defaults are
fn build_agent_tools_permissions(agent_tools: &[crate::crds::AgentTool]) -> (Vec<String>, Vec<String>) {
    let mut allow_rules = Vec::new();
    let mut deny_rules = Vec::new();

    for tool in agent_tools {
        if tool.enabled {
            // Translate API tool names to Claude permission format
            let tool_rule = match tool.name.as_str() {
                "bash" => {
                    // Add restrictions as deny rules
                    for restriction in &tool.restrictions {
                        deny_rules.push(format!("Bash({restriction})"));
                    }
                    "Bash(*)".to_string()
                }
                "edit" => "Edit(*)".to_string(),
                "read" => "Read(*)".to_string(),
                "write" => "Write(*)".to_string(),
                "multiedit" => "MultiEdit(*)".to_string(),
                "glob" => "Glob(*)".to_string(),
                "grep" => "Grep(*)".to_string(),
                "ls" => "LS(*)".to_string(),
                "webfetch" => "WebFetch(*)".to_string(),
                "websearch" => "WebSearch(*)".to_string(),
                _ => {
                    // Log unknown tools but don't fail - allows for future extensibility
                    tracing::warn!("Unknown agent tool '{}' - skipping", tool.name);
                    continue;
                }
            };
            allow_rules.push(tool_rule);
        }
    }

    (allow_rules, deny_rules)
}

/// Build telemetry configuration data
fn build_telemetry_data(config: &ControllerConfig) -> serde_json::Value {
    if config.telemetry.enabled {
        // Provide the raw endpoint and protocol values for the template
        json!({
            "enabled": true,
            "otlpEndpoint": config.telemetry.otlp_endpoint,
            "otlpProtocol": config.telemetry.otlp_protocol
        })
    } else {
        json!({
            "enabled": false
        })
    }
}

/// Build retry configuration data
fn build_retry_data(tr: &TaskRun) -> serde_json::Value {
    let attempt_number = tr.status.as_ref().map_or(1, |s| s.attempts);
    json!({
        "is_retry": attempt_number > 1
    })
}

/// Generate all hook scripts from the hooks directory based on task type
fn generate_hook_scripts(tr: &TaskRun) -> Result<Vec<(String, String)>> {
    let mut hook_scripts = Vec::new();
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    // Determine task type directory
    let task_type = if is_docs_generation(tr) { "docs" } else { "implementation" };

    // Get list of hook templates for this task type
    let hook_templates = get_hook_templates(task_type);

    // Extract working directory from the TaskRun (for docs generation compatibility)
    let working_directory = if tr.spec.repository.is_some() {
        tr.spec.working_directory.as_deref().unwrap_or(".").to_string()
    } else {
        ".".to_string()
    };

    // Prepare template data
    let data = json!({
        "task_id": if tr.spec.task_id == DOCS_GENERATION_TASK_ID { json!(null) } else { json!(tr.spec.task_id) },
        "service_name": tr.spec.service_name,
        "repository": tr.spec.repository.as_ref(),
        "working_directory": working_directory,
        "attempts": tr.status.as_ref().map_or(1, |s| s.attempts),
        "is_docs_generation": is_docs_generation(tr),
    });

    // Process each hook template
    for (hook_name, template_content) in hook_templates {
        handlebars
            .register_template_string(&hook_name, &template_content)
            .map_err(|e| Error::ConfigError(format!("Failed to register hook template {}: {e}", hook_name)))?;

        let rendered = handlebars
            .render(&hook_name, &data)
            .map_err(|e| Error::ConfigError(format!("Failed to render hook template {}: {e}", hook_name)))?;

        // Generate output filename (remove .hbs extension)
        let output_filename = if hook_name.ends_with(".hbs") {
            hook_name.trim_end_matches(".hbs").to_string()
        } else {
            hook_name
        };

        hook_scripts.push((output_filename, rendered));
    }

    Ok(hook_scripts)
}

/// Get all hook templates for a specific task type
fn get_hook_templates(task_type: &str) -> Vec<(String, String)> {
    // For now, return the hardcoded templates based on task type
    // This can be made more dynamic in the future if needed
    match task_type {
        "docs" => vec![
            ("stop-pr-creation.sh.hbs".to_string(), include_str!("../../templates/docs/hooks/stop-pr-creation.sh.hbs").to_string()),
            ("early-test.sh.hbs".to_string(), include_str!("../../templates/docs/hooks/early-test.sh.hbs").to_string()),
        ],
        "implementation" => vec![
            ("stop-commit.sh.hbs".to_string(), include_str!("../../templates/implementation/hooks/stop-commit.sh.hbs").to_string()),
            ("early-test.sh.hbs".to_string(), include_str!("../../templates/implementation/hooks/early-test.sh.hbs").to_string()),
        ],
        _ => vec![],
    }
}

/// Generate prompt content from prompt template
fn generate_prompt(tr: &TaskRun, template: &str) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false); // Allow missing fields

    handlebars
        .register_template_string("prompt", template)
        .map_err(|e| Error::ConfigError(format!("Failed to register prompt template: {e}")))?;

    // Extract task_id for docs generation (999999 means "all tasks")
    let task_id = if tr.spec.task_id == DOCS_GENERATION_TASK_ID {
        None // Generate docs for all tasks
    } else {
        Some(tr.spec.task_id) // Generate docs for specific task
    };

    let data = json!({
        "task_id": task_id,
        "service_name": tr.spec.service_name,
        "repository": tr.spec.repository.as_ref(),
    });

    handlebars
        .render("prompt", &data)
        .map_err(|e| Error::ConfigError(format!("Failed to render prompt template: {e}")))
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
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let config = ControllerConfig::default();
        let cm = build_configmap(&tr, "test-cm", &config).unwrap();
        let data = cm.data.unwrap();

        // ConfigMap should contain generated template files instead of markdown files
        assert!(data.contains_key("CLAUDE.md"));
        assert!(data.contains_key("settings-local.json"));
        assert!(data.contains_key("mcp.json"));
        assert!(data.contains_key("coding-guidelines.md"));
        assert!(data.contains_key("github-guidelines.md"));
    }

    #[test]
    fn test_generate_claude_settings_implementation() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let config = ControllerConfig::default();
        let settings_json = generate_claude_settings(&tr, &config).unwrap();

        // Parse the JSON to verify it's valid
        let settings: serde_json::Value = serde_json::from_str(&settings_json).unwrap();

        // Verify key structure
        assert!(settings.get("permissions").is_some());
        assert!(settings.get("env").is_some());
        assert!(settings.get("model").is_some());
        assert_eq!(settings["model"], "sonnet");
        assert_eq!(settings["permissions"]["defaultMode"], "acceptEdits");
        assert!(settings.get("hooks").is_none()); // No hooks for implementation jobs
    }

    #[test]
    fn test_generate_claude_settings_docs() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: DOCS_GENERATION_TASK_ID, // Use docs generation task ID
                service_name: "docs-generator".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(), // This should be overridden
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let config = ControllerConfig::default();
        let settings_json = generate_claude_settings(&tr, &config).unwrap();

        // Parse the JSON to verify it's valid
        let settings: serde_json::Value = serde_json::from_str(&settings_json).unwrap();

        // Verify key structure
        assert!(settings.get("permissions").is_some());
        assert!(settings.get("env").is_some());
        assert!(settings.get("model").is_some());
        assert!(settings.get("hooks").is_some()); // Docs jobs have hooks
        assert_eq!(settings["model"], "claude-opus-4-20250514"); // Hard-coded in docs template
        assert_eq!(settings["permissions"]["defaultMode"], "acceptEdits");
        // Check the new hooks format with Stop array
        assert!(settings["hooks"]["Stop"].is_array());
        let stop_hooks = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop_hooks.len(), 1);
        assert_eq!(stop_hooks[0]["hooks"][0]["command"], "./.stop-hook-docs-pr.sh");
    }

    #[test]
    fn test_agent_tools_translation() {
        // Test that API format is correctly translated to Claude format
        let agent_tools = vec![
            crate::crds::AgentTool {
                name: "bash".to_string(),
                enabled: true,
                config: None,
                restrictions: vec!["rm:*".to_string(), "sudo:*".to_string()],
            },
            crate::crds::AgentTool {
                name: "websearch".to_string(),
                enabled: true,
                config: None,
                restrictions: vec![],
            },
            crate::crds::AgentTool {
                name: "edit".to_string(),
                enabled: false, // Should be ignored
                config: None,
                restrictions: vec![],
            },
        ];

        let (allow_rules, deny_rules) = build_agent_tools_permissions(&agent_tools);

        // Should translate enabled tools to Claude format
        assert!(allow_rules.contains(&"Bash(*)".to_string()));
        assert!(allow_rules.contains(&"WebSearch(*)".to_string()));

        // Should NOT include disabled tools
        assert!(!allow_rules.contains(&"Edit(*)".to_string()));

        // Should translate restrictions to deny rules
        assert!(deny_rules.contains(&"Bash(rm:*)".to_string()));
        assert!(deny_rules.contains(&"Bash(sudo:*)".to_string()));

        // Should have 2 allow rules (bash + websearch)
        assert_eq!(allow_rules.len(), 2);

        // Should have 2 deny rules (rm + sudo restrictions)
        assert_eq!(deny_rules.len(), 2);
    }

    #[test]
    fn test_generate_claude_memory_template_substitution() {
        // Test that template variables are properly substituted in claude memory generation
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: Some(crate::crds::taskrun::RepositorySpec {
                    url: "https://github.com/test/repo".to_string(),
                    branch: "main".to_string(),
                    github_user: "testuser".to_string(),
                    token: None,
                }),
                working_directory: Some("service-dir".to_string()),
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let memory = generate_claude_memory(&tr, IMPLEMENTATION_CLAUDE_TEMPLATE).unwrap();

        // Verify template variables were substituted correctly
        assert!(memory.contains("https://github.com/test/repo"));
        assert!(memory.contains("main"));
        assert!(memory.contains("testuser"));
        assert!(memory.contains("service-dir"));
        assert!(memory.contains("task 1001"));
        assert!(memory.contains("task/task.md"));
        assert!(memory.contains("task/acceptance-criteria.md"));
    }

    #[test]
    fn test_generate_mcp_config_template() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let mcp_config = generate_mcp_config(&tr).unwrap();

        // Parse as JSON to verify it's valid
        let config: serde_json::Value = serde_json::from_str(&mcp_config).unwrap();

        // Verify MCP config structure
        assert!(config.get("mcpServers").is_some());
        assert!(config["mcpServers"].get("toolman").is_some());
        assert_eq!(config["mcpServers"]["toolman"]["type"], "stdio");
        assert_eq!(config["mcpServers"]["toolman"]["command"], "/usr/local/bin/toolman-client");
    }

    #[test]
    fn test_generate_coding_guidelines_template() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let guidelines = generate_coding_guidelines(&tr).unwrap();

        // Verify template generated content
        assert!(guidelines.contains("# Rust Coding Guidelines"));
        assert!(guidelines.contains("Error Handling"));
        assert!(guidelines.contains("Memory Management"));
        assert!(guidelines.contains("Async Programming"));
        assert!(guidelines.contains("Testing Guidelines"));
    }

    #[test]
    fn test_generate_github_guidelines_template() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let guidelines = generate_github_guidelines(&tr).unwrap();

        // Verify template generated content
        assert!(guidelines.contains("# GitHub Workflow Guidelines"));
        assert!(guidelines.contains("Commit Message Format"));
        assert!(guidelines.contains("Branch Naming"));
        assert!(guidelines.contains("Pull Request Guidelines"));
        assert!(guidelines.contains("Code Review Standards"));
    }

    #[test]
    fn test_generate_hook_scripts_implementation() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: Some(crate::crds::taskrun::RepositorySpec {
                    url: "https://github.com/test/repo".to_string(),
                    branch: "main".to_string(),
                    github_user: "testuser".to_string(),
                    token: None,
                }),
                working_directory: Some("service-dir".to_string()),
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let hooks = generate_hook_scripts(&tr).unwrap();

        // Should generate 2 hook scripts for implementation tasks
        assert_eq!(hooks.len(), 2);

        // Verify hook script names
        let hook_names: Vec<&String> = hooks.iter().map(|(name, _)| name).collect();
        assert!(hook_names.contains(&&"stop-commit.sh".to_string()));
        assert!(hook_names.contains(&&"early-test.sh".to_string()));

        // Verify template content is generated correctly
        for (_, content) in &hooks {
            assert!(content.contains("#!/bin/bash"));
            assert!(content.len() > 100); // Hook scripts should be non-trivial
        }
    }

    #[test]
    fn test_build_settings_template_data() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "opus".to_string(),
                context_version: 1,
                agent_tools: vec![
                    crate::crds::AgentTool {
                        name: "bash".to_string(),
                        enabled: true,
                        config: None,
                        restrictions: vec!["rm:*".to_string()],
                    }
                ],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: Some(crate::crds::taskrun::TaskRunStatus {
                phase: Some(crate::crds::taskrun::TaskRunPhase::Running),
                job_name: None,
                config_map_name: None,
                attempts: 2,
                last_updated: None,
                message: None,
                session_id: None,
                conditions: vec![],
            }),
        };

        let config = crate::config::controller_config::ControllerConfig::default();
        let data = build_settings_template_data(&tr, &config).unwrap();

        // Verify template data structure
        assert_eq!(data["model"], "opus");
        assert_eq!(data["agent_tools_override"], true);
        assert_eq!(data["retry"]["is_retry"], true);
        assert!(data.get("permissions").is_some());

        // Verify telemetry data structure
        assert!(data.get("telemetry").is_some());
        assert_eq!(data["telemetry"]["enabled"], true); // Default config has telemetry enabled
    }

    #[test]
    fn test_template_conditional_logic() {
        // Test conditional logic in templates with docs generation
        let tr_docs = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: DOCS_GENERATION_TASK_ID,
                service_name: "docs-generator".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: Some(crate::crds::taskrun::RepositorySpec {
                    url: "https://github.com/test/repo".to_string(),
                    branch: "main".to_string(),
                    github_user: "testuser".to_string(),
                    token: None,
                }),
                working_directory: Some("docs".to_string()),
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let memory_docs = generate_claude_memory(&tr_docs, DOCS_CLAUDE_TEMPLATE).unwrap();

        // Should not include task ID for docs generation (uses null)
        assert!(!memory_docs.contains("task 999999"));

        // Test implementation task
        let tr_impl = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        let memory_impl = generate_claude_memory(&tr_impl, IMPLEMENTATION_CLAUDE_TEMPLATE).unwrap();

        // Should include specific task ID for implementation
        assert!(memory_impl.contains("task 1001"));
    }

    #[test]
    fn test_template_error_handling() {
        let tr = TaskRun {
            metadata: Default::default(),
            spec: TaskRunSpec {
                task_id: 1001,
                service_name: "test-service".to_string(),
                agent_name: "claude-agent-1".to_string(),
                model: "sonnet".to_string(),
                context_version: 1,
                agent_tools: vec![],
                repository: None,
                working_directory: None,
                platform_repository: None,
                prompt_modification: None,
                prompt_mode: "append".to_string(),
            },
            status: None,
        };

        // Test with invalid template - should return error
        let invalid_template = "{{invalid_syntax";
        let result = generate_claude_memory(&tr, invalid_template);
        assert!(result.is_err());
    }
}
