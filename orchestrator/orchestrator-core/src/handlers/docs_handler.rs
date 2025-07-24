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
use orchestrator_common::models::DocsRequest;

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
        model: request.model.clone(),
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
