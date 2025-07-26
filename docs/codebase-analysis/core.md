# core Analysis

**Path:** `orchestrator/core`
**Type:** RustBinary
**Lines of Code:** 3295
**Description:** 5D Labs platform orchestrator core API server (for Kubernetes deployment)

## Dependencies

- axum
- tokio
- tower
- tower-http
- kube
- kube-derive
- k8s-openapi
- schemars
- serde
- serde_json
- serde_yaml
- anyhow
- thiserror
- tracing
- tracing-subscriber
- opentelemetry
- opentelemetry-otlp
- opentelemetry_sdk
- tracing-opentelemetry
- reqwest
- futures
- async-trait
- chrono
- regex
- handlebars
- tempfile
- common

## Source Files

### src/crds/mod.rs (5 lines)

**Full Content:**
```rust
pub mod coderun;
pub mod docsrun;

pub use coderun::*;
pub use docsrun::*;

```

### src/crds/coderun.rs (181 lines)

**Key Definitions:**
```rust
10:pub struct SecretEnvVar {
51:pub struct CodeRunSpec {
121:pub struct CodeRunStatus {
162:pub struct CodeRunCondition {
```

**Full Content:**
```rust
//! `CodeRun` Custom Resource Definition for code implementation tasks

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reference to a secret for environment variable
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecretEnvVar {
    /// Name of the environment variable
    pub name: String,
    /// Name of the secret
    #[serde(rename = "secretName")]
    pub secret_name: String,
    /// Key within the secret
    #[serde(rename = "secretKey")]
    pub secret_key: String,
}

/// Default function for `context_version` field
fn default_context_version() -> u32 {
    1
}

/// Default function for `docs_branch` field
fn default_docs_branch() -> String {
    "main".to_string()
}

/// Default function for `continue_session` field
fn default_continue_session() -> bool {
    false
}

/// Default function for `overwrite_memory` field
fn default_overwrite_memory() -> bool {
    false
}

/// `CodeRun` CRD for code implementation tasks
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.platform", version = "v1", kind = "CodeRun")]
#[kube(namespaced)]
#[kube(status = "CodeRunStatus")]
#[kube(printcolumn = r#"{"name":"Task","type":"integer","jsonPath":".spec.taskId"}"#)]
#[kube(printcolumn = r#"{"name":"Service","type":"string","jsonPath":".spec.service"}"#)]
#[kube(printcolumn = r#"{"name":"Model","type":"string","jsonPath":".spec.model"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct CodeRunSpec {
    /// Task ID to implement
    #[serde(rename = "taskId")]
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    #[serde(rename = "docsRepositoryUrl")]
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    #[serde(default, rename = "docsProjectDirectory")]
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    #[serde(default, rename = "workingDirectory")]
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus)
    pub model: String,

    /// GitHub username for authentication and commits
    #[serde(rename = "githubUser")]
    pub github_user: String,

    /// Local MCP tools/servers to enable (comma-separated)
    #[serde(default, rename = "localTools")]
    pub local_tools: Option<String>,

    /// Remote MCP tools/servers to enable (comma-separated)
    #[serde(default, rename = "remoteTools")]
    pub remote_tools: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version", rename = "contextVersion")]
    pub context_version: u32,

    /// Additional context for retry attempts
    #[serde(default, rename = "promptModification")]
    pub prompt_modification: Option<String>,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch", rename = "docsBranch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default = "default_continue_session", rename = "continueSession")]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default = "default_overwrite_memory", rename = "overwriteMemory")]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default, rename = "envFromSecrets")]
    pub env_from_secrets: Vec<SecretEnvVar>,
}

/// Status of the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeRunStatus {
    /// Current phase of the code implementation
    pub phase: String,

    /// Human-readable message about the current state
    pub message: Option<String>,

    /// Timestamp when this phase was reached
    pub last_update: Option<String>,

    /// Associated Kubernetes Job name
    pub job_name: Option<String>,

    /// Pull request URL if created
    pub pull_request_url: Option<String>,

    /// Current retry attempt (if applicable)
    pub retry_count: Option<u32>,

    /// Conditions for the `CodeRun`
    pub conditions: Option<Vec<CodeRunCondition>>,

    /// Name of the `ConfigMap` containing the prompt and context
    pub configmap_name: Option<String>,

    /// Version of the context and prompt used
    pub context_version: Option<u32>,

    /// Modification to the prompt if any
    pub prompt_modification: Option<String>,

    /// Mode of prompt (e.g., "direct", "indirect")
    pub prompt_mode: Option<String>,

    /// Session ID for tracking
    pub session_id: Option<String>,
}

/// Condition for the `CodeRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

