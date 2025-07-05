use crate::config::controller_config::ControllerConfig;
#[cfg(test)]
use crate::crds::{MarkdownFile, MarkdownFileType};
use crate::crds::{TaskRun, TaskRunPhase};
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::{
    api::{batch::v1::Job, core::v1::ConfigMap},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
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

    let name = tr.name_any();
    info!("Reconciling TaskRun: {}", name);

    // Handle finalizers for cleanup
    let _result = finalizer(&taskruns, FINALIZER_NAME, tr.clone(), |event| async {
        match event {
            FinalizerEvent::Apply(tr) => {
                // Create or update resources
                reconcile_create_or_update(tr, &jobs, &configmaps, &taskruns, &ctx.config).await
            }
            FinalizerEvent::Cleanup(tr) => {
                // Cleanup resources when TaskRun is deleted
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
    taskruns: &Api<TaskRun>,
    config: &ControllerConfig,
) -> Result<Action> {
    let name = tr.name_any();

    // Update status to Pending if not set
    if tr.status.is_none() {
        update_status(taskruns, &name, TaskRunPhase::Pending, "TaskRun created").await?;
    }

    // Check for existing jobs with older versions
    let job_list = jobs
        .list(&ListParams::default().labels(&format!("task-id={}", tr.spec.task_id)))
        .await?;

    // Delete older versions
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

    // Create ConfigMap with matching name pattern
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

    // Create Job with descriptive name: agent-service-task-attempt
    let job_name = format!(
        "{}-{}-task{}-attempt{}",
        tr.spec.agent_name.replace('_', "-"),
        tr.spec.service_name.replace('_', "-"),
        tr.spec.task_id,
        tr.spec.context_version
    );

    let job = build_job(&tr, &job_name, &cm_name, config)?;
    match jobs.create(&PostParams::default(), &job).await {
        Ok(_) => {
            info!("Created Job: {}", job_name);
            update_status_with_details(
                taskruns,
                &name,
                TaskRunPhase::Running,
                "Job created successfully",
                Some(job_name.clone()),
                Some(cm_name),
            )
            .await?;
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
            update_status(taskruns, &name, TaskRunPhase::Running, "Job already exists").await?;
        }
        Err(e) => {
            update_status(
                taskruns,
                &name,
                TaskRunPhase::Failed,
                &format!("Failed to create job: {e}"),
            )
            .await?;
            return Err(e.into());
        }
    }

    Ok(Action::requeue(Duration::from_secs(30)))
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
    data.insert(".claude.json".to_string(), settings_json);

    // Generate CLAUDE.md if not provided by Task Master system
    let has_claude_md = tr
        .spec
        .markdown_files
        .iter()
        .any(|f| f.filename == "CLAUDE.md");
    if !has_claude_md {
        let claude_md_content = generate_claude_md(tr);
        data.insert("CLAUDE.md".to_string(), claude_md_content);
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

/// Build Job from TaskRun
fn build_job(
    tr: &TaskRun,
    job_name: &str,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    // API key will be injected from secret

    // Build telemetry environment variables from config
    // Build volume mounts for init container
    let mut init_volume_mounts = vec![
        json!({
            "name": "task-files",
            "mountPath": "/config"
        }),
        json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }),
    ];

    // Build volumes list
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
                "claimName": "shared-workspace"
            }
        }),
    ];

    // Add secret volume and mount if repository is configured
    if let Some(repo) = &tr.spec.repository {
        // Auto-resolve secret name from GitHub user
        let secret_name = format!("github-pat-{}", repo.github_user);
        let secret_volume_name = format!("{secret_name}-secret");

        // Add volume mount to init container
        init_volume_mounts.push(json!({
            "name": secret_volume_name.clone(),
            "mountPath": format!("/secrets/{}", secret_name),
            "readOnly": true
        }));

        // Add secret volume
        volumes.push(json!({
            "name": secret_volume_name,
            "secret": {
                "secretName": secret_name
            }
        }));
    }

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
                    "initContainers": [{
                        "name": "prepare-workspace",
                        "image": format!("{}:{}", config.init_container.image.repository, config.init_container.image.tag),
                        "command": ["/bin/sh", "-c"],
                        "args": [build_init_script(tr, config)],
                        "volumeMounts": init_volume_mounts
                    }],
                    "containers": [{
                        "name": "claude-agent",
                        "image": format!("{}:{}", config.agent.image.repository, config.agent.image.tag),
                        "command": ["/bin/sh", "-c"],
                        "args": [build_agent_startup_script(tr, config)],
                        "env": build_env_vars(tr, telemetry_env, config),
                        "volumeMounts": [{
                            "name": "workspace",
                            "mountPath": "/workspace"
                        }],
                        "workingDir": format!("/workspace/{}", tr.spec.service_name),
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

/// Build init container script for workspace preparation
fn build_init_script(tr: &TaskRun, _config: &ControllerConfig) -> String {
    let service = &tr.spec.service_name;
    let task_id = tr.spec.task_id;
    let _version = tr.spec.context_version;

    let mut script = String::new();

    // Install gh CLI if not present (alpine/git image already has git)
    script.push_str("which gh >/dev/null 2>&1 || apk add --no-cache github-cli\n");

    // Create workspace directory (no per-attempt subdirectory for --continue support)
    script.push_str(&format!("mkdir -p /workspace/{service}/.task/{task_id}\n"));

    // Clone repository if specified
    if let Some(repo) = &tr.spec.repository {
        script.push_str(&format!("echo 'Cloning repository: {}'\n", repo.url));

        // Setup authentication using GitHub user to resolve secret
        let secret_name = format!("github-pat-{}", repo.github_user);
        let secret_key = "token"; // Standard convention

        // Export GitHub token from secret
        script.push_str(&format!(
            "export GITHUB_TOKEN=$(cat /secrets/{secret_name}/{secret_key} 2>/dev/null)\n"
        ));
        script.push_str("if [ -n \"$GITHUB_TOKEN\" ]; then\n");
        script.push_str(&format!(
            "  echo \"GitHub token loaded from secret: {secret_name}\"\n"
        ));
        script.push_str("  \n");
        script.push_str("  # Configure git global settings\n");
        script.push_str("  git config --global user.name \"Claude Agent\"\n");
        script.push_str("  git config --global user.email \"claude@5dlabs.com\"\n");
        script.push_str("  \n");
        script.push_str("  # Configure git to use the token for HTTPS authentication\n");
        script.push_str(
            "  git config --global credential.helper 'store --file=/workspace/.git-credentials'\n",
        );
        script.push_str(
            "  echo \"https://oauth2:${GITHUB_TOKEN}@github.com\" > /workspace/.git-credentials\n",
        );
        script.push_str("  chmod 600 /workspace/.git-credentials\n");
        script.push_str("  \n");
        script.push_str("  # Also configure for the specific service directory\n");
        script.push_str(&format!("  mkdir -p /workspace/{service}/.git\n"));
        script.push_str(&format!(
            "  cp /workspace/.git-credentials /workspace/{service}/.git-credentials\n"
        ));
        script.push_str("  \n");
        script.push_str("  # Configure gh CLI authentication\n");
        script.push_str("  echo \"${GITHUB_TOKEN}\" > /workspace/.gh-token\n");
        script.push_str("  gh auth login --with-token < /workspace/.gh-token 2>/dev/null || echo \"gh auth already configured\"\n");
        script.push_str("  rm -f /workspace/.gh-token\n");
        script.push_str("  \n");
        script.push_str("  # Write GITHUB_TOKEN to a file for the main container\n");
        script
            .push_str("  echo \"export GITHUB_TOKEN=${GITHUB_TOKEN}\" > /workspace/.github-env\n");
        script.push_str("  chmod 644 /workspace/.github-env\n");
        script.push_str("  # Ensure git config is accessible to the agent user\n");
        script.push_str("  chmod 644 /workspace/.git-credentials\n");
        script.push_str("else\n");
        script.push_str(&format!(
            "  echo \"Warning: GitHub token not found in secret: {secret_name}\"\n"
        ));
        script.push_str("fi\n");

        // Smart repository management: only clone if needed, otherwise sync
        script.push_str(&format!(
            "cd /workspace/{service}\n\
            if [ ! -d '.git' ]; then\n\
              if [ -z \"$(ls -A . 2>/dev/null)\" ]; then\n\
                echo 'Directory is empty, cloning repository...'\n\
                git clone --depth 1 --branch {} {} . || echo 'Clone failed, continuing...'\n\
              else\n\
                echo 'Directory not empty but not a Git repo, backing up existing files and cloning...'\n\
                mkdir -p .backup\n\
                mv ./* .backup/ 2>/dev/null || true\n\
                mv .[^.]* .backup/ 2>/dev/null || true\n\
                git clone --depth 1 --branch {} {} . || echo 'Clone failed, continuing...'\n\
                echo 'Repository cloned, existing files backed up to .backup/'\n\
              fi\n\
            else\n\
              echo 'Git repository exists, checking if it matches the target repository...'\n\
              CURRENT_REMOTE=$(git remote get-url origin 2>/dev/null || echo '')\n\
              if [ \"$CURRENT_REMOTE\" = \"{}\" ]; then\n\
                echo 'Repository matches, updating to latest changes...'\n\
                git fetch origin {} --depth 1 2>/dev/null || echo 'Fetch failed, continuing with existing code'\n\
                git reset --hard origin/{} 2>/dev/null || echo 'Reset failed, continuing with existing code'\n\
              else\n\
                echo 'Different repository detected, updating remote and fetching...'\n\
                git remote set-url origin {}\n\
                git fetch origin {} --depth 1 || echo 'Fetch failed, continuing...'\n\
                git reset --hard origin/{} || echo 'Reset failed, continuing...'\n\
              fi\n\
            fi\n",
            repo.branch, repo.url, repo.branch, repo.url, repo.url, repo.branch, repo.branch, repo.url, repo.branch, repo.branch
        ));
    } else {
        script.push_str(&format!(
            "echo 'No repository specified, using empty workspace for service: {service}'\n"
        ));
    }

    // Copy task files to .task directory
    script.push_str(&format!(
        "cp /config/* /workspace/{service}/.task/{task_id}/ 2>/dev/null || echo 'No config files to copy'\n"
    ));

    // Copy all task files to service root for @import access
    script.push_str(&format!(
        "cp /workspace/{service}/.task/{task_id}/*.md /workspace/{service}/ 2>/dev/null || echo 'No markdown files to copy to root'\n"
    ));

    // Setup Claude Code configuration directory and copy settings
    script.push_str("mkdir -p /home/node/.claude/todos\n");
    script.push_str("cp /config/.claude.json /home/node/.claude.json 2>/dev/null || echo 'No .claude.json to copy'\n");
    script.push_str("chmod -R 755 /home/node/.claude\n");
    script.push_str("chown -R 1000:1000 /home/node/.claude\n");

    // Also copy config to service directory for Claude Code (multiple locations for reliability)
    script.push_str(&format!("mkdir -p /workspace/{service}/.claude\n"));
    script.push_str(&format!("cp /config/.claude.json /workspace/{service}/.claude.json 2>/dev/null || echo 'No .claude.json to copy to service dir'\n"));
    script.push_str(&format!("cp /config/.claude.json /workspace/{service}/.claude/settings.local.json 2>/dev/null || echo 'No .claude.json to copy as settings.local.json'\n"));

    // Create .gitignore to prevent committing Claude internal files
    script.push_str(&format!("cat > /workspace/{service}/.gitignore << 'EOF'\n"));
    script.push_str("# Claude Code internal files - do not commit\n");
    script.push_str(".claude/\n");
    script.push_str(".claude.json\n");
    script.push_str(".task/\n");
    script.push_str("task.md\n");
    script.push_str("CLAUDE.md\n");
    script.push_str(".gitconfig\n");
    script.push_str(".git-credentials\n");
    script.push_str("EOF\n");

    // Clean up any credentials from parent workspace to prevent accidental commits
    script.push_str("echo 'Cleaning up credentials from parent workspace'\n");
    script.push_str("rm -f /workspace/.git-credentials 2>/dev/null || true\n");
    script.push_str("rm -f /workspace/.github-env 2>/dev/null || true\n");

    // DEBUG: Show configuration details
    script.push_str("echo '=== DEBUGGING CONFIGURATION ==='\n");
    if let Some(repo) = &tr.spec.repository {
        script.push_str(&format!("echo 'Repository URL: {}'\n", repo.url));
        script.push_str(&format!("echo 'Repository Branch: {}'\n", repo.branch));
        script.push_str(&format!("echo 'GitHub User: {}'\n", repo.github_user));
        script.push_str(&format!(
            "echo 'Secret Name: github-pat-{}'\n",
            repo.github_user
        ));
    } else {
        script.push_str("echo 'No repository specified in TaskRun spec'\n");
    }
    script.push_str("echo 'Claude configuration contents:'\n");
    script.push_str("cat /config/.claude.json 2>/dev/null || echo 'No .claude.json found'\n");
    script.push_str("echo 'File permissions in service .claude directory:'\n");
    script.push_str(&format!(
        "ls -la /workspace/{service}/.claude/ 2>/dev/null || echo 'No .claude directory found'\n"
    ));
    script.push_str("echo '=== END DEBUGGING ==='\n");

    script.push_str("echo 'Workspace prepared successfully'\n");
    script.push_str("ls -la /workspace/\n");
    script.push_str(&format!(
        "ls -la /workspace/{service}/ || echo 'Service directory not found'\n"
    ));

    script
}

/// Build startup script for the agent container
fn build_agent_startup_script(tr: &TaskRun, config: &ControllerConfig) -> String {
    let mut script = String::new();

    // Source GitHub environment if it exists (using . instead of source for sh compatibility)
    script.push_str("if [ -f /workspace/.github-env ]; then\n");
    script.push_str("  . /workspace/.github-env\n");
    script.push_str("  echo \"GitHub authentication configured\"\n");
    script.push_str("fi\n\n");

    // Configure git credentials for HTTPS authentication if GITHUB_TOKEN is available
    script.push_str("if [ -n \"$GITHUB_TOKEN\" ]; then\n");
    script.push_str("  echo 'Configuring git credentials for GitHub authentication'\n");
    script.push_str("  git config --global user.name \"Claude Agent\"\n");
    script.push_str("  git config --global user.email \"claude@5dlabs.com\"\n");
    script.push_str(
        "  git config --global credential.helper 'store --file=$HOME/.git-credentials'\n",
    );
    script.push_str(
        "  echo \"https://oauth2:${GITHUB_TOKEN}@github.com\" > \"$HOME/.git-credentials\"\n",
    );
    script.push_str("  chmod 600 \"$HOME/.git-credentials\"\n");
    script.push_str("  echo 'Git credentials configured successfully'\n");
    script.push_str("else\n");
    script.push_str("  echo 'No GITHUB_TOKEN found, skipping git credential setup'\n");
    script.push_str("fi\n\n");

    // COMPREHENSIVE DEBUGGING BEFORE CLAUDE STARTS
    script.push_str("echo '=== COMPREHENSIVE CLAUDE DEBUGGING ==='\n");
    script.push_str("echo 'Installing tree utility for filesystem debugging...'\n");
    script.push_str("apk add --no-cache tree 2>/dev/null || echo 'Tree installation failed'\n");
    script.push_str("echo '\n--- Environment Variables ---'\n");
    script.push_str("env | grep -E '(HOME|PWD|WORKDIR|CLAUDE)' | sort\n");
    script.push_str("echo '\n--- Current Working Directory ---'\n");
    script.push_str("pwd\n");
    script.push_str("echo '\n--- User Information ---'\n");
    script.push_str("whoami\n");
    script.push_str("id\n");
    script.push_str("echo '\n--- Current Directory Tree ---'\n");
    script.push_str("tree -a . 2>/dev/null || find . -type f 2>/dev/null | head -20\n");
    script.push_str("echo '\n--- HOME Directory Contents ---'\n");
    script.push_str("echo \"HOME is set to: $HOME\"\n");
    script.push_str("ls -la \"$HOME\" 2>/dev/null || echo 'HOME directory not accessible'\n");
    script.push_str("echo '\n--- Claude Config Directory ---'\n");
    script
        .push_str("ls -la \"$HOME/.claude\" 2>/dev/null || echo 'No .claude directory in HOME'\n");
    script.push_str("echo '\n--- Settings.json Content ---'\n");
    script.push_str("cat \"$HOME/.claude/settings.json\" 2>/dev/null || echo 'No settings.json found in HOME/.claude'\n");
    script.push_str("echo '\n--- Alternative Claude Config Locations ---'\n");
    script.push_str("find . -name 'settings.json' -type f 2>/dev/null\n");
    script.push_str("find .. -name 'settings.json' -type f 2>/dev/null | head -5\n");
    script.push_str("echo '\n--- File Permissions on All Settings.json ---'\n");
    script.push_str("find . -name 'settings.json' -exec ls -la {} \\; 2>/dev/null\n");
    script.push_str("find .. -name 'settings.json' -exec ls -la {} \\; 2>/dev/null | head -5\n");
    script.push_str("echo '\n--- Parent Directory Structure ---'\n");
    script.push_str("ls -la .. 2>/dev/null || echo 'Cannot access parent directory'\n");
    script.push_str("echo '=== END DEBUGGING - STARTING CLAUDE ==='\n\n");

    // Print Claude Code's actual loaded settings before execution
    script.push_str("echo '\n--- CLAUDE CODE SETTINGS DEBUG ---'\n");
    script.push_str("echo 'Testing Claude Code settings loading...'\n");
    let command = config.agent.command.join(" ");
    script.push_str(&format!("echo 'Claude command: {command}'\n"));
    script.push_str(&format!(
        "{command} --version 2>/dev/null || echo 'Claude version failed'\n"
    ));
    script.push_str(&format!(
        "{command} --help 2>&1 | head -20 || echo 'Claude help failed'\n"
    ));
    script.push_str("echo '\n--- CLAUDE CONFIG COMMANDS DEBUG ---'\n");
    script.push_str(&format!(
        "{command} config --help 2>&1 || echo 'Claude config help failed'\n"
    ));
    script.push_str(&format!(
        "{command} config list 2>&1 || echo 'Claude config list failed'\n"
    ));
    script.push_str(&format!(
        "{command} config show 2>&1 || echo 'Claude config show failed'\n"
    ));
    script.push_str(&format!(
        "{command} config get defaultMode 2>&1 || echo 'Claude config get defaultMode failed'\n"
    ));
    script.push_str(&format!(
        "{command} config get permissions 2>&1 || echo 'Claude config get permissions failed'\n"
    ));
    script.push_str("echo '\n--- CLAUDE SETTINGS FILE DISCOVERY ---'\n");
    script.push_str(&format!(
        "{command} --print-config-path 2>&1 || echo 'Claude print-config-path not available'\n"
    ));
    script.push_str(&format!(
        "{command} --debug 2>&1 | head -10 || echo 'Claude debug output failed'\n"
    ));
    script.push_str("echo '\n--- DEVCONTAINER AND ENVIRONMENT VARIABLES ---'\n");
    script.push_str("env | grep -i devcontainer || echo 'No DEVCONTAINER variables found'\n");
    script.push_str("env | grep -i claude || echo 'No CLAUDE environment variables found'\n");
    script.push_str("echo '\n--- SETTING CLAUDE CONFIG DIRECTORY ---'\n");
    script.push_str("export CLAUDE_CONFIG_DIR=/workspace/debug-api/.claude\n");
    script.push_str("echo \"CLAUDE_CONFIG_DIR set to: $CLAUDE_CONFIG_DIR\"\n");
    script.push_str("echo '\n--- UNSETTING DEVCONTAINER VARIABLES ---'\n");
    script.push_str("unset DEVCONTAINER 2>/dev/null || true\n");
    script.push_str("unset DEVCONTAINER_CONFIG 2>/dev/null || true\n");
    script.push_str("echo 'DEVCONTAINER variables unset'\n");
    script.push_str("echo '\n--- FINAL SETTINGS CHECK ---'\n");
    script.push_str("echo 'Attempting to read settings.json with cat:'\n");
    script.push_str("find . -name 'settings.json' -exec echo 'Found settings.json:' {} \\; -exec cat {} \\; 2>/dev/null\n");
    script.push_str("echo '\n--- TESTING PERMISSIVE MODE EXPLICITLY ---'\n");
    script.push_str(&format!(
        "{command} --help | grep -i permission || echo 'No permission flags found in help'\n"
    ));
    script.push_str(&format!(
        "{command} --help | grep -i allow || echo 'No allow flags found in help'\n"
    ));
    script.push_str(&format!(
        "{command} --help | grep -i mode || echo 'No mode flags found in help'\n"
    ));
    script.push_str("echo '\n--- TESTING SIMPLE CLAUDE COMMAND WITHOUT PROMPT ---'\n");
    script.push_str(&format!("{command} --version 2>&1\n"));
    script.push_str("echo '\n--- STARTING CLAUDE WITH FULL ARGS ---'\n");

    // Execute the Claude command with model selection and continuation
    let mut args = config.agent.args.clone();

    // Add model argument if not "sonnet" (default)
    if tr.spec.model != "sonnet" {
        args.insert(0, format!("--model={}", tr.spec.model));
    }

    // Add --continue flag for retry attempts (attempts > 1)
    let attempts = tr.status.as_ref().map(|s| s.attempts).unwrap_or(0);
    if attempts > 1 {
        args.push("--continue".to_string());
        script.push_str(&format!(
            "echo 'Adding --continue flag for attempt {attempts}'\n"
        ));
    }

    let args_str = args.join(" ");
    script.push_str(&format!(
        "echo 'Final Claude command with model: {command} {args_str}'\n"
    ));
    script.push_str(&format!("exec {command} {args_str}"));

    script
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
            "value": format!("/workspace/{}", tr.spec.service_name)  // Set HOME to working directory for Claude settings
        }),
        json!({
            "name": "WORKDIR",
            "value": format!("/workspace/{}", tr.spec.service_name)
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

/// Generate CLAUDE.md file that imports task files
fn generate_claude_md(tr: &TaskRun) -> String {
    let mut content = String::new();

    content.push_str("# Task Context\n\n");
    content.push_str(
        "This workspace contains all the necessary files to complete the assigned task.\n\n",
    );

    // Add @import statements for each markdown file
    content.push_str("## Task Files\n\n");
    for file in &tr.spec.markdown_files {
        if file.filename != "CLAUDE.md" {
            content.push_str(&format!("@{}\n\n", file.filename));
        }
    }

    // Add repository information if available
    if let Some(repo) = &tr.spec.repository {
        content.push_str("## Repository\n\n");
        content.push_str(&format!("- **URL**: {}\n", repo.url));
        content.push_str(&format!("- **Branch**: {}\n", repo.branch));
        content.push_str(&format!("- **GitHub User**: {}\n", repo.github_user));
        content.push('\n');
    }

    // Add task metadata
    content.push_str("## Task Metadata\n\n");
    content.push_str(&format!("- **Task ID**: {}\n", tr.spec.task_id));
    content.push_str(&format!("- **Service**: {}\n", tr.spec.service_name));
    content.push_str(&format!("- **Agent**: {}\n", tr.spec.agent_name));
    content.push_str(&format!(
        "- **Context Version**: {}\n",
        tr.spec.context_version
    ));
    content.push('\n');

    content.push_str("## Instructions\n\n");
    content.push_str("1. Review all task files using the @import statements above\n");
    content.push_str("2. Follow the design specification and implementation guidelines\n");
    content.push_str("3. Ensure all acceptance criteria are met\n");
    content.push_str("4. Create a pull request when implementation is complete\n");

    content
}

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
    let workspace_path = format!("/workspace/{}", tr.spec.service_name);
    let settings = json!({
        "projects": {
            workspace_path: {
                "allowedTools": allow_rules.iter().map(|rule| {
                    // Convert "Bash(*)" format to "Bash" format
                    rule.replace("(*)", "")
                }).collect::<Vec<String>>()
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
