/*
 * 5D Labs Agent Platform - Controller Service
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

//! Controller Service - Kubernetes Controller for CodeRun and DocsRun CRDs
//!
//! This service manages the lifecycle of AI agent jobs by:
//! - Watching for CodeRun and DocsRun custom resources
//! - Creating and managing Kubernetes Jobs for agent execution
//! - Handling resource cleanup and status updates
//! - Providing health and metrics endpoints

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use controller::tasks::run_task_controller;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    // Could be extended with shared state if needed
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,core=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting 5D Labs Controller Service v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize Kubernetes client and controller
    let client = kube::Client::try_default().await?;
    info!("Connected to Kubernetes cluster");

    let state = AppState {};

    // Start the controller in the background
    let controller_handle = {
        let client = client.clone();
        tokio::spawn(async move {
            if let Err(e) = run_task_controller(client, "agent-platform".to_string()).await {
                tracing::error!("Controller error: {}", e);
            }
        })
    };

    // Build the HTTP router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics))
        .route("/webhook", post(webhook_handler))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::new(Duration::from_secs(60))),
        )
        .with_state(state);

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Controller HTTP server listening on 0.0.0.0:8080");

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Wait for controller to finish
    controller_handle.abort();
    info!("Controller service stopped");

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Check if controller is ready (basic check)
    Json(json!({
        "status": "ready",
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .pipe(Ok)
}

async fn metrics() -> Json<Value> {
    // Basic metrics endpoint - can be extended with prometheus metrics
    Json(json!({
        "service": "controller",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

async fn webhook_handler() -> Result<Json<Value>, StatusCode> {
    // Placeholder for webhook handling
    Json(json!({
        "message": "Webhook received"
    }))
    .pipe(Ok)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully");
        },
    }
}

// Helper trait for more ergonomic Result handling
trait Pipe<T> {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R;
}

impl<T> Pipe<T> for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        f(self)
    }
}
