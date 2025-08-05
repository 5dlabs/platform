# Project: controller

## Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "core",
    "mcp",
    "common",
]

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["5D team"]
license = "AGPL-3.0"
repository = "https://github.com/5dlabs/cto"

[workspace.dependencies]
# Web framework
axum = "0.8.4"
tokio = { version = "1.40", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors", "limit", "timeout"] }

# Kubernetes
kube = { version = "0.93", features = ["runtime", "derive", "client", "ws"] }
kube-derive = "0.93"
k8s-openapi = { version = "0.22", features = ["v1_30"] }
schemars = "0.8"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "2.0.12"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# OpenTelemetry
opentelemetry = "0.30.0"
opentelemetry-otlp = { version = "0.17", features = ["tonic"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
tracing-opentelemetry = "0.31.0"

# CLI
clap = { version = "4.5", features = ["derive", "env", "cargo"] }
dialoguer = "0.11"
indicatif = "0.17"
colored = "3.0.0"

# HTTP Client
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
eventsource-client = "0.15.0"

# Async utilities
futures = "0.3"
async-trait = "0.1"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Text processing
regex = "1.10"
handlebars = "6.3.2"

# Testing
mockall = "0.13"
wiremock = "0.6"

# UUID generation
uuid = { version = "1.10", features = ["v4", "serde"] }

[profile.release]
lto = true
opt-level = 3
codegen-units = 1

# The profile that 'dist' will build with
[profile.dist]
inherits = "release"
lto = "thin"

```

## Source Files

### core/src/crds/mod.rs

```rust
pub mod coderun;
pub mod docsrun;

pub use coderun::*;
pub use docsrun::*;

```

### core/src/crds/coderun.rs

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
#[kube(group = "agents.platform", version = "v1", kind = "CodeRun")]
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

    /// GitHub username for authentication and commits (deprecated - use githubApp)
    #[serde(rename = "githubUser", default)]
    pub github_user: Option<String>,

    /// GitHub App name for authentication (e.g., "5DLabs-Rex")
    #[serde(rename = "githubApp", default)]
    pub github_app: Option<String>,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version", rename = "contextVersion")]
    pub context_version: u32,

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

    /// Tracks whether the code implementation work has been completed successfully
    /// This field is used for idempotent reconciliation and TTL safety
    pub work_completed: Option<bool>,
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

### core/src/crds/docsrun.rs

```rust
//! `DocsRun` Custom Resource Definition for documentation generation

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "agents.platform", version = "v1", kind = "DocsRun")]
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
    #[serde(default)]
    pub model: Option<String>,
    #[serde(rename = "githubUser", default)]
    pub github_user: Option<String>,
    #[serde(rename = "githubApp", default)]
    pub github_app: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocsRunStatus {
    pub phase: String,
    pub message: Option<String>,
    pub last_update: Option<String>,
    pub job_name: Option<String>,
    pub pull_request_url: Option<String>,
    pub conditions: Option<Vec<DocsRunCondition>>,
    pub configmap_name: Option<String>,
    /// Tracks whether the documentation work has been completed successfully
    /// This field is used for idempotent reconciliation and TTL safety
    pub work_completed: Option<bool>,
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

### core/src/tasks/types.rs

```rust
use super::config::ControllerConfig;
use kube::Client;
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
#[derive(Clone)]
pub struct Context {
    pub client: Client,
    pub namespace: String,
    pub config: Arc<ControllerConfig>,
}

// Finalizer names for cleanup
pub(crate) const DOCS_FINALIZER_NAME: &str = "docsruns.orchestrator.io/finalizer";
pub(crate) const CODE_FINALIZER_NAME: &str = "coderuns.orchestrator.io/finalizer";

// Helper functions for SSH and GitHub token secret names
pub fn ssh_secret_name(github_user: &str) -> String {
    format!("github-ssh-{github_user}")
}

pub fn github_token_secret_name(github_user: &str) -> String {
    format!("github-token-{github_user}")
}

// Helper function for GitHub App secret names
pub fn github_app_secret_name(github_app: &str) -> String {
    // Convert GitHub App name to secret name (e.g., "5DLabs-Morgan" -> "github-app-5dlabs-morgan")
    let normalized = github_app.to_lowercase().replace(['_', ' '], "-");
    format!("github-app-{normalized}")
}

```

### core/src/tasks/config.rs

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
                api_key_secret_name: "orchestrator-secrets".to_string(),
                api_key_secret_key: "ANTHROPIC_API_KEY".to_string(),
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
        assert_eq!(config.secrets.api_key_secret_name, "orchestrator-secrets");
        assert!(!config.telemetry.enabled);
        assert!(!config.permissions.agent_tools_override);
    }
}

```

### core/src/tasks/code/controller.rs

```rust
use super::resources::CodeResourceManager;
use crate::tasks::types::{Context, Result, CODE_FINALIZER_NAME};
use crate::crds::CodeRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, instrument};

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_code_run(code_run: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    info!(
        "🎯 CODE DEBUG: Starting reconcile for CodeRun: {}",
        code_run.name_any()
    );

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = code_run.name_any();

    info!("🔄 CODE DEBUG: Reconciling CodeRun: {}", name);

    // Create APIs
    info!("🔗 CODE DEBUG: Creating Kubernetes API clients...");
    let coderuns: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
    info!("✅ CODE DEBUG: API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(
        &coderuns,
        CODE_FINALIZER_NAME,
        code_run.clone(),
        |event| async {
            match event {
                FinalizerEvent::Apply(cr) => reconcile_code_create_or_update(cr, &ctx).await,
                FinalizerEvent::Cleanup(cr) => cleanup_code_resources(cr, &ctx).await,
            }
        },
    )
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => crate::tasks::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => crate::tasks::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => crate::tasks::types::Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            crate::tasks::types::Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    info!(
        "🏁 CODE DEBUG: reconcile completed with result: {:?}",
        result
    );

    Ok(result)
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_code_create_or_update(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    let code_run_name = code_run.name_any();
    info!(
        "Starting status-first idempotent reconcile for CodeRun: {}",
        code_run_name
    );

    // STEP 1: Check CodeRun status first (status-first idempotency)
    if let Some(status) = &code_run.status {
        // Check for completion based on work_completed field (TTL-safe)
        if status.work_completed == Some(true) {
            info!("Work already completed (work_completed=true), no further action needed");
            return Ok(Action::await_change());
        }

        // Check legacy completion states
        match status.phase.as_str() {
            "Succeeded" => {
                info!("Already succeeded, ensuring work_completed is set");
                update_code_status_with_completion(
                    &code_run,
                    ctx,
                    "Succeeded",
                    "Code implementation completed successfully",
                    true,
                )
                .await?;
                return Ok(Action::await_change());
            }
            "Failed" => {
                info!("Already failed, no retry logic");
                return Ok(Action::await_change());
            }
            "Running" => {
                info!("Status shows running, checking actual job state");
                // Continue to job state check below
            }
            _ => {
                info!("Status is '{}', proceeding with job creation", status.phase);
                // Continue to job creation below
            }
        }
    } else {
        info!("No status found, initializing");
    }

    // STEP 2: Check job state for running jobs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let job_name = generate_code_job_name(&code_run);
    info!("Generated job name: {}", job_name);

    let job_state = check_code_job_state(&jobs, &job_name).await?;
    info!("Current job state: {:?}", job_state);

    match job_state {
        CodeJobState::NotFound => {
            info!("No existing job found, using optimistic job creation");

            // STEP 3: Optimistic job creation with conflict handling (copied from working docs controller)
            let ctx_arc = Arc::new(ctx.clone());
            let resource_manager =
                CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);

            // This handles 409 conflicts gracefully (same as docs controller)
            resource_manager
                .reconcile_create_or_update(&code_run)
                .await?;

            // Update status to Running (same pattern as docs)
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Running",
                "Code implementation started",
                false,
            )
            .await?;

            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        CodeJobState::Running => {
            info!("Job is still running, monitoring progress");

            // Update status to Running with workCompleted=false
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Running",
                "Code task in progress",
                false,
            )
            .await?;

            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        CodeJobState::Completed => {
            info!("Job completed successfully - marking work as completed");

            // CRITICAL: Update with work_completed=true for TTL safety
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Succeeded",
                "Code implementation completed successfully",
                true,
            )
            .await?;

            // Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }

        CodeJobState::Failed => {
            info!("Job failed - marking as failed");

            // Update to failed status (no work_completed=true for failures)
            update_code_status_with_completion(
                &code_run,
                ctx,
                "Failed",
                "Code implementation failed",
                false,
            )
            .await?;

            // Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(code_run_name = %code_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_code_resources(code_run: Arc<CodeRun>, ctx: &Context) -> Result<Action> {
    info!("🧹 CODE DEBUG: Cleaning up resources for CodeRun");

    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager =
        CodeResourceManager::new(&jobs, &configmaps, &pvcs, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&code_run).await
}

// Helper functions for idempotent reconciliation - CodeRun version

#[derive(Debug, Clone)]
pub enum CodeJobState {
    NotFound,
    Running,
    Completed,
    Failed,
}

fn generate_code_job_name(code_run: &CodeRun) -> String {
    let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
    let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
    let uid_suffix = code_run
        .metadata
        .uid
        .as_deref()
        .map(|uid| &uid[..8])
        .unwrap_or("nouid");
    let task_id = code_run.spec.task_id;
    let context_version = code_run.spec.context_version;

    format!("code-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}")
        .replace(['_', '.'], "-")
        .to_lowercase()
}

async fn check_code_job_state(jobs: &Api<Job>, job_name: &str) -> Result<CodeJobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_code_job_state(status))
            } else {
                Ok(CodeJobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(CodeJobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_code_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> CodeJobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return CodeJobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return CodeJobState::Failed;
            }
        }
    }

    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return CodeJobState::Completed;
        }
    }

    if let Some(failed) = status.failed {
        if failed > 0 {
            return CodeJobState::Failed;
        }
    }

    CodeJobState::Running
}

async fn update_code_status_with_completion(
    code_run: &CodeRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
    work_completed: bool,
) -> Result<()> {
    // Only update if status actually changed or work_completed changed
    let current_phase = code_run
        .status
        .as_ref()
        .map(|s| s.phase.as_str())
        .unwrap_or("");
    let current_work_completed = code_run
        .status
        .as_ref()
        .and_then(|s| s.work_completed)
        .unwrap_or(false);

    if current_phase == new_phase && current_work_completed == work_completed {
        info!(
            "Status already '{}' with work_completed={}, skipping update to prevent reconciliation",
            new_phase, work_completed
        );
        return Ok(());
    }

    info!(
        "Updating status from '{}' (work_completed={}) to '{}' (work_completed={})",
        current_phase, current_work_completed, new_phase, work_completed
    );

    let coderuns: Api<CodeRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
        }
    });

    // Use status subresource to avoid triggering spec reconciliation
    coderuns
        .patch_status(
            &code_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await?;

    info!(
        "Status updated successfully to '{}' with work_completed={}",
        new_phase, work_completed
    );
    Ok(())
}

```

### core/src/tasks/code/mod.rs

```rust
pub mod controller;
pub mod resources;
pub mod status;
pub mod templates;

pub use controller::*;
```

### core/src/tasks/code/status.rs

```rust
use crate::tasks::types::{Context, Result};
use crate::crds::{CodeRun, CodeRunCondition};
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use kube::ResourceExt;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct CodeStatusManager;

