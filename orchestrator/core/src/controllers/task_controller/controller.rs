use super::code_controller::reconcile_code_run;
use super::config::ControllerConfig;
use super::docs_controller::reconcile_docs_run;
use super::types::{Context, Error, Result};
use crate::crds::{CodeRun, DocsRun};
use futures::StreamExt;
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher::Config;
use kube::{Api, Client, ResourceExt};
use std::sync::Arc;
use tracing::{error, info, instrument, Instrument};

/// Main entry point for the separated task controllers
#[instrument(skip(client), fields(namespace = %namespace))]
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    error!(
        "🚀 TASK_CONTROLLER DEBUG: Starting separated task controllers in namespace: {}",
        namespace
    );

    error!("🔧 TASK_CONTROLLER DEBUG: Loading controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            error!("✅ TASK_CONTROLLER DEBUG: Successfully loaded controller configuration");
            error!(
                "🔧 TASK_CONTROLLER DEBUG: Configuration cleanup enabled = {}",
                cfg.cleanup.enabled
            );

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!(
                    "❌ TASK_CONTROLLER DEBUG: Configuration validation failed: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ TASK_CONTROLLER DEBUG: Configuration validation passed");
            cfg
        }
        Err(e) => {
            error!(
                "❌ TASK_CONTROLLER DEBUG: Failed to load configuration, using defaults: {}",
                e
            );
            error!("🔧 TASK_CONTROLLER DEBUG: Creating default configuration...");
            let default_config = ControllerConfig::default();

            // Validate default configuration
            if let Err(validation_error) = default_config.validate() {
                error!(
                    "❌ TASK_CONTROLLER DEBUG: Default configuration is invalid: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            error!("✅ TASK_CONTROLLER DEBUG: Default configuration validation passed");
            default_config
        }
    };

    error!("🏗️ TASK_CONTROLLER DEBUG: Creating controller context...");

    // Create shared context
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    error!("✅ TASK_CONTROLLER DEBUG: Controller context created successfully");

    // Run both controllers concurrently
    error!("🚀 TASK_CONTROLLER DEBUG: Starting DocsRun and CodeRun controllers...");
    
    let docs_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move {
            run_docs_controller(client, namespace, context).await
        }
    });

    let code_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move {
            run_code_controller(client, namespace, context).await
        }
    });

    error!("🔄 TASK_CONTROLLER DEBUG: Both controllers started, waiting for completion...");

    // Wait for both controllers to complete (they should run indefinitely)
    match tokio::try_join!(docs_controller_handle, code_controller_handle) {
        Ok((docs_result, code_result)) => {
            if let Err(e) = docs_result {
                error!("DocsRun controller failed: {:?}", e);
            }
            if let Err(e) = code_result {
                error!("CodeRun controller failed: {:?}", e);
            }
        }
        Err(e) => {
            error!("Controller task join error: {:?}", e);
        }
    }

    error!("🏁 TASK_CONTROLLER DEBUG: Task controller shutting down");
    Ok(())
}

/// Run the DocsRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_docs_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    error!("🚀 DOCS_CONTROLLER DEBUG: Starting DocsRun controller");

    let docs_api: Api<DocsRun> = Api::namespaced(client, &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(docs_api, watcher_config)
        .run(reconcile_docs_run, error_policy_docs, context)
        .for_each(|reconciliation_result| {
            let docs_span = tracing::info_span!("docs_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(docs_run_resource) => {
                        info!(
                            resource = ?docs_run_resource,
                            "✅ DOCS_CONTROLLER: Reconciliation successful for DocsRun"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "❌ DOCS_CONTROLLER: Reconciliation error"
                        );
                    }
                }
            }.instrument(docs_span)
        })
        .await;

    error!("🏁 DOCS_CONTROLLER DEBUG: DocsRun controller shutting down");
    Ok(())
}

/// Run the CodeRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_code_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    error!("🚀 CODE_CONTROLLER DEBUG: Starting CodeRun controller");

    let code_api: Api<CodeRun> = Api::namespaced(client, &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(code_api, watcher_config)
        .run(reconcile_code_run, error_policy_code, context)
        .for_each(|reconciliation_result| {
            let code_span = tracing::info_span!("code_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(code_run_resource) => {
                        info!(
                            resource = ?code_run_resource,
                            "✅ CODE_CONTROLLER: Reconciliation successful for CodeRun"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "❌ CODE_CONTROLLER: Reconciliation error"
                        );
                    }
                }
            }.instrument(code_span)
        })
        .await;

    error!("🏁 CODE_CONTROLLER DEBUG: CodeRun controller shutting down");
    Ok(())
}

/// Error policy for DocsRun controller - limit to single retry
#[instrument(skip(_ctx), fields(docs_run_name = %_docs_run.name_any(), namespace = %_ctx.namespace))]
fn error_policy_docs(_docs_run: Arc<DocsRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?error,
        docs_run_name = %_docs_run.name_any(),
        "DocsRun reconciliation failed - no retries, stopping"
    );
    // Don't retry - just stop on first failure
    Action::await_change()
}

/// Error policy for CodeRun controller - limit to single retry
#[instrument(skip(_ctx), fields(code_run_name = %_code_run.name_any(), namespace = %_ctx.namespace))]
fn error_policy_code(_code_run: Arc<CodeRun>, error: &Error, _ctx: Arc<Context>) -> Action {
    error!(
        error = ?error,
        code_run_name = %_code_run.name_any(),
        "CodeRun reconciliation failed - no retries, stopping"
    );
    // Don't retry - just stop on first failure
    Action::await_change()
}