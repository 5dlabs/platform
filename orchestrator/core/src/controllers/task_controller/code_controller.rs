use super::code_resources::CodeResourceManager;
use super::types::{Context, Result, CODE_FINALIZER_NAME};
use crate::crds::CodeRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::runtime::controller::Action;
use kube::{Api, ResourceExt};
use kube::api::{Patch, PatchParams};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_code_run(code_run: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    info!(
        "🎯 CODE DEBUG: Starting reconcile for CodeRun: {}",
        code_run.name_any()
    );

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = code_run.name_any();

    info!("🔄 CODE DEBUG: Reconciling CodeRun: {}", name);

    // Create APIs
    info!("🔗 CODE DEBUG: Creating Kubernetes API clients...");
    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
    info!("✅ CODE DEBUG: API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(&coderuns, CODE_FINALIZER_NAME, code_run.clone(), |event| async {
        match event {
            FinalizerEvent::Apply(cr) => {
                reconcile_code_create_or_update(cr, &ctx).await
            }
            FinalizerEvent::Cleanup(cr) => {
                cleanup_code_resources(cr, &ctx).await
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

    info!(
        "🏁 CODE DEBUG: reconcile completed with result: {:?}",
        result
    );

    Ok(result)
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_code_create_or_update(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    let code_run_name = code_run.name_any();
    info!("🚀 CODE DEBUG: Starting idempotent reconcile for: {}", code_run_name);
    
    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Generate deterministic job name
    let job_name = generate_code_job_name(&code_run);
    info!("🏷️ CODE DEBUG: Generated job name: {}", job_name);
    
    // Step 1: Check current job state
    let job_state = check_code_job_state(&jobs, &job_name).await?;
    info!("📊 CODE DEBUG: Current job state: {:?}", job_state);
    
    match job_state {
        CodeJobState::NotFound => {
            // CRITICAL: Check if this is a retry scenario where job was already completed
            // Look at CodeRun status to see if we previously completed successfully
            if let Some(status) = &code_run.status {
                if status.phase == "Succeeded" {
                    info!("✅ CODE DEBUG: CodeRun already succeeded, maintaining final state");
                    return Ok(Action::await_change());
                }
                if status.phase == "Failed" {
                    info!("❌ CODE DEBUG: CodeRun already failed, maintaining final state");
                    return Ok(Action::await_change());
                }
            }
            
            info!("📝 CODE DEBUG: No existing job found, creating resources and job");
            
            // Use the existing resource manager pattern to create ConfigMap, PVC, and Job
            let ctx_arc = Arc::new(ctx.clone()); 
            let resource_manager = CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
            
            // This will create all resources atomically
            resource_manager.reconcile_create_or_update(&code_run).await?;
            
            // Update status to Running (only if not already Running)
            update_code_status_if_changed(&code_run, ctx, "Running", "Code task started").await?;
            
            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }
        
        CodeJobState::Running => {
            info!("🔄 CODE DEBUG: Job is still running, monitoring progress");
            
            // Update status to Running if not already
            update_code_status_if_changed(&code_run, ctx, "Running", "Code task in progress").await?;
            
            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }
        
        CodeJobState::Completed => {
            info!("🎉 CODE DEBUG: Job completed successfully - final state reached");
            
            // Update to completed status
            update_code_status_if_changed(&code_run, ctx, "Succeeded", "Code task completed successfully").await?;
            
            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
        
        CodeJobState::Failed => {
            info!("💥 CODE DEBUG: Job failed - final state reached");
            
            // Update to failed status
            update_code_status_if_changed(&code_run, ctx, "Failed", "Code task failed").await?;
            
            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_code_resources(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    info!("🧹 CODE DEBUG: Cleaning up resources for CodeRun");
    
    // Create APIs  
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&code_run).await
}

// Helper functions for idempotent reconciliation - CodeRun version

#[derive(Debug, Clone)]
pub enum CodeJobState {
    NotFound,
    Running, 
    Completed,
    Failed,
}

fn generate_code_job_name(code_run: &CodeRun) -> String {
    let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
    let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
    let uid_suffix = code_run.metadata.uid.as_deref()
        .map(|uid| &uid[..8])
        .unwrap_or("nouid");
    
    format!("code-{namespace}-{name}-{uid_suffix}")
        .replace(['_', '.'], "-")
        .to_lowercase()
}

async fn check_code_job_state(jobs: &Api<Job>, job_name: &str) -> Result<CodeJobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_code_job_state(status))
            } else {
                Ok(CodeJobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(CodeJobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_code_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> CodeJobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return CodeJobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return CodeJobState::Failed;
            }
        }
    }
    
    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return CodeJobState::Completed;
        }
    }
    
    if let Some(failed) = status.failed {
        if failed > 0 {
            return CodeJobState::Failed;
        }
    }
    
    CodeJobState::Running
}

async fn update_code_status_if_changed(
    code_run: &CodeRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
) -> Result<()> {
    // Only update if status actually changed
    let current_phase = code_run.status.as_ref().map(|s| s.phase.as_str()).unwrap_or("");
    
    if current_phase == new_phase {
        info!("📊 CODE DEBUG: Status already '{}', skipping update to prevent reconciliation", new_phase);
        return Ok(());
    }
    
    info!("📊 CODE DEBUG: Updating status from '{}' to '{}'", current_phase, new_phase);
    
    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,            
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
        }
    });
    
    // Use status subresource to avoid triggering spec reconciliation
    coderuns.patch_status(
        &code_run.name_any(),
        &PatchParams::default(),
        &Patch::Merge(&status_patch)
    ).await?;
    
    info!("✅ CODE DEBUG: Status updated successfully to '{}'", new_phase);
    Ok(())
}