#[allow(dead_code)]
impl CodeStatusManager {
    /// Monitor Job status and update CodeRun CRD accordingly
    pub async fn monitor_job_status(
        code_run: &Arc<CodeRun>,
        jobs: &Api<Job>,
        ctx: &Arc<Context>,
    ) -> Result<()> {
        let job_name = Self::get_current_job_name(code_run);

        if let Some(job_name) = job_name {
            // Get the current job
            match jobs.get(&job_name).await {
                Ok(job) => {
                    let (phase, message) = Self::analyze_job_status(&job);
                    Self::update_status(code_run, ctx, &phase, &message).await?;

                    // Schedule cleanup if job is complete and cleanup is enabled
                    if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                        Self::schedule_job_cleanup(code_run, ctx, &job_name, &phase).await?;
                    }
                }
                Err(kube::Error::Api(ae)) if ae.code == 404 => {
                    warn!(
                        "Job {} not found for CodeRun {}",
                        job_name,
                        code_run.name_any()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to get job {} for CodeRun {}: {}",
                        job_name,
                        code_run.name_any(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Update the status when a job starts
    pub async fn update_job_started(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        _cm_name: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let current_retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));

        let status_patch = json!({
            "status": {
                "phase": "Running",
                "message": "Code implementation job started",
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "jobName": job_name,
                "retryCount": current_retry_count,
                "conditions": Self::build_conditions("Running", "Code implementation job started", &chrono::Utc::now().to_rfc3339())
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!("Updated CodeRun status: {} -> Running", name);
            }
            Err(e) => {
                error!("Failed to update CodeRun status for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Increment retry count for failed attempts
    #[allow(dead_code)]
    pub async fn increment_retry_count(code_run: &Arc<CodeRun>, ctx: &Arc<Context>) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);
        let current_retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        let new_retry_count = current_retry_count + 1;

        let status_patch = json!({
            "status": {
                "retryCount": new_retry_count,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Retry attempt {} scheduled", new_retry_count)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!(
                    "Updated CodeRun retry count: {} -> {}",
                    name, new_retry_count
                );
            }
            Err(e) => {
                error!("Failed to update CodeRun retry count for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Update session info for retries
    #[allow(dead_code)]
    pub async fn update_session_info(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        session_id: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "sessionId": session_id,
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "message": format!("Session {} started", session_id)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(_) => {
                info!("Updated CodeRun session info: {} -> {}", name, session_id);
            }
            Err(e) => {
                error!("Failed to update CodeRun session info for {}: {}", name, e);
            }
        }

        Ok(())
    }

    /// Update the CodeRun CRD status
    async fn update_status(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        phase: &str,
        message: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = code_run.name_any();

        let current_time = chrono::Utc::now().to_rfc3339();
        let code_api: Api<CodeRun> = Api::namespaced(client.clone(), namespace);

        let current_retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        let session_id = code_run
            .status
            .as_ref()
            .and_then(|s| s.session_id.as_deref());

        let mut status_patch = json!({
            "status": {
                "phase": phase,
                "message": message,
                "lastUpdate": current_time,
                "retryCount": current_retry_count,
                "conditions": Self::build_conditions(phase, message, &current_time)
            }
        });

        // Include session ID if present
        if let Some(sid) = session_id {
            status_patch["status"]["sessionId"] = json!(sid);
        }

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match code_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_code_run) => {
                info!(
                    "✅ Successfully updated CodeRun status: {} -> {}",
                    name, phase
                );
                info!(
                    "✅ Updated resource version: {:?}",
                    updated_code_run.metadata.resource_version
                );
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update CodeRun status for {}: {}", name, e);
                error!("❌ Error type: {}", std::any::type_name_of_val(&e));
                error!("❌ Full error details: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// Get the current job name for a code task
    fn get_current_job_name(code_run: &CodeRun) -> Option<String> {
        code_run.status.as_ref().and_then(|s| s.job_name.clone())
    }

    /// Analyze job status and return (phase, message)
    fn analyze_job_status(job: &Job) -> (String, String) {
        if let Some(status) = &job.status {
            // Check completion time first
            if status.completion_time.is_some() {
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        if condition.type_ == "Complete" && condition.status == "True" {
                            return (
                                "Succeeded".to_string(),
                                "Code implementation completed successfully".to_string(),
                            );
                        } else if condition.type_ == "Failed" && condition.status == "True" {
                            let message = condition
                                .message
                                .as_deref()
                                .unwrap_or("Code implementation failed");
                            return ("Failed".to_string(), message.to_string());
                        }
                    }
                }
            }

            // Check if job is running
            if let Some(active) = status.active {
                if active > 0 {
                    return (
                        "Running".to_string(),
                        "Code implementation is running".to_string(),
                    );
                }
            }

            // Check for failure conditions
            if let Some(failed) = status.failed {
                if failed > 0 {
                    return (
                        "Failed".to_string(),
                        "Code implementation failed".to_string(),
                    );
                }
            }
        }

        (
            "Pending".to_string(),
            "Code implementation job pending".to_string(),
        )
    }

    /// Build CodeRun conditions
    fn build_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<CodeRunCondition> {
        vec![CodeRunCondition {
            condition_type: phase.to_string(),
            status: "True".to_string(),
            last_transition_time: Some(timestamp.to_string()),
            reason: Some(match phase {
                "Running" => "JobStarted".to_string(),
                "Succeeded" => "JobCompleted".to_string(),
                "Failed" => "JobFailed".to_string(),
                _ => "Unknown".to_string(),
            }),
            message: Some(message.to_string()),
        }]
    }

    /// Schedule cleanup of completed job
    async fn schedule_job_cleanup(
        code_run: &Arc<CodeRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        phase: &str,
    ) -> Result<()> {
        info!(
            "Scheduling cleanup for CodeRun {} job {} (phase: {})",
            code_run.name_any(),
            job_name,
            phase
        );

        // For code jobs, we might want to keep them longer for debugging
        // or implement different cleanup policies based on success/failure
        let cleanup_delay_minutes = if phase == "Succeeded" {
            ctx.config.cleanup.completed_job_delay_minutes
        } else {
            ctx.config.cleanup.failed_job_delay_minutes
        };

        if cleanup_delay_minutes > 0 {
            info!(
                "Delaying cleanup for {} minutes for CodeRun job {}",
                cleanup_delay_minutes, job_name
            );
            // In a real implementation, you might schedule this with a timer or job queue
            // For now, just log the intent
        } else {
            // Clean up immediately
            let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

            if let Err(e) = jobs
                .delete(job_name, &kube::api::DeleteParams::default())
                .await
            {
                warn!("Failed to delete completed code job {}: {}", job_name, e);
            } else {
                info!("Successfully deleted completed code job: {}", job_name);
            }
        }

        Ok(())
    }
}

```

### core/src/tasks/code/resources.rs

```rust
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::{github_app_secret_name, Context, Result};
use crate::crds::CodeRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, info};

pub struct CodeResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub pvcs: &'a Api<PersistentVolumeClaim>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> CodeResourceManager<'a> {
    pub fn new(
        jobs: &'a Api<Job>,
        configmaps: &'a Api<ConfigMap>,
        pvcs: &'a Api<PersistentVolumeClaim>,
        config: &'a Arc<ControllerConfig>,
        ctx: &'a Arc<Context>,
    ) -> Self {
        Self {
            jobs,
            configmaps,
            pvcs,
            config,
            ctx,
        }
    }

    pub async fn reconcile_create_or_update(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        info!(
            "🚀 CODE DEBUG: Creating/updating code resources for: {}",
            name
        );

        // Ensure PVC exists for code tasks (persistent workspace)
        let service_name = &code_run.spec.service;
        let pvc_name = format!("workspace-{service_name}");
        info!("📦 CODE DEBUG: Ensuring PVC exists: {}", pvc_name);
        self.ensure_pvc_exists(&pvc_name, service_name).await?;
        info!("✅ CODE DEBUG: PVC check completed");

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 CODE DEBUG: Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(code_run);
        info!("📄 CODE DEBUG: Generated ConfigMap name: {}", cm_name);

        info!("🔧 CODE DEBUG: Creating ConfigMap template data...");
        let configmap = self.create_configmap(code_run, &cm_name, None)?;
        info!("✅ CODE DEBUG: ConfigMap template created successfully");

        // Always create or update ConfigMap to ensure latest template content
        info!("📤 CODE DEBUG: Attempting to create ConfigMap: {}", cm_name);
        match self
            .configmaps
            .create(&PostParams::default(), &configmap)
            .await
        {
            Ok(_) => {
                info!("✅ CODE DEBUG: Created ConfigMap: {}", cm_name);
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // ConfigMap exists, update it with latest content
                info!(
                    "📝 CODE DEBUG: ConfigMap exists, updating with latest content: {}",
                    cm_name
                );
                match self
                    .configmaps
                    .replace(&cm_name, &PostParams::default(), &configmap)
                    .await
                {
                    Ok(_) => {
                        info!("✅ CODE DEBUG: Updated ConfigMap: {}", cm_name);
                    }
                    Err(e) => {
                        error!(
                            "❌ CODE DEBUG: Failed to update ConfigMap {}: {}",
                            cm_name, e
                        );
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ CODE DEBUG: Failed to create ConfigMap {}: {}",
                    cm_name, e
                );
                return Err(e.into());
            }
        }

        // Create Job using idempotent creation (now it can successfully mount the existing ConfigMap)
        info!("🚀 CODE DEBUG: Creating job with ConfigMap: {}", cm_name);
        let job_ref = self.create_or_get_job(code_run, &cm_name).await?;
        info!("✅ CODE DEBUG: Job creation completed");

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            info!("🔗 CODE DEBUG: Updating ConfigMap owner reference");
            self.update_configmap_owner(code_run, &cm_name, owner_ref)
                .await?;
            info!("✅ CODE DEBUG: ConfigMap owner reference updated");
        } else {
            info!("⚠️ CODE DEBUG: No job owner reference to set");
        }

        info!(
            "🎉 CODE DEBUG: Reconciliation completed successfully for: {}",
            name
        );
        Ok(Action::await_change())
    }

    pub async fn cleanup_resources(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        info!("Cleaning up code resources for: {}", name);

        // Clean up any remaining jobs and configmaps (but keep PVCs for session continuity)
        self.cleanup_old_jobs(code_run).await?;
        self.cleanup_old_configmaps(code_run).await?;

        Ok(Action::await_change())
    }

    async fn ensure_pvc_exists(&self, pvc_name: &str, service_name: &str) -> Result<()> {
        match self.pvcs.get(pvc_name).await {
            Ok(_) => {
                info!("PVC {} already exists", pvc_name);
                Ok(())
            }
            Err(kube::Error::Api(ae)) if ae.code == 404 => {
                info!("Creating PVC: {}", pvc_name);
                let pvc = self.build_pvc_spec(pvc_name, service_name);
                match self.pvcs.create(&PostParams::default(), &pvc).await {
                    Ok(_) => {
                        info!("Successfully created PVC: {}", pvc_name);
                        Ok(())
                    }
                    Err(kube::Error::Api(ae)) if ae.code == 409 => {
                        info!("PVC {} was created concurrently", pvc_name);
                        Ok(())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn build_pvc_spec(&self, pvc_name: &str, service_name: &str) -> PersistentVolumeClaim {
        let mut spec = json!({
            "accessModes": ["ReadWriteOnce"],
            "resources": {
                "requests": {
                    "storage": self.config.storage.workspace_size.clone()
                }
            }
        });

        // Add storageClassName if specified in config
        if let Some(ref storage_class) = self.config.storage.storage_class_name {
            spec["storageClassName"] = json!(storage_class);
        }

        let pvc_spec = json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": {
                "name": pvc_name,
                "labels": {
                    "app": "orchestrator",
                    "component": "code-runner",
                    "service": service_name
                }
            },
            "spec": spec
        });

        serde_json::from_value(pvc_spec).expect("Failed to build PVC spec")
    }

    fn generate_configmap_name(&self, code_run: &CodeRun) -> String {
        // Generate unique ConfigMap name per CodeRun to prevent conflicts between sequential jobs
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = code_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
        let task_id = code_run.spec.task_id;
        let service_name = code_run.spec.service.replace('_', "-");
        let context_version = code_run.spec.context_version;
        
        format!("code-{namespace}-{name}-{uid_suffix}-{service_name}-t{task_id}-v{context_version}-files")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn create_configmap(
        &self,
        code_run: &CodeRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for code
        let templates = super::templates::CodeTemplateGenerator::generate_all_templates(
            code_run,
            self.config,
        )?;
        for (filename, content) in templates {
            data.insert(filename, content);
        }

        let labels = self.create_task_labels(code_run);
        let mut metadata = ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            ..Default::default()
        };

        if let Some(owner) = owner_ref {
            metadata.owner_references = Some(vec![owner]);
        }

        Ok(ConfigMap {
            metadata,
            data: Some(data),
            ..Default::default()
        })
    }

    /// Idempotent job creation: create if doesn't exist, get if it does
    async fn create_or_get_job(
        &self,
        code_run: &CodeRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(code_run);

        // Try to get existing job first (idempotent check)
        match self.jobs.get(&job_name).await {
            Ok(existing_job) => {
                info!("Found existing job: {}, using it", job_name);
                Ok(Some(OwnerReference {
                    api_version: "batch/v1".to_string(),
                    kind: "Job".to_string(),
                    name: job_name,
                    uid: existing_job.metadata.uid.unwrap_or_default(),
                    controller: Some(false),
                    block_owner_deletion: Some(true),
                }))
            }
            Err(_) => {
                // Job doesn't exist, create it
                info!("Job {} doesn't exist, creating it", job_name);
                self.create_job(code_run, cm_name).await
            }
        }
    }

    async fn create_job(
        &self,
        code_run: &CodeRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(code_run);
        let job = self.build_job_spec(code_run, &job_name, cm_name)?;

        match self.jobs.create(&PostParams::default(), &job).await {
            Ok(created_job) => {
                info!("Created code job: {}", job_name);
                // Update status
                super::status::CodeStatusManager::update_job_started(
                    &Arc::new(code_run.clone()),
                    self.ctx,
                    &job_name,
                    cm_name,
                )
                .await?;

                // Return owner reference for the created job
                if let (Some(uid), Some(name)) =
                    (created_job.metadata.uid, created_job.metadata.name)
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
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!("Job already exists: {}", job_name);
                // Try to get existing job for owner reference
                match self.jobs.get(&job_name).await {
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

    fn generate_job_name(&self, code_run: &CodeRun) -> String {
        // Use deterministic naming based on the CodeRun's actual name and UID
        // This ensures the same CodeRun always generates the same Job name
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = code_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
        let task_id = code_run.spec.task_id;
        let context_version = code_run.spec.context_version;

        let job_name = format!("code-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}")
            .replace(['_', '.'], "-")
            .to_lowercase();

        // Kubernetes has a 63-character limit for resource names and labels
        // Truncate if necessary while preserving uniqueness
        if job_name.len() > 63 {
            let uid_and_suffix = format!("-{uid_suffix}-t{task_id}-v{context_version}");
            let available_len = 63 - uid_and_suffix.len();
            let prefix = format!("code-{namespace}-{name}").replace(['_', '.'], "-").to_lowercase();
            
            if prefix.len() > available_len {
                format!("{}-{uid_suffix}-t{task_id}-v{context_version}", &prefix[..available_len])
            } else {
                job_name
            }
        } else {
            job_name
        }
    }

    fn build_job_spec(&self, code_run: &CodeRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(code_run);

        // Create owner reference to CodeRun for proper event handling
        let owner_ref = OwnerReference {
            api_version: "agents.platform/v1".to_string(),
            kind: "CodeRun".to_string(),
            name: code_run.name_any(),
            uid: code_run.metadata.uid.clone().unwrap_or_default(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        };

        // Build volumes for code (PVC for persistence)
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
            "mountPath": "/task-files"
        }));

        // Mount settings.json as managed-settings.json for enterprise compatibility
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/etc/claude-code/managed-settings.json",
            "subPath": "settings.json"
        }));

        // PVC workspace volume for code (persistent across sessions)
        let pvc_name = format!("workspace-{}", code_run.spec.service);
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

        // GitHub App authentication only - no SSH volumes needed
        let github_app = code_run.spec.github_app.as_ref()
            .ok_or_else(|| {
                tracing::error!("GitHub App is required for CodeRun authentication");
                crate::tasks::types::Error::ConfigError("GitHub App is required for CodeRun authentication".to_string())
            })?;
        
        tracing::info!("Using GitHub App authentication for CodeRun: {}", github_app);

        let image = format!(
            "{}:{}",
            self.config.agent.image.repository, self.config.agent.image.tag
        );

        // Build environment variables for code tasks
        let env_vars = vec![
            json!({
                "name": "GITHUB_APP_ID",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_app_secret_name(github_app),
                        "key": "app-id"
                    }
                }
            }),
            json!({
                "name": "GITHUB_APP_PRIVATE_KEY",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_app_secret_name(github_app),
                        "key": "private-key"
                    }
                }
            }),
            json!({
                "name": "ANTHROPIC_API_KEY",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": self.config.secrets.api_key_secret_name,
                        "key": self.config.secrets.api_key_secret_key
                    }
                }
            }),
        ];

        // Code-specific environment variables will be added here when needed

        let job_spec = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_name,
                "labels": labels,
                "ownerReferences": [{
                    "apiVersion": owner_ref.api_version,
                    "kind": owner_ref.kind,
                    "name": owner_ref.name,
                    "uid": owner_ref.uid,
                    "controller": owner_ref.controller,
                    "blockOwnerDeletion": owner_ref.block_owner_deletion
                }]
            },
            "spec": {
                "backoffLimit": 0,
                "ttlSecondsAfterFinished": 30,
                "template": {
                    "metadata": {
                        "labels": labels
                    },
                    "spec": {
                        "restartPolicy": "Never",
                        "containers": [{
                            "name": "claude-code",
                            "image": image,
                            "env": env_vars,
                            "command": ["/bin/bash"],
                            "args": ["/task-files/container.sh"],
                            "workingDir": "/workspace",
                            "volumeMounts": volume_mounts
                        }],
                        "volumes": volumes
                    }
                }
            }
        });

        Ok(serde_json::from_value(job_spec)?)
    }

    fn create_task_labels(&self, code_run: &CodeRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        labels.insert("app".to_string(), "orchestrator".to_string());
        labels.insert("component".to_string(), "code-runner".to_string());
        let github_identifier = code_run.spec.github_app.as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        labels.insert(
            "github-user".to_string(),
            self.sanitize_label_value(github_identifier),
        );
        labels.insert(
            "context-version".to_string(),
            code_run.spec.context_version.to_string(),
        );

        // Code-specific labels
        labels.insert("task-type".to_string(), "code".to_string());
        labels.insert("task-id".to_string(), code_run.spec.task_id.to_string());
        labels.insert(
            "service".to_string(),
            self.sanitize_label_value(&code_run.spec.service),
        );

        labels
    }


    async fn update_configmap_owner(
        &self,
        _code_run: &CodeRun,
        cm_name: &str,
        owner_ref: OwnerReference,
    ) -> Result<()> {
        let mut existing_cm = self.configmaps.get(cm_name).await?;

        // Add owner reference
        let owner_refs = existing_cm
            .metadata
            .owner_references
            .get_or_insert_with(Vec::new);
        owner_refs.push(owner_ref);

        // Update the ConfigMap
        self.configmaps
            .replace(cm_name, &PostParams::default(), &existing_cm)
            .await?;
        info!("Updated ConfigMap {} with owner reference", cm_name);

        Ok(())
    }

