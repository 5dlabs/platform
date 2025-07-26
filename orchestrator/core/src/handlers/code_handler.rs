//! Code task submission handler

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use kube::Api;
use std::collections::HashMap;
use tracing::{error, info};

use crate::crds::{CodeRun, CodeRunSpec, CodeRunStatus};
use crate::handlers::common::{ApiResponse, AppState};
use common::models::CodeRequest;

/// Convert comma-separated tools string to vector
fn parse_tools_string(tools: Option<String>) -> Vec<String> {
    tools
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub async fn submit_code_task(
    State(state): State<AppState>,
    Json(request): Json<CodeRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!(
        "Received code task request: task_id={}, service={}",
        request.task_id, request.service
    );

    // Convert string-based tools to structured format
    let tools = if request.local_tools.is_some() || request.remote_tools.is_some() {
        Some(crate::crds::coderun::ToolConfig {
            local: parse_tools_string(request.local_tools.clone()),
            remote: parse_tools_string(request.remote_tools.clone()),
        })
    } else {
        None
    };

    // Validate tools if specified
    if let Some(ref tool_config) = tools {
        if let Err(e) = super::common::validate_tools(
            &tool_config.local,
            &tool_config.remote,
            state.k8s_client.clone(),
        ).await {
            error!("Tool validation failed: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

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
        tools,
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