```

### src/crds/docsrun.rs (73 lines)

**Key Definitions:**
```rust
13:pub struct DocsRunSpec {
26:pub struct DocsRunStatus {
39:pub struct DocsRunCondition {
62:pub enum DocsRunPhase {
```

**Full Content:**
```rust
//! `DocsRun` Custom Resource Definition for documentation generation

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "orchestrator.platform", version = "v1", kind = "DocsRun")]
#[kube(namespaced)]
#[kube(status = "DocsRunStatus")]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
pub struct DocsRunSpec {
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,
    #[serde(rename = "sourceBranch")]
    pub source_branch: String,
    pub model: String,
    #[serde(rename = "githubUser")]
    pub github_user: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DocsRunStatus {
    pub phase: String,
    pub message: Option<String>,
    pub last_update: Option<String>,
    pub job_name: Option<String>,
    pub pull_request_url: Option<String>,
    pub conditions: Option<Vec<DocsRunCondition>>,
    pub configmap_name: Option<String>,
}

/// Condition for the `DocsRun`
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocsRunCondition {
    /// Type of condition
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, or Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Phase of `DocsRun` execution
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum DocsRunPhase {
    /// `DocsRun` has been created but not yet processed
    Pending,
    /// Documentation generation is in progress
    Running,
    /// Documentation generation completed successfully
    Succeeded,
    /// Documentation generation failed
    Failed,
    /// `DocsRun` was manually cancelled
    Cancelled,
}

```

### src/bin/test_templates.rs (148 lines)

**Full Content:**
```rust
#!/usr/bin/env cargo
//! Template testing utility for local handlebars template validation
//!
//! Usage: cargo run --bin `test_templates`

#![allow(clippy::disallowed_macros)]

use handlebars::Handlebars;
use serde_json::json;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Handlebars Templates...\n");

    // Initialize handlebars engine
    let mut handlebars = Handlebars::new();

    // Template directory
    let template_dir = Path::new("orchestrator-core/templates");

    // Test docs templates
    test_docs_templates(&mut handlebars, template_dir)?;

    // Test code templates
    test_code_templates(&mut handlebars, template_dir)?;

    println!("✅ All templates rendered successfully!");
    Ok(())
}

fn test_docs_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Testing Docs Templates:");

    // Mock DocsRunSpec data
    let docs_data = json!({
        "repository_url": "https://github.com/5dlabs/platform",
        "working_directory": "_projects/simple-api",
        "source_branch": "feature/example-project-and-cli",
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs"
    });

    // Test docs templates
    let docs_templates = [
        "docs/claude.md.hbs",
        "docs/settings.json.hbs",
        "docs/container.sh.hbs",
    ];

    for template_name in &docs_templates {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");

            // Register template
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;

            // Render template
            let result = handlebars.render(template_name, &docs_data)?;

            println!("    ✅ Rendered successfully ({} chars)", result.len());

            // Show first few lines of output for verification
            let lines: Vec<&str> = result.lines().take(3).collect();
            for line in lines {
                println!("    │ {line}");
            }

            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    Ok(())
}

fn test_code_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("💻 Testing Code Templates:");

    // Mock CodeRunSpec data
    let code_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/platform",
        "platform_repository_url": "https://github.com/5dlabs/platform",
        "branch": "feature/example-project-and-cli",
        "working_directory": "_projects/simple-api",
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs",
        "local_tools": "bash,edit,read",
        "remote_tools": "github_create_issue",
        "tool_config": "default",
        "context_version": 1,
        "prompt_modification": null,
        "prompt_mode": "append"
    });

    // Test code templates
    let code_templates = [
        "code/claude.md.hbs",
        "code/settings.json.hbs",
        "code/container.sh.hbs",
    ];

    for template_name in &code_templates {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");

            // Register template
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;

            // Render template
            let result = handlebars.render(template_name, &code_data)?;

            println!("    ✅ Rendered successfully ({} chars)", result.len());

            // Show first few lines of output for verification
            let lines: Vec<&str> = result.lines().take(3).collect();
            for line in lines {
                println!("    │ {line}");
            }

            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    Ok(())
}

```

### src/lib.rs (31 lines)

**Full Content:**
```rust
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

//! Orchestrator core library
//!
//! This crate provides the core functionality for the unified orchestration service,
//! including Kubernetes client wrapper, job orchestration, and request handling.

pub mod controllers;
pub mod crds;
pub mod handlers;

// Re-export commonly used types
pub use controllers::task_controller::ControllerConfig;
pub use crds::{CodeRun, CodeRunSpec, CodeRunStatus, DocsRun, DocsRunSpec, DocsRunStatus};
pub use handlers::*;

```

### src/main.rs (159 lines)

**Full Content:**
```rust
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
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
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

```

### src/controllers/mod.rs (8 lines)

**Full Content:**
```rust
// TODO: Remove this old controller once new one is complete
// pub mod taskrun_old;
// pub use taskrun_old::run_taskrun_controller;

pub mod task_controller;

// Re-export the main controller function for easy access
pub use task_controller::run_task_controller;

```

### src/controllers/task_controller/types.rs (212 lines)

**Key Definitions:**
```rust
9:pub enum Error {
37:pub enum TaskType {
42:impl TaskType {
43:pub fn name(&self) -> String {
50:pub fn is_docs(&self) -> bool {
54:pub fn service_name(&self) -> &str {
61:pub fn model(&self) -> &str {
68:pub fn github_user(&self) -> &str {
75:pub fn repository_url(&self) -> &str {
82:pub fn source_branch(&self) -> Option<&str> {
90:pub fn working_directory(&self) -> &str {
103:pub fn task_id(&self) -> Option<u32> {
111:pub fn context_version(&self) -> u32 {
118:pub fn retry_count(&self) -> u32 {
125:pub fn session_id(&self) -> Option<&str> {
132:pub fn prompt_modification(&self) -> Option<&str> {
140:pub fn local_tools(&self) -> Option<&str> {
147:pub fn remote_tools(&self) -> Option<&str> {
155:pub fn docs_repository_url(&self) -> Option<&str> {
163:pub fn uses_ssh() -> bool {
168:pub fn ssh_secret_name(&self) -> String {
173:pub fn github_token_secret_name(&self) -> String {
178:pub fn docs_branch(&self) -> &str {
187:pub fn continue_session(&self) -> bool {
198:pub fn overwrite_memory(&self) -> bool {
206:pub fn docs_project_directory(&self) -> Option<&str> {
```

**Full Content:**
```rust
use super::config::ControllerConfig;
use crate::crds::{CodeRun, DocsRun};
use kube::{Client, ResourceExt};
use std::sync::Arc;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Missing object key")]
    MissingObjectKey,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Context shared across controller operations
pub(crate) struct Context {
    pub client: Client,
    pub namespace: String,
    pub config: Arc<ControllerConfig>,
}

// Finalizer names for cleanup
pub(crate) const DOCS_FINALIZER_NAME: &str = "docsruns.orchestrator.io/finalizer";
pub(crate) const CODE_FINALIZER_NAME: &str = "coderuns.orchestrator.io/finalizer";

// Enum to represent either task type for shared functionality
pub enum TaskType {
    Docs(Arc<DocsRun>),
    Code(Arc<CodeRun>),
}

impl TaskType {
    pub fn name(&self) -> String {
        match self {
            TaskType::Docs(dr) => dr.name_any(),
            TaskType::Code(cr) => cr.name_any(),
        }
    }

    pub fn is_docs(&self) -> bool {
        matches!(self, TaskType::Docs(_))
    }

    pub fn service_name(&self) -> &str {
        match self {
            TaskType::Docs(_) => "docs-generator", // Fixed service name for docs
            TaskType::Code(cr) => &cr.spec.service,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.model,
            TaskType::Code(cr) => &cr.spec.model,
        }
    }

    pub fn github_user(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.github_user,
            TaskType::Code(cr) => &cr.spec.github_user,
        }
    }

    pub fn repository_url(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.repository_url,
            TaskType::Code(cr) => &cr.spec.repository_url,
        }
    }

    pub fn source_branch(&self) -> Option<&str> {
        match self {
            TaskType::Docs(dr) => Some(&dr.spec.source_branch),
            TaskType::Code(_) => None, // CodeRun uses platform_branch instead
        }
    }

    /// Get working directory (defaults to service name if not specified)
    pub fn working_directory(&self) -> &str {
        match self {
            TaskType::Docs(dr) => &dr.spec.working_directory,
            TaskType::Code(cr) => {
                // Default to service name if working_directory is None or empty
                match &cr.spec.working_directory {
                    Some(wd) if !wd.is_empty() => wd,
                    _ => &cr.spec.service,
                }
            }
        }
    }

    pub fn task_id(&self) -> Option<u32> {
        match self {
            TaskType::Docs(_) => None, // Docs generation doesn't have a specific task ID
            TaskType::Code(cr) => Some(cr.spec.task_id),
        }
    }

    /// Get retry/versioning information for `CodeRun` (docs don't have retries)
    pub fn context_version(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 1, // Docs don't have context versions
            TaskType::Code(cr) => cr.spec.context_version,
        }
    }

    pub fn retry_count(&self) -> u32 {
        match self {
            TaskType::Docs(_) => 0, // Docs don't retry
            TaskType::Code(cr) => cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.session_id.as_deref()),
        }
    }

    pub fn prompt_modification(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.prompt_modification.as_deref(),
        }
    }

    /// Get tool configuration for the task
    pub fn local_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.local_tools.as_deref(),
        }
    }

    pub fn remote_tools(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None, // Docs use fixed tool set
            TaskType::Code(cr) => cr.spec.remote_tools.as_deref(),
        }
    }

    /// Get docs repository info (only for `CodeRun`)
    pub fn docs_repository_url(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => Some(&cr.spec.docs_repository_url),
        }
    }

    /// Always use SSH authentication (we're SSH-only now)
    pub fn uses_ssh() -> bool {
        true
    }

    /// Get SSH secret name for this GitHub user
    pub fn ssh_secret_name(&self) -> String {
        format!("github-ssh-{}", self.github_user())
    }

    /// Get GitHub token secret name for this GitHub user
    pub fn github_token_secret_name(&self) -> String {
        format!("github-token-{}", self.github_user())
    }

    /// Get docs branch (only for `CodeRun`)
    pub fn docs_branch(&self) -> &str {
        match self {
            TaskType::Docs(_) => "main", // Docs use default branch
            TaskType::Code(cr) => &cr.spec.docs_branch,
        }
    }

    /// Get continue session flag - true for retries or user-requested continuation
    #[allow(dead_code)]
    pub fn continue_session(&self) -> bool {
        match self {
            TaskType::Docs(_) => false, // Docs don't continue sessions
            TaskType::Code(cr) => {
                // Continue if it's a retry attempt OR user explicitly requested it
                self.retry_count() > 0 || cr.spec.continue_session
            }
        }
    }

    /// Get overwrite memory flag (only for `CodeRun`)
    pub fn overwrite_memory(&self) -> bool {
        match self {
            TaskType::Docs(_) => true, // Docs always overwrite memory
            TaskType::Code(cr) => cr.spec.overwrite_memory,
        }
    }

    /// Get docs project directory (only for `CodeRun`)
    pub fn docs_project_directory(&self) -> Option<&str> {
        match self {
            TaskType::Docs(_) => None,
            TaskType::Code(cr) => cr.spec.docs_project_directory.as_deref(),
        }
    }
}

```

### src/controllers/task_controller/config.rs (348 lines)

**Key Definitions:**
```rust
12:pub struct ControllerConfig {
38:pub struct JobConfig {
46:pub struct AgentConfig {
57:pub struct ImageConfig {
67:pub struct SecretsConfig {
79:pub struct PermissionsConfig {
93:pub struct TelemetryConfig {
116:pub struct StorageConfig {
132:pub struct CleanupConfig {
169:impl Default for CleanupConfig {
180:impl ControllerConfig {
182:pub fn validate(&self) -> Result<(), anyhow::Error> {
195:pub fn from_mounted_file(config_path: &str) -> Result<Self, anyhow::Error> {
226:impl Default for ControllerConfig {
```

**Full Content:**
```rust
//! Task Controller Configuration
//!
//! Simplified configuration structure for the new DocsRun/CodeRun controller.
//! Contains only the essential configuration needed for our current implementation.

use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::Api, Client};
use serde::{Deserialize, Serialize};

/// Main controller configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControllerConfig {
    /// Job configuration
    pub job: JobConfig,

    /// Agent configuration
    pub agent: AgentConfig,

    /// Secrets configuration
    pub secrets: SecretsConfig,

    /// Tool permissions configuration
    pub permissions: PermissionsConfig,

    /// Telemetry configuration
    pub telemetry: TelemetryConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Cleanup configuration
    #[serde(default)]
    pub cleanup: CleanupConfig,
}

/// Job configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobConfig {
    /// Job timeout in seconds
    #[serde(rename = "activeDeadlineSeconds")]
    pub active_deadline_seconds: i64,
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    /// Container image configuration
    pub image: ImageConfig,

    /// Image pull secrets for private registries
    #[serde(default, rename = "imagePullSecrets")]
    pub image_pull_secrets: Vec<String>,
}

/// Image configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageConfig {
    /// Image repository (e.g., "ghcr.io/5dlabs/claude")
    pub repository: String,

    /// Image tag (e.g., "latest", "v2.1.0")
    pub tag: String,
}

/// Secrets configuration - only what we actually use
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretsConfig {
    /// Anthropic API key secret name (for rotation)
    #[serde(rename = "apiKeySecretName")]
    pub api_key_secret_name: String,

    /// Anthropic API key secret key
    #[serde(rename = "apiKeySecretKey")]
    pub api_key_secret_key: String,
}

/// Tool permissions configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionsConfig {
    /// Whether to override default tool permissions
    #[serde(rename = "agentToolsOverride")]
    pub agent_tools_override: bool,

    /// Allowed tool patterns
    pub allow: Vec<String>,

    /// Denied tool patterns
    pub deny: Vec<String>,
}

/// Telemetry configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,

    /// OTLP endpoint URL
    #[serde(rename = "otlpEndpoint")]
    pub otlp_endpoint: String,

    /// OTLP protocol (grpc/http)
    #[serde(rename = "otlpProtocol")]
    pub otlp_protocol: String,

    /// Logs endpoint (for code tasks)
    #[serde(rename = "logsEndpoint")]
    pub logs_endpoint: String,

    /// Logs protocol (for code tasks)
    #[serde(rename = "logsProtocol")]
    pub logs_protocol: String,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Storage class name for PVCs (e.g., "local-path" for local development)
    #[serde(rename = "storageClassName")]
    pub storage_class_name: Option<String>,

    /// Storage size for workspace PVCs
    #[serde(rename = "workspaceSize", default = "default_workspace_size")]
    pub workspace_size: String,
}

fn default_workspace_size() -> String {
    "10Gi".to_string()
}

/// Cleanup configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    /// Whether automatic cleanup is enabled
    #[serde(default = "default_cleanup_enabled")]
    pub enabled: bool,

    /// Minutes to wait before cleaning up completed (successful) jobs
    #[serde(
        rename = "completedJobDelayMinutes",
        default = "default_completed_delay"
    )]
    pub completed_job_delay_minutes: u64,

    /// Minutes to wait before cleaning up failed jobs
    #[serde(rename = "failedJobDelayMinutes", default = "default_failed_delay")]
    pub failed_job_delay_minutes: u64,

    /// Whether to delete the ConfigMap when cleaning up the job
    #[serde(rename = "deleteConfigMap", default = "default_delete_configmap")]
    pub delete_configmap: bool,
}

fn default_cleanup_enabled() -> bool {
    true
}

fn default_completed_delay() -> u64 {
    5 // 5 minutes
}

fn default_failed_delay() -> u64 {
    60 // 60 minutes (1 hour)
}

fn default_delete_configmap() -> bool {
    true
}

impl Default for CleanupConfig {
    fn default() -> Self {
        CleanupConfig {
            enabled: default_cleanup_enabled(),
            completed_job_delay_minutes: default_completed_delay(),
            failed_job_delay_minutes: default_failed_delay(),
            delete_configmap: default_delete_configmap(),
        }
    }
}

impl ControllerConfig {
    /// Validate that configuration has required fields
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.agent.image.repository == "MISSING_IMAGE_CONFIG"
            || self.agent.image.tag == "MISSING_IMAGE_CONFIG"
        {
            return Err(anyhow::anyhow!(
                "Agent image configuration is missing! This indicates the controller ConfigMap was not loaded properly. \
                Please ensure the 'agent.image.repository' and 'agent.image.tag' are set in the Helm values."
            ));
        }
        Ok(())
    }

    /// Load configuration from mounted ConfigMap file
    pub fn from_mounted_file(config_path: &str) -> Result<Self, anyhow::Error> {
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;

        let config: ControllerConfig = serde_yaml::from_str(&config_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse config YAML: {}", e))?;

        Ok(config)
    }

    /// Load configuration from a `ConfigMap` (legacy API-based method)
    pub async fn from_configmap(
        client: &Client,
        namespace: &str,
        name: &str,
    ) -> Result<Self, anyhow::Error> {
        let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
        let cm = api.get(name).await?;

        let data = cm
            .data
            .ok_or_else(|| anyhow::anyhow!("ConfigMap has no data"))?;
        let config_str = data
            .get("config.yaml")
            .ok_or_else(|| anyhow::anyhow!("ConfigMap missing config.yaml"))?;

        let config: ControllerConfig = serde_yaml::from_str(config_str)?;
        Ok(config)
    }
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            job: JobConfig {
                active_deadline_seconds: 7200, // 2 hours
            },
            agent: AgentConfig {
                image: ImageConfig {
                    repository: "MISSING_IMAGE_CONFIG".to_string(),
                    tag: "MISSING_IMAGE_CONFIG".to_string(),
                },
                image_pull_secrets: vec!["ghcr-secret".to_string()],
            },
            secrets: SecretsConfig {
                api_key_secret_name: "anthropic-api-key".to_string(),
                api_key_secret_key: "api-key".to_string(),
            },
            permissions: PermissionsConfig {
                agent_tools_override: false,
                allow: vec![
                    "Bash(*)".to_string(),
                    "Edit(*)".to_string(),
                    "Read(*)".to_string(),
                    "Write(*)".to_string(),
                    "MultiEdit(*)".to_string(),
                    "Glob(*)".to_string(),
                    "Grep(*)".to_string(),
                    "LS(*)".to_string(),
                ],
                deny: vec![
                    "Bash(npm:install*, yarn:install*, cargo:install*, docker:*, kubectl:*, rm:-rf*, git:*)".to_string(),
                ],
            },
            // Telemetry configuration with environment variable overrides:
            // - OTLP_ENDPOINT: OTLP traces endpoint (default: http://localhost:4317)
            // - LOGS_ENDPOINT: Logs endpoint (default: http://localhost:4318)
            // - LOGS_PROTOCOL: Logs protocol (default: http)
            telemetry: TelemetryConfig {
                enabled: false,
                otlp_endpoint: std::env::var("OTLP_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4317".to_string()),
                otlp_protocol: "grpc".to_string(),
                logs_endpoint: std::env::var("LOGS_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4318".to_string()),
                logs_protocol: std::env::var("LOGS_PROTOCOL")
                    .unwrap_or_else(|_| "http".to_string()),
            },
            storage: StorageConfig {
                storage_class_name: None, // Let K8s use default storage class
                workspace_size: "10Gi".to_string(),
            },
            cleanup: CleanupConfig {
                enabled: true,
                completed_job_delay_minutes: 5,
                failed_job_delay_minutes: 60,
                delete_configmap: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
job:
  activeDeadlineSeconds: 3600

agent:
  image:
    repository: "test/image"
    tag: "latest"

secrets:
  apiKeySecretName: "test-secret"
  apiKeySecretKey: "key"

permissions:
  agentToolsOverride: true
  allow: ["*"]
  deny: []

telemetry:
  enabled: true
  otlpEndpoint: "localhost:4317"
  otlpProtocol: "grpc"
  logsEndpoint: "localhost:4318"
  logsProtocol: "http"

storage:
  storageClassName: "local-path"
  workspaceSize: "5Gi"

cleanup:
  enabled: true
  completedJobDelayMinutes: 5
  failedJobDelayMinutes: 60
  deleteConfigMap: true
"#;

        let config: ControllerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.job.active_deadline_seconds, 3600);
        assert_eq!(config.agent.image.repository, "test/image");
        assert!(config.telemetry.enabled);
        assert_eq!(config.permissions.allow, vec!["*"]);
        assert!(config.cleanup.enabled);
        assert_eq!(config.cleanup.completed_job_delay_minutes, 5);
        assert_eq!(config.cleanup.failed_job_delay_minutes, 60);
    }

    #[test]
    fn test_default_config() {
        let config = ControllerConfig::default();
        assert_eq!(config.job.active_deadline_seconds, 7200);
        assert_eq!(config.agent.image.repository, "MISSING_IMAGE_CONFIG");
        assert_eq!(config.secrets.api_key_secret_name, "anthropic-api-key");
        assert!(!config.telemetry.enabled);
        assert!(!config.permissions.agent_tools_override);
    }
}

```

### src/controllers/task_controller/auth.rs (31 lines)

**Key Definitions:**
```rust
5:pub fn generate_ssh_volumes(task: &TaskType) -> Vec<serde_json::Value> {
```

**Full Content:**
```rust
use super::types::TaskType;
use serde_json::json;

/// Generate SSH key volume configuration if needed
pub fn generate_ssh_volumes(task: &TaskType) -> Vec<serde_json::Value> {
    if !TaskType::uses_ssh() {
        return vec![];
    }

    let ssh_secret_name = task.ssh_secret_name();

    vec![json!({
        "name": "ssh-key",
        "secret": {
            "secretName": ssh_secret_name,
            "defaultMode": 0o600,
            "items": [
                {
                    "key": "ssh-privatekey",
                    "path": "id_ed25519",
                    "mode": 0o600
                },
                {
                    "key": "ssh-publickey",
                    "path": "id_ed25519.pub",
                    "mode": 0o644
                }
            ]
        }
    })]
}

```

### src/controllers/task_controller/mod.rs (19 lines)

**Full Content:**
```rust
//! Task Controller
//!
//! Unified controller for both `DocsRun` and `CodeRun` resources.
//! Handles job orchestration, resource management, and status tracking.

// Public API - re-export the main controller function
pub use reconcile::run_task_controller;

// Public types - re-export config for external use
pub use config::ControllerConfig;

// Internal modules
pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod reconcile;
pub(crate) mod resources;
pub(crate) mod status;
pub(crate) mod templates;
pub(crate) mod types;

```

### src/controllers/task_controller/reconcile.rs (266 lines)

**Full Content:**
```rust
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
use tracing::error;

use super::resources::{cleanup_resources, reconcile_create_or_update};
use super::status::monitor_job_status;
use super::types::{Context, Error, Result, TaskType, CODE_FINALIZER_NAME, DOCS_FINALIZER_NAME};

/// Run the task controller for both `DocsRun` and `CodeRun` resources
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    error!(
        "🚀 AGGRESSIVE DEBUG: Starting task controller in namespace: {}",
        namespace
    );

    error!("🔧 AGGRESSIVE DEBUG: About to load controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            error!("✅ AGGRESSIVE DEBUG: Successfully loaded controller configuration from mounted file");
            error!(
                "🔧 AGGRESSIVE DEBUG: Configuration cleanup enabled = {}",
                cfg.cleanup.enabled
            );

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!(
                    "❌ AGGRESSIVE DEBUG: Configuration validation failed: {}",
                    validation_error
                );
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
                error!(
                    "❌ AGGRESSIVE DEBUG: Default configuration is invalid: {}",
                    validation_error
                );
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
    error!(
        "📝 AGGRESSIVE DEBUG: Starting reconcile_docs for: {}",
        docs_run
            .metadata
            .name
            .as_ref()
            .unwrap_or(&"unnamed".to_string())
    );

    let task = TaskType::Docs(docs_run.clone());
    error!("🔍 AGGRESSIVE DEBUG: Created task type, calling reconcile_common...");

    let result = reconcile_common(task, ctx, DOCS_FINALIZER_NAME).await;
    error!(
        "🏁 AGGRESSIVE DEBUG: reconcile_common completed with result: {:?}",
        result.is_ok()
    );

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
    error!(
        "🎯 AGGRESSIVE DEBUG: Starting reconcile_common for: {}",
        task.name()
    );

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

```

### src/controllers/task_controller/status.rs (347 lines)

**Full Content:**
```rust
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::types::{Context, Result, TaskType};
use crate::crds::{CodeRun, CodeRunCondition, DocsRun, DocsRunCondition};

/// Monitor Job status and update CRD accordingly
pub async fn monitor_job_status(
    task: &TaskType,
    jobs: &Api<Job>,
    ctx: &Arc<Context>,
) -> Result<()> {
    let job_name = get_current_job_name(task);

    if let Some(job_name) = job_name {
        // Get the current job
        match jobs.get(&job_name).await {
            Ok(job) => {
                let (phase, message, pull_request_url) = analyze_job_status(&job);
                update_task_status(task, ctx, &phase, &message, pull_request_url).await?;

                // Schedule cleanup if job is complete and cleanup is enabled
                if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                    schedule_job_cleanup(task, ctx, &job_name, &phase).await?;
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                // Job doesn't exist yet, which is fine for newly created tasks
                info!("Job {} not found yet for task {}", job_name, task.name());
            }
            Err(e) => {
                warn!(
                    "Failed to get job {} for task {}: {}",
                    job_name,
                    task.name(),
                    e
                );
            }
        }
    }

    Ok(())
}

/// Get the current job name for a task
fn get_current_job_name(task: &TaskType) -> Option<String> {
    match task {
        TaskType::Docs(dr) => dr.status.as_ref().and_then(|s| s.job_name.clone()),
        TaskType::Code(cr) => cr.status.as_ref().and_then(|s| s.job_name.clone()),
    }
}

/// Analyze job status and return (phase, message, `pull_request_url`)
fn analyze_job_status(job: &Job) -> (String, String, Option<String>) {
    if let Some(status) = &job.status {
        // Check completion time first
        if status.completion_time.is_some() {
            if let Some(conditions) = &status.conditions {
                for condition in conditions {
                    if condition.type_ == "Complete" && condition.status == "True" {
                        return (
                            "Succeeded".to_string(),
                            "Job completed successfully".to_string(),
                            None,
                        );
                    } else if condition.type_ == "Failed" && condition.status == "True" {
                        let message = condition.message.as_deref().unwrap_or("Job failed");
                        return ("Failed".to_string(), message.to_string(), None);
                    }
                }
            }
        }

        // Check if job is running
        if let Some(active) = status.active {
            if active > 0 {
                return ("Running".to_string(), "Job is running".to_string(), None);
            }
        }

        // Check for failure conditions
        if let Some(failed) = status.failed {
            if failed > 0 {
                return ("Failed".to_string(), "Job failed".to_string(), None);
            }
        }
    }

    // Default to pending if we can't determine status
    (
        "Pending".to_string(),
        "Job status unknown".to_string(),
        None,
    )
}

/// Update the task CRD status
async fn update_task_status(
    task: &TaskType,
    ctx: &Arc<Context>,
    phase: &str,
    message: &str,
    pull_request_url: Option<String>,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();

    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_dr) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "conditions": build_docs_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match docs_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated DocsRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update DocsRun status for {}: {}", name, e);
                }
            }
        }
        TaskType::Code(cr) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": phase,
                    "message": message,
                    "lastUpdate": current_time,
                    "pullRequestUrl": pull_request_url,
                    "retryCount": cr.status.as_ref().map_or(0, |s| s.retry_count.unwrap_or(0)),
                    "conditions": build_code_conditions(phase, message, &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            let pp = PatchParams::default();

            match code_api.patch_status(&name, &pp, &patch).await {
                Ok(_) => {
                    info!("Updated CodeRun status: {} -> {}", name, phase);
                }
                Err(e) => {
                    error!("Failed to update CodeRun status for {}: {}", name, e);
                }
            }
        }
    }

    Ok(())
}

/// Build conditions for `DocsRun` status
fn build_docs_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<DocsRunCondition> {
    vec![DocsRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Build conditions for `CodeRun` status
fn build_code_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<CodeRunCondition> {
    vec![CodeRunCondition {
        condition_type: "Ready".to_string(),
        status: if phase == "Succeeded" {
            "True"
        } else {
            "False"
        }
        .to_string(),
        last_transition_time: Some(timestamp.to_string()),
        reason: Some(phase.to_string()),
        message: Some(message.to_string()),
    }]
}

/// Update task status when job starts (called from reconcile logic)
pub async fn update_job_started(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    configmap_name: &str,
) -> Result<()> {
    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = task.name();
    let current_time = chrono::Utc::now().to_rfc3339();

    match task {
        TaskType::Docs(_) => {
            let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_docs_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            docs_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
        TaskType::Code(_) => {
            let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

            let status_patch = json!({
                "status": {
                    "phase": "Running",
                    "message": "Job started",
                    "lastUpdate": current_time,
                    "jobName": job_name,
                    "configmapName": configmap_name,
                    "conditions": build_code_conditions("Running", "Job started", &current_time)
                }
            });

            let patch = Patch::Merge(&status_patch);
            code_api
                .patch_status(&name, &PatchParams::default(), &patch)
                .await?;
        }
    }

    info!("Updated {} status to Running with job: {}", name, job_name);
    Ok(())
}

/// Schedule cleanup of completed job after configured delay
async fn schedule_job_cleanup(
    task: &TaskType,
    ctx: &Arc<Context>,
    job_name: &str,
    phase: &str,
) -> Result<()> {
    let delay_minutes = if phase == "Succeeded" {
        ctx.config.cleanup.completed_job_delay_minutes
    } else {
        ctx.config.cleanup.failed_job_delay_minutes
    };

    let job_name = job_name.to_string();
    let task_name = task.name();
    let namespace = ctx.namespace.clone();
    let client = ctx.client.clone();
    let delete_configmap = ctx.config.cleanup.delete_configmap;

    info!(
        "Scheduling cleanup for job {} in {} minutes (phase: {})",
        job_name, delay_minutes, phase
    );

    // Spawn background task to handle cleanup after delay
    tokio::spawn(async move {
        // Wait for the configured delay
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_minutes * 60)).await;

        info!("Starting scheduled cleanup for job: {}", job_name);

        // Delete the job
        let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
        match jobs_api
            .delete(&job_name, &kube::api::DeleteParams::background())
            .await
        {
            Ok(_) => info!("Successfully deleted job: {}", job_name),
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Job {} already deleted", job_name);
            }
            Err(e) => {
                error!("Failed to delete job {}: {}", job_name, e);
            }
        }

        // Delete associated ConfigMap if enabled
        if delete_configmap {
            let configmaps_api: Api<k8s_openapi::api::core::v1::ConfigMap> =
                Api::namespaced(client.clone(), &namespace);

            // Find ConfigMap associated with this job
            let labels_selector = "app=orchestrator".to_string();
            let list_params = kube::api::ListParams::default().labels(&labels_selector);

            match configmaps_api.list(&list_params).await {
                Ok(cms) => {
                    for cm in cms.items {
                        if let Some(cm_name) = &cm.metadata.name {
                            // Check if ConfigMap is associated with this job
                            if cm_name.starts_with(&task_name.replace('_', "-")) {
                                match configmaps_api
                                    .delete(cm_name, &kube::api::DeleteParams::default())
                                    .await
                                {
                                    Ok(_) => info!("Successfully deleted ConfigMap: {}", cm_name),
                                    Err(kube::Error::Api(ae)) if ae.code == 404 => {
                                        info!("ConfigMap {} already deleted", cm_name);
                                    }
                                    Err(e) => {
                                        error!("Failed to delete ConfigMap {}: {}", cm_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to list ConfigMaps for cleanup: {}", e);
                }
            }
        }

        info!("Completed cleanup for job: {}", job_name);
    });

    Ok(())
}

```

### src/controllers/task_controller/resources.rs (612 lines)

**Full Content:**
```rust
use super::config::ControllerConfig;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use super::auth::generate_ssh_volumes;
use super::status::update_job_started;
use super::templates::generate_templates;
use super::types::{Result, TaskType};

/// Reconciliation logic for create/update operations
pub async fn reconcile_create_or_update(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
    pvcs: &Api<PersistentVolumeClaim>,
    config: &Arc<ControllerConfig>,
    ctx: &Arc<super::types::Context>,
) -> Result<Action> {
    let name = task.name();
    info!("Creating/updating resources for task: {}", name);

    // Ensure PVC exists for code tasks (docs use emptyDir)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");
        ensure_pvc_exists(pvcs, &pvc_name, service_name, config).await?;
    }

    // Clean up older versions for retries
    cleanup_old_jobs(&task, jobs).await?;
    cleanup_old_configmaps(&task, configmaps).await?;

    // Create ConfigMap FIRST (without owner reference) so Job can mount it
    let cm_name = generate_configmap_name(&task);
    let configmap = create_configmap(&task, &cm_name, config, None)?;

    match configmaps.create(&PostParams::default(), &configmap).await {
        Ok(_) => info!("Created ConfigMap: {}", cm_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("ConfigMap already exists: {}", cm_name);
        }
        Err(e) => return Err(e.into()),
    }

    // Create Job SECOND (now it can successfully mount the existing ConfigMap)
    let job_ref = create_job(&task, jobs, &cm_name, config, ctx).await?;

    // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
    if let Some(owner_ref) = job_ref {
        update_configmap_owner(&task, configmaps, &cm_name, owner_ref).await?;
    }

    Ok(Action::await_change())
}

/// Generate a unique `ConfigMap` name for the task
fn generate_configmap_name(task: &TaskType) -> String {
    let task_id = task.task_id().unwrap_or(0); // Fallback for docs
    let service_name = task.service_name().replace('_', "-");
    let context_version = task.context_version();

    if task.is_docs() {
        format!("{service_name}-docs-v{context_version}-files")
    } else {
        format!("{service_name}-task{task_id}-v{context_version}-files")
    }
}

/// Create `ConfigMap` with all template files
fn create_configmap(
    task: &TaskType,
    name: &str,
    config: &ControllerConfig,
    owner_ref: Option<OwnerReference>,
) -> Result<ConfigMap> {
    let mut data = BTreeMap::new();

    // Generate all templates for this task
    let templates = generate_templates(task, config)?;
    for (filename, content) in templates {
        data.insert(filename, content);
    }

    let labels = create_task_labels(task);
    let mut metadata = ObjectMeta {
        name: Some(name.to_string()),
        labels: Some(labels),
        ..Default::default()
    };

    // Set owner reference if provided (for automatic cleanup)
    if let Some(owner) = owner_ref {
        metadata.owner_references = Some(vec![owner]);
    }

    Ok(ConfigMap {
        metadata,
        data: Some(data),
        ..Default::default()
    })
}

/// Create the main job for the task
async fn create_job(
    task: &TaskType,
    jobs: &Api<Job>,
    cm_name: &str,
    config: &ControllerConfig,
    ctx: &Arc<super::types::Context>,
) -> Result<Option<OwnerReference>> {
    let job_name = generate_job_name(task);
    let job = build_job_spec(task, &job_name, cm_name, config)?;

    match jobs.create(&PostParams::default(), &job).await {
        Ok(created_job) => {
            info!("Created job: {}", job_name);
            update_job_started(task, ctx, &job_name, cm_name).await?;

            // Return owner reference for the created job
            if let (Some(uid), Some(name)) = (created_job.metadata.uid, created_job.metadata.name) {
                Ok(Some(OwnerReference {
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    name,
                    uid,
                    controller: Some(true),
                    block_owner_deletion: Some(true),
                }))
            } else {
                Ok(None)
            }
        }
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            info!("Job already exists: {}", job_name);
            // Try to get existing job for owner reference
            match jobs.get(&job_name).await {
                Ok(existing_job) => {
                    if let (Some(uid), Some(name)) =
                        (existing_job.metadata.uid, existing_job.metadata.name)
                    {
                        Ok(Some(OwnerReference {
                            api_version: "batch/v1".to_string(),
                            kind: "Job".to_string(),
                            name,
                            uid,
                            controller: Some(true),
                            block_owner_deletion: Some(true),
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// Generate a deterministic job name for the task (based on resource name, not timestamp)
fn generate_job_name(task: &TaskType) -> String {
    let resource_name = task.name().replace(['_', '.'], "-");
    match task {
        TaskType::Docs(_) => {
            format!("docs-gen-{resource_name}")
        }
        TaskType::Code(_) => {
            let task_id = task.task_id().unwrap_or(0);
            let context_version = task.context_version();
            format!("code-impl-{resource_name}-task{task_id}-v{context_version}")
        }
    }
}

/// Build the complete Job specification
fn build_job_spec(
    task: &TaskType,
    job_name: &str,
    cm_name: &str,
    config: &ControllerConfig,
) -> Result<Job> {
    let labels = create_task_labels(task);

    // Build volumes based on task type
    let mut volumes = vec![];
    let mut volume_mounts = vec![];

    // ConfigMap volume (always needed)
    volumes.push(json!({
        "name": "task-files",
        "configMap": {
            "name": cm_name
        }
    }));
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/config"
    }));

    // Workspace volume (only for code tasks)
    if !task.is_docs() {
        let service_name = task.service_name();
        let pvc_name = format!("workspace-{service_name}");

        volumes.push(json!({
            "name": "workspace",
            "persistentVolumeClaim": {
                "claimName": pvc_name
            }
        }));
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));
    }

    // SSH volumes if needed
    if TaskType::uses_ssh() {
        let ssh_volumes = generate_ssh_volumes(task);
        volumes.extend(ssh_volumes);

        volume_mounts.push(json!({
            "name": "ssh-key",
            "mountPath": "/workspace/.ssh",
            "readOnly": true
        }));
    }

    // Mount settings.json directly to /etc/claude-code/managed-settings.json
    volume_mounts.push(json!({
        "name": "task-files",
        "mountPath": "/etc/claude-code/managed-settings.json",
        "subPath": "settings.json",
        "readOnly": true
    }));

    // Guidelines files will be copied from ConfigMap to working directory by container.sh
    // No need to mount them separately since they need to be in the working directory

    // Environment variables
    let mut env_vars = vec![
        json!({"name": "ANTHROPIC_API_KEY", "valueFrom": {"secretKeyRef": {"name": config.secrets.api_key_secret_name, "key": config.secrets.api_key_secret_key}}}),
        json!({"name": "TASK_TYPE", "value": if task.is_docs() { "docs" } else { "code" }}),
        json!({"name": "MODEL", "value": task.model()}),
        json!({"name": "GITHUB_USER", "value": task.github_user()}),
        json!({"name": "REPOSITORY_URL", "value": task.repository_url()}),
        json!({"name": "WORKING_DIRECTORY", "value": task.working_directory()}),
    ];

    // Add GitHub token from secret for API operations (PR creation, etc.)
    env_vars.push(json!({
        "name": "GH_TOKEN",
        "valueFrom": {
            "secretKeyRef": {
                "name": task.github_token_secret_name(),
                "key": "token"
            }
        }
    }));

    // Add task-specific environment variables
    match task {
        TaskType::Docs(dr) => {
            env_vars.push(json!({"name": "SOURCE_BRANCH", "value": dr.spec.source_branch}));
        }
        TaskType::Code(cr) => {
            env_vars.push(json!({"name": "TASK_ID", "value": cr.spec.task_id.to_string()}));
            env_vars.push(json!({"name": "SERVICE_NAME", "value": cr.spec.service}));
            env_vars
                .push(json!({"name": "DOCS_REPOSITORY_URL", "value": cr.spec.docs_repository_url}));
            env_vars
                .push(json!({"name": "MCP_CLIENT_CONFIG", "value": "/.claude/client-config.json"}));

            if let Some(local_tools) = &cr.spec.local_tools {
                env_vars.push(json!({"name": "LOCAL_TOOLS", "value": local_tools}));
            }
            if let Some(remote_tools) = &cr.spec.remote_tools {
                env_vars.push(json!({"name": "REMOTE_TOOLS", "value": remote_tools}));
            }

            // Add toolman server URL for MCP integration
            // Environment variable: TOOLMAN_SERVER_URL (default: http://toolman.mcp.svc.cluster.local:3000/mcp)
            let toolman_url = std::env::var("TOOLMAN_SERVER_URL")
                .unwrap_or_else(|_| "http://toolman.mcp.svc.cluster.local:3000/mcp".to_string());
            env_vars.push(json!({"name": "TOOLMAN_SERVER_URL", "value": toolman_url}));

            // Add custom environment variables
            for (name, value) in &cr.spec.env {
                env_vars.push(json!({"name": name, "value": value}));
            }

            // Add environment variables from secrets
            for secret_env in &cr.spec.env_from_secrets {
                env_vars.push(json!({
                    "name": secret_env.name,
                    "valueFrom": {
                        "secretKeyRef": {
                            "name": secret_env.secret_name,
                            "key": secret_env.secret_key
                        }
                    }
                }));
            }
        }
    }

    // Job deadline from config
    let job_deadline = config.job.active_deadline_seconds;

    // Agent image from config
    let agent_image = format!(
        "{}:{}",
        config.agent.image.repository, config.agent.image.tag
    );

    let job_spec = json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": job_name,
            "labels": labels
        },
        "spec": {
            "activeDeadlineSeconds": job_deadline,
            "backoffLimit": 0,
            "template": {
                "metadata": {
                    "labels": labels
                },
                "spec": {
                    "restartPolicy": "Never",
                    "securityContext": {
                        "fsGroup": 1000,
                        "runAsUser": 1000,
                        "runAsGroup": 1000
                    },
                    "imagePullSecrets": config.agent.image_pull_secrets.iter().map(|name| {
                        json!({"name": name})
                    }).collect::<Vec<_>>(),
                    "containers": [{
                        "name": "claude",
                        "image": agent_image,
                        "command": ["/bin/bash", "/config/container.sh"],
                        "env": env_vars,
                        "volumeMounts": volume_mounts,
                        "resources": {
                            "requests": {
                                "cpu": "100m",
                                "memory": "256Mi"
                            },
                            "limits": {
                                "cpu": "2",
                                "memory": "4Gi"
                            }
                        }
                    }],
                    "volumes": volumes
                }
            }
        }
    });

    Ok(serde_json::from_value(job_spec)?)
}

/// Create standard labels for task resources
fn create_task_labels(task: &TaskType) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();

    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert(
        "component".to_string(),
        if task.is_docs() {
            "docs-generator"
        } else {
            "code-runner"
        }
        .to_string(),
    );
    labels.insert(
        "github-user".to_string(),
        sanitize_label_value(task.github_user()),
    );
    labels.insert(
        "context-version".to_string(),
        task.context_version().to_string(),
    );

    match task {
        TaskType::Docs(_) => {
            labels.insert("task-type".to_string(), "docs".to_string());
        }
        TaskType::Code(_) => {
            labels.insert("task-type".to_string(), "code".to_string());
            if let Some(task_id) = task.task_id() {
                labels.insert("task-id".to_string(), task_id.to_string());
            }
            labels.insert("service-name".to_string(), task.service_name().to_string());
        }
    }

    labels
}

/// Sanitize a string value for use as a Kubernetes label value
/// Kubernetes labels must be an empty string or consist of alphanumeric characters, '-', '_' or '.',
/// and must start and end with an alphanumeric character
fn sanitize_label_value(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Replace spaces with hyphens, convert to lowercase
    let mut sanitized = input.to_lowercase().replace([' ', '_'], "-"); // Normalize spaces and underscores to hyphens

    // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
    sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    // Ensure it starts with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().next().unwrap().is_alphanumeric() {
        sanitized.remove(0);
    }

    // Ensure it ends with an alphanumeric character
    while !sanitized.is_empty() && !sanitized.chars().last().unwrap().is_alphanumeric() {
        sanitized.pop();
    }

    // If we ended up with an empty string, provide a fallback
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

/// Ensure PVC exists for the given service
async fn ensure_pvc_exists(
    pvcs: &Api<PersistentVolumeClaim>,
    pvc_name: &str,
    service_name: &str,
    config: &ControllerConfig,
) -> Result<()> {
    match pvcs.get(pvc_name).await {
        Ok(_) => {
            info!("PVC already exists: {}", pvc_name);
            return Ok(());
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // PVC doesn't exist, create it
        }
        Err(e) => return Err(e.into()),
    }

    let mut pvc_spec = json!({
        "apiVersion": "v1",
        "kind": "PersistentVolumeClaim",
        "metadata": {
            "name": pvc_name,
            "labels": {
                "app": "orchestrator",
                "service": service_name
            }
        },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": config.storage.workspace_size
                }
            }
        }
    });

    // Add storage class if specified
    if let Some(storage_class) = &config.storage.storage_class_name {
        pvc_spec["spec"]["storageClassName"] = json!(storage_class);
    }

    let pvc: PersistentVolumeClaim = serde_json::from_value(pvc_spec)?;
    pvcs.create(&PostParams::default(), &pvc).await?;
    info!("Created PVC: {}", pvc_name);

    Ok(())
}

/// Clean up older job versions for retry attempts
async fn cleanup_old_jobs(task: &TaskType, jobs: &Api<Job>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let job_list = jobs
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for job in job_list.items {
            if let Some(version) = job
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(job_name) = &job.metadata.name {
                        jobs.delete(job_name, &DeleteParams::background()).await?;
                        info!("Deleted older job version: {}", job_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Clean up older configmap versions for retry attempts
async fn cleanup_old_configmaps(task: &TaskType, configmaps: &Api<ConfigMap>) -> Result<()> {
    if let Some(task_id) = task.task_id() {
        let current_version = task.context_version();

        let cm_list = configmaps
            .list(&ListParams::default().labels(&format!("task-id={task_id}")))
            .await?;

        for cm in cm_list.items {
            if let Some(version) = cm
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("context-version"))
                .and_then(|v| v.parse::<u32>().ok())
            {
                if version < current_version {
                    if let Some(cm_name) = &cm.metadata.name {
                        configmaps.delete(cm_name, &DeleteParams::default()).await?;
                        info!("Deleted older configmap version: {}", cm_name);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Update the owner reference of an existing `ConfigMap`
async fn update_configmap_owner(
    _task: &TaskType,
    configmaps: &Api<ConfigMap>,
    cm_name: &str,
    owner_ref: OwnerReference,
) -> Result<()> {
    let mut configmap = configmaps.get(cm_name).await?;
    configmap.metadata.owner_references = Some(vec![owner_ref]);
    configmaps
        .replace(cm_name, &PostParams::default(), &configmap)
        .await?;
    info!("Updated ConfigMap owner reference for: {}", cm_name);
    Ok(())
}

/// Cleanup resources when task is deleted
pub async fn cleanup_resources(
    task: TaskType,
    jobs: &Api<Job>,
    configmaps: &Api<ConfigMap>,
) -> Result<Action> {
    let task_label = if let Some(task_id) = task.task_id() {
        format!("task-id={task_id}")
    } else {
        format!(
            "task-type=docs,github-user={}",
            sanitize_label_value(task.github_user())
        )
    };

    info!("Cleaning up resources for task: {}", task.name());

    // Delete all jobs for this task
    let job_list = jobs
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for job in job_list.items {
        if let Some(name) = &job.metadata.name {
            jobs.delete(name, &DeleteParams::background()).await?;
            info!("Deleted job: {}", name);
        }
    }

    // Delete all configmaps for this task
    let cm_list = configmaps
        .list(&ListParams::default().labels(&task_label))
        .await?;
    for cm in cm_list.items {
        if let Some(name) = &cm.metadata.name {
            configmaps.delete(name, &DeleteParams::default()).await?;
            info!("Deleted configmap: {}", name);
        }
    }

    Ok(Action::await_change())
}

```

### src/controllers/task_controller/templates.rs (541 lines)

**Key Definitions:**
```rust
32:pub fn generate_templates(
```

**Full Content:**
```rust
use super::config::ControllerConfig;
use super::types::{Result, TaskType};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

/// Load a template file from the mounted `ConfigMap`
fn load_template(relative_path: &str) -> Result<String> {
    // Convert path separators to underscores for ConfigMap key lookup
    let configmap_key = relative_path.replace('/', "_");
    let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
    debug!(
        "Loading template from: {} (key: {})",
        full_path.display(),
        configmap_key
    );

    fs::read_to_string(&full_path).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to load template {relative_path} (key: {configmap_key}): {e}"
        ))
    })
}

/// Generate all template files for a task
pub fn generate_templates(
    task: &TaskType,
    config: &ControllerConfig,
) -> Result<BTreeMap<String, String>> {
    let mut templates = BTreeMap::new();

    // Generate container startup script
    templates.insert("container.sh".to_string(), generate_container_script(task)?);

    // Generate Claude memory
    templates.insert("CLAUDE.md".to_string(), generate_claude_memory(task)?);

    // Generate Claude settings
    templates.insert(
        "settings.json".to_string(),
        generate_claude_settings(task, config)?,
    );

    // Generate task-specific templates
    if task.is_docs() {
        // Generate docs prompt
        templates.insert("prompt.md".to_string(), generate_docs_prompt(task)?);
    } else {
        // Generate code-specific templates
        templates.insert("mcp.json".to_string(), generate_mcp_config(task, config)?);
        templates.insert(
            "client-config.json".to_string(),
            generate_client_config(task, config)?,
        );
        templates.insert(
            "coding-guidelines.md".to_string(),
            generate_coding_guidelines(task)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            generate_github_guidelines(task)?,
        );
        templates.insert("mcp-tools.md".to_string(), generate_mcp_tools_doc(task)?);
    }

    // Generate hook scripts
    let hook_scripts = generate_hook_scripts(task)?;
    for (filename, content) in hook_scripts {
        // Use hooks- prefix to comply with ConfigMap key constraints
        templates.insert(format!("hooks-{filename}"), content);
    }

    Ok(templates)
}

/// Generate CLAUDE.md content from memory template
fn generate_claude_memory(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/claude.md.hbs"
    } else {
        "code/claude.md.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("claude_memory", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register CLAUDE.md template: {e}"
            ))
        })?;

    let data = json!({
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "task_id": task.task_id(),
        "docs_repository_url": task.docs_repository_url()
    });

    handlebars.render("claude_memory", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render CLAUDE.md template: {e}"
        ))
    })
}

/// Generate Claude Code settings.json for tool permissions
fn generate_claude_settings(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/settings.json.hbs"
    } else {
        "code/settings.json.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("settings", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register settings template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("settings", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render settings template: {e}"
        ))
    })
}

/// Generate container startup script from template
fn generate_container_script(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template_path = if task.is_docs() {
        "docs/container.sh.hbs"
    } else {
        "code/container.sh.hbs"
    };

    let template = load_template(template_path)?;

    handlebars
        .register_template_string("container_script", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register container script template: {e}"
            ))
        })?;

    // Prompt content is now embedded inline in container script - no template needed

    let data = json!({
        "repository_url": task.repository_url(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "model": task.model(),
        "service_name": task.service_name(),
        "task_id": task.task_id(),
        "source_branch": task.source_branch(),
        "docs_repository_url": task.docs_repository_url(),
        "docs_branch": task.docs_branch(),
        "docs_project_directory": task.docs_project_directory(),
        "overwrite_memory": task.overwrite_memory(),
        "continue_session": task.continue_session(),
        "user_requested": match task {
            crate::controllers::task_controller::types::TaskType::Code(cr) => cr.spec.continue_session,
            _ => false
        }
    });

    handlebars.render("container_script", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render container script template: {e}"
        ))
    })
}

/// Generate MCP configuration for implementation tasks
fn generate_mcp_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp.json.hbs")?;

    handlebars
        .register_template_string("mcp", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("mcp", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP template: {e}"
        ))
    })
}

/// Generate MCP tools documentation based on task configuration
fn generate_mcp_tools_doc(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/mcp-tools.md.hbs")?;

    handlebars
        .register_template_string("mcp_tools", template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register MCP tools template: {e}"
            ))
        })?;

    // Parse comma-separated tool strings into arrays
    let local_tools: Vec<String> = task
        .local_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let remote_tools: Vec<String> = task
        .remote_tools()
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let data = json!({
        "localTools": local_tools,
        "remoteTools": remote_tools,
        "service": task.service_name(),
        "task_id": task.task_id()
    });

    handlebars.render("mcp_tools", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render MCP tools template: {e}"
        ))
    })
}

/// Generate client configuration for dynamic tool selection
fn generate_client_config(task: &TaskType, config: &ControllerConfig) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/client-config.json.hbs")?;

    handlebars
        .register_template_string("client_config", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register client config template: {e}"
            ))
        })?;

    let data = build_settings_template_data(task, config);

    handlebars.render("client_config", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render client config template: {e}"
        ))
    })
}