    // Legacy cleanup method for backward compatibility
    async fn cleanup_old_jobs(&self, code_run: &CodeRun) -> Result<()> {
        let github_identifier = code_run.spec.github_app.as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(github_identifier),
            self.sanitize_label_value(&code_run.spec.service)
        ));

        let jobs = self.jobs.list(&list_params).await?;

        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old code job: {}", job_name);
                let _ = self.jobs.delete(&job_name, &DeleteParams::default()).await;
            }
        }

        Ok(())
    }

    async fn cleanup_old_configmaps(&self, code_run: &CodeRun) -> Result<()> {
        // Generate current ConfigMap name to avoid deleting it
        let current_cm_name = self.generate_configmap_name(code_run);
        
        let github_identifier = code_run.spec.github_app.as_deref()
            .or(code_run.spec.github_user.as_deref())
            .unwrap_or("unknown");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(github_identifier),
            self.sanitize_label_value(&code_run.spec.service)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;

        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                // Skip deleting the current ConfigMap - this prevents deletion of active job's ConfigMap
                if cm_name == current_cm_name {
                    info!("Skipping deletion of current ConfigMap: {}", cm_name);
                    continue;
                }
                
                // Check if ConfigMap has an owner reference to a Job that's still running
                let has_active_job = cm.metadata.owner_references
                    .as_ref()
                    .map(|owners| {
                        owners.iter().any(|owner| {
                            owner.kind == "Job" && owner.api_version.starts_with("batch/")
                        })
                    })
                    .unwrap_or(false);
                
                if has_active_job {
                    // If ConfigMap is owned by a Job, let Kubernetes handle cleanup when Job completes
                    info!("Skipping cleanup of ConfigMap with active Job owner: {}", cm_name);
                    continue;
                }
                
                info!("Deleting old code ConfigMap: {}", cm_name);
                let _ = self
                    .configmaps
                    .delete(&cm_name, &DeleteParams::default())
                    .await;
            }
        }

        Ok(())
    }

    fn sanitize_label_value(&self, input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        // Replace spaces with hyphens, convert to lowercase
        let mut sanitized = input.to_lowercase().replace([' ', '_'], "-");

        // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
        sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        // Ensure it starts and ends with alphanumeric
        let chars: Vec<char> = sanitized.chars().collect();
        let start = chars.iter().position(|c| c.is_alphanumeric()).unwrap_or(0);
        let end = chars
            .iter()
            .rposition(|c| c.is_alphanumeric())
            .unwrap_or(chars.len().saturating_sub(1));

        if start <= end {
            sanitized = chars[start..=end].iter().collect();
        }

        // Truncate to 63 characters (Kubernetes label limit)
        if sanitized.len() > 63 {
            sanitized.truncate(63);
            // Ensure it still ends with alphanumeric after truncation
            if let Some(last_alphanumeric) = sanitized.rfind(|c: char| c.is_alphanumeric()) {
                sanitized.truncate(last_alphanumeric + 1);
            }
        }

        sanitized
    }
}


```

### core/src/tasks/code/templates.rs

```rust
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::Result;
use crate::crds::CodeRun;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

pub struct CodeTemplateGenerator;

impl CodeTemplateGenerator {
    /// Generate all template files for a code task
    pub fn generate_all_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core code templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(code_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(code_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(code_run, config)?,
        );

        // Generate code-specific templates
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(code_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_container_script(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/container.sh.hbs")?;

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/claude.md.hbs")?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
            "context_version": code_run.spec.context_version,
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render CLAUDE.md: {e}"
            ))
        })
    }

    fn generate_claude_settings(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/settings.json.hbs")?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        let context = json!({
            "model": code_run.spec.model,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render settings.json: {e}"
            ))
        })
    }

    fn generate_mcp_config(_code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        // MCP config is currently static, so just load and return the template content
        Self::load_template("code/mcp.json.hbs")
    }



    fn generate_coding_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/coding-guidelines.md.hbs")?;

        handlebars
            .register_template_string("coding_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register coding-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
        });

        handlebars
            .render("coding_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render coding-guidelines.md: {e}"
                ))
            })
    }

    fn generate_github_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/github-guidelines.md.hbs")?;

        handlebars
            .register_template_string("github_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register github-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
        });

        handlebars
            .render("github_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render github-guidelines.md: {e}"
                ))
            })
    }


    fn generate_hook_scripts(code_run: &CodeRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "code_hooks_";

        debug!(
            "Scanning for code hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for code
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                                match std::fs::read_to_string(&path) {
                                    Ok(template_content) => {
                                        debug!(
                                            "Loaded code hook template: {} (from {})",
                                            hook_name, filename
                                        );

                                        let mut handlebars = Handlebars::new();
                                        handlebars.set_strict_mode(false);

                                        if let Err(e) = handlebars
                                            .register_template_string("hook", template_content)
                                        {
                                            debug!(
                                                "Failed to register hook template {}: {}",
                                                hook_name, e
                                            );
                                            continue;
                                        }

                                        let context = json!({
                                            "task_id": code_run.spec.task_id,
                                            "service": code_run.spec.service,
                                            "repository_url": code_run.spec.repository_url,
                                            "docs_repository_url": code_run.spec.docs_repository_url,
                                            "working_directory": Self::get_working_directory(code_run),
                                            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
                                        });

                                        match handlebars.render("hook", &context) {
                                            Ok(rendered_script) => {
                                                // Remove .hbs extension for the final filename
                                                let script_name = hook_name
                                                    .strip_suffix(".hbs")
                                                    .unwrap_or(hook_name);
                                                hook_scripts.insert(
                                                    script_name.to_string(),
                                                    rendered_script,
                                                );
                                            }
                                            Err(e) => {
                                                debug!(
                                                    "Failed to render code hook script {}: {}",
                                                    hook_name, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!(
                                            "Failed to load code hook template {}: {}",
                                            filename, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        Ok(hook_scripts)
    }

    /// Get working directory (defaults to service name if not specified)
    fn get_working_directory(code_run: &CodeRun) -> &str {
        match &code_run.spec.working_directory {
            Some(wd) if !wd.is_empty() => wd,
            _ => &code_run.spec.service,
        }
    }

    /// Get continue session flag - true for retries or user-requested continuation
    fn get_continue_session(code_run: &CodeRun) -> bool {
        // Continue if it's a retry attempt OR user explicitly requested it
        let retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        retry_count > 0 || code_run.spec.continue_session
    }

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading code template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load code template {relative_path} (key: {configmap_key}): {e}"
            ))
        })
    }
}

```

### core/src/tasks/docs/controller.rs

```rust
use super::resources::DocsResourceManager;
use crate::tasks::types::{Context, Result, DOCS_FINALIZER_NAME};
use crate::crds::DocsRun;
use k8s_openapi::api::{batch::v1::Job, core::v1::ConfigMap};
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event as FinalizerEvent};
use kube::{Api, ResourceExt};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, instrument};

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
pub async fn reconcile_docs_run(docs_run: Arc<DocsRun>, ctx: Arc<Context>) -> Result<Action> {
    info!("Starting reconcile for DocsRun: {}", docs_run.name_any());

    let namespace = &ctx.namespace;
    let client = &ctx.client;
    let name = docs_run.name_any();

    debug!("Reconciling DocsRun: {}", name);

    // Create APIs
    debug!("Creating Kubernetes API clients...");
    let docsruns: Api<DocsRun> = Api::namespaced(client.clone(), namespace);
    debug!("API clients created successfully");

    // Handle finalizers for cleanup
    let result = finalizer(
        &docsruns,
        DOCS_FINALIZER_NAME,
        docs_run.clone(),
        |event| async {
            match event {
                FinalizerEvent::Apply(dr) => reconcile_docs_create_or_update(dr, &ctx).await,
                FinalizerEvent::Cleanup(dr) => cleanup_docs_resources(dr, &ctx).await,
            }
        },
    )
    .await
    .map_err(|e| match e {
        kube::runtime::finalizer::Error::ApplyFailed(err) => err,
        kube::runtime::finalizer::Error::CleanupFailed(err) => err,
        kube::runtime::finalizer::Error::AddFinalizer(e) => crate::tasks::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::RemoveFinalizer(e) => crate::tasks::types::Error::KubeError(e),
        kube::runtime::finalizer::Error::UnnamedObject => crate::tasks::types::Error::MissingObjectKey,
        kube::runtime::finalizer::Error::InvalidFinalizer => {
            crate::tasks::types::Error::ConfigError("Invalid finalizer name".to_string())
        }
    })?;

    debug!("Reconcile completed with result: {:?}", result);

    Ok(result)
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn reconcile_docs_create_or_update(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    let docs_run_name = docs_run.name_any();
    info!(
        "Starting status-first idempotent reconcile for DocsRun: {}",
        docs_run_name
    );

    // STEP 1: Check DocsRun status first (status-first idempotency)
    if let Some(status) = &docs_run.status {
        // Check for completion based on work_completed field (TTL-safe)
        if status.work_completed == Some(true) {
            info!("Work already completed (work_completed=true), no further action needed");
            return Ok(Action::await_change());
        }

        // Check legacy completion states
        match status.phase.as_str() {
            "Succeeded" => {
                info!("Already succeeded, ensuring work_completed is set");
                update_docs_status_with_completion(
                    &docs_run,
                    ctx,
                    "Succeeded",
                    "Documentation generation completed successfully",
                    true,
                )
                .await?;
                return Ok(Action::await_change());
            }
            "Failed" => {
                info!("Already failed, no retry logic");
                return Ok(Action::await_change());
            }
            "Running" => {
                debug!("Status shows running, checking actual job state");
                // Continue to job state check below
            }
            _ => {
                debug!("Status is '{}', proceeding with job creation", status.phase);
                // Continue to job creation below
            }
        }
    } else {
        debug!("No status found, initializing");
    }

    // STEP 2: Check job state for running jobs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let job_name = generate_job_name(&docs_run);
    debug!("Generated job name: {}", job_name);

    let job_state = check_job_state(&jobs, &job_name).await?;
    debug!("Current job state: {:?}", job_state);

    match job_state {
        JobState::NotFound => {
            debug!("No existing job found, using optimistic job creation");

            // STEP 3: Optimistic job creation with conflict handling
            let ctx_arc = Arc::new(ctx.clone());
            let resource_manager =
                DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);

            // This handles 409 conflicts gracefully
            resource_manager
                .reconcile_create_or_update(&docs_run)
                .await?;

            // Update status to Running
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Running",
                "Documentation generation started",
                false,
            )
            .await?;

            // Requeue to check job progress
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        JobState::Running => {
            debug!("Job is still running, monitoring progress");

            // Update status to Running if needed
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Running",
                "Documentation generation in progress",
                false,
            )
            .await?;

            // Continue monitoring
            Ok(Action::requeue(std::time::Duration::from_secs(30)))
        }

        JobState::Completed => {
            info!("Job completed successfully - marking work as complete");

            // Mark work as completed (TTL-safe)
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Succeeded",
                "Documentation generation completed successfully",
                true,
            )
            .await?;

            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }

        JobState::Failed => {
            info!("Job failed - final state reached");

            // Update to failed status (work_completed remains false for potential retry)
            update_docs_status_with_completion(
                &docs_run,
                ctx,
                "Failed",
                "Documentation generation failed",
                false,
            )
            .await?;

            // CRITICAL: Use await_change() to stop reconciliation
            Ok(Action::await_change())
        }
    }
}

#[instrument(skip(ctx), fields(docs_run_name = %docs_run.name_any(), namespace = %ctx.namespace))]
async fn cleanup_docs_resources(docs_run: Arc<DocsRun>, ctx: &Context) -> Result<Action> {
    debug!("Cleaning up resources for DocsRun");

    // Create APIs
    let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);
    let configmaps: Api<ConfigMap> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    // Create resource manager and delegate
    let ctx_arc = Arc::new(ctx.clone());
    let resource_manager = DocsResourceManager::new(&jobs, &configmaps, &ctx.config, &ctx_arc);
    resource_manager.cleanup_resources(&docs_run).await
}

// Helper functions for idempotent reconciliation

#[derive(Debug, Clone)]
pub enum JobState {
    NotFound,
    Running,
    Completed,
    Failed,
}

fn generate_job_name(docs_run: &DocsRun) -> String {
    let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
    let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
    let uid_suffix = docs_run
        .metadata
        .uid
        .as_deref()
        .map(|uid| &uid[..8])
        .unwrap_or("nouid");

    format!("docs-{namespace}-{name}-{uid_suffix}")
        .replace(['_', '.'], "-")
        .to_lowercase()
}

async fn check_job_state(jobs: &Api<Job>, job_name: &str) -> Result<JobState> {
    match jobs.get(job_name).await {
        Ok(job) => {
            if let Some(status) = &job.status {
                Ok(determine_job_state(status))
            } else {
                Ok(JobState::Running) // Job exists but no status yet
            }
        }
        Err(kube::Error::Api(response)) if response.code == 404 => Ok(JobState::NotFound),
        Err(e) => Err(e.into()),
    }
}

fn determine_job_state(status: &k8s_openapi::api::batch::v1::JobStatus) -> JobState {
    // Check completion conditions first
    if let Some(conditions) = &status.conditions {
        for condition in conditions {
            if condition.type_ == "Complete" && condition.status == "True" {
                return JobState::Completed;
            }
            if condition.type_ == "Failed" && condition.status == "True" {
                return JobState::Failed;
            }
        }
    }

    // Check legacy status fields
    if let Some(succeeded) = status.succeeded {
        if succeeded > 0 {
            return JobState::Completed;
        }
    }

    if let Some(failed) = status.failed {
        if failed > 0 {
            return JobState::Failed;
        }
    }

    JobState::Running
}

async fn update_docs_status_with_completion(
    docs_run: &DocsRun,
    ctx: &Context,
    new_phase: &str,
    new_message: &str,
    work_completed: bool,
) -> Result<()> {
    // Only update if status actually changed
    let current_phase = docs_run
        .status
        .as_ref()
        .map(|s| s.phase.as_str())
        .unwrap_or("");
    let current_work_completed = docs_run
        .status
        .as_ref()
        .and_then(|s| s.work_completed)
        .unwrap_or(false);

    if current_phase == new_phase && current_work_completed == work_completed {
        debug!(
            "Status already '{}' with work_completed={}, skipping update to prevent reconciliation",
            new_phase, work_completed
        );
        return Ok(());
    }

    debug!(
        "Updating status from '{}' (work_completed={}) to '{}' (work_completed={})",
        current_phase, current_work_completed, new_phase, work_completed
    );

    let docsruns: Api<DocsRun> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

    let status_patch = json!({
        "status": {
            "phase": new_phase,
            "message": new_message,
            "lastUpdate": chrono::Utc::now().to_rfc3339(),
            "workCompleted": work_completed,
        }
    });

    // Use status subresource to avoid triggering spec reconciliation
    docsruns
        .patch_status(
            &docs_run.name_any(),
            &PatchParams::default(),
            &Patch::Merge(&status_patch),
        )
        .await?;

    debug!(
        "Status updated successfully to '{}' with work_completed={}",
        new_phase, work_completed
    );
    Ok(())
}

```

### core/src/tasks/docs/mod.rs

```rust
pub mod controller;
pub mod resources;
pub mod status;
pub mod templates;

pub use controller::*;
```

### core/src/tasks/docs/status.rs

```rust
use crate::tasks::types::{Context, Result};
use crate::crds::{DocsRun, DocsRunCondition};
use k8s_openapi::api::batch::v1::Job;
use kube::api::{Api, Patch, PatchParams};
use kube::ResourceExt;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct DocsStatusManager;

