/*
 * 5D Labs Agent Platform - Kubernetes Orchestrator for AI Coding Agents
 * Copyright (C) 2025 5D Labs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Main entry point for the Orchestrator service

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use core::{
    controllers::run_task_controller,
    handlers::{code_handler::submit_code_task, common::AppState, docs_handler::generate_docs},
};
use kube::Client;
use serde_json::{json, Value};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn create_app_state() -> Result<AppState> {
    // Initialize Kubernetes client
    let k8s_client = Client::try_default()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create K8s client: {}", e))?;

    // Get namespace from environment or use default
    let namespace =
        std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| "orchestrator".to_string());

    info!("Initialized orchestrator for namespace: {}", namespace);

    Ok(AppState {
        k8s_client,
        namespace,
    })
}

/// Health check endpoint
async fn health_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Create API routes
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/pm/tasks", post(submit_code_task))
        .route("/pm/docs/generate", post(generate_docs))
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
    let app_state = create_app_state().await?;

    // Start task controller
    let client = app_state.k8s_client.clone();
    let namespace = app_state.namespace.clone();

    info!("Starting task controller in namespace: {}", namespace);

    // Spawn the controller in the background
    tokio::spawn(async move {
        if let Err(e) = run_task_controller(client, namespace).await {
            error!("Task controller error: {}", e);
        }
    });

    // Build the application with middleware layers
    let app = Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(health_check)) // Root health check for load balancers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()), // Simplified for now
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown");
}
