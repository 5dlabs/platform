use super::config::ControllerConfig;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use super::auth::{generate_ssh_volumes};
use super::status::update_job_started;
use super::templates::generate_templates;
use super::types::{Result, TaskType};

/// Reconciliation logic for create/update operations
pub async fn reconcile_create_or_update(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
    pvcs: &Api<PersistentVolumeClaim>,
    config: &Arc<ControllerConfig>,
    ctx: &Arc<super::types::Context>,
) -> Result<Action> {
    let name = task.name();
    info!("Creating/updating resources for task: {}", name);

    // Ensure PVC exists for code tasks (docs use emptyDir)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");
        ensure_pvc_exists(pvcs, &pvc_name, service_name).await?;
    }

    // Create ConfigMap with all templates
    let cm_name = generate_configmap_name(&task);
    let configmap = create_configmap(&task, &cm_name, config)?;

    match configmaps.create(&PostParams::default(), &configmap).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Clean up older job versions for retries
    cleanup_old_jobs(&task, jobs).await?;

    // Create the main job
    create_job(&task, jobs, &cm_name, config, ctx).await?;

    Ok(Action::await_change())
}

/// Generate a unique ConfigMap name for the task
fn generate_configmap_name(task: &TaskType) -> String {
    let task_id = task.task_id().unwrap_or(0); // Fallback for docs
    let service_name = task.service_name().replace('_', "-");
    let context_version = task.context_version();

    if task.is_docs() {
        format!("{service_name}-docs-v{context_version}-files")
    } else {
        format!("{service_name}-task{task_id}-v{context_version}-files")
    }
}

/// Create ConfigMap with all template files
fn create_configmap(task: &TaskType, name: &str, config: &ControllerConfig) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Generate all templates for this task
    let templates = generate_templates(task, config)?;
    for (filename, content) in templates {
        data.insert(filename, content);
    }

    let labels = create_task_labels(task);

    Ok(ConfigMap {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    })
}

/// Create the main job for the task
async fn create_job(
    task: &TaskType,
    jobs: &Api<Job>,
    cm_name: &str,
    config: &ControllerConfig,
    ctx: &Arc<super::types::Context>,
) -> Result<()> {
    let job_name = generate_job_name(task);
    let job = build_job_spec(task, &job_name, cm_name, config)?;

    match jobs.create(&PostParams::default(), &job).await {
        Ok(_) => {
            info!("Created job: {}", job_name);
            update_job_started(task, ctx, &job_name, cm_name).await?;
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Generate a deterministic job name for the task (based on resource name, not timestamp)
fn generate_job_name(task: &TaskType) -> String {
    let resource_name = task.name().replace(['_', '.'], "-");
    match task {
        TaskType::Docs(_) => {
            format!("docs-gen-{resource_name}")
        }
        TaskType::Code(_) => {
            let context_version = task.context_version();
            format!("code-impl-{resource_name}-v{context_version}")
        }
    }
}

/// Build the complete Job specification
fn build_job_spec(task: &TaskType, job_name: &str, cm_name: &str, config: &ControllerConfig) -> Result<Job> {
    let labels = create_task_labels(task);

    // Build volumes based on task type
    let mut volumes = vec![];
    let mut volume_mounts = vec![];

    // ConfigMap volume (always needed)
    volumes.push(json!({
        "name": "task-files",
        "configMap": {
            "name": cm_name
        }
    }));
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/config"
    }));

    // Workspace volume (only for code tasks)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");

        volumes.push(json!({
            "name": "workspace",
            "persistentVolumeClaim": {
                "claimName": pvc_name
            }
        }));
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));
    }

    // SSH volumes if needed
    if task.uses_ssh() {
        let ssh_volumes = generate_ssh_volumes(task);
        volumes.extend(ssh_volumes);

            volume_mounts.push(json!({
        "name": "ssh-key",
        "mountPath": "/workspace/.ssh",
        "readOnly": true
    }));
}

// Mount settings.json directly to /etc/claude-code/managed-settings.json
volume_mounts.push(json!({
    "name": "task-files",
    "mountPath": "/etc/claude-code/managed-settings.json",
    "subPath": "settings.json",
    "readOnly": true
}));

// Mount guidelines files directly to workspace for code tasks
if !task.is_docs() {
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/workspace/coding-guidelines.md",
        "subPath": "coding-guidelines.md",
        "readOnly": true
    }));
    
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/workspace/github-guidelines.md",
        "subPath": "github-guidelines.md",
        "readOnly": true
    }));
}