/// Generate coding guidelines for implementation tasks
fn generate_coding_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/coding-guidelines.md.hbs")?;

    handlebars
        .register_template_string("coding_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register coding guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name()
    });

    handlebars.render("coding_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render coding guidelines template: {e}"
        ))
    })
}

/// Generate GitHub guidelines for implementation tasks
fn generate_github_guidelines(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("code/github-guidelines.md.hbs")?;

    handlebars
        .register_template_string("github_guidelines", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register GitHub guidelines template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user()
    });

    handlebars.render("github_guidelines", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render GitHub guidelines template: {e}"
        ))
    })
}

/// Generate docs prompt for documentation generation tasks
fn generate_docs_prompt(task: &TaskType) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    let template = load_template("docs/prompt.md.hbs")?;

    handlebars
        .register_template_string("docs_prompt", &template)
        .map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to register docs prompt template: {e}"
            ))
        })?;

    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "github_user": task.github_user(),
        "working_directory": task.working_directory(),
        "repository_url": task.repository_url()
    });

    handlebars.render("docs_prompt", &data).map_err(|e| {
        crate::controllers::task_controller::types::Error::ConfigError(format!(
            "Failed to render docs prompt template: {e}"
        ))
    })
}

/// Build template data for settings/MCP/client config templates
fn build_settings_template_data(task: &TaskType, config: &ControllerConfig) -> serde_json::Value {
    let mut data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "model": task.model(),
        "github_user": task.github_user(),
        "repository": {
            "url": task.repository_url(),
            "githubUser": task.github_user()
        },
        "working_directory": task.working_directory(),
        "agent_tools_override": config.permissions.agent_tools_override,
        "permissions": {
            "allow": config.permissions.allow,
            "deny": config.permissions.deny
        },
        "telemetry": {
            "enabled": config.telemetry.enabled,
            "otlpEndpoint": config.telemetry.otlp_endpoint,
            "otlpProtocol": config.telemetry.otlp_protocol,
            "logs_endpoint": config.telemetry.logs_endpoint,
            "logs_protocol": config.telemetry.logs_protocol
        }
    });

    // Add retry information for code tasks
    if !task.is_docs() {
        let retry_data = json!({
            "context_version": task.context_version(),
            "prompt_modification": task.prompt_modification(),
            "session_id": task.session_id()
        });
        data["retry"] = retry_data;

        // Add tool configuration
        let (local_tools, remote_tools) = parse_tool_configuration(task);
        data["tools"] = json!({
            "local": local_tools,
            "remote": remote_tools
        });

        // Add docs repository info
        if let Some(docs_url) = task.docs_repository_url() {
            data["docs_repository_url"] = json!(docs_url);
        }
    }

    data
}

