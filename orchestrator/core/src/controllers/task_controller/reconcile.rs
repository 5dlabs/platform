use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use futures::StreamExt;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::{
    api::Api,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
        watcher::Config,
    },
    Client,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info, warn, debug};

use super::resources::{cleanup_resources, reconcile_create_or_update};
use super::status::monitor_job_status;
use super::types::{Context, Error, Result, TaskType, CODE_FINALIZER_NAME, DOCS_FINALIZER_NAME};

/// Run the task controller for both `DocsRun` and `CodeRun` resources
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    error!("🚀 AGGRESSIVE DEBUG: Starting task controller in namespace: {}", namespace);

    error!("🔧 AGGRESSIVE DEBUG: About to load controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            error!("✅ AGGRESSIVE DEBUG: Successfully loaded controller configuration from mounted file");
            error!("🔧 AGGRESSIVE DEBUG: Configuration cleanup enabled = {}", cfg.cleanup.enabled);

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!("❌ AGGRESSIVE DEBUG: Configuration validation failed: {}", validation_error);
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ AGGRESSIVE DEBUG: Configuration validation passed");
            cfg
        }
        Err(e) => {
            error!(
                "❌ AGGRESSIVE DEBUG: Failed to load configuration from mounted file, using defaults: {}",
                e
            );
            error!("🔧 AGGRESSIVE DEBUG: About to create default configuration...");
            let default_config = ControllerConfig::default();

            // Validate default configuration - this should fail if image config is missing
            if let Err(validation_error) = default_config.validate() {
                error!("❌ AGGRESSIVE DEBUG: Default configuration is invalid: {}", validation_error);
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ AGGRESSIVE DEBUG: Default configuration validation passed");
            default_config
        }
    };

    error!("🏗️ AGGRESSIVE DEBUG: Creating controller context...");
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    error!("✅ AGGRESSIVE DEBUG: Controller context created successfully");

    // Start controllers for both DocsRun and CodeRun
    error!("🔗 AGGRESSIVE DEBUG: Creating API clients for DocsRun and CodeRun...");
    let docs_runs = Api::<DocsRun>::namespaced(client.clone(), &namespace);
    let code_runs = Api::<CodeRun>::namespaced(client.clone(), &namespace);

    error!("✅ AGGRESSIVE DEBUG: API clients created, starting controllers...");

    let docs_controller = Controller::new(docs_runs, Config::default())
        .shutdown_on_signal()
        .run(reconcile_docs, error_policy_docs, context.clone())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()));

    let code_controller = Controller::new(code_runs, Config::default())
        .shutdown_on_signal()
        .run(reconcile_code, error_policy_code, context.clone())
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()));

    error!("🚀 AGGRESSIVE DEBUG: Both controllers started, entering main loop...");

    // Run both controllers concurrently
    tokio::select! {
        () = docs_controller => error!("DocsRun controller finished"),
        () = code_controller => error!("CodeRun controller finished"),
    }

    Ok(())
}

/// Reconciliation logic for `DocsRun` resources
async fn reconcile_docs(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    error!("📝 AGGRESSIVE DEBUG: Starting reconcile_docs for: {}", docs_run.metadata.name.as_ref().unwrap_or(&"unnamed".to_string()));

    let task = TaskType::Docs(docs_run.clone());
    error!("🔍 AGGRESSIVE DEBUG: Created task type, calling reconcile_common...");

    let result = reconcile_common(task, ctx, DOCS_FINALIZER_NAME).await;
    error!("🏁 AGGRESSIVE DEBUG: reconcile_common completed with result: {:?}", result.is_ok());

    result
}

/// Reconcile function for `CodeRun` resources
async fn reconcile_code(cr: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    let task = TaskType::Code(cr.clone());
    reconcile_common(task, ctx, CODE_FINALIZER_NAME).await
}

/// Common reconciliation logic for both `DocsRun` and `CodeRun`
async fn reconcile_common(
    task: TaskType,
    ctx: Arc<Context>,
    finalizer_name: &str,
) -> Result<Action> {
    error!("🎯 AGGRESSIVE DEBUG: Starting reconcile_common for: {}", task.name());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    error!(
        "🔄 AGGRESSIVE DEBUG: Reconciling {}: {}",
        if task.is_docs() { "DocsRun" } else { "CodeRun" },
        name
    );

    // Create APIs
    error!("🔗 AGGRESSIVE DEBUG: Creating Kubernetes API clients...");
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    error!("✅ AGGRESSIVE DEBUG: API clients created successfully");

    // Handle finalizers for cleanup based on task type
    let _result = match &task {
        TaskType::Docs(dr) => {
            let docsruns: Api<DocsRun> = Api::namespaced(client.clone(), namespace);
            finalizer(&docsruns, finalizer_name, dr.clone(), |event| async {
                match event {
                    FinalizerEvent::Apply(dr) => {
                        let task = TaskType::Docs(dr);
                        reconcile_create_or_update(
                            task,
                            &jobs,
                            &configmaps,
                            &pvcs,
                            &ctx.config,
                            &ctx,
                        )
                        .await
                    }
                    FinalizerEvent::Cleanup(dr) => {
                        let task = TaskType::Docs(dr);
                        cleanup_resources(task, &jobs, &configmaps).await
                    }
                }
            })
            .await
        }
        TaskType::Code(cr) => {
            let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
            finalizer(&coderuns, finalizer_name, cr.clone(), |event| async {
                match event {
                    FinalizerEvent::Apply(cr) => {
                        let task = TaskType::Code(cr);
                        reconcile_create_or_update(
                            task,
                            &jobs,
                            &configmaps,
                            &pvcs,
                            &ctx.config,
                            &ctx,
                        )
                        .await
                    }
                    FinalizerEvent::Cleanup(cr) => {
                        let task = TaskType::Code(cr);
                        cleanup_resources(task, &jobs, &configmaps).await
                    }
                }
            })
            .await
        }
    };

    // Handle finalizer errors
    let _result = _result.map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    // Monitor running jobs
    monitor_running_job(&task, &jobs, &ctx).await?;

    // Requeue after 30 seconds to check status
    Ok(Action::requeue(Duration::from_secs(30)))
}

/// Monitor running job status for both task types
async fn monitor_running_job(task: &TaskType, jobs: &Api<Job>, ctx: &Arc<Context>) -> Result<()> {
    let is_running = match task {
        TaskType::Docs(dr) => dr.status.as_ref().is_some_and(|s| s.phase == "Running"),
        TaskType::Code(cr) => cr.status.as_ref().is_some_and(|s| s.phase == "Running"),
    };

    if is_running {
        monitor_job_status(task, jobs, ctx).await?;
    }

    Ok(())
}



/// Error policy for `DocsRun` controller
fn error_policy_docs(_dr: Arc<DocsRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("DocsRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}

/// Error policy for `CodeRun` controller
fn error_policy_code(_cr: Arc<CodeRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("CodeRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}
