//! Main entry point for the Orchestrator service

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use kube::Client;
use orchestrator_core::{
    handlers::pm_taskrun::{
        add_context, get_task, get_task_status, list_tasks, submit_task, update_session,
        AppState as TaskRunAppState,
    },
    run_taskrun_controller,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state shared across handlers  
#[derive(Clone)]
pub struct AppState {
    taskrun_state: Arc<TaskRunAppState>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Initialize Kubernetes client
        let k8s_client = Client::try_default()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create K8s client: {}", e))?;

        // Get namespace from environment or use default
        let namespace =
            std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "orchestrator".to_string());

        // Create TaskRun app state
        let taskrun_state = TaskRunAppState {
            k8s_client: k8s_client.clone(),
            namespace: namespace.clone(),
        };

        info!("Initialized orchestrator for namespace: {}", namespace);

        Ok(Self {
            taskrun_state: Arc::new(taskrun_state),
        })
    }
}

/// Health check endpoint
async fn health_check(State(_state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Error handling middleware
async fn error_handler(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(request).await;

    // Log errors if status code indicates a problem
    if response.status().is_server_error() {
        warn!("Server error: {}", response.status());
    }

    response
}

/// Create API routes
fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route(
            "/pm/tasks",
            post({
                let taskrun_state = state.taskrun_state.clone();
                move |req| submit_task(State(taskrun_state), req)
            })
            .get({
                let taskrun_state = state.taskrun_state.clone();
                move || list_tasks(State(taskrun_state))
            }),
        )
        .route(
            "/pm/tasks/:task_id",
            get({
                let taskrun_state = state.taskrun_state.clone();
                move |path| get_task(State(taskrun_state), path)
            }),
        )
        .route(
            "/pm/tasks/:task_id/status",
            get({
                let taskrun_state = state.taskrun_state.clone();
                move |path| get_task_status(State(taskrun_state), path)
            }),
        )
        .route(
            "/pm/tasks/:task_id/context",
            post({
                let taskrun_state = state.taskrun_state.clone();
                move |path, req| add_context(State(taskrun_state), path, req)
            }),
        )
        .route(
            "/pm/tasks/:task_id/session",
            post({
                let taskrun_state = state.taskrun_state.clone();
                move |path, req| update_session(State(taskrun_state), path, req)
            }),
        )
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with OpenTelemetry support
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orchestrator=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting Orchestrator service v{} with TaskRun CRD support",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize application state
    let app_state = Arc::new(AppState::new().await?);

    // Start TaskRun controller if enabled
    let controller_enabled = std::env::var("CONTROLLER_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if controller_enabled {
        let client = app_state.taskrun_state.k8s_client.clone();
        let namespace = app_state.taskrun_state.namespace.clone();

        info!("Starting TaskRun controller in namespace: {}", namespace);

        // Spawn the controller in the background
        tokio::spawn(async move {
            if let Err(e) = run_taskrun_controller(client, namespace).await {
                error!("TaskRun controller error: {}", e);
            }
        });
    } else {
        info!("TaskRun controller disabled");
    }

    // Build the application with middleware layers
    let app = Router::new()
        .nest("/api/v1", api_routes(app_state.clone()))
        .route("/health", get(health_check)) // Root health check for load balancers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // TODO: Configure proper CORS for production
                .layer(middleware::from_fn(error_handler)),
        )
        .with_state(app_state);

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}", listener.local_addr()?);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}

#[cfg(test)]
mod tests {
    // Note: Skipping tests that require actual K8s cluster connection
    // These would be better as integration tests
}
// Trigger build: Thu  3 Jul 2025 02:35:13 PDT
