use super::config::ControllerConfig;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use super::auth::generate_ssh_volumes;
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
        ensure_pvc_exists(pvcs, &pvc_name, service_name, config).await?;
    }

    // Clean up older versions for retries
    cleanup_old_jobs(&task, jobs).await?;
    cleanup_old_configmaps(&task, configmaps).await?;

    // Create ConfigMap FIRST (without owner reference) so Job can mount it
    let cm_name = generate_configmap_name(&task);
    let configmap = create_configmap(&task, &cm_name, config, None)?;

    match configmaps.create(&PostParams::default(), &configmap).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Create Job SECOND (now it can successfully mount the existing ConfigMap)
    let job_ref = create_job(&task, jobs, &cm_name, config, ctx).await?;

    // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
    if let Some(owner_ref) = job_ref {
        update_configmap_owner(&task, configmaps, &cm_name, owner_ref).await?;
    }

    Ok(Action::await_change())
}

/// Generate a unique `ConfigMap` name for the task
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

/// Create `ConfigMap` with all template files
fn create_configmap(
    task: &TaskType,
    name: &str,
    config: &ControllerConfig,
    owner_ref: Option<OwnerReference>,
) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Generate all templates for this task
    let templates = generate_templates(task, config)?;
    for (filename, content) in templates {
        data.insert(filename, content);
    }

    let labels = create_task_labels(task);
    let mut metadata = ObjectMeta {
        name: Some(name.to_string()),
        labels: Some(labels),
        ..Default::default()
    };

    // Set owner reference if provided (for automatic cleanup)
    if let Some(owner) = owner_ref {
        metadata.owner_references = Some(vec![owner]);
    }

    Ok(ConfigMap {
        metadata,
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
) -> Result<Option<OwnerReference>> {
    let job_name = generate_job_name(task);
    let job = build_job_spec(task, &job_name, cm_name, config)?;

    match jobs.create(&PostParams::default(), &job).await {
        Ok(created_job) => {
            info!("Created job: {}", job_name);
            update_job_started(task, ctx, &job_name, cm_name).await?;

            // Return owner reference for the created job
            if let (Some(uid), Some(name)) = (created_job.metadata.uid, created_job.metadata.name) {
                Ok(Some(OwnerReference {
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    name,
                    uid,
                    controller: Some(true),
                    block_owner_deletion: Some(true),
                }))
            } else {
                Ok(None)
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
            // Try to get existing job for owner reference
            match jobs.get(&job_name).await {
                Ok(existing_job) => {
                    if let (Some(uid), Some(name)) =
                        (existing_job.metadata.uid, existing_job.metadata.name)
                    {
                        Ok(Some(OwnerReference {
                            api_version: "batch/v1".to_string(),
                            kind: "Job".to_string(),
                            name,
                            uid,
                            controller: Some(true),
                            block_owner_deletion: Some(true),
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// Generate a deterministic job name for the task (based on resource name, not timestamp)
fn generate_job_name(task: &TaskType) -> String {
    let resource_name = task.name().replace(['_', '.'], "-");
    match task {
        TaskType::Docs(_) => {
            format!("docs-gen-{resource_name}")
        }
        TaskType::Code(_) => {
            let task_id = task.task_id().unwrap_or(0);
            let context_version = task.context_version();
            format!("code-impl-{resource_name}-task{task_id}-v{context_version}")
        }
    }
}

/// Build the complete Job specification
fn build_job_spec(
    task: &TaskType,
    job_name: &str,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
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
    if TaskType::uses_ssh() {
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

    // Guidelines files will be copied from ConfigMap to working directory by container.sh
    // No need to mount them separately since they need to be in the working directory

    // Environment variables
    let mut env_vars = vec![
        json!({"name": "ANTHROPIC_API_KEY", "valueFrom": {"secretKeyRef": {"name": config.secrets.api_key_secret_name, "key": config.secrets.api_key_secret_key}}}),
        json!({"name": "TASK_TYPE", "value": if task.is_docs() { "docs" } else { "code" }}),
        json!({"name": "MODEL", "value": task.model()}),
        json!({"name": "GITHUB_USER", "value": task.github_user()}),
        json!({"name": "REPOSITORY_URL", "value": task.repository_url()}),
        json!({"name": "WORKING_DIRECTORY", "value": task.working_directory()}),
    ];

    // Add GitHub token from secret for API operations (PR creation, etc.)
    env_vars.push(json!({
        "name": "GH_TOKEN",
        "valueFrom": {
            "secretKeyRef": {
                "name": task.github_token_secret_name(),
                "key": "token"
            }
        }
    }));

    // Add task-specific environment variables
    match task {
        TaskType::Docs(dr) => {
            env_vars.push(json!({"name": "SOURCE_BRANCH", "value": dr.spec.source_branch}));
        }
        TaskType::Code(cr) => {
            env_vars.push(json!({"name": "TASK_ID", "value": cr.spec.task_id.to_string()}));
            env_vars.push(json!({"name": "SERVICE_NAME", "value": cr.spec.service}));
            env_vars
                .push(json!({"name": "DOCS_REPOSITORY_URL", "value": cr.spec.docs_repository_url}));
            env_vars
                .push(json!({"name": "MCP_CLIENT_CONFIG", "value": "/.claude/client-config.json"}));

            if let Some(local_tools) = &cr.spec.local_tools {
                env_vars.push(json!({"name": "LOCAL_TOOLS", "value": local_tools}));
            }
            if let Some(remote_tools) = &cr.spec.remote_tools {
                env_vars.push(json!({"name": "REMOTE_TOOLS", "value": remote_tools}));
            }

            // Add toolman server URL for MCP integration
            // Environment variable: TOOLMAN_SERVER_URL (default: http://toolman.mcp.svc.cluster.local:3000/mcp)
            let toolman_url = std::env::var("TOOLMAN_SERVER_URL")
                .unwrap_or_else(|_| "http://toolman.mcp.svc.cluster.local:3000/mcp".to_string());
            env_vars.push(json!({"name": "TOOLMAN_SERVER_URL", "value": toolman_url}));

            // Add custom environment variables
            for (name, value) in &cr.spec.env {
                env_vars.push(json!({"name": name, "value": value}));
            }

            // Add environment variables from secrets
            for secret_env in &cr.spec.env_from_secrets {
                env_vars.push(json!({
                    "name": secret_env.name,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_env.secret_name,
                            "key": secret_env.secret_key
                        }
                    }
                }));
            }
        }
    }

    // Job deadline from config
    let job_deadline = config.job.active_deadline_seconds;

    // Agent image from config
    let agent_image = format!(
        "{}:{}",
        config.agent.image.repository, config.agent.image.tag
    );

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
    labels.insert(
        "component".to_string(),
        if task.is_docs() {
            "docs-generator"
        } else {
            "code-runner"
        }
        .to_string(),
    );
    labels.insert(
        "github-user".to_string(),
        sanitize_label_value(task.github_user()),
    );
    labels.insert(
        "context-version".to_string(),
        task.context_version().to_string(),
    );

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

/// Sanitize a string value for use as a Kubernetes label value
/// Kubernetes labels must be an empty string or consist of alphanumeric characters, '-', '_' or '.',
/// and must start and end with an alphanumeric character
fn sanitize_label_value(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Replace spaces with hyphens, convert to lowercase
    let mut sanitized = input.to_lowercase().replace([' ', '_'], "-"); // Normalize spaces and underscores to hyphens

    // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
    sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    // Ensure it starts with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().next().unwrap().is_alphanumeric() {
        sanitized.remove(0);
    }

    // Ensure it ends with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().last().unwrap().is_alphanumeric() {
        sanitized.pop();
    }

    // If we ended up with an empty string, provide a fallback
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

/// Ensure PVC exists for the given service
async fn ensure_pvc_exists(
    pvcs: &Api<PersistentVolumeClaim>,
    pvc_name: &str,
    service_name: &str,
    config: &ControllerConfig,
) -> Result<()> {
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

    let mut pvc_spec = json!({
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
                    "storage": config.storage.workspace_size
                }
            }
        }
    });

    // Add storage class if specified
    if let Some(storage_class) = &config.storage.storage_class_name {
        pvc_spec["spec"]["storageClassName"] = json!(storage_class);
    }

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

/// Clean up older configmap versions for retry attempts
async fn cleanup_old_configmaps(task: &TaskType, configmaps: &Api<ConfigMap>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let cm_list = configmaps
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for cm in cm_list.items {
            if let Some(version) = cm
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(cm_name) = &cm.metadata.name {
                        configmaps.delete(cm_name, &DeleteParams::default()).await?;
                        info!("Deleted older configmap version: {}", cm_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Update the owner reference of an existing `ConfigMap`
async fn update_configmap_owner(
    _task: &TaskType,
    configmaps: &Api<ConfigMap>,
    cm_name: &str,
    owner_ref: OwnerReference,
) -> Result<()> {
    let mut configmap = configmaps.get(cm_name).await?;
    configmap.metadata.owner_references = Some(vec![owner_ref]);
    configmaps
        .replace(cm_name, &PostParams::default(), &configmap)
        .await?;
    info!("Updated ConfigMap owner reference for: {}", cm_name);
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
        format!(
            "task-type=docs,github-user={}",
            sanitize_label_value(task.github_user())
        )
    };

    info!("Cleaning up resources for task: {}", task.name());

    // Delete all jobs for this task
    let job_list = jobs
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }

    // Delete all configmaps for this task
    let cm_list = configmaps
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }

    Ok(Action::await_change())
}
