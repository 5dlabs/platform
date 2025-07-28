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
use std::sync::Arc;
use tracing::error;

pub async fn reconcile_code_run(code_run: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    error!(
        "🎯 CODE DEBUG: Starting reconcile for CodeRun: {}",
        code_run.name_any()
    );

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = code_run.name_any();

    error!("🔄 CODE DEBUG: Reconciling CodeRun: {}", name);

    // Create APIs
    error!("🔗 CODE DEBUG: Creating Kubernetes API clients...");
    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
    error!("✅ CODE DEBUG: API clients created successfully");

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

    error!(
        "🏁 CODE DEBUG: reconcile completed with result: {:?}",
        result
    );

    Ok(result)
}

async fn reconcile_code_create_or_update(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    error!("🚀 CODE DEBUG: Creating or updating resources for CodeRun");
    
    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Create resource manager and delegate  
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
    
    // First handle resource creation/updates
    let result = resource_manager.reconcile_create_or_update(&code_run).await;
    
    // Then monitor any existing job status
    use super::code_status::CodeStatusManager;
    if let Err(e) = CodeStatusManager::monitor_job_status(&code_run, &jobs, &ctx_arc).await {
        error!("Failed to monitor job status: {}", e);
        // Don't fail the reconciliation for monitoring errors
    }
    
    result
}

async fn cleanup_code_resources(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    error!("🧹 CODE DEBUG: Cleaning up resources for CodeRun");
    
    // Create APIs  
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    
    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&code_run).await
}