/// Parse tool configuration into local and remote tool lists
fn parse_tool_configuration(task: &TaskType) -> (Vec<String>, Vec<String>) {
    let local_tools = task
        .local_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let remote_tools = task
        .remote_tools()
        .map(|tools| tools.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    (local_tools, remote_tools)
}

/// Generate hook scripts from the hooks directory
fn generate_hook_scripts(task: &TaskType) -> Result<BTreeMap<String, String>> {
    let mut hook_scripts = BTreeMap::new();
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(false);

    // Get hook templates based on task type
    let hook_templates = get_hook_templates(task)?;

    // Prepare template data
    let data = json!({
        "task_id": task.task_id(),
        "service_name": task.service_name(),
        "repository": json!({
            "url": task.repository_url(),
            "githubUser": task.github_user()
        }),
        "working_directory": task.working_directory(),
        "attempts": task.retry_count() + 1, // retry_count + 1 = attempt number
        "is_docs_generation": task.is_docs(),
        "docs_repository_url": task.docs_repository_url()
    });

    // Process each hook template
    for (hook_name, template_content) in hook_templates {
        handlebars
            .register_template_string(&hook_name, &template_content)
            .map_err(|e| {
                crate::controllers::task_controller::types::Error::ConfigError(format!(
                    "Failed to register hook template {hook_name}: {e}"
                ))
            })?;

        let rendered = handlebars.render(&hook_name, &data).map_err(|e| {
            crate::controllers::task_controller::types::Error::ConfigError(format!(
                "Failed to render hook template {hook_name}: {e}"
            ))
        })?;

        // Remove .hbs extension for the final filename
        let filename = hook_name.strip_suffix(".hbs").unwrap_or(&hook_name);
        hook_scripts.insert(filename.to_string(), rendered);
    }

    Ok(hook_scripts)
}

/// Get all hook templates for a specific task type by scanning the filesystem
fn get_hook_templates(task: &TaskType) -> Result<Vec<(String, String)>> {
    let hooks_prefix = match task {
        TaskType::Docs(_) => "docs_hooks_",
        TaskType::Code(_) => "code_hooks_",
    };

    debug!("Scanning for hook templates with prefix: {}", hooks_prefix);

    let mut templates = Vec::new();

    // Read the ConfigMap directory and find files with the hook prefix
    match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if this is a hook template for our task type
                        if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                            // Extract just the hook filename (remove prefix and convert back)
                            let hook_name = filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    debug!(
                                        "Loaded hook template: {} (from {})",
                                        hook_name, filename
                                    );
                                    templates.push((hook_name.to_string(), content));
                                }
                                Err(e) => {
                                    debug!("Failed to load hook template {}: {}", filename, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!(
                "Templates directory {} not found or not accessible: {}",
                CLAUDE_TEMPLATES_PATH, e
            );
            // Don't fail - hooks are optional
        }
    }

    Ok(templates)
}

