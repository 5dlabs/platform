//! Main entry point for the Orchestrator service with TaskRun CRD support
//!
//! This version replaces Helm-based deployment with TaskRun CRD management

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
    controllers::run_taskrun_controller,
    handlers::{pm_taskrun::{submit_task, add_context, AppState as HandlerAppState}},
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::{signal, task};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Health check endpoint
async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "features": {
            "taskrun_crd": true,
            "helm_support": false
        }
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
fn api_routes(app_state: Arc<HandlerAppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/pm/tasks", post(submit_task))
        .route("/pm/tasks/:id/context", post(add_context))
        .with_state(app_state)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with OpenTelemetry support
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orchestrator=info,orchestrator_core=debug,kube=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting Orchestrator service v{} with TaskRun CRD support",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize Kubernetes client
    let k8s_client = Client::try_default().await?;
    let namespace = std::env::var("ORCHESTRATOR_NAMESPACE").unwrap_or_else(|_| "orchestrator".to_string());
    
    info!("Using namespace: {}", namespace);

    // Initialize application state for handlers
    let app_state = Arc::new(HandlerAppState {
        k8s_client: k8s_client.clone(),
        namespace: namespace.clone(),
    });

    // Spawn the TaskRun controller in a separate task
    let controller_client = k8s_client.clone();
    let controller_namespace = namespace.clone();
    let controller_handle = task::spawn(async move {
        info!("Starting TaskRun controller");
        if let Err(e) = run_taskrun_controller(controller_client, controller_namespace).await {
            error!("TaskRun controller error: {}", e);
        }
    });

    // Build the application with middleware layers
    let app = Router::new()
        .nest("/api/v1", api_routes(app_state))
        .route("/health", get(health_check)) // Root health check for load balancers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // TODO: Configure proper CORS for production
                .layer(middleware::from_fn(error_handler)),
        );

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}", listener.local_addr()?);

    // Start server with graceful shutdown
    let server_handle = task::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .expect("Server error");
    });

    // Wait for either the server or controller to finish
    tokio::select! {
        _ = server_handle => {
            info!("Server task completed");
        }
        _ = controller_handle => {
            info!("Controller task completed");
        }
    }

    info!("Orchestrator shutdown complete");
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
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.unwrap();
        let json = response.0;
        assert_eq!(json["status"], "healthy");
        assert!(json["features"]["taskrun_crd"].as_bool().unwrap());
        assert!(!json["features"]["helm_support"].as_bool().unwrap());
    }
}