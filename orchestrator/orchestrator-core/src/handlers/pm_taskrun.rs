//! PM task submission handler for TaskRun CRD
//!
//! This handler replaces the Helm-based deployment with TaskRun CRD management

use crate::crds::{AgentTool, MarkdownFile, MarkdownFileType, TaskRun, TaskRunSpec};
use axum::{extract::State, http::StatusCode, response::Json};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams, PostParams},
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

    // Convert agent tools to CRD format
    let agent_tools = request
        .agent_tools
        .into_iter()
        .map(|tool| AgentTool {
            name: tool.name,
            enabled: tool.enabled,
            config: tool.config,
            restrictions: tool.restrictions,
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
            agent_tools,
            repository: request.repository.map(|repo| crate::crds::taskrun::RepositorySpec {
                url: repo.url,
                branch: repo.branch,
                path: repo.path,
                auth: repo.auth.map(|auth| crate::crds::taskrun::RepositoryAuth {
                    auth_type: match auth.auth_type {
                        orchestrator_common::models::pm_task::RepositoryAuthType::Token => 
                            crate::crds::taskrun::RepositoryAuthType::Token,
                        orchestrator_common::models::pm_task::RepositoryAuthType::SshKey => 
                            crate::crds::taskrun::RepositoryAuthType::SshKey,
                        orchestrator_common::models::pm_task::RepositoryAuthType::BasicAuth => 
                            crate::crds::taskrun::RepositoryAuthType::BasicAuth,
                    },
                    secret_name: auth.secret_name,
                    secret_key: auth.secret_key,
                }),
            }),
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

/// Request for updating task session
#[derive(serde::Deserialize)]
pub struct UpdateSessionRequest {
    pub session_id: String,
}

/// Get all tasks with optional filtering
pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!("Listing all tasks");

    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);

    match api.list(&ListParams::default()).await {
        Ok(task_list) => {
            let tasks: Vec<Value> = task_list
                .items
                .into_iter()
                .map(|tr| {
                    json!({
                        "name": tr.metadata.name.unwrap_or_default(),
                        "task_id": tr.spec.task_id,
                        "service_name": tr.spec.service_name,
                        "agent_name": tr.spec.agent_name,
                        "context_version": tr.spec.context_version,
                        "phase": tr.status.as_ref().and_then(|s| s.phase.as_ref()).map(|p| p.to_string()),
                        "session_id": tr.status.as_ref().and_then(|s| s.session_id.clone()),
                        "attempts": tr.status.as_ref().map(|s| s.attempts).unwrap_or(0),
                        "last_updated": tr.status.as_ref().and_then(|s| s.last_updated.clone()),
                        "message": tr.status.as_ref().and_then(|s| s.message.clone()),
                    })
                })
                .collect();

            Ok(Json(ApiResponse {
                success: true,
                message: format!("Found {} tasks", tasks.len()),
                data: Some(json!({ "tasks": tasks })),
            }))
        }
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to list tasks")),
            ))
        }
    }
}

/// Get a specific task by ID
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(task_id): axum::extract::Path<u32>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!("Getting task {}", task_id);

    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);
    let name = format!("task-{task_id}");

    match api.get(&name).await {
        Ok(taskrun) => {
            let task_data = json!({
                "name": taskrun.metadata.name.unwrap_or_default(),
                "task_id": taskrun.spec.task_id,
                "service_name": taskrun.spec.service_name,
                "agent_name": taskrun.spec.agent_name,
                "context_version": taskrun.spec.context_version,
                "markdown_files": taskrun.spec.markdown_files.iter().map(|f| {
                    json!({
                        "filename": f.filename,
                        "file_type": f.file_type,
                        "content_length": f.content.len(),
                    })
                }).collect::<Vec<_>>(),
                "status": taskrun.status.as_ref().map(|s| {
                    json!({
                        "phase": s.phase.as_ref().map(|p| p.to_string()),
                        "session_id": s.session_id,
                        "job_name": s.job_name,
                        "config_map_name": s.config_map_name,
                        "attempts": s.attempts,
                        "last_updated": s.last_updated,
                        "message": s.message,
                        "conditions": s.conditions.iter().map(|c| {
                            json!({
                                "type": c.condition_type,
                                "status": c.status.to_string(),
                                "reason": c.reason,
                                "message": c.message,
                                "last_transition_time": c.last_transition_time,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }),
            });

            Ok(Json(ApiResponse {
                success: true,
                message: "Task retrieved successfully".to_string(),
                data: Some(task_data),
            }))
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Task {task_id} not found"))),
        )),
        Err(e) => {
            error!("Failed to get task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to get task")),
            ))
        }
    }
}

/// Get task status only
pub async fn get_task_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(task_id): axum::extract::Path<u32>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!("Getting status for task {}", task_id);

    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);
    let name = format!("task-{task_id}");

    match api.get(&name).await {
        Ok(taskrun) => {
            let status_data = match taskrun.status {
                Some(status) => json!({
                    "phase": status.phase.map(|p| p.to_string()).unwrap_or("Unknown".to_string()),
                    "session_id": status.session_id,
                    "job_name": status.job_name,
                    "attempts": status.attempts,
                    "last_updated": status.last_updated,
                    "message": status.message,
                }),
                None => json!({
                    "phase": "Pending",
                    "message": "Task has not started yet",
                }),
            };

            Ok(Json(ApiResponse {
                success: true,
                message: "Status retrieved successfully".to_string(),
                data: Some(status_data),
            }))
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Task {task_id} not found"))),
        )),
        Err(e) => {
            error!("Failed to get task status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to get task status")),
            ))
        }
    }
}

/// Update task session ID
pub async fn update_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(task_id): axum::extract::Path<u32>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    info!(
        "Updating session for task {}: {}",
        task_id, request.session_id
    );

    let api: Api<TaskRun> = Api::namespaced(state.k8s_client.clone(), &state.namespace);
    let name = format!("task-{task_id}");

    // Get current TaskRun
    let _current_tr = match api.get(&name).await {
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

    // Use Server-Side Apply to update only the session ID
    let patch = json!({
        "apiVersion": "orchestrator.io/v1",
        "kind": "TaskRun",
        "metadata": {
            "name": name,
            "namespace": state.namespace,
        },
        "status": {
            "sessionId": request.session_id,
        }
    });

    let patch_params = kube::api::PatchParams::apply("session-updater").force();

    match api
        .patch_status(&name, &patch_params, &kube::api::Patch::Apply(patch))
        .await
    {
        Ok(_) => {
            info!("Successfully updated session ID for TaskRun: {}", name);
            Ok(Json(ApiResponse {
                success: true,
                message: "Session ID updated successfully".to_string(),
                data: Some(json!({
                    "name": name,
                    "task_id": task_id,
                    "session_id": request.session_id,
                })),
            }))
        }
        Err(e) => {
            error!("Failed to update session ID: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Failed to update session ID")),
            ))
        }
    }
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