// Environment variables
    let mut env_vars = vec![
        json!({"name": "ANTHROPIC_API_KEY", "valueFrom": {"secretKeyRef": {"name": config.secrets.api_key_secret_name, "key": config.secrets.api_key_secret_key}}}),
        json!({"name": "TASK_TYPE", "value": if task.is_docs() { "docs" } else { "code" }}),
        json!({"name": "MODEL", "value": task.model()}),
        json!({"name": "GITHUB_USER", "value": task.github_user()}),
        json!({"name": "REPOSITORY_URL", "value": task.repository_url()}),
        json!({"name": "WORKING_DIRECTORY", "value": task.working_directory()}),
        json!({"name": "BRANCH", "value": task.branch()}),
    ];

    // Add task-specific environment variables
    match task {
        TaskType::Docs(dr) => {
            env_vars.push(json!({"name": "SOURCE_BRANCH", "value": dr.spec.source_branch}));
        }
        TaskType::Code(cr) => {
            env_vars.push(json!({"name": "TASK_ID", "value": cr.spec.task_id.to_string()}));
            env_vars.push(json!({"name": "SERVICE_NAME", "value": cr.spec.service}));
            env_vars.push(json!({"name": "PLATFORM_REPOSITORY_URL", "value": cr.spec.platform_repository_url}));
            env_vars.push(json!({"name": "MCP_CLIENT_CONFIG", "value": "/.claude/client-config.json"}));

            if let Some(local_tools) = &cr.spec.local_tools {
                env_vars.push(json!({"name": "LOCAL_TOOLS", "value": local_tools}));
            }
            if let Some(remote_tools) = &cr.spec.remote_tools {
                env_vars.push(json!({"name": "REMOTE_TOOLS", "value": remote_tools}));
            }
            env_vars.push(json!({"name": "TOOL_CONFIG", "value": cr.spec.tool_config}));
            
            // Add toolman server URL for MCP integration
            env_vars.push(json!({"name": "TOOLMAN_SERVER_URL", "value": "http://toolman.mcp.svc.cluster.local:3000/mcp"}));
        }
    }


    // Job deadline from config
    let job_deadline = config.job.active_deadline_seconds;

    // Agent image from config
    let agent_image = format!("{}:{}", config.agent.image.repository, config.agent.image.tag);

    let job_spec = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "labels": labels
        },
        "spec": {
            "activeDeadlineSeconds": job_deadline,
            "backoffLimit": 0,
            "template": {
                "metadata": {
                    "labels": labels
                },
                "spec": {
                    "restartPolicy": "Never",
                    "securityContext": {
                        "fsGroup": 1000,
                        "runAsUser": 1000,
                        "runAsGroup": 1000
                    },
                    "imagePullSecrets": config.agent.image_pull_secrets.iter().map(|name| {
                        json!({"name": name})
                    }).collect::<Vec<_>>(),
                    "containers": [{
                        "name": "claude",
                        "image": agent_image,
                        "command": ["/bin/bash", "/config/container.sh"],
                        "env": env_vars,
                        "volumeMounts": volume_mounts,
                        "resources": {
                            "requests": {
                                "cpu": "100m",
                                "memory": "256Mi"
                            },
                            "limits": {
                                "cpu": "2",
                                "memory": "4Gi"
                            }
                        }
                    }],
                    "volumes": volumes
                }
            }
        }
    });

    Ok(serde_json::from_value(job_spec)?)
}

/// Create standard labels for task resources
fn create_task_labels(task: &TaskType) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();

    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert("component".to_string(), if task.is_docs() { "docs-generator" } else { "code-runner" }.to_string());
    labels.insert("github-user".to_string(), task.github_user().to_string());

    match task {
        TaskType::Docs(_) => {
            labels.insert("task-type".to_string(), "docs".to_string());
        }
        TaskType::Code(_) => {
            labels.insert("task-type".to_string(), "code".to_string());
            if let Some(task_id) = task.task_id() {
                labels.insert("task-id".to_string(), task_id.to_string());
            }
            labels.insert("service-name".to_string(), task.service_name().to_string());
        }
    }

    labels
}

/// Ensure PVC exists for the given service
async fn ensure_pvc_exists(pvcs: &Api<PersistentVolumeClaim>, pvc_name: &str, service_name: &str) -> Result<()> {
    match pvcs.get(pvc_name).await {
        Ok(_) => {
            info!("PVC already exists: {}", pvc_name);
            return Ok(());
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // PVC doesn't exist, create it
        }
        Err(e) => return Err(e.into()),
    }

    let pvc_spec = json!({
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": pvc_name,
            "labels": {
                "app": "orchestrator",
                "service": service_name
            }
        },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": "10Gi"
                }
            }
        }
    });

    let pvc: PersistentVolumeClaim = serde_json::from_value(pvc_spec)?;
    pvcs.create(&PostParams::default(), &pvc).await?;
    info!("Created PVC: {}", pvc_name);

    Ok(())
}

/// Clean up older job versions for retry attempts
async fn cleanup_old_jobs(task: &TaskType, jobs: &Api<Job>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let job_list = jobs
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for job in job_list.items {
            if let Some(version) = job
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(job_name) = &job.metadata.name {
                        jobs.delete(job_name, &DeleteParams::background()).await?;
                        info!("Deleted older job version: {}", job_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Cleanup resources when task is deleted
pub async fn cleanup_resources(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
) -> Result<Action> {
    let task_label = if let Some(task_id) = task.task_id() {
        format!("task-id={task_id}")
    } else {
        format!("task-type=docs,github-user={}", task.github_user())
    };

    info!("Cleaning up resources for task: {}", task.name());

    // Delete all jobs for this task
    let job_list = jobs.list(&ListParams::default().labels(&task_label)).await?;
    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }

    // Delete all configmaps for this task
    let cm_list = configmaps.list(&ListParams::default().labels(&task_label)).await?;
    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }

    Ok(Action::await_change())
}