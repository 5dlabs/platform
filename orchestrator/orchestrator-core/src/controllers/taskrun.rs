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

    // Create ConfigMap
    let cm_name = format!(
        "{}-{}-v{}-files",
        tr.spec.service_name, tr.spec.task_id, tr.spec.context_version
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
    let job_name = format!(
        "{}-{}-v{}",
        tr.spec.service_name, tr.spec.task_id, tr.spec.context_version
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

/// Update TaskRun status with additional details
async fn update_status_with_details(
    api: &Api<TaskRun>,
    name: &str,
    phase: TaskRunPhase,
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
        &Patch::Merge(json!({"status": status})),
    )
    .await
    .map_err(Error::KubeError)?;

    Ok(())
}

/// Build ConfigMap from TaskRun
fn build_configmap(tr: &TaskRun, name: &str) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Add all markdown files
    for file in &tr.spec.markdown_files {
        data.insert(file.filename.clone(), file.content.clone());
    }

    // Generate Claude Code settings.json file for tool permissions
    let settings_json = generate_claude_settings(tr)?;
    data.insert("settings.json".to_string(), settings_json);

    // Note: CLAUDE.md should be provided by Task Master system as one of the markdown files
    // No hard-coded content generation in orchestrator

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
                        "volumeMounts": [
                            {
                                "name": "task-files",
                                "mountPath": "/config"
                            },
                            {
                                "name": "workspace",
                                "mountPath": "/workspace"
                            }
                        ]
                    }],
                    "containers": [{
                        "name": "claude-agent",
                        "image": format!("{}:{}", config.agent.image.repository, config.agent.image.tag),
                        "command": config.agent.command.clone(),
                        "args": config.agent.args.clone(),
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
                    "volumes": [
                        {
                            "name": "task-files",
                            "configMap": {
                                "name": cm_name
                            }
                        },
                        {
                            "name": "workspace",
                            "persistentVolumeClaim": {
                                "claimName": "shared-workspace"
                            }
                        }
                    ]
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
    let version = tr.spec.context_version;

    let mut script = String::new();

    // Install git if not present
    script.push_str("apk add --no-cache git || apt-get update && apt-get install -y git || yum install -y git || echo 'Git already available'\n");

    // Create workspace directory
    script.push_str(&format!(
        "mkdir -p /workspace/{service}/.task/{task_id}/run-{version}\n"
    ));

    // Clone repository if specified
    if let Some(repo) = &tr.spec.repository {
        script.push_str(&format!("echo 'Cloning repository: {}'\n", repo.url));

        // Clone the repository
        script.push_str(&format!(
            "cd /workspace/{service} && git clone --depth 1 --branch {} {} . || echo 'Clone failed, continuing...'\n",
            repo.branch,
            repo.url
        ));

        // If a specific path is specified, restructure
        if let Some(path) = &repo.path {
            script.push_str(&format!(
                "if [ -d './{path}' ]; then \
                 mv ./{path}/* . && \
                 mv ./{path}/.[^.]* . 2>/dev/null || true && \
                 rmdir ./{path}; \
                 fi\n"
            ));
        }
    } else {
        script.push_str(&format!(
            "echo 'No repository specified, using empty workspace for service: {service}'\n"
        ));
    }

    // Copy task files to .task directory
    script.push_str(&format!(
        "cp /config/* /workspace/{service}/.task/{task_id}/run-{version}/ 2>/dev/null || echo 'No config files to copy'\n"
    ));

    // Copy all task files to service root for @import access
    script.push_str(&format!(
        "cp /workspace/{service}/.task/{task_id}/run-{version}/*.md /workspace/{service}/ 2>/dev/null || echo 'No markdown files to copy to root'\n"
    ));

    // Setup Claude Code configuration directory and copy settings
    script.push_str("mkdir -p /workspace/.claude\n");
    script.push_str("cp /config/settings.json /workspace/.claude/settings.json 2>/dev/null || echo 'No settings.json to copy'\n");

    // Also copy settings to service directory for Claude Code
    script.push_str(&format!("mkdir -p /workspace/{service}/.claude\n"));
    script.push_str(&format!("cp /config/settings.json /workspace/{service}/.claude/settings.json 2>/dev/null || echo 'No settings.json to copy to service dir'\n"));

    script.push_str("echo 'Workspace prepared successfully'\n");
    script.push_str("ls -la /workspace/\n");
    script.push_str(&format!(
        "ls -la /workspace/{service}/ || echo 'Service directory not found'\n"
    ));

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
            "value": format!("/workspace/{}", tr.spec.service_name)
        }),
    ];

    // Add telemetry environment variables from config
    env_vars.extend(telemetry_env);

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

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(tr: &TaskRun) -> Result<String> {
    let mut allow_rules = Vec::new();
    let mut deny_rules = Vec::new();

    if tr.spec.agent_tools.is_empty() {
        // Default tool permissions for standard development tasks
        allow_rules.extend(vec![
            "Bash(git *)".to_string(),
            "Bash(npm run *)".to_string(),
            "Bash(cargo *)".to_string(),
            "Bash(ls *)".to_string(),
            "Bash(find *)".to_string(),
            "Bash(grep *)".to_string(),
            "Bash(cat *)".to_string(),
            "Bash(head *)".to_string(),
            "Bash(tail *)".to_string(),
            "Bash(tree *)".to_string(),
            "Edit(*)".to_string(),
            "Read(*)".to_string(),
            "Write(*)".to_string(),
            "MultiEdit(*)".to_string(),
            "Glob(*)".to_string(),
            "Grep(*)".to_string(),
        ]);

        // Deny potentially dangerous operations
        deny_rules.extend(vec![
            "Bash(rm -rf *)".to_string(),
            "Bash(curl *)".to_string(),
            "Bash(wget *)".to_string(),
            "WebFetch(*)".to_string(),
            "WebSearch(*)".to_string(),
        ]);
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

    let settings = json!({
        "permissions": {
            "allow": allow_rules,
            "deny": deny_rules
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

        let cm = build_configmap(&tr, "test-cm").unwrap();
        let data = cm.data.unwrap();
        assert!(data.contains_key("task.md"));
        assert!(data.contains_key("design-spec.md"));
        assert_eq!(data.get("task.md").unwrap(), "Task content");
        assert_eq!(data.get("design-spec.md").unwrap(), "Design spec");
    }
}
