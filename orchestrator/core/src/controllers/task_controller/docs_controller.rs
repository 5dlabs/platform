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
use std::sync::Arc;
use tracing::{error, instrument};

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
    error!("🚀 DOCS DEBUG: Starting reconcile_docs_create_or_update for: {}", docs_run_name);
    
    // Create APIs
    error!("🔗 DOCS DEBUG: Creating Kubernetes API clients for namespace: {}", ctx.namespace);
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    error!("✅ DOCS DEBUG: API clients created successfully");
    
    // Create resource manager and delegate
    error!("🏗️ DOCS DEBUG: Creating DocsResourceManager");
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);
    error!("✅ DOCS DEBUG: DocsResourceManager created successfully");
    
    // First handle resource creation/updates  
    error!("📦 DOCS DEBUG: Calling resource_manager.reconcile_create_or_update");
    let result = resource_manager.reconcile_create_or_update(&docs_run).await;
    
    match &result {
        Ok(action) => {
            error!("✅ DOCS DEBUG: resource_manager.reconcile_create_or_update succeeded with action: {:?}", action);
        }
        Err(e) => {
            error!("❌ DOCS DEBUG: resource_manager.reconcile_create_or_update FAILED with error: {:?}", e);
            error!("❌ DOCS DEBUG: Error type: {}", std::any::type_name_of_val(e));
            error!("❌ DOCS DEBUG: Error details: {}", e);
        }
    }
    
    // Then monitor any existing job status
    error!("📊 DOCS DEBUG: Monitoring job status");
    use super::docs_status::DocsStatusManager;
    if let Err(e) = DocsStatusManager::monitor_job_status(&docs_run, &jobs, &ctx_arc).await {
        error!("⚠️ DOCS DEBUG: Failed to monitor job status (non-fatal): {:?}", e);
        // Don't fail the reconciliation for monitoring errors
    } else {
        error!("✅ DOCS DEBUG: Job status monitoring completed");
    }
    
    error!("🏁 DOCS DEBUG: reconcile_docs_create_or_update completed for: {}", docs_run_name);
    result
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