#[allow(dead_code)]
impl DocsStatusManager {
    /// Monitor Job status and update DocsRun CRD accordingly
    pub async fn monitor_job_status(
        docs_run: &Arc<DocsRun>,
        jobs: &Api<Job>,
        ctx: &Arc<Context>,
    ) -> Result<()> {
        error!(
            "🔍 STATUS_MANAGER: Starting monitor_job_status for DocsRun: {}",
            docs_run.name_any()
        );
        let job_name = Self::get_current_job_name(docs_run);

        if let Some(job_name) = job_name {
            error!("✅ STATUS_MANAGER: Found job_name to monitor: {}", job_name);
            // Get the current job
            match jobs.get(&job_name).await {
                Ok(job) => {
                    let (phase, message) = Self::analyze_job_status(&job);
                    Self::update_status(docs_run, ctx, &phase, &message).await?;

                    // Schedule cleanup if job is complete and cleanup is enabled
                    if ctx.config.cleanup.enabled && (phase == "Succeeded" || phase == "Failed") {
                        Self::schedule_job_cleanup(docs_run, ctx, &job_name, &phase).await?;
                    }
                }
                Err(kube::Error::Api(ae)) if ae.code == 404 => {
                    warn!(
                        "Job {} not found for DocsRun {}",
                        job_name,
                        docs_run.name_any()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to get job {} for DocsRun {}: {}",
                        job_name,
                        docs_run.name_any(),
                        e
                    );
                }
            }
        } else {
            error!("❌ STATUS_MANAGER: No job_name found in DocsRun status - cannot monitor job!");
            error!(
                "❌ STATUS_MANAGER: This means the initial status update failed or was overwritten"
            );
        }

        Ok(())
    }

    /// Update the status when a job starts
    pub async fn update_job_started(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        _cm_name: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = docs_run.name_any();

        let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "phase": "Running",
                "message": "Documentation generation job started",
                "lastUpdate": chrono::Utc::now().to_rfc3339(),
                "jobName": job_name,
                "conditions": Self::build_conditions("Running", "Documentation generation job started", &chrono::Utc::now().to_rfc3339())
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        error!(
            "🔄 STATUS_MANAGER: Attempting to update DocsRun status with job_name: {}",
            job_name
        );
        error!(
            "🔄 STATUS_MANAGER: Status patch: {}",
            serde_json::to_string_pretty(&status_patch)
                .unwrap_or_else(|e| format!("Failed to serialize patch: {e}"))
        );

        match docs_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_docs_run) => {
                error!(
                    "✅ STATUS_MANAGER: Successfully updated DocsRun status: {} -> Running",
                    name
                );
                error!(
                    "✅ STATUS_MANAGER: Updated resource version: {:?}",
                    updated_docs_run.metadata.resource_version
                );
                error!(
                    "✅ STATUS_MANAGER: Updated job_name in status: {:?}",
                    updated_docs_run
                        .status
                        .as_ref()
                        .and_then(|s| s.job_name.as_ref())
                );
            }
            Err(e) => {
                error!(
                    "❌ STATUS_MANAGER: Failed to update DocsRun status for {}: {}",
                    name, e
                );
                error!(
                    "❌ STATUS_MANAGER: Error type: {}",
                    std::any::type_name_of_val(&e)
                );
                error!("❌ STATUS_MANAGER: Full error details: {:?}", e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Update the DocsRun CRD status
    async fn update_status(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        phase: &str,
        message: &str,
    ) -> Result<()> {
        let namespace = &ctx.namespace;
        let client = &ctx.client;
        let name = docs_run.name_any();

        let current_time = chrono::Utc::now().to_rfc3339();
        let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), namespace);

        let status_patch = json!({
            "status": {
                "phase": phase,
                "message": message,
                "lastUpdate": current_time,
                "conditions": Self::build_conditions(phase, message, &current_time)
            }
        });

        let patch = Patch::Merge(&status_patch);
        let pp = PatchParams::default();

        match docs_api.patch_status(&name, &pp, &patch).await {
            Ok(updated_docs_run) => {
                info!(
                    "✅ Successfully updated DocsRun status: {} -> {}",
                    name, phase
                );
                info!(
                    "✅ Updated resource version: {:?}",
                    updated_docs_run.metadata.resource_version
                );
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to update DocsRun status for {}: {}", name, e);
                error!("❌ Error type: {}", std::any::type_name_of_val(&e));
                error!("❌ Full error details: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// Get the current job name for a docs task
    fn get_current_job_name(docs_run: &DocsRun) -> Option<String> {
        let job_name = docs_run.status.as_ref().and_then(|s| s.job_name.clone());
        error!(
            "🔍 STATUS_MANAGER: get_current_job_name for {}: {:?}",
            docs_run.name_any(),
            job_name
        );
        error!("🔍 STATUS_MANAGER: DocsRun status: {:?}", docs_run.status);
        job_name
    }

    /// Analyze job status and return (phase, message)
    fn analyze_job_status(job: &Job) -> (String, String) {
        let job_name = job.metadata.name.as_deref().unwrap_or("unknown");
        error!(
            "🔍 STATUS_MANAGER: analyze_job_status for job: {}",
            job_name
        );

        if let Some(status) = &job.status {
            error!("📊 STATUS_MANAGER: Job status - active: {:?}, succeeded: {:?}, failed: {:?}, completion_time: {:?}",
                status.active, status.succeeded, status.failed, status.completion_time);
            // Check completion time first
            if status.completion_time.is_some() {
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        error!(
                            "🏷️ STATUS_MANAGER: Job condition - type: {}, status: {}",
                            condition.type_, condition.status
                        );
                        if condition.type_ == "Complete" && condition.status == "True" {
                            error!("🎉 STATUS_MANAGER: Job COMPLETED successfully! Setting phase to Succeeded");
                            return (
                                "Succeeded".to_string(),
                                "Documentation generation completed successfully".to_string(),
                            );
                        } else if condition.type_ == "Failed" && condition.status == "True" {
                            let message = condition
                                .message
                                .as_deref()
                                .unwrap_or("Documentation generation failed");
                            error!(
                                "💥 STATUS_MANAGER: Job FAILED! Setting phase to Failed: {}",
                                message
                            );
                            return ("Failed".to_string(), message.to_string());
                        }
                    }
                }
            }

            // Check if job is running
            if let Some(active) = status.active {
                if active > 0 {
                    return (
                        "Running".to_string(),
                        "Documentation generation is running".to_string(),
                    );
                }
            }

            // Check for failure conditions
            if let Some(failed) = status.failed {
                if failed > 0 {
                    return (
                        "Failed".to_string(),
                        "Documentation generation failed".to_string(),
                    );
                }
            }
        }

        (
            "Pending".to_string(),
            "Documentation generation job pending".to_string(),
        )
    }

    /// Build DocsRun conditions
    fn build_conditions(phase: &str, message: &str, timestamp: &str) -> Vec<DocsRunCondition> {
        vec![DocsRunCondition {
            condition_type: phase.to_string(),
            status: "True".to_string(),
            last_transition_time: Some(timestamp.to_string()),
            reason: Some(match phase {
                "Running" => "JobStarted".to_string(),
                "Succeeded" => "JobCompleted".to_string(),
                "Failed" => "JobFailed".to_string(),
                _ => "Unknown".to_string(),
            }),
            message: Some(message.to_string()),
        }]
    }

    /// Schedule cleanup of completed job
    async fn schedule_job_cleanup(
        docs_run: &Arc<DocsRun>,
        ctx: &Arc<Context>,
        job_name: &str,
        phase: &str,
    ) -> Result<()> {
        info!(
            "Scheduling cleanup for DocsRun {} job {} (phase: {})",
            docs_run.name_any(),
            job_name,
            phase
        );

        // For docs jobs, we can clean up immediately since they don't need session persistence
        let jobs: Api<Job> = Api::namespaced(ctx.client.clone(), &ctx.namespace);

        if let Err(e) = jobs
            .delete(job_name, &kube::api::DeleteParams::default())
            .await
        {
            warn!("Failed to delete completed docs job {}: {}", job_name, e);
        } else {
            info!("Successfully deleted completed docs job: {}", job_name);
        }

        Ok(())
    }
}

```

### core/src/tasks/docs/resources.rs

```rust
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::{github_app_secret_name, ssh_secret_name, Context, Result};
use crate::crds::DocsRun;
use k8s_openapi::api::{batch::v1::Job, core::v1::ConfigMap};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::ResourceExt;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, info};

pub struct DocsResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> DocsResourceManager<'a> {
    pub fn new(
        jobs: &'a Api<Job>,
        configmaps: &'a Api<ConfigMap>,
        config: &'a Arc<ControllerConfig>,
        ctx: &'a Arc<Context>,
    ) -> Self {
        Self {
            jobs,
            configmaps,
            config,
            ctx,
        }
    }

    pub async fn reconcile_create_or_update(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!(
            "🚀 RESOURCE_MANAGER: Starting reconcile_create_or_update for: {}",
            name
        );

        // Don't cleanup resources at start - let idempotent creation handle it
        info!("🔄 RESOURCE_MANAGER: Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(docs_run);
        info!("📝 RESOURCE_MANAGER: Generated ConfigMap name: {}", cm_name);

        info!("🏗️ RESOURCE_MANAGER: Creating ConfigMap object");
        let configmap = match self.create_configmap(docs_run, &cm_name, None) {
            Ok(cm) => {
                info!("✅ RESOURCE_MANAGER: ConfigMap object created successfully");
                cm
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Failed to create ConfigMap object: {:?}",
                    e
                );
                error!(
                    "❌ RESOURCE_MANAGER: Error type: {}",
                    std::any::type_name_of_val(&e)
                );
                return Err(e);
            }
        };

        // Always create or update ConfigMap to ensure latest template content
        info!(
            "🔄 RESOURCE_MANAGER: Attempting to create ConfigMap: {}",
            cm_name
        );
        error!(
            "📝 RESOURCE_MANAGER: Attempting to create ConfigMap: {}",
            cm_name
        );
        match self
            .configmaps
            .create(&PostParams::default(), &configmap)
            .await
        {
            Ok(_) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully created ConfigMap: {}",
                    cm_name
                );
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // ConfigMap exists, update it with latest content
                error!("🔄 RESOURCE_MANAGER: ConfigMap {} already exists (409), attempting to update with latest content", cm_name);

                // First get the existing ConfigMap to preserve resourceVersion
                match self.configmaps.get(&cm_name).await {
                    Ok(existing_cm) => {
                        let mut updated_configmap = configmap;
                        updated_configmap.metadata.resource_version =
                            existing_cm.metadata.resource_version;

                        match self
                            .configmaps
                            .replace(&cm_name, &PostParams::default(), &updated_configmap)
                            .await
                        {
                            Ok(_) => {
                                error!("✅ RESOURCE_MANAGER: Successfully updated existing ConfigMap: {}", cm_name);
                            }
                            Err(e) => {
                                error!("❌ RESOURCE_MANAGER: Failed to replace existing ConfigMap {}: {:?}", cm_name, e);
                                error!(
                                    "❌ RESOURCE_MANAGER: Replace error type: {}",
                                    std::any::type_name_of_val(&e)
                                );

                                // Fall back to creating a new one with a different name
                                error!("🔄 RESOURCE_MANAGER: Replace failed, falling back to create-only approach");
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ RESOURCE_MANAGER: Failed to get existing ConfigMap {} for update: {:?}", cm_name, e);
                        error!(
                            "🔄 RESOURCE_MANAGER: Get failed, falling back to create-only approach"
                        );
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Failed to create ConfigMap {}: {:?}",
                    cm_name, e
                );
                error!(
                    "❌ RESOURCE_MANAGER: Kubernetes error type: {}",
                    std::any::type_name_of_val(&e)
                );
                return Err(e.into());
            }
        }

        // Create Job using idempotent creation (now it can successfully mount the existing ConfigMap)
        let job_ref = self.create_or_get_job(docs_run, &cm_name).await?;

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            self.update_configmap_owner(docs_run, &cm_name, owner_ref)
                .await?;
        }

        Ok(Action::await_change())
    }

    pub async fn cleanup_resources(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!("Cleaning up docs resources for: {}", name);

        // Clean up any remaining jobs and configmaps
        self.cleanup_old_jobs(docs_run).await?;
        self.cleanup_old_configmaps(docs_run).await?;

        Ok(Action::await_change())
    }

    fn generate_configmap_name(&self, docs_run: &DocsRun) -> String {
        // Generate unique ConfigMap name per DocsRun to prevent conflicts between sequential jobs
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
        let context_version = 1; // Docs don't have context versions, always 1
        
        format!("docs-{namespace}-{name}-{uid_suffix}-v{context_version}-files")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn create_configmap(
        &self,
        docs_run: &DocsRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for docs
        error!(
            "🔧 RESOURCE_MANAGER: Generating templates for ConfigMap: {}",
            name
        );
        let templates = match super::templates::DocsTemplateGenerator::generate_all_templates(
            docs_run,
            self.config,
        ) {
            Ok(tmpl) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully generated {} templates",
                    tmpl.len()
                );
                for filename in tmpl.keys() {
                    error!("📄 RESOURCE_MANAGER: Generated template file: {}", filename);
                }
                tmpl
            }
            Err(e) => {
                error!("❌ RESOURCE_MANAGER: Failed to generate templates: {:?}", e);
                error!(
                    "❌ RESOURCE_MANAGER: Template error type: {}",
                    std::any::type_name_of_val(&e)
                );
                error!("❌ RESOURCE_MANAGER: Template error details: {}", e);
                return Err(e);
            }
        };

        for (filename, content) in templates {
            data.insert(filename, content);
        }

        error!(
            "🏷️ RESOURCE_MANAGER: Creating labels for ConfigMap: {}",
            name
        );
        let labels = self.create_task_labels(docs_run);
        error!("✅ RESOURCE_MANAGER: Created {} labels", labels.len());

        error!("📝 RESOURCE_MANAGER: Building ConfigMap metadata");
        let mut metadata = ObjectMeta {
            name: Some(name.to_string()),
            labels: Some(labels),
            ..Default::default()
        };

        if let Some(owner) = owner_ref {
            error!("👤 RESOURCE_MANAGER: Adding owner reference to ConfigMap");
            metadata.owner_references = Some(vec![owner]);
        }

        error!(
            "🏗️ RESOURCE_MANAGER: Constructing final ConfigMap object with {} data entries",
            data.len()
        );
        let configmap = ConfigMap {
            metadata,
            data: Some(data),
            ..Default::default()
        };

