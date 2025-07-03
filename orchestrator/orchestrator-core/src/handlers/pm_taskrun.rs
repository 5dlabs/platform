//! PM task submission handler for TaskRun CRD
//!
//! This handler replaces the Helm-based deployment with TaskRun CRD management

use crate::crds::{MarkdownFile, MarkdownFileType, TaskRun, TaskRunSpec};
use axum::{extract::State, http::StatusCode, response::Json};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, PostParams},
    Client,
};
use orchestrator_common::models::pm_task::PmTaskRequest;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Application state for the handler
pub struct AppState {
    pub k8s_client: Client,
    pub namespace: String,
}

/// Error type for PM handler
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

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
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// Handle PM task submission with validation
pub async fn submit_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PmTaskRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!(
        "Received PM task submission: task_id={}, service={}",
        request.id, request.service_name
    );

    // Validate request
    if request.markdown_files.is_empty() {
        warn!("Task {} has no markdown files", request.id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("No markdown files provided")),
        ));
    }

    // Check if TaskRun already exists
    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);
    let name = format!("task-{}", request.id);

    match api.get(&name).await {
        Ok(_) => {
            warn!("TaskRun {} already exists", name);
            return Err((
                StatusCode::CONFLICT,
                Json(ApiResponse::error(&format!(
                    "Task {} already exists",
                    request.id
                ))),
            ));
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Expected - task doesn't exist yet
        }
        Err(e) => {
            error!("Error checking for existing TaskRun: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to check existing task")),
            ));
        }
    }

    // Convert request markdown files to CRD format
    let markdown_files = request
        .markdown_files
        .into_iter()
        .map(|f| MarkdownFile {
            filename: f.filename,
            content: f.content,
            file_type: match f.file_type.as_str() {
                "task" => Some(MarkdownFileType::Task),
                "design-spec" => Some(MarkdownFileType::DesignSpec),
                "prompt" => Some(MarkdownFileType::Prompt),
                "context" => Some(MarkdownFileType::Context),
                "acceptance-criteria" => Some(MarkdownFileType::AcceptanceCriteria),
                _ => None,
            },
        })
        .collect();

    // Create TaskRun
    let taskrun = TaskRun {
        metadata: ObjectMeta {
            name: Some(name.clone()),
            namespace: Some(state.namespace.clone()),
            labels: Some(BTreeMap::from([
                ("task-id".to_string(), request.id.to_string()),
                ("service-name".to_string(), request.service_name.clone()),
                ("agent-name".to_string(), request.agent_name.clone()),
            ])),
            ..Default::default()
        },
        spec: TaskRunSpec {
            task_id: request.id,
            service_name: request.service_name.clone(),
            agent_name: request.agent_name.clone(),
            context_version: 1,
            markdown_files,
        },
        status: None,
    };

    // Create the TaskRun
    match api.create(&PostParams::default(), &taskrun).await {
        Ok(_) => {
            info!("Successfully created TaskRun: {}", name);
            Ok(Json(ApiResponse {
                success: true,
                message: "Task submitted successfully".to_string(),
                data: Some(json!({
                    "name": name,
                    "namespace": state.namespace,
                    "service": request.service_name,
                    "task_id": request.id,
                })),
            }))
        }
        Err(e) => {
            error!("Failed to create TaskRun: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to create task")),
            ))
        }
    }
}

/// Add context to an existing task using Server-Side Apply
pub async fn add_context(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(task_id): axum::extract::Path<u32>,
    Json(context): Json<AddContextRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!("Adding context to task {}", task_id);

    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);
    let name = format!("task-{task_id}");

    // Get current TaskRun to determine next version
    let current_tr = match api.get(&name).await {
        Ok(tr) => tr,
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(&format!("Task {task_id} not found"))),
            ));
        }
        Err(e) => {
            error!("Error fetching TaskRun: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to fetch task")),
            ));
        }
    };

    let next_version = current_tr.spec.context_version + 1;

    // Use Server-Side Apply for conflict-free updates
    let patch = json!({
        "apiVersion": "orchestrator.io/v1",
        "kind": "TaskRun",
        "metadata": {
            "name": name,
            "namespace": state.namespace,
        },
        "spec": {
            "taskId": task_id,
            "serviceName": current_tr.spec.service_name,
            "agentName": current_tr.spec.agent_name,
            "contextVersion": next_version,
            "markdownFiles": [{
                "filename": format!("context-v{}.md", next_version),
                "content": context.additional_context,
                "fileType": "context",
            }],
        }
    });

    let patch_params = kube::api::PatchParams::apply("pm-handler").force();

    match api
        .patch(&name, &patch_params, &kube::api::Patch::Apply(patch))
        .await
    {
        Ok(_) => {
            info!("Successfully added context to TaskRun: {}", name);
            Ok(Json(ApiResponse {
                success: true,
                message: "Context added successfully".to_string(),
                data: Some(json!({
                    "name": name,
                    "context_version": next_version,
                })),
            }))
        }
        Err(e) => {
            error!("Failed to add context: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to add context")),
            ))
        }
    }
}

/// Request for adding context to an existing task
#[derive(serde::Deserialize)]
pub struct AddContextRequest {
    pub additional_context: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taskrun_name_generation() {
        let task_id = 1001;
        let expected = "task-1001";
        let actual = format!("task-{task_id}");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("Task created");
        assert!(response.success);
        assert_eq!(response.message, "Task created");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::error("Validation failed");
        assert!(!response.success);
        assert_eq!(response.message, "Validation failed");
        assert!(response.data.is_none());
    }
}
