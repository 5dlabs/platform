use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::types::{Context, Result, TaskType};
use crate::crds::{CodeRun, CodeRunCondition, DocsRun, DocsRunCondition};

/// Monitor Job status and update CRD accordingly
pub async fn monitor_job_status(
    task: &TaskType,
    jobs: &Api<Job>,
    ctx: &Arc<Context>,
) -> Result<()> {
    let job_name = get_current_job_name(task);

    if let Some(job_name) = job_name {
        // Get the current job
        match jobs.get(&job_name).await {
            Ok(job) => {
                let (phase, message, pull_request_url) = analyze_job_status(&job);
                update_task_status(task, ctx, &phase, &message, pull_request_url).await?;

                // Schedule cleanup if job is complete and cleanup is enabled
                if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                    schedule_job_cleanup(task, ctx, &job_name, &phase).await?;
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // Job doesn't exist yet, which is fine for newly created tasks
                info!("Job {} not found yet for task {}", job_name, task.name());
            }
            Err(e) => {
                warn!(
                    "Failed to get job {} for task {}: {}",
                    job_name,
                    task.name(),
                    e
                );
            }
        }
    }

    Ok(())
}

/// Get the current job name for a task
fn get_current_job_name(task: &TaskType) -> Option<String> {
    match task {
        TaskType::Docs(dr) => dr.status.as_ref().and_then(|s| s.job_name.clone()),
        TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.job_name.clone()),
    }
}

/// Analyze job status and return (phase, message, `pull_request_url`)
fn analyze_job_status(job: &Job) -> (String, String, Option<String>) {
    if let Some(status) = &job.status {
        // Check completion time first
        if status.completion_time.is_some() {
            if let Some(conditions) = &status.conditions {
                for condition in conditions {
                    if condition.type_ == "Complete" && condition.status == "True" {
                        return (
                            "Succeeded".to_string(),
                            "Job completed successfully".to_string(),
                            None,
                        );
                    } else if condition.type_ == "Failed" && condition.status == "True" {
                        let message = condition.message.as_deref().unwrap_or("Job failed");
                        return ("Failed".to_string(), message.to_string(), None);
                    }
                }
            }
        }

        // Check if job is running
        if let Some(active) = status.active {
            if active > 0 {
                return ("Running".to_string(), "Job is running".to_string(), None);
            }
        }

        // Check for failure conditions
        if let Some(failed) = status.failed {
            if failed > 0 {
                return ("Failed".to_string(), "Job failed".to_string(), None);
            }
        }
    }

    // Default to pending if we can't determine status
    (
        "Pending".to_string(),
        "Job status unknown".to_string(),
        None,
    )
}

/// Update the task CRD status
async fn update_task_status(
    task: &TaskType,
    ctx: &Arc<Context>,
    phase: &str,
    message: &str,
    pull_request_url: Option<String>,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_dr) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "conditions": build_docs_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match docs_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated DocsRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update DocsRun status for {}: {}", name, e);
                }
            }
        }
        TaskType::Code(cr) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "retryCount": cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
                    "conditions": build_code_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match code_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated CodeRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update CodeRun status for {}: {}", name, e);
                }
            }
        }
    }

    Ok(())
}

/// Build conditions for `DocsRun` status
fn build_docs_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<DocsRunCondition> {
    vec![DocsRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Build conditions for `CodeRun` status
fn build_code_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<CodeRunCondition> {
    vec![CodeRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Update task status when job starts (called from reconcile logic)
pub async fn update_job_started(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    configmap_name: &str,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();
    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_docs_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            docs_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
        TaskType::Code(_) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_code_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            code_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
    }

    info!("Updated {} status to Running with job: {}", name, job_name);
    Ok(())
}

/// Schedule cleanup of completed job after configured delay
async fn schedule_job_cleanup(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    phase: &str,
) -> Result<()> {
    let delay_minutes = if phase == "Succeeded" {
        ctx.config.cleanup.completed_job_delay_minutes
    } else {
        ctx.config.cleanup.failed_job_delay_minutes
    };

    let job_name = job_name.to_string();
    let task_name = task.name();
    let namespace = ctx.namespace.clone();
    let client = ctx.client.clone();
    let delete_configmap = ctx.config.cleanup.delete_configmap;

    info!(
        "Scheduling cleanup for job {} in {} minutes (phase: {})",
        job_name, delay_minutes, phase
    );

    // Spawn background task to handle cleanup after delay
    tokio::spawn(async move {
        // Wait for the configured delay
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_minutes * 60)).await;

        info!("Starting scheduled cleanup for job: {}", job_name);

        // Delete the job
        let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
        match jobs_api
            .delete(&job_name, &kube::api::DeleteParams::background())
            .await
        {
            Ok(_) => info!("Successfully deleted job: {}", job_name),
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Job {} already deleted", job_name);
            }
            Err(e) => {
                error!("Failed to delete job {}: {}", job_name, e);
            }
        }

        // Delete associated ConfigMap if enabled
        if delete_configmap {
            let configmaps_api: Api<k8s_openapi::api::core::v1::ConfigMap> =
                Api::namespaced(client.clone(), &namespace);

            // Find ConfigMap associated with this job
            let labels_selector = "app=orchestrator".to_string();
            let list_params = kube::api::ListParams::default().labels(&labels_selector);

            match configmaps_api.list(&list_params).await {
                Ok(cms) => {
                    for cm in cms.items {
                        if let Some(cm_name) = &cm.metadata.name {
                            // Check if ConfigMap is associated with this job
                            if cm_name.starts_with(&task_name.replace('_', "-")) {
                                match configmaps_api
                                    .delete(cm_name, &kube::api::DeleteParams::default())
                                    .await
                                {
                                    Ok(_) => info!("Successfully deleted ConfigMap: {}", cm_name),
                                    Err(kube::Error::Api(ae)) if ae.code == 404 => {
                                        info!("ConfigMap {} already deleted", cm_name);
                                    }
                                    Err(e) => {
                                        error!("Failed to delete ConfigMap {}: {}", cm_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to list ConfigMaps for cleanup: {}", e);
                }
            }
        }

        info!("Completed cleanup for job: {}", job_name);
    });

    Ok(())
}