```

### src/handlers/code_handler.rs (126 lines)

**Full Content:**
```rust
//! Code task submission handler

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use kube::Api;
use std::collections::HashMap;
use tracing::{error, info};

use crate::crds::{CodeRun, CodeRunSpec, CodeRunStatus};
use crate::handlers::common::{ApiResponse, AppState};
use common::models::CodeRequest;

pub async fn submit_code_task(
    State(state): State<AppState>,
    Json(request): Json<CodeRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!(
        "Received code task request: task_id={}, service={}",
        request.task_id, request.service
    );

    let spec = CodeRunSpec {
        task_id: request.task_id,
        service: request.service.clone(),
        repository_url: request.repository_url,
        docs_repository_url: request.docs_repository_url,
        docs_project_directory: request.docs_project_directory,
        working_directory: request.working_directory,
        model: request.model.unwrap_or_else(|| {
            std::env::var("DEFAULT_CODE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string())
        }),
        github_user: request.github_user,
        local_tools: request.local_tools,
        remote_tools: request.remote_tools,
        context_version: request.context_version,
        prompt_modification: request.prompt_modification,
        docs_branch: request.docs_branch,
        continue_session: request.continue_session,
        overwrite_memory: request.overwrite_memory,
        env: request.env,
        env_from_secrets: request
            .env_from_secrets
            .into_iter()
            .map(|s| crate::crds::coderun::SecretEnvVar {
                name: s.name,
                secret_name: s.secret_name,
                secret_key: s.secret_key,
            })
            .collect(),
    };

    let coderun = CodeRun {
        metadata: kube::api::ObjectMeta {
            name: Some(format!(
                "code-{}-{}",
                request.task_id,
                Utc::now().timestamp()
            )),
            namespace: Some(state.namespace.clone()),
            ..Default::default()
        },
        spec,
        status: Some(CodeRunStatus {
            phase: "Pending".to_string(),
            message: Some("CodeRun created successfully".to_string()),
            last_update: Some(Utc::now().to_rfc3339()),
            job_name: None,
            pull_request_url: None,
            retry_count: Some(0),
            conditions: None,
            configmap_name: None,
            context_version: Some(1),
            prompt_modification: None,
            prompt_mode: Some("direct".to_string()),
            session_id: None,
        }),
    };

    let api: Api<CodeRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);

    // Check if a CodeRun already exists for this task
    let existing_name = format!("code-{}", request.task_id);
    if let Ok(_existing) = api.get(&existing_name).await {
        error!("CodeRun already exists for task {}", request.task_id);
        return Ok(Json(ApiResponse {
            success: false,
            message: format!("CodeRun already exists for task {}", request.task_id),
            data: None,
        }));
    }

    match api.create(&Default::default(), &coderun).await {
        Ok(created) => {
            info!("CodeRun created successfully: {:?}", created.metadata.name);

            let mut response_data = HashMap::new();
            if let Some(name) = &created.metadata.name {
                response_data.insert(
                    "coderun_name".to_string(),
                    serde_json::Value::String(name.clone()),
                );
            }
            response_data.insert(
                "namespace".to_string(),
                serde_json::Value::String(state.namespace.clone()),
            );

            Ok(Json(ApiResponse {
                success: true,
                message: "Code task submitted successfully".to_string(),
                data: Some(serde_json::Value::Object(
                    response_data.into_iter().collect(),
                )),
            }))
        }
        Err(e) => {
            error!("Failed to create CodeRun: {}", e);
            Ok(Json(ApiResponse {
                success: false,
                message: format!("Failed to create CodeRun: {e}"),
                data: None,
            }))
        }
    }
}

