use super::docs_resources::DocsResourceManager;
use super::types::{Context, Result, DOCS_FINALIZER_NAME};
use crate::crds::DocsRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::ConfigMap,
};
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::runtime::controller::Action;
use kube::{Api, ResourceExt};
use kube::api::{Patch, PatchParams};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, instrument};

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_docs_run(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    error!(
        "🎯 DOCS DEBUG: Starting reconcile for DocsRun: {}",
        docs_run.name_any()
    );

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = docs_run.name_any();

    error!("🔄 DOCS DEBUG: Reconciling DocsRun: {}", name);

    // Create APIs
    error!("🔗 DOCS DEBUG: Creating Kubernetes API clients...");
    let docsruns: Api<DocsRun> = Api::namespaced(client.clone(), namespace);
    error!("✅ DOCS DEBUG: API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(&docsruns, DOCS_FINALIZER_NAME, docs_run.clone(), |event| async {
        match event {
            FinalizerEvent::Apply(dr) => {
                reconcile_docs_create_or_update(dr, &ctx).await
            }
            FinalizerEvent::Cleanup(dr) => {
                cleanup_docs_resources(dr, &ctx).await
            }
        }
    })
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => super::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => super::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => super::types::Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            super::types::Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    error!(
        "🏁 DOCS DEBUG: reconcile completed with result: {:?}",
        result
    );

    Ok(result)
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_docs_create_or_update(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    let docs_run_name = docs_run.name_any();
    error!("🚀 DOCS DEBUG: Starting idempotent reconcile for: {}", docs_run_name);
    
    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Generate deterministic job name
    let job_name = generate_job_name(&docs_run);
    error!("🏷️ DOCS DEBUG: Generated job name: {}", job_name);
    
    // Step 1: Check current job state
    let job_state = check_job_state(&jobs, &job_name).await?;
    error!("📊 DOCS DEBUG: Current job state: {:?}", job_state);
    
    match job_state {
        JobState::NotFound => {
            error!("📝 DOCS DEBUG: No existing job found, creating resources and job");
            
            // Use the existing resource manager pattern to create ConfigMap and Job
            let ctx_arc = Arc::new(ctx.clone()); 
            let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);
            
            // This will create both ConfigMap and Job atomically
            resource_manager.reconcile_create_or_update(&docs_run).await?;
            
            // Update status to Running (only if not already Running)
            update_docs_status_if_changed(&docs_run, ctx, "Running", "Documentation generation started").await?;
            
            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }
        
        JobState::Running => {
            error!("🔄 DOCS DEBUG: Job is still running, monitoring progress");
            
            // Update status to Running if not already
            update_docs_status_if_changed(&docs_run, ctx, "Running", "Documentation generation in progress").await?;
            
            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }
        
        JobState::Completed => {
            error!("🎉 DOCS DEBUG: Job completed successfully - final state reached");
            
            // Update to completed status
            update_docs_status_if_changed(&docs_run, ctx, "Succeeded", "Documentation generation completed successfully").await?;
            
            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
        
        JobState::Failed => {
            error!("💥 DOCS DEBUG: Job failed - final state reached");
            
            // Update to failed status
            update_docs_status_if_changed(&docs_run, ctx, "Failed", "Documentation generation failed").await?;
            
            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_docs_resources(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    error!("🧹 DOCS DEBUG: Cleaning up resources for DocsRun");
    
    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&docs_run).await
}

// Helper functions for idempotent reconciliation

#[derive(Debug, Clone)]
pub enum JobState {
    NotFound,
    Running, 
    Completed,
    Failed,
}

fn generate_job_name(docs_run: &DocsRun) -> String {
    let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
    let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
    let uid_suffix = docs_run.metadata.uid.as_deref()
        .map(|uid| &uid[..8])
        .unwrap_or("nouid");
    
    format!("docs-{namespace}-{name}-{uid_suffix}")
        .replace(['_', '.'], "-")
        .to_lowercase()
}

async fn check_job_state(jobs: &Api<Job>, job_name: &str) -> Result<JobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_job_state(status))
            } else {
                Ok(JobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(JobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> JobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return JobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return JobState::Failed;
            }
        }
    }
    
    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return JobState::Completed;
        }
    }
    
    if let Some(failed) = status.failed {
        if failed > 0 {
            return JobState::Failed;
        }
    }
    
    JobState::Running
}

async fn update_docs_status_if_changed(
    docs_run: &DocsRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
) -> Result<()> {
    // Only update if status actually changed
    let current_phase = docs_run.status.as_ref().map(|s| s.phase.as_str()).unwrap_or("");
    
    if current_phase == new_phase {
        info!("📊 DOCS DEBUG: Status already '{}', skipping update to prevent reconciliation", new_phase);
        return Ok(());
    }
    
    error!("📊 DOCS DEBUG: Updating status from '{}' to '{}'", current_phase, new_phase);
    
    let docsruns: Api<DocsRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,            
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
        }
    });
    
    // Use status subresource to avoid triggering spec reconciliation
    docsruns.patch_status(
        &docs_run.name_any(),
        &PatchParams::default(),
        &Patch::Merge(&status_patch)
    ).await?;
    
    error!("✅ DOCS DEBUG: Status updated successfully to '{}'", new_phase);
    Ok(())
}