        error!("✅ RESOURCE_MANAGER: ConfigMap object created successfully");
        Ok(configmap)
    }

    /// Optimistic job creation: create job directly, handle conflicts gracefully
    async fn create_or_get_job(
        &self,
        docs_run: &DocsRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(docs_run);

        // OPTIMISTIC APPROACH: Try to create job directly first
        error!(
            "🎯 RESOURCE_MANAGER: Using optimistic job creation for: {}",
            job_name
        );
        match self.create_job(docs_run, cm_name).await {
            Ok(owner_ref) => {
                error!(
                    "✅ RESOURCE_MANAGER: Successfully created new job: {}",
                    job_name
                );
                Ok(owner_ref)
            }
            Err(crate::tasks::types::Error::KubeError(kube::Error::Api(ae))) if ae.code == 409 => {
                // Job already exists due to race condition, get the existing one
                error!("🔄 RESOURCE_MANAGER: Job {} already exists (409 conflict), getting existing job", job_name);
                match self.jobs.get(&job_name).await {
                    Ok(existing_job) => {
                        error!("✅ RESOURCE_MANAGER: Retrieved existing job: {}", job_name);
                        Ok(Some(OwnerReference {
                            api_version: "batch/v1".to_string(),
                            kind: "Job".to_string(),
                            name: job_name,
                            uid: existing_job.metadata.uid.unwrap_or_default(),
                            controller: Some(false),
                            block_owner_deletion: Some(true),
                        }))
                    }
                    Err(e) => {
                        error!("❌ RESOURCE_MANAGER: Failed to get existing job after 409 conflict: {:?}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                error!(
                    "❌ RESOURCE_MANAGER: Job creation failed with non-conflict error: {:?}",
                    e
                );
                Err(e)
            }
        }
    }

    async fn create_job(
        &self,
        docs_run: &DocsRun,
        cm_name: &str,
    ) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(docs_run);
        let job = self.build_job_spec(docs_run, &job_name, cm_name)?;

        let created_job = self.jobs.create(&PostParams::default(), &job).await?;

        error!("✅ RESOURCE_MANAGER: Created docs job: {}", job_name);

        // Update status using legacy status manager if needed
                        if let Err(e) = super::status::DocsStatusManager::update_job_started(
            &Arc::new(docs_run.clone()),
            self.ctx,
            &job_name,
            cm_name,
        )
        .await
        {
            error!(
                "⚠️ RESOURCE_MANAGER: Failed to update job started status: {:?}",
                e
            );
            // Continue anyway, status will be updated by main controller
        }

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
            error!("⚠️ RESOURCE_MANAGER: Created job missing UID or name metadata");
            Ok(None)
        }
    }

    fn generate_job_name(&self, docs_run: &DocsRun) -> String {
        // Use deterministic naming based on the DocsRun's actual name and UID
        // This ensures the same DocsRun always generates the same Job name
        let namespace = docs_run.metadata.namespace.as_deref().unwrap_or("default");
        let name = docs_run.metadata.name.as_deref().unwrap_or("unknown");
        let uid_suffix = docs_run
            .metadata
            .uid
            .as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");

        format!("docs-{namespace}-{name}-{uid_suffix}")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn build_job_spec(&self, docs_run: &DocsRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(docs_run);

        // Create owner reference to DocsRun for proper event handling
        let owner_ref = OwnerReference {
            api_version: "agents.platform/v1".to_string(),
            kind: "DocsRun".to_string(),
            name: docs_run.name_any(),
            uid: docs_run.metadata.uid.clone().unwrap_or_default(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        };

        // Build volumes for docs (emptyDir, no PVCs)
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
            "mountPath": "/task-files"
        }));

        // Mount settings.json as managed-settings.json for enterprise compatibility
        volume_mounts.push(json!({
            "name": "task-files",
            "mountPath": "/etc/claude-code/managed-settings.json",
            "subPath": "settings.json"
        }));

        // EmptyDir workspace volume for docs (no persistence needed)
        volumes.push(json!({
            "name": "workspace",
            "emptyDir": {}
        }));
        volume_mounts.push(json!({
            "name": "workspace",
            "mountPath": "/workspace"
        }));

        // SSH volumes
        let ssh_volumes = self.generate_ssh_volumes(docs_run);
        volumes.extend(ssh_volumes.volumes);
        volume_mounts.extend(ssh_volumes.volume_mounts);

        let image = format!(
            "{}:{}",
            self.config.agent.image.repository, self.config.agent.image.tag
        );
        let job_spec = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_name,
                "labels": labels,
                "ownerReferences": [{
                    "apiVersion": owner_ref.api_version,
                    "kind": owner_ref.kind,
                    "name": owner_ref.name,
                    "uid": owner_ref.uid,
                    "controller": owner_ref.controller,
                    "blockOwnerDeletion": owner_ref.block_owner_deletion
                }]
            },
            "spec": {
                "backoffLimit": 0,
                "ttlSecondsAfterFinished": 30,
                "template": {
                    "metadata": {
                        "labels": labels
                    },
                    "spec": {
                        "restartPolicy": "Never",
                        "containers": [{
                            "name": "claude-docs",
                            "image": image,
                            "env": [
                                {
                                    "name": "GITHUB_APP_PRIVATE_KEY",
                                    "valueFrom": {
                                        "secretKeyRef": {
                                            "name": github_app_secret_name(docs_run.spec.github_app.as_deref()
                                                .or(docs_run.spec.github_user.as_deref())
                                                .unwrap_or("")),
                                            "key": "private-key"
                                        }
                                    }
                                },
                                {
                                    "name": "GITHUB_APP_ID",
                                    "valueFrom": {
                                        "secretKeyRef": {
                                            "name": github_app_secret_name(docs_run.spec.github_app.as_deref()
                                                .or(docs_run.spec.github_user.as_deref())
                                                .unwrap_or("")),
                                            "key": "app-id"
                                        }
                                    }
                                },
                                {
                                    "name": "ANTHROPIC_API_KEY",
                                    "valueFrom": {
                                        "secretKeyRef": {
                                            "name": self.config.secrets.api_key_secret_name,
                                            "key": self.config.secrets.api_key_secret_key
                                        }
                                    }
                                }
                            ],
                            "command": ["/bin/bash"],
                            "args": ["/task-files/container.sh"],
                            "workingDir": "/workspace",
                            "volumeMounts": volume_mounts
                        }],
                        "volumes": volumes
                    }
                }
            }
        });

        Ok(serde_json::from_value(job_spec)?)
    }

    fn create_task_labels(&self, docs_run: &DocsRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        labels.insert("app".to_string(), "orchestrator".to_string());
        labels.insert("component".to_string(), "docs-generator".to_string());
        // Use github_app if available, fallback to github_user for backward compatibility
        let github_identity = docs_run.spec.github_app.as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        labels.insert(
            "github-identity".to_string(),
            self.sanitize_label_value(github_identity),
        );
        labels.insert("context-version".to_string(), "1".to_string()); // Docs always version 1

        // Docs-specific labels
        labels.insert("task-type".to_string(), "docs".to_string());
        labels.insert(
            "repository".to_string(),
            self.sanitize_label_value(&docs_run.spec.repository_url),
        );

        labels
    }

    fn generate_ssh_volumes(&self, docs_run: &DocsRun) -> SshVolumes {
        // Only mount SSH keys when using github_user authentication (not GitHub Apps)
        if docs_run.spec.github_app.is_some() || docs_run.spec.github_user.is_none() {
            // GitHub App authentication doesn't need SSH keys
            return SshVolumes {
                volumes: vec![],
                volume_mounts: vec![],
            };
        }

        let ssh_secret = ssh_secret_name(docs_run.spec.github_user.as_deref().unwrap_or(""));

        let volumes = vec![json!({
            "name": "ssh-key",
            "secret": {
                "secretName": ssh_secret,
                "defaultMode": 0o644,
                "items": [{
                    "key": "ssh-privatekey",
                    "path": "id_ed25519"
                }]
            }
        })];

        let volume_mounts = vec![json!({
            "name": "ssh-key",
            "mountPath": "/workspace/.ssh",
            "readOnly": true
        })];

        SshVolumes {
            volumes,
            volume_mounts,
        }
    }

    async fn update_configmap_owner(
        &self,
        _docs_run: &DocsRun,
        cm_name: &str,
        owner_ref: OwnerReference,
    ) -> Result<()> {
        let mut existing_cm = self.configmaps.get(cm_name).await?;

        // Add owner reference
        let owner_refs = existing_cm
            .metadata
            .owner_references
            .get_or_insert_with(Vec::new);
        owner_refs.push(owner_ref);

        // Update the ConfigMap
        self.configmaps
            .replace(cm_name, &PostParams::default(), &existing_cm)
            .await?;
        info!("Updated ConfigMap {} with owner reference", cm_name);

        Ok(())
    }

    // Legacy cleanup method for backward compatibility
    async fn cleanup_old_jobs(&self, docs_run: &DocsRun) -> Result<()> {
        let github_identity = docs_run.spec.github_app.as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-identity={}",
            self.sanitize_label_value(github_identity)
        ));

        let jobs = self.jobs.list(&list_params).await?;

        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old docs job: {}", job_name);
                let _ = self.jobs.delete(&job_name, &DeleteParams::default()).await;
            }
        }

        Ok(())
    }

    async fn cleanup_old_configmaps(&self, docs_run: &DocsRun) -> Result<()> {
        // Generate current ConfigMap name to avoid deleting it
        let current_cm_name = self.generate_configmap_name(docs_run);
        
        let github_identity = docs_run.spec.github_app.as_deref()
            .or(docs_run.spec.github_user.as_deref())
            .unwrap_or("");
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-identity={}",
            self.sanitize_label_value(github_identity)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;

        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                // Skip deleting the current ConfigMap - this prevents deletion of active job's ConfigMap
                if cm_name == current_cm_name {
                    info!("Skipping deletion of current ConfigMap: {}", cm_name);
                    continue;
                }
                
                // Check if ConfigMap has an owner reference to a Job that's still running
                let has_active_job = cm.metadata.owner_references
                    .as_ref()
                    .map(|owners| {
                        owners.iter().any(|owner| {
                            owner.kind == "Job" && owner.api_version.starts_with("batch/")
                        })
                    })
                    .unwrap_or(false);
                
                if has_active_job {
                    // If ConfigMap is owned by a Job, let Kubernetes handle cleanup when Job completes
                    info!("Skipping cleanup of ConfigMap with active Job owner: {}", cm_name);
                    continue;
                }
                
                info!("Deleting old docs ConfigMap: {}", cm_name);
                let _ = self
                    .configmaps
                    .delete(&cm_name, &DeleteParams::default())
                    .await;
            }
        }

        Ok(())
    }

    fn sanitize_label_value(&self, input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }

        // Replace spaces with hyphens, convert to lowercase
        let mut sanitized = input.to_lowercase().replace([' ', '_'], "-");

        // Remove any characters that aren't alphanumeric, hyphens, underscores, or dots
        sanitized.retain(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        // Ensure it starts and ends with alphanumeric
        let chars: Vec<char> = sanitized.chars().collect();
        let start = chars.iter().position(|c| c.is_alphanumeric()).unwrap_or(0);
        let end = chars
            .iter()
            .rposition(|c| c.is_alphanumeric())
            .unwrap_or(chars.len().saturating_sub(1));

        if start <= end {
            sanitized = chars[start..=end].iter().collect();
        }

        // Truncate to 63 characters (Kubernetes label limit)
        if sanitized.len() > 63 {
            sanitized.truncate(63);
            // Ensure it still ends with alphanumeric after truncation
            if let Some(last_alphanumeric) = sanitized.rfind(|c: char| c.is_alphanumeric()) {
                sanitized.truncate(last_alphanumeric + 1);
            }
        }

        sanitized
    }
}

struct SshVolumes {
    volumes: Vec<serde_json::Value>,
    volume_mounts: Vec<serde_json::Value>,
}

```

### core/src/tasks/docs/templates.rs

```rust
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::Result;
use crate::crds::DocsRun;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

pub struct DocsTemplateGenerator;

impl DocsTemplateGenerator {
    /// Generate all template files for a docs task
    pub fn generate_all_templates(
        docs_run: &DocsRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core docs templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(docs_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(docs_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(docs_run, config)?,
        );
        templates.insert(
            "prompt.md".to_string(),
            Self::generate_docs_prompt(docs_run)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(docs_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_container_script(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/container.sh.hbs")?;

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "github_app": docs_run.spec.github_app,
            "service_name": "docs-generator"
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/claude.md.hbs")?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "service_name": "docs-generator"
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render CLAUDE.md: {e}"
            ))
        })
    }

    fn generate_claude_settings(docs_run: &DocsRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/settings.json.hbs")?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        let context = json!({
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render settings.json: {e}"
            ))
        })
    }

    fn generate_docs_prompt(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/prompt.md.hbs")?;

        handlebars
            .register_template_string("docs_prompt", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register docs prompt template: {e}"
                ))
            })?;

        // Load toolman catalog for embedding in prompt
        let catalog_data = Self::load_toolman_catalog_data()?;
        let catalog_markdown = Self::render_toolman_catalog_markdown(&catalog_data)?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "service_name": "docs-generator",
            "toolman_catalog_markdown": catalog_markdown
        });

        handlebars.render("docs_prompt", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render docs prompt: {e}"
            ))
        })
    }

    // Removed generate_toolman_catalog - catalog is now embedded as markdown in prompt

    fn load_toolman_catalog_data() -> Result<serde_json::Value> {
        const TOOLMAN_CATALOG_PATH: &str = "/toolman-catalog/tool-catalog.json";
        
        match fs::read_to_string(TOOLMAN_CATALOG_PATH) {
            Ok(catalog_json) => {
                serde_json::from_str(&catalog_json).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to parse toolman catalog JSON: {e}"
                    ))
                })
            }
            Err(e) => {
                debug!("Toolman catalog not found at {}: {}", TOOLMAN_CATALOG_PATH, e);
                // Return empty catalog structure if toolman ConfigMap is not available
                Ok(json!({
                    "local": {},
                    "remote": {},
                    "last_updated": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                }))
            }
        }
    }

    fn count_total_tools(catalog_data: &serde_json::Value) -> u32 {
        let mut count = 0;
        
        if let Some(local) = catalog_data.get("local").and_then(|v| v.as_object()) {
            for server in local.values() {
                if let Some(tools) = server.get("tools").and_then(|v| v.as_array()) {
                    count += tools.len() as u32;
                }
            }
        }
        
        if let Some(remote) = catalog_data.get("remote").and_then(|v| v.as_object()) {
            for server in remote.values() {
                if let Some(tools) = server.get("tools").and_then(|v| v.as_array()) {
                    count += tools.len() as u32;
                }
            }
        }
        
        count
    }

    fn render_toolman_catalog_markdown(catalog_data: &serde_json::Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        
        // Register json helper for proper JSON serialization
        handlebars.register_helper(
            "json",
            Box::new(|h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
                let param = h.param(0).ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex("json", 0))?;
                let json_str = serde_json::to_string(param.value())
                    .map_err(|e| handlebars::RenderErrorReason::NestedError(Box::new(e)))?;
                out.write(&json_str)?;
                Ok(())
            }),
        );

        let template = Self::load_template("docs/toolman-catalog.md.hbs")?;

        handlebars
            .register_template_string("toolman_catalog_markdown", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register toolman catalog markdown template: {e}"
                ))
            })?;
        
        let context = json!({
            "toolman_catalog": catalog_data,
            "generated_timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "total_tool_count": Self::count_total_tools(catalog_data)
        });

        handlebars.render("toolman_catalog_markdown", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render toolman catalog markdown: {e}"
            ))
        })
    }

    fn generate_hook_scripts(docs_run: &DocsRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "docs_hooks_";

        debug!(
            "Scanning for docs hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for docs
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

                                match std::fs::read_to_string(&path) {
                                    Ok(template_content) => {
                                        debug!(
                                            "Loaded docs hook template: {} (from {})",
                                            hook_name, filename
                                        );

                                        let mut handlebars = Handlebars::new();
                                        handlebars.set_strict_mode(false);

                                        if let Err(e) = handlebars
                                            .register_template_string("hook", template_content)
                                        {
                                            debug!(
                                                "Failed to register hook template {}: {}",
                                                hook_name, e
                                            );
                                            continue;
                                        }

                                        let context = json!({
                                            "repository_url": docs_run.spec.repository_url,
                                            "source_branch": docs_run.spec.source_branch,
                                            "working_directory": docs_run.spec.working_directory,
                                            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
                                            "service_name": "docs-generator"
                                        });

                                        match handlebars.render("hook", &context) {
                                            Ok(rendered_script) => {
                                                // Remove .hbs extension for the final filename
                                                let script_name = hook_name
                                                    .strip_suffix(".hbs")
                                                    .unwrap_or(hook_name);
                                                hook_scripts.insert(
                                                    script_name.to_string(),
                                                    rendered_script,
                                                );
                                            }
                                            Err(e) => {
                                                debug!(
                                                    "Failed to render docs hook script {}: {}",
                                                    hook_name, e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!(
                                            "Failed to load docs hook template {}: {}",
                                            filename, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        Ok(hook_scripts)
    }

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading docs template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load docs template {relative_path} (key: {configmap_key}): {e}"
            ))
        })
    }
}

```

### core/src/tasks/mod.rs

```rust
use crate::crds::{CodeRun, DocsRun};
use futures::StreamExt;
use k8s_openapi::api::batch::v1::Job;
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher::Config;
use kube::{Api, Client, ResourceExt};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, Instrument};