```

### src/handlers/mod.rs (9 lines)

**Full Content:**
```rust
//! Request handlers for the orchestrator service

pub mod code_handler;
pub mod common;
pub mod docs_handler;

pub use code_handler::submit_code_task;
pub use common::{ApiResponse, AppError, AppState};
pub use docs_handler::generate_docs;

```

### src/handlers/docs_handler.rs (102 lines)

**Full Content:**
```rust
//! Documentation generation handler

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, PostParams};
use serde_json::json;
use std::collections::BTreeMap;
use tracing::{error, info};

use crate::crds::{DocsRun, DocsRunSpec, DocsRunStatus};
use crate::handlers::common::{ApiResponse, AppError, AppState};
use common::models::DocsRequest;

/// Generate documentation for Task Master tasks
pub async fn generate_docs(
    State(state): State<AppState>,
    Json(request): Json<DocsRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), (StatusCode, Json<ApiResponse>)> {
    info!(
        "Generate documentation request received for repository: {}",
        request.repository_url
    );

    // Generate a unique DocsRun name using timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let docsrun_name = format!("docs-gen-{timestamp}");

    // Create DocsRun spec for documentation generation
    let spec = DocsRunSpec {
        repository_url: request.repository_url.clone(),
        working_directory: request.working_directory.clone(),
        source_branch: request.source_branch.clone(),
        model: request.model.unwrap_or_else(|| {
            std::env::var("DEFAULT_DOCS_MODEL")
                .unwrap_or_else(|_| "claude-opus-4-20250514".to_string())
        }),
        github_user: request.github_user.clone(),
    };

    // Create DocsRun
    let docsrun = DocsRun {
        metadata: ObjectMeta {
            name: Some(docsrun_name.clone()),
            namespace: Some(state.namespace.clone()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app".to_string(), "orchestrator".to_string());
                labels.insert("type".to_string(), "docs".to_string());
                labels
            }),
            ..Default::default()
        },
        spec,
        status: Some(DocsRunStatus {
            phase: "Pending".to_string(),
            message: Some("DocsRun created successfully".to_string()),
            last_update: Some(chrono::Utc::now().to_rfc3339()),
            job_name: None,
            pull_request_url: None,
            conditions: None,
            configmap_name: None,
        }),
    };

    // Create DocsRun in Kubernetes
    let api: Api<DocsRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);

    match api.create(&PostParams::default(), &docsrun).await {
        Ok(created) => {
            info!("Created documentation generation DocsRun: {}", docsrun_name);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse {
                    success: true,
                    message: "Documentation generation job submitted successfully".to_string(),
                    data: Some(json!({
                        "docsrun_name": docsrun_name,
                        "namespace": state.namespace,
                        "repository_url": created.spec.repository_url,
                        "model": created.spec.model,
                    })),
                }),
            ))
        }
        Err(e) => {
            error!("Failed to create documentation generation DocsRun: {}", e);
            let status_code = StatusCode::from(AppError::from(e));
            Err((
                status_code,
                Json(ApiResponse::error(&format!(
                    "Failed to submit documentation generation job: {}",
                    status_code.canonical_reason().unwrap_or("Unknown error")
                ))),
            ))
        }
    }
}

```

### src/handlers/common.rs (77 lines)

**Key Definitions:**
```rust
9:pub struct AppState {
16:pub enum AppError {
22:impl std::fmt::Display for AppError {
32:impl std::error::Error for AppError {}
34:impl From<kube::Error> for AppError {
40:impl From<AppError> for StatusCode {
52:pub struct ApiResponse {
59:impl ApiResponse {
61:pub fn success(message: &str) -> Self {
70:pub fn error(message: &str) -> Self {
```

**Full Content:**
```rust
//! Shared types and utilities for API handlers

use axum::http::StatusCode;
use kube::Client;
use serde_json::Value;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub k8s_client: Client,
    pub namespace: String,
}

/// Error type for API handlers
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            AppError::Conflict(msg) => write!(f, "Conflict: {msg}"),
            AppError::Internal(msg) => write!(f, "Internal Error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<kube::Error> for AppError {
    fn from(e: kube::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<AppError> for StatusCode {
    fn from(err: AppError) -> Self {
        match err {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// API response structure
#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ApiResponse {
    #[must_use]
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    #[must_use]
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

```

