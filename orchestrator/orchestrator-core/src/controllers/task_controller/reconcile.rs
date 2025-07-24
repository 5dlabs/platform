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
use tracing::{error, info, warn};

use super::resources::{cleanup_resources, reconcile_create_or_update};
use super::status::monitor_job_status;
use super::types::{Context, Error, Result, TaskType, CODE_FINALIZER_NAME, DOCS_FINALIZER_NAME};

/// Run the task controller for both DocsRun and CodeRun resources
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    info!("Starting task controller in namespace: {}", namespace);

    // Load controller configuration from ConfigMap
    let config =
        match ControllerConfig::from_configmap(&client, &namespace, "task-controller-config").await
        {
            Ok(cfg) => {
                info!("Loaded controller configuration from ConfigMap");
                // Validate configuration has required fields
                if let Err(validation_error) = cfg.validate() {
                    return Err(Error::ConfigError(validation_error.to_string()));
                }
                cfg
            }
            Err(e) => {
                warn!(
                    "Failed to load configuration from ConfigMap, using defaults: {}",
                    e
                );
                let default_config = ControllerConfig::default();
                // Validate default configuration - this should fail if image config is missing
                if let Err(validation_error) = default_config.validate() {
                    error!("Default configuration is invalid: {}", validation_error);
                    return Err(Error::ConfigError(validation_error.to_string()));
                }
                default_config
            }
        };

    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    // Start controllers for both DocsRun and CodeRun
    let docs_runs = Api::<DocsRun>::namespaced(client.clone(), &namespace);
    let code_runs = Api::<CodeRun>::namespaced(client.clone(), &namespace);

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

    // Run both controllers concurrently
    tokio::select! {
        _ = docs_controller => info!("DocsRun controller finished"),
        _ = code_controller => info!("CodeRun controller finished"),
    }

    Ok(())
}

/// Reconcile function for DocsRun resources
async fn reconcile_docs(dr: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    let task = TaskType::Docs(dr.clone());
    reconcile_common(task, ctx, DOCS_FINALIZER_NAME).await
}

/// Reconcile function for CodeRun resources
async fn reconcile_code(cr: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    let task = TaskType::Code(cr.clone());
    reconcile_common(task, ctx, CODE_FINALIZER_NAME).await
}

/// Common reconciliation logic for both DocsRun and CodeRun
async fn reconcile_common(
    task: TaskType,
    ctx: Arc<Context>,
    finalizer_name: &str,
) -> Result<Action> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    info!(
        "Reconciling {}: {}",
        if task.is_docs() { "DocsRun" } else { "CodeRun" },
        name
    );

    // Create APIs
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);

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

/// Error policy for DocsRun controller
fn error_policy_docs(_dr: Arc<DocsRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("DocsRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}

/// Error policy for CodeRun controller
fn error_policy_code(_cr: Arc<CodeRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!("CodeRun reconciliation error: {:?}", error);
    Action::requeue(Duration::from_secs(30))
}