pub mod code;
pub mod docs;
pub mod config;
pub mod types;

// Re-export commonly used items
pub use code::reconcile_code_run;
pub use config::ControllerConfig;
pub use docs::reconcile_docs_run;
pub use types::{Error, Result};

// Context is crate-internal only
use types::Context;

/// Main entry point for the separated task controllers
#[instrument(skip(client), fields(namespace = %namespace))]
pub async fn run_task_controller(client: Client, namespace: String) -> Result<()> {
    info!(
        "Starting separated task controllers in namespace: {}",
        namespace
    );

    debug!("Loading controller configuration from mounted file...");

    // Load controller configuration from mounted file
    let config = match ControllerConfig::from_mounted_file("/config/config.yaml") {
        Ok(cfg) => {
            debug!("Successfully loaded controller configuration");
            debug!("Configuration cleanup enabled = {}", cfg.cleanup.enabled);

            // Validate configuration has required fields
            if let Err(validation_error) = cfg.validate() {
                error!(
                    "❌ TASK_CONTROLLER DEBUG: Configuration validation failed: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            debug!("Configuration validation passed");
            cfg
        }
        Err(e) => {
            error!(
                "❌ TASK_CONTROLLER DEBUG: Failed to load configuration, using defaults: {}",
                e
            );
            debug!("Creating default configuration...");
            let default_config = ControllerConfig::default();

            // Validate default configuration
            if let Err(validation_error) = default_config.validate() {
                error!(
                    "❌ TASK_CONTROLLER DEBUG: Default configuration is invalid: {}",
                    validation_error
                );
                return Err(Error::ConfigError(validation_error.to_string()));
            }
            debug!("Default configuration validation passed");
            default_config
        }
    };

    debug!("Creating controller context...");

    // Create shared context
    let context = Arc::new(Context {
        client: client.clone(),
        namespace: namespace.clone(),
        config: Arc::new(config),
    });

    debug!("Controller context created successfully");

    // Run both controllers concurrently
    info!("Starting DocsRun and CodeRun controllers...");

    let docs_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move { run_docs_controller(client, namespace, context).await }
    });

    let code_controller_handle = tokio::spawn({
        let context = context.clone();
        let client = client.clone();
        let namespace = namespace.clone();
        async move { run_code_controller(client, namespace, context).await }
    });

    debug!("Both controllers started, waiting for completion...");

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

    info!("Task controller shutting down");
    Ok(())
}

/// Run the DocsRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_docs_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    info!("Starting DocsRun controller");

    let docs_api: Api<DocsRun> = Api::namespaced(client.clone(), &namespace);
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(docs_api, watcher_config.clone())
        .owns(jobs_api, watcher_config)
        .run(reconcile_docs_run, error_policy_docs, context)
        .for_each(|reconciliation_result| {
            let docs_span = tracing::info_span!("docs_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(docs_run_resource) => {
                        info!(
                            resource = ?docs_run_resource,
                            "DocsRun reconciliation successful"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "DocsRun reconciliation error"
                        );
                    }
                }
            }
            .instrument(docs_span)
        })
        .await;

    info!("DocsRun controller shutting down");
    Ok(())
}

/// Run the CodeRun controller
#[instrument(skip(client, context), fields(namespace = %namespace))]
async fn run_code_controller(
    client: Client,
    namespace: String,
    context: Arc<Context>,
) -> Result<()> {
    info!("Starting CodeRun controller");

    let code_api: Api<CodeRun> = Api::namespaced(client.clone(), &namespace);
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let watcher_config = Config::default().any_semantic();

    Controller::new(code_api, watcher_config.clone())
        .owns(jobs_api, watcher_config)
        .run(reconcile_code_run, error_policy_code, context)
        .for_each(|reconciliation_result| {
            let code_span = tracing::info_span!("code_reconciliation_result");
            async move {
                match reconciliation_result {
                    Ok(code_run_resource) => {
                        info!(
                            resource = ?code_run_resource,
                            "CodeRun reconciliation successful"
                        );
                    }
                    Err(reconciliation_err) => {
                        error!(
                            error = ?reconciliation_err,
                            "CodeRun reconciliation error"
                        );
                    }
                }
            }
            .instrument(code_span)
        })
        .await;

    info!("CodeRun controller shutting down");
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
```

### core/src/bin/test_templates.rs

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
        "repository_url": "https://github.com/5dlabs/cto",
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
        "docs/prompt.md.hbs",
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
        "repository_url": "https://github.com/5dlabs/cto",
        "platform_repository_url": "https://github.com/5dlabs/cto",
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

### core/src/bin/agent_controller.rs

```rust
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
use core::tasks::run_task_controller;
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

    info!("Starting 5D Labs Controller Service v{}", env!("CARGO_PKG_VERSION"));

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
        "uptime": "TODO: implement uptime tracking"
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
```

### core/src/lib.rs

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

pub mod tasks;
pub mod crds;

// Re-export commonly used types
pub use tasks::config::ControllerConfig;
pub use crds::{CodeRun, CodeRunSpec, CodeRunStatus, DocsRun, DocsRunSpec, DocsRunStatus};

```

### mcp/src/tools.rs

```rust
use serde_json::{json, Value};

/// Get tool schemas for MCP protocol with rich descriptions
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_task_schema(),
            get_export_schema()
        ]
    })
}


fn get_docs_schema() -> Value {
    json!({
        "name": "docs",
        "description": "Initialize documentation for Task Master tasks using Claude",
        "inputSchema": {
            "type": "object",
            "properties": {
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like 'projects/market-research'."
                },
                "agent": {
                    "type": "string",
                    "description": "Agent name for task assignment (e.g., morgan, rex, blaze, cipher)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "include_codebase": {
                    "type": "boolean",
                    "description": "Include existing codebase as markdown context (optional, defaults to false)"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_task_schema() -> Value {
    json!({
        "name": "task",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "Target service name (creates workspace-{service} PVC)",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., https://github.com/5dlabs/cto)"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g., projects/market-research)"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "Documentation repository URL (optional, defaults to configured value)"
                },
                "agent": {
                    "type": "string",
                    "description": "Agent name for task assignment (e.g., morgan, rex, blaze, cipher)"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (optional, defaults to '.')"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (optional, defaults to false)"
                },
                "overwrite_memory": {
                    "type": "boolean",
                    "description": "Whether to overwrite CLAUDE.md memory file (optional, defaults to false)"
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables to set in the container (optional)",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "env_from_secrets": {
                    "type": "array",
                    "description": "Environment variables from secrets (optional)",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Name of the environment variable"
                            },
                            "secretName": {
                                "type": "string",
                                "description": "Name of the secret"
                            },
                            "secretKey": {
                                "type": "string",
                                "description": "Key within the secret"
                            }
                        },
                        "required": ["name", "secretName", "secretKey"]
                    }
                }
            },
            "required": ["task_id", "service", "repository", "docs_project_directory"]
        }
    })
}

fn get_export_schema() -> Value {
    json!({
        "name": "export",
        "description": "Export Rust codebase to markdown for documentation context",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    })
}
```

### mcp/src/main.rs

```rust
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;

mod tools;

// Global agents configuration loaded once at startup
static AGENTS_CONFIG: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Load agent configuration from environment variables
/// Looks for AGENT_* environment variables (e.g., AGENT_MORGAN=5DLabs-Morgan)
fn load_agents_from_env() -> Result<HashMap<String, String>> {
    let mut agents = HashMap::new();
    
    for (key, value) in std::env::vars() {
        if let Some(agent_name) = key.strip_prefix("AGENT_") {
            let agent_name = agent_name.to_lowercase();
            agents.insert(agent_name, value);
        }
    }
    
    if agents.is_empty() {
        return Err(anyhow!("No AGENT_* environment variables found. Required format: AGENT_MORGAN=5DLabs-Morgan"));
    }
    
    Ok(agents)
}

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcSuccessResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Serialize)]
struct RpcErrorResponse {
    jsonrpc: String,
    error: RpcError,
    id: Option<Value>,
}

fn extract_params(params: Option<&Value>) -> HashMap<String, Value> {
    params
        .and_then(|p| p.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn handle_mcp_methods(method: &str, _params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "initialize" => {
            Some(Ok(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "agent-platform-mcp",
                    "title": "Agent Platform MCP Server",
                    "version": "1.0.0"
                }
            })))
        }
        "tools/list" => {
            Some(Ok(tools::get_tool_schemas()))
        }
        _ => None,
    }
}

fn run_argo_cli(args: &[&str]) -> Result<String> {
    let output = Command::new("argo")
        .args(args)
        .output()
        .context("Failed to execute argo command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Argo command failed: {}", stderr))
    }
}

/// Get the remote URL for the current git repository
fn get_git_remote_url() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {}", stderr))
    }
}

/// Get the current git branch
fn get_git_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to execute git command")?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?.trim().to_string();
        if branch.is_empty() {
            Ok("main".to_string()) // fallback to main if no branch (detached HEAD)
        } else {
            Ok(branch)
        }
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {}", stderr))
    }
}

/// Validate repository URL format
fn validate_repository_url(repo_url: &str) -> Result<()> {
    if !repo_url.starts_with("https://github.com/") {
        return Err(anyhow!(
            "Repository URL must be a GitHub HTTPS URL (e.g., 'https://github.com/org/repo')"
        ));
    }
    
    // Basic validation - should have org/repo structure
    let path = repo_url.trim_start_matches("https://github.com/");
    let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(anyhow!(
            "Repository URL must be in format 'https://github.com/org/repo'"
        ));
    }
    
    Ok(())
}

fn handle_docs_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: working_directory"))?;
    
    let agents_config = AGENTS_CONFIG.get().unwrap();
    
    // Auto-detect repository URL (fail if not available)
    let repository_url = get_git_remote_url()
        .context("Failed to auto-detect repository URL. Ensure you're in a git repository with origin remote.")?;
    validate_repository_url(&repository_url)?;
    
    // Auto-detect source branch (fail if not available)
    let source_branch = get_git_current_branch()
        .context("Failed to auto-detect git branch. Ensure you're in a git repository.")?;
    
    // Handle agent name resolution with validation
    let agent_name = arguments.get("agent").and_then(|v| v.as_str());
    let github_app = if let Some(agent) = agent_name {
        // Validate agent name exists in config
        if !agents_config.contains_key(agent) {
            let available_agents: Vec<&String> = agents_config.keys().collect();
            return Err(anyhow!(
                "Unknown agent '{}'. Available agents: {:?}", 
                agent, available_agents
            ));
        }
        agents_config[agent].clone()
    } else {
        return Err(anyhow!("No agent specified. Please provide an 'agent' parameter (e.g., 'morgan', 'rex', 'blaze', 'cipher')"));
    };
    
    // Handle model (use Helm defaults if not provided)  
    let model = arguments.get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No model specified. Please provide a 'model' parameter (e.g., 'claude-opus-4-20250514')"))?;
    
    // Validate model name
    if !model.starts_with("claude-") {
        return Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name", model));
    }
    
    let params = vec![
        format!("working-directory={working_directory}"),
        format!("repository-url={repository_url}"),
        format!("source-branch={source_branch}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
    ];
    
    let mut args = vec![
        "submit",
        "--from", "workflowtemplate/docsrun-template",
        "-n", "agent-platform",
    ];
    
    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Documentation generation workflow submitted successfully",
            "output": output,
            "working_directory": working_directory,
            "repository_url": repository_url,
            "source_branch": source_branch,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit docs workflow: {}", e)),
    }
}

fn handle_task_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let task_id = arguments
        .get("task_id")
        .and_then(|v| v.as_u64())
        .ok_or(anyhow!("Missing required parameter: task_id"))?;
    
    let service = arguments
        .get("service")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: service"))?;
        
    let repository = arguments
        .get("repository")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: repository"))?;
        
    let docs_project_directory = arguments
        .get("docs_project_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: docs_project_directory"))?;
    
    // Validate repository URL
    validate_repository_url(repository)?;
    
    // Validate service name (must be valid for PVC naming)
    if !service.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(anyhow!("Invalid service name '{}'. Must contain only lowercase letters, numbers, and hyphens", service));
    }
    
    let agents_config = AGENTS_CONFIG.get().unwrap();
    
    // Handle docs repository (require explicit value or rely on Helm defaults)
    let docs_repository = arguments.get("docs_repository")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No docs_repository specified. Please provide a 'docs_repository' parameter"))?;
    
    validate_repository_url(docs_repository)?;
    
    // Handle working directory (require explicit value or rely on Helm defaults)
    let working_directory = arguments.get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No working_directory specified. Please provide a 'working_directory' parameter"))?;
        
    // Handle agent name resolution with validation
    let agent_name = arguments.get("agent").and_then(|v| v.as_str());
    let github_app = if let Some(agent) = agent_name {
        // Validate agent name exists in config
        if !agents_config.contains_key(agent) {
            let available_agents: Vec<&String> = agents_config.keys().collect();
            return Err(anyhow!(
                "Unknown agent '{}'. Available agents: {:?}", 
                agent, available_agents
            ));
        }
        agents_config[agent].clone()
    } else {
        return Err(anyhow!("No agent specified. Please provide an 'agent' parameter (e.g., 'rex', 'blaze', 'cipher')"));
    };
    
    // Handle model (require explicit value or rely on Helm defaults)
    let model = arguments.get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("No model specified. Please provide a 'model' parameter (e.g., 'claude-3-5-sonnet-20241022')"))?;
    
    if !model.starts_with("claude-") {
        return Err(anyhow!("Invalid model '{}'. Must be a valid Claude model name", model));
    }
    
    // Auto-detect docs branch (fail if not available)
    let docs_branch = get_git_current_branch()
        .context("Failed to auto-detect git branch. Ensure you're in a git repository.")?;
    
    // Handle continue session (require explicit value or rely on Helm defaults)
    let continue_session = arguments.get("continue_session")
        .and_then(|v| v.as_bool())
        .ok_or(anyhow!("No continue_session specified. Please provide a 'continue_session' parameter (true/false)"))?;
    
    // Handle overwrite memory (require explicit value or rely on Helm defaults)
    let overwrite_memory = arguments.get("overwrite_memory")
        .and_then(|v| v.as_bool())
        .ok_or(anyhow!("No overwrite_memory specified. Please provide an 'overwrite_memory' parameter (true/false)"))?;
    
    let mut params = vec![
        format!("task-id={task_id}"),
        format!("service-id={service}"),
        format!("repository-url={repository}"),
        format!("docs-repository-url={docs_repository}"),
        format!("docs-project-directory={docs_project_directory}"),
        format!("working-directory={working_directory}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
        format!("continue-session={continue_session}"),
        format!("overwrite-memory={overwrite_memory}"),
        format!("docs-branch={docs_branch}"),
        format!("context-version=0"), // Auto-assign by controller
    ];
    
    // Handle env object - convert to JSON string for workflow parameter
    if let Some(env) = arguments.get("env").and_then(|v| v.as_object()) {
        let env_json = serde_json::to_string(env)?;
        params.push(format!("env={env_json}"));
    }
    
    // Handle env_from_secrets array - convert to JSON string for workflow parameter
    if let Some(env_from_secrets) = arguments.get("env_from_secrets").and_then(|v| v.as_array()) {
        let env_from_secrets_json = serde_json::to_string(env_from_secrets)?;
        params.push(format!("envFromSecrets={env_from_secrets_json}"));
    }
    
    let mut args = vec![
        "submit",
        "--from", "workflowtemplate/coderun-template", 
        "-n", "agent-platform",
    ];
    
    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Task implementation workflow submitted successfully",
            "output": output,
            "task_id": task_id,
            "service": service,
            "repository": repository,
            "docs_repository": docs_repository,
            "docs_project_directory": docs_project_directory,
            "working_directory": working_directory,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
            "continue_session": continue_session,
            "overwrite_memory": overwrite_memory,
            "docs_branch": docs_branch,
            "context_version": 0,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit task workflow: {}", e)),
    }
}

fn handle_tool_calls(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => {
            let name = params_map
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(anyhow!("Missing tool name"));
            
            let arguments = params_map
                .get("arguments")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default();
            
            match name {
                Ok("docs") => Some(handle_docs_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("task") => Some(handle_task_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("export") => Some(handle_export_workflow().map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": result
                    }]
                }))),
                Ok(unknown) => Some(Err(anyhow!("Unknown tool: {}", unknown))),
                Err(e) => Some(Err(e)),
            }
        }
        _ => None,
    }
}

fn handle_method(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
    let params_map = extract_params(params);

    // Try MCP protocol methods first
    if let Some(result) = handle_mcp_methods(method, &params_map) {
        return Some(result);
    }

    // Handle notifications (no response)
    if method.starts_with("notifications/") {
        return None;
    }

    // Try tool calls
    if let Some(result) = handle_tool_calls(method, &params_map) {
        return Some(result);
    }

    Some(Err(anyhow!("Unknown method: {}", method)))
}

#[allow(clippy::disallowed_macros)]
async fn rpc_loop() -> Result<()> {
    eprintln!("Starting RPC loop");
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        eprintln!("Received line: {line}");
        let request: RpcRequest = serde_json::from_str(&line).context("Invalid JSON request")?;
        eprintln!("Parsed request for method: {}", request.method);

        let result = handle_method(&request.method, request.params.as_ref());
        if let Some(method_result) = result {
            let resp_json = match method_result {
                Ok(res) => {
                    let response = RpcSuccessResponse {
                        jsonrpc: "2.0".to_string(),
                        result: res,
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
                Err(err) => {
                    let response = RpcErrorResponse {
                        jsonrpc: "2.0".to_string(),
                        error: RpcError {
                            code: -32600,
                            message: err.to_string(),
                            data: None,
                        },
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
            };
            stdout.write_all((resp_json + "\n").as_bytes()).await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

/// Handle export workflow - convert current directory's Rust code to markdown
fn handle_export_workflow() -> Result<String> {
    // Use PWD environment variable to get Cursor's current working directory
    let project_dir = std::env::var("PWD")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    
    eprintln!("🔍 Exporting Rust codebase from: {}", project_dir.display());
    
    // Create .taskmaster/docs directory if it doesn't exist
    let taskmaster_dir = project_dir.join(".taskmaster");
    let docs_dir = taskmaster_dir.join("docs");
    
    eprintln!("📁 Creating directory: {}", docs_dir.display());
    eprintln!("📁 Project dir exists: {}", project_dir.exists());
    eprintln!("📁 Project dir is_dir: {}", project_dir.is_dir());
    
    std::fs::create_dir_all(&docs_dir)
        .with_context(|| format!("Failed to create .taskmaster/docs directory at: {}", docs_dir.display()))?;
    
    let output_file = docs_dir.join("codebase.md");
    
    // Generate markdown content
    let markdown_content = generate_codebase_markdown(&project_dir)
        .context("Failed to generate codebase markdown")?;
    
    // Write to file
    std::fs::write(&output_file, &markdown_content)
        .context("Failed to write codebase.md")?;
    
    Ok(format!("✅ Exported codebase to: {}", output_file.display()))
}

/// Generate markdown representation of Rust codebase
fn generate_codebase_markdown(project_dir: &std::path::Path) -> Result<String> {
    let mut markdown = String::new();
    
    // Add header
    let project_name = project_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project");
    
    markdown.push_str(&format!("# Project: {}\n\n", project_name));
    
    // Read Cargo.toml if it exists
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(cargo_content) = std::fs::read_to_string(&cargo_toml_path) {
            markdown.push_str("## Cargo.toml\n\n```toml\n");
            markdown.push_str(&cargo_content);
            markdown.push_str("\n```\n\n");
        }
    }
    
    // Find and process all .rs files
    markdown.push_str("## Source Files\n\n");
    
    process_rust_files(&mut markdown, project_dir, project_dir)?;
    
    Ok(markdown)
}

/// Recursively process Rust files
fn process_rust_files(
    markdown: &mut String, 
    current_dir: &std::path::Path,
    project_root: &std::path::Path
) -> Result<()> {
    let entries = std::fs::read_dir(current_dir)
        .context("Failed to read directory")?;
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip target directory and hidden directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "target" || name.starts_with('.') {
                continue;
            }
        }
        
        if path.is_dir() {
            process_rust_files(markdown, &path, project_root)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Get relative path from project root
            let relative_path = path.strip_prefix(project_root)
                .context("Failed to get relative path")?;
            
            markdown.push_str(&format!("### {}\n\n", relative_path.display()));
            
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    markdown.push_str("```rust\n");
                    markdown.push_str(&content);
                    markdown.push_str("\n```\n\n");
                }
                Err(e) => {
                    markdown.push_str(&format!("*Error reading file: {}*\n\n", e));
                }
            }
        }
    }
    
    Ok(())
}

#[allow(clippy::disallowed_macros)]
fn main() -> Result<()> {
    eprintln!("🚀 Starting 5D Labs MCP Server...");
    
    // Initialize agents configuration from environment variables
    let agents_config = load_agents_from_env()
        .context("Failed to load agents configuration")?;
    eprintln!("📋 Loaded {} agents from environment: {:?}", 
              agents_config.len(), 
              agents_config.keys().collect::<Vec<_>>());
    
    // Store in global static
    AGENTS_CONFIG.set(agents_config).map_err(|_| anyhow!("Failed to set agents config"))?;
    eprintln!("✅ Agents configuration loaded");
    
    eprintln!("Creating runtime...");
    let rt = Runtime::new()?;
    eprintln!("Runtime created, starting RPC loop");
    rt.block_on(rpc_loop())?;
    eprintln!("RPC loop completed");
    Ok(())
}
```

### common/src/error.rs

```rust
//! Common error types for the orchestrator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes operation failed: {0}")]
    Kubernetes(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Job failed: {0}")]
    JobFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversion from anyhow::Error for easier error handling
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

```

### common/src/lib.rs

```rust
/*
 * 5D Labs Agent Platform - Common Types and Utilities
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

//! Shared types and utilities for the Orchestrator project

pub mod error;
pub mod models;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

// Re-export commonly used types for convenience
pub use models::{AgentType, Job, JobStatus, JobType, Request, RequestSource, Task, TaskStatus};

```

### common/src/models/response.rs

```rust
//! Response models for API endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: ResponseStatus,
    pub data: Option<T>,
    pub error: Option<ErrorDetails>,
    pub metadata: ResponseMetadata,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Error,
    Partial,
}

/// Error details in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub version: String,
}

/// Task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: super::task::TaskStatus,
    pub priority: super::task::TaskPriority,
    pub microservice: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub job_ids: Vec<String>,
}

/// Job response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub task_id: String,
    pub job_type: super::job::JobType,
    pub status: super::job::JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub logs_url: Option<String>,
}

/// Job list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Task list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<TaskResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: HashMap<String, ComponentHealth>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Webhook processing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub accepted: bool,
    pub task_id: Option<String>,
    pub job_ids: Vec<String>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// Create a success response
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(error: ErrorDetails, request_id: String) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: None,
            error: Some(error),
            metadata: ResponseMetadata {
                request_id,
                timestamp: Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Set the duration in milliseconds
    #[must_use]
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.metadata.duration_ms = Some(duration_ms);
        self
    }
}

impl From<super::task::Task> for TaskResponse {
    fn from(task: super::Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            microservice: task.microservice,
            created_at: task.created_at,
            updated_at: task.updated_at,
            job_ids: Vec::new(), // To be populated by service layer
        }
    }
}

impl From<super::job::Job> for JobResponse {
    fn from(job: super::Job) -> Self {
        Self {
            id: job.id,
            task_id: job.task_id,
            job_type: job.job_type,
            status: job.status,
            k8s_job_name: job.k8s_job_name,
            namespace: job.namespace,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            logs_url: None, // To be populated by service layer
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::{TaskPriority, TaskStatus};

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success(
            TaskResponse {
                id: "task-123".to_string(),
                title: "Test Task".to_string(),
                description: "A test task".to_string(),
                status: TaskStatus::Pending,
                priority: TaskPriority::Medium,
                microservice: "auth".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                job_ids: vec![],
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<TaskResponse> = ApiResponse::error(
            ErrorDetails {
                code: "TASK_NOT_FOUND".to_string(),
                message: "Task not found".to_string(),
                details: None,
            },
            "req-123".to_string(),
        );

        assert_eq!(response.status, ResponseStatus::Error);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}

```

### common/src/models/request.rs

```rust
//! Request models for unified orchestration

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Unified request interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub source: RequestSource,
    pub action: RequestAction,
    pub payload: Value,
    pub metadata: RequestMetadata,
}

/// Source of incoming requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestSource {
    Cli,
    PmAgent,
    GitHub,
    Grafana,
    Discord,
}

/// Action to be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestAction {
    CreateTask,
    UpdateTask,
    GetTaskStatus,
    TriggerAssistance,
    ListJobs,
    GetJobLogs,
    ReviewPR,
    HandleAlert,
}

/// Additional request metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestMetadata {
    pub user: Option<String>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub channel: Option<String>,
    pub timestamp: String,
    pub trace_id: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Parsed request after normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequest {
    pub action: RequestAction,
    pub task_id: Option<String>,
    pub microservice: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub metadata: Value,
}

/// CLI request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRequest {
    pub command: String,
    pub args: Vec<String>,
    pub options: HashMap<String, String>,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub microservice: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: Option<String>,
    pub agent_type: Option<super::AgentType>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Task update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<super::TaskStatus>,
    pub priority: Option<super::task::TaskPriority>,
    pub description: Option<String>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Assistance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceRequest {
    pub task_id: String,
    pub reason: String,
    pub assist_type: AssistanceType,
    pub context: Option<Value>,
    pub priority: AssistancePriority,
}

/// Type of assistance needed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistanceType {
    ImplementationGuidance,
    ArchitectureReview,
    ErrorDiagnosis,
    TestDebugging,
    PerformanceOptimization,
}

/// Priority of assistance request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssistancePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Request {
    /// Create a new request
    #[must_use]
    pub fn new(source: RequestSource, action: RequestAction, payload: Value) -> Self {
        use chrono::Utc;
        use uuid::Uuid;

        Self {
            id: Uuid::new_v4().to_string(),
            source,
            action,
            payload,
            metadata: RequestMetadata {
                timestamp: Utc::now().to_rfc3339(),
                ..Default::default()
            },
        }
    }

    /// Add trace ID for distributed tracing
    #[must_use]
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }

    /// Add user information
    #[must_use]
    pub fn with_user(mut self, user: String) -> Self {
        self.metadata.user = Some(user);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_creation() {
        let request = Request::new(
            RequestSource::Cli,
            RequestAction::CreateTask,
            json!({
                "title": "Test Task",
                "description": "A test task"
            }),
        );

        assert_eq!(request.source, RequestSource::Cli);
        assert_eq!(request.action, RequestAction::CreateTask);
        assert!(!request.id.is_empty());
        assert!(!request.metadata.timestamp.is_empty());
    }

    #[test]
    fn test_create_task_request_serialization() {
        let req = CreateTaskRequest {
            microservice: "auth".to_string(),
            title: "Implement JWT validation".to_string(),
            description: "Add JWT token validation".to_string(),
            acceptance_criteria: vec!["Validate tokens".to_string()],
            priority: Some("high".to_string()),
            agent_type: Some(super::super::AgentType::Claude),
            metadata: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: CreateTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.title, deserialized.title);
        assert_eq!(req.microservice, deserialized.microservice);
    }
}

```

### common/src/models/job.rs

```rust
//! Job-related data models for Kubernetes job orchestration

use chrono::{DateTime, Utc};
use k8s_openapi::api::batch::v1::Job as K8sJob;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Kubernetes job for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub task_id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub k8s_job_name: String,
    pub namespace: String,
    pub spec: JobSpec,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Type of job in the orchestration pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Prepare job that sets up workspace and context files
    Prepare,
    /// Execute job that runs the primary agent (Claude)
    Execute,
    /// Assist job that runs helper agent (Gemini)
    Assist,
    /// Review job for code review tasks
    Review,
}

/// Job execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

/// Job specification details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// Container image to use
    pub image: String,
    /// Agent type for execution jobs
    pub agent: Option<super::AgentType>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Resource limits and requests
    pub resources: super::ResourceLimits,
    /// Volume mounts
    pub volumes: Vec<VolumeSpec>,
    /// Command to execute
    pub command: Option<Vec<String>>,
    /// Working directory
    pub working_dir: Option<String>,
    /// Job timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Number of retries
    pub retry_limit: Option<u32>,
}

/// Volume specification for job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub mount_path: String,
    pub volume_type: VolumeType,
    pub read_only: bool,
}

/// Types of volumes that can be mounted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeType {
    /// `ConfigMap` volume
    ConfigMap { name: String },
    /// `PersistentVolumeClaim`
    Pvc { claim_name: String },
    /// `EmptyDir` volume
    EmptyDir,
    /// Secret volume
    Secret { name: String },
}

impl Job {
    /// Create a new job
    #[must_use]
    pub fn new(
        id: String,
        task_id: String,
        job_type: JobType,
        k8s_job_name: String,
        namespace: String,
        spec: JobSpec,
    ) -> Self {
        Self {
            id,
            task_id,
            job_type,
            status: JobStatus::Pending,
            k8s_job_name,
            namespace,
            spec,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Update job status based on Kubernetes job status
    ///
    /// # Panics
    ///
    /// Panics if the Kubernetes job completion time cannot be parsed as RFC3339
    pub fn update_from_k8s_job(&mut self, k8s_job: &K8sJob) {
        if let Some(status) = &k8s_job.status {
            if status.succeeded == Some(1) {
                self.status = JobStatus::Succeeded;
                self.completed_at = status.completion_time.as_ref().map(|t| {
                    DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                        .unwrap()
                        .with_timezone(&Utc)
                });
            } else if status.failed.unwrap_or(0) > 0 {
                self.status = JobStatus::Failed;
                self.completed_at = Some(Utc::now());
            } else if status.active == Some(1) {
                self.status = JobStatus::Running;
                if self.started_at.is_none() {
                    self.started_at = status.start_time.as_ref().map(|t| {
                        DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                            .unwrap()
                            .with_timezone(&Utc)
                    });
                }
            }
        }
    }

    /// Check if the job is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, JobStatus::Succeeded | JobStatus::Failed)
    }

    /// Get job duration if available
    #[must_use]
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            image: "busybox:latest".to_string(),
            agent: None,
            env_vars: HashMap::new(),
            resources: super::ResourceLimits::default(),
            volumes: Vec::new(),
            command: None,
            working_dir: None,
            timeout_seconds: Some(1800), // 30 minutes default
            retry_limit: Some(2),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Prepare => write!(f, "Prepare"),
            JobType::Execute => write!(f, "Execute"),
            JobType::Assist => write!(f, "Assist"),
            JobType::Review => write!(f, "Review"),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Succeeded => write!(f, "Succeeded"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let spec = JobSpec::default();
        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());
        assert!(!job.is_terminal());
    }

    #[test]
    fn test_job_serialization() {
        let spec = JobSpec {
            image: "claude:latest".to_string(),
            agent: Some(super::super::AgentType::Claude),
            ..Default::default()
        };

        let job = Job::new(
            "job-123".to_string(),
            "task-123".to_string(),
            JobType::Execute,
            "claude-task-123".to_string(),
            "default".to_string(),
            spec,
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(job.id, deserialized.id);
        assert_eq!(job.job_type, deserialized.job_type);
    }
}

```

### common/src/models/config.rs

```rust
//! Configuration-related models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent types that can execute tasks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    #[default]
    Claude,
    Gemini,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub image: String,
    pub version: String,
    pub env_vars: HashMap<String, String>,
    pub resources: ResourceLimits,
    pub capabilities: Vec<String>,
    pub mcp_servers: Vec<String>,
}

/// Resource limits and requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
    pub ephemeral_storage_request: Option<String>,
    pub ephemeral_storage_limit: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub capabilities: Vec<String>,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub namespace: String,
    pub agents: HashMap<AgentType, AgentConfig>,
    pub default_timeout_seconds: u32,
    pub max_retry_attempts: u32,
    pub workspace_pvc_template: String,
    pub prepare_job_image: String,
    pub node_selector: HashMap<String, String>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "2Gi".to_string(),
            ephemeral_storage_request: None,
            ephemeral_storage_limit: None,
        }
    }
}

impl AgentType {
    /// Get display name for the agent
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::Claude => "Claude Code",
            AgentType::Gemini => "Gemini CLI",
        }
    }

    /// Get the default image for the agent
    #[must_use]
    pub fn default_image(&self) -> &'static str {
        match self {
            AgentType::Claude => "anthropic/claude-code:latest",
            AgentType::Gemini => "google/gemini-cli:latest",
        }
    }

    /// Check if this agent can be a primary implementer
    #[must_use]
    pub fn can_implement(&self) -> bool {
        match self {
            AgentType::Claude => true,
            AgentType::Gemini => false, // Gemini is assistance-only in our pattern
        }
    }

    /// Check if this agent can provide assistance
    #[must_use]
    pub fn can_assist(&self) -> bool {
        match self {
            AgentType::Claude => false, // Claude is implementation-only in our pattern
            AgentType::Gemini => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_capabilities() {
        assert!(AgentType::Claude.can_implement());
        assert!(!AgentType::Claude.can_assist());
        assert!(!AgentType::Gemini.can_implement());
        assert!(AgentType::Gemini.can_assist());
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu_request, "100m");
        assert_eq!(limits.memory_limit, "2Gi");
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig {
            agent_type: AgentType::Claude,
            image: "claude:v1".to_string(),
            version: "1.0.0".to_string(),
            env_vars: HashMap::new(),
            resources: ResourceLimits::default(),
            capabilities: vec!["code".to_string(), "test".to_string()],
            mcp_servers: vec!["taskmaster".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.agent_type, deserialized.agent_type);
        assert_eq!(config.capabilities, deserialized.capabilities);
    }
}

```

### common/src/models/webhook.rs

```rust
//! Webhook payload models for various sources

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Generic webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub source: super::RequestSource,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

/// GitHub webhook payload for issue events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub pull_request: Option<GitHubPullRequest>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// GitHub issue structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub pull request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub head: GitHubRef,
    pub base: GitHubRef,
    pub created_at: String,
    pub updated_at: String,
    pub user: GitHubUser,
}

/// GitHub repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub private: bool,
    pub default_branch: String,
}

/// GitHub user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(rename = "type")]
    pub user_type: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

/// GitHub ref (branch/tag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    pub label: String,
    pub ref_field: String,
    pub sha: String,
}

/// Grafana alert webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaAlert {
    pub status: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub values: HashMap<String, f64>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
}

/// Grafana webhook payload wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaWebhookPayload {
    pub receiver: String,
    pub status: String,
    pub alerts: Vec<GrafanaAlert>,
    #[serde(rename = "groupLabels")]
    pub group_labels: HashMap<String, String>,
    #[serde(rename = "commonLabels")]
    pub common_labels: HashMap<String, String>,
    #[serde(rename = "commonAnnotations")]
    pub common_annotations: HashMap<String, String>,
}

/// PM Agent webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmAgentPayload {
    pub action: String,
    pub project_id: String,
    pub task: PmTaskData,
}

/// PM Agent task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Discord webhook payload (via relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub command: String,
    pub args: Vec<String>,
    pub message_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_webhook_deserialization() {
        let json = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 42,
                "title": "Test Issue",
                "body": "Test body",
                "state": "open",
                "labels": [],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "user": {
                    "login": "testuser",
                    "id": 456,
                    "type": "User"
                }
            },
            "repository": {
                "id": 789,
                "name": "test-repo",
                "full_name": "org/test-repo",
                "owner": {
                    "login": "org",
                    "id": 999,
                    "type": "Organization"
                },
                "private": false,
                "default_branch": "main"
            },
            "sender": {
                "login": "testuser",
                "id": 456,
                "type": "User"
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.action, "opened");
        assert!(payload.issue.is_some());
        assert_eq!(payload.issue.unwrap().number, 42);
    }

    #[test]
    fn test_grafana_alert_deserialization() {
        let json = r#"{
            "receiver": "webhook",
            "status": "firing",
            "alerts": [{
                "status": "firing",
                "labels": {
                    "alertname": "HighErrorRate",
                    "task_id": "123"
                },
                "annotations": {
                    "summary": "High error rate detected"
                },
                "values": {
                    "error_rate": 0.45
                },
                "startsAt": "2024-01-01T00:00:00Z",
                "endsAt": null,
                "generatorURL": "http://grafana/alert"
            }],
            "groupLabels": {},
            "commonLabels": {},
            "commonAnnotations": {}
        }"#;

        let payload: GrafanaWebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.status, "firing");
        assert_eq!(payload.alerts.len(), 1);
        assert_eq!(
            payload.alerts[0].labels.get("task_id"),
            Some(&"123".to_string())
        );
    }
}

```

### common/src/models/code_request.rs

```rust
//! Clean code task submission request structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export SecretEnvVar from orchestrator-core crate to avoid duplication
// For now, we'll define it locally until we can reorganize the type sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    /// Task ID to implement
    pub task_id: u32,

    /// Target service name
    pub service: String,

    /// Target project repository URL (where implementation work happens)
    pub repository_url: String,

    /// Documentation repository URL (where Task Master definitions come from)
    pub docs_repository_url: String,

    /// Project directory within docs repository (e.g. "_projects/simple-api")
    pub docs_project_directory: Option<String>,

    /// Working directory within target repository (defaults to service name)
    pub working_directory: Option<String>,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Context version for retry attempts (incremented on each retry)
    #[serde(default = "default_context_version")]
    pub context_version: u32,

    /// Docs branch to use (e.g., "main", "feature/branch")
    #[serde(default = "default_docs_branch")]
    pub docs_branch: String,

    /// Whether to continue a previous session (auto-continue on retries or user-requested)
    #[serde(default)]
    pub continue_session: bool,

    /// Whether to overwrite memory before starting
    #[serde(default)]
    pub overwrite_memory: bool,

    /// Environment variables to set in the container
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Environment variables from secrets
    #[serde(default)]
    pub env_from_secrets: Vec<SecretEnvVar>,
}

/// Default context version
fn default_context_version() -> u32 {
    1
}

/// Default docs branch
fn default_docs_branch() -> String {
    "main".to_string()
}

```

### common/src/models/task.rs

```rust
//! Task-related data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a task to be executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub microservice: String,
    pub agent_type: Option<super::AgentType>,
    pub metadata: TaskMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Blocked,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Additional metadata for tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskMetadata {
    /// Source that created this task
    pub source: Option<String>,
    /// GitHub issue number if applicable
    pub github_issue: Option<u64>,
    /// Task Master task ID if applicable
    pub task_master_id: Option<String>,
    /// Custom labels
    pub labels: HashMap<String, String>,
    /// Additional arbitrary data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Task {
    /// Create a new task with default values
    #[must_use]
    pub fn new(id: String, title: String, description: String, microservice: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            acceptance_criteria: Vec::new(),
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            microservice,
            agent_type: None,
            metadata: TaskMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the task is in a terminal state
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }

    /// Update the task status and timestamp
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert_eq!(task.id, "test-123");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(!task.is_terminal());
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.status, deserialized.status);
    }

    #[test]
    fn test_terminal_states() {
        let mut task = Task::new(
            "test-123".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "auth".to_string(),
        );

        assert!(!task.is_terminal());

        task.update_status(TaskStatus::Completed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::Failed);
        assert!(task.is_terminal());

        task.update_status(TaskStatus::InProgress);
        assert!(!task.is_terminal());
    }
}

```

### common/src/models/docs_request.rs

```rust
//! Clean documentation generation request structure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsRequest {
    /// Git repository URL
    pub repository_url: String,

    /// Working directory within the repository
    pub working_directory: String,

    /// Claude model to use (sonnet, opus) - optional, defaults handled by MCP tools
    pub model: Option<String>,

    /// GitHub username for authentication
    pub github_user: String,

    /// Source branch (auto-detected)
    pub source_branch: String,
}

```

### common/src/models/mod.rs

```rust
//! Core data models module

pub mod code_request;
pub mod config;
pub mod docs_request;
pub mod job;
pub mod pm_task;
pub mod request;
pub mod response;
pub mod task;
pub mod webhook;

// Re-export commonly used types
pub use code_request::CodeRequest;
pub use config::{AgentConfig, AgentType, ResourceLimits};
pub use docs_request::DocsRequest;
pub use job::{Job, JobSpec, JobStatus, JobType};
pub use pm_task::{
    DocsGenerationRequest, MarkdownPayload, PmTaskRequest, Subtask, Task as PmTask, TaskMaster,
    TaskMasterFile,
};
pub use request::{ParsedRequest, Request, RequestAction, RequestSource};
pub use response::{ApiResponse, JobResponse, TaskResponse};
pub use task::{Task, TaskMetadata, TaskStatus};
pub use webhook::{GitHubWebhookPayload, GrafanaAlert, WebhookPayload};

```

### common/src/models/pm_task.rs

```rust
//! PM task submission models

use serde::{Deserialize, Serialize};

/// PM task request structure according to design document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmTaskRequest {
    // Task Master schema fields
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,

    // PM-specific fields
    pub service_name: String,
    pub agent_name: String,

    // Claude model selection (sonnet, opus)
    pub model: String,

    // Markdown files as structured payloads
    pub markdown_files: Vec<MarkdownPayload>,

    // Agent tools specification
    #[serde(default)]
    pub agent_tools: Vec<AgentToolSpec>,

    // Repository specification for code access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositorySpec>,

    // Working directory within target repository (defaults to service_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    // Additional prompt instructions for retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_modification: Option<String>,

    // How to apply prompt_modification: 'append' or 'replace'
    #[serde(
        default = "default_prompt_mode",
        skip_serializing_if = "is_default_prompt_mode"
    )]
    pub prompt_mode: String,

    // Local Claude Code tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_tools: Vec<String>,

    // Remote MCP tools to enable
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remote_tools: Vec<String>,

    // Tool configuration preset
    #[serde(
        default = "default_tool_config",
        skip_serializing_if = "is_default_tool_config"
    )]
    pub tool_config: String,
}

/// Subtask structure from Task Master
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<u32>,
    pub details: String,
    pub status: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
}

/// Markdown file payload for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownPayload {
    pub content: String,
    pub filename: String,
    pub file_type: String,
}

/// Agent tool specification for PM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolSpec {
    pub name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

/// Repository specification for cloning source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySpec {
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    pub github_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>, // Reserved for future use - TODO: Implement direct token submission
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_prompt_mode() -> String {
    "append".to_string()
}

fn is_default_prompt_mode(mode: &str) -> bool {
    mode == "append"
}

fn default_tool_config() -> String {
    "default".to_string()
}

fn is_default_tool_config(config: &str) -> bool {
    config == "default"
}

/// Documentation generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsGenerationRequest {
    /// Repository URL to clone
    pub repository_url: String,

    /// Working directory within the repository (path to .taskmaster)
    pub working_directory: String,

    /// Source branch to checkout and base new branch from
    pub source_branch: String,

    /// Target branch for the PR
    pub target_branch: String,

    /// Service name for the job
    pub service_name: String,

    /// Agent name for the job
    pub agent_name: String,

    /// Claude model selection (sonnet, opus)
    pub model: String,

    /// GitHub user for authentication
    pub github_user: String,

    /// Optional specific task ID to generate docs for (if None, generates all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,

    /// Force overwrite existing documentation
    #[serde(default)]
    pub force: bool,

    /// Dry run mode (preview only)
    #[serde(default)]
    pub dry_run: bool,
}

/// Task Master JSON file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMasterFile {
    pub master: TaskMaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMaster {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: String,
    #[serde(default, alias = "testStrategy")]
    pub test_strategy: String,
    pub priority: String,
    pub dependencies: Vec<u32>,
    pub status: String,
    pub subtasks: Vec<Subtask>,
}

impl PmTaskRequest {
    /// Create a new PM task request from Task Master task and markdown files
    #[must_use]
    pub fn new(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
    ) -> Self {
        Self::new_with_tools(
            task,
            service_name,
            agent_name,
            model,
            markdown_files,
            Vec::new(),
        )
    }

    /// Create a new PM task request with agent tools specification
    #[must_use]
    pub fn new_with_tools(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository: None,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request from Task Master task with repository support
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_repository(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory: None,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full specification including working directory
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_full_spec(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification: None,
            prompt_mode: "append".to_string(),
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with prompt modification support for retries
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_prompt_modification(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools: Vec::new(),
            remote_tools: Vec::new(),
            tool_config: "default".to_string(),
        }
    }

    /// Create a new PM task request with full tool configuration support
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_tool_config(
        task: Task,
        service_name: String,
        agent_name: String,
        model: String,
        markdown_files: Vec<MarkdownPayload>,
        agent_tools: Vec<AgentToolSpec>,
        repository: Option<RepositorySpec>,
        working_directory: Option<String>,
        prompt_modification: Option<String>,
        prompt_mode: String,
        local_tools: Vec<String>,
        remote_tools: Vec<String>,
        tool_config: String,
    ) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            details: task.details,
            test_strategy: task.test_strategy,
            priority: task.priority,
            dependencies: task.dependencies,
            status: task.status,
            subtasks: task.subtasks,
            service_name,
            agent_name,
            model,
            markdown_files,
            agent_tools,
            repository,
            working_directory,
            prompt_modification,
            prompt_mode,
            local_tools,
            remote_tools,
            tool_config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_task_request_creation() {
        let task = Task {
            id: 1001,
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            details: "Test details".to_string(),
            test_strategy: "Test strategy".to_string(),
            priority: "high".to_string(),
            dependencies: vec![],
            status: "pending".to_string(),
            subtasks: vec![],
        };

        let markdown_files = vec![MarkdownPayload {
            content: "# Task Content".to_string(),
            filename: "task.md".to_string(),
            file_type: "task".to_string(),
        }];

        let request = PmTaskRequest::new(
            task,
            "test-service".to_string(),
            "claude-agent-1".to_string(),
            "sonnet".to_string(),
            markdown_files,
        );

        assert_eq!(request.id, 1001);
        assert_eq!(request.service_name, "test-service");
        assert_eq!(request.model, "sonnet");
        assert_eq!(request.markdown_files.len(), 1);
    }
}

```

