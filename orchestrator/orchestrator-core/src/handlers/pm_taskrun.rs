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
use tokio::process::Command;
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

/// Validate GitHub repository permissions for the given user account
async fn validate_github_permissions(
    k8s_client: &Client,
    namespace: &str,
    repository_url: &str,
    secret_name: &str,
    secret_key: &str,
) -> Result<(), AppError> {
    info!(
        "Validating GitHub permissions for repository: {} using secret: {}",
        repository_url, secret_name
    );

    // Extract repository owner and name from URL
    let repo_parts = extract_repo_info(repository_url)?;
    let (owner, repo) = repo_parts;

    // Get GitHub token from Kubernetes secret
    let secret_api: Api<k8s_openapi::api::core::v1::Secret> =
        Api::namespaced(k8s_client.clone(), namespace);

    let secret = secret_api.get(secret_name).await.map_err(|e| {
        AppError::BadRequest(format!("Failed to get GitHub secret '{secret_name}': {e}"))
    })?;

    let token_bytes = secret
        .data
        .and_then(|data| data.get(secret_key).cloned())
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "Secret '{secret_name}' does not contain key '{secret_key}'"
            ))
        })?;

    let token = String::from_utf8(token_bytes.0)
        .map_err(|_| AppError::BadRequest("Invalid token encoding in secret".to_string()))?;

    // Check repository permissions using wget (GitHub REST API)
    let output = Command::new("wget")
        .args([
            "-q", "-O", "-",
            "--header", "Accept: application/vnd.github+json",
            "--header", &format!("Authorization: Bearer {token}"),
            &format!("https://api.github.com/repos/{owner}/{repo}/collaborators")
        ])
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to execute wget command: {e}")))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::BadRequest(format!(
            "GitHub API error: {error_msg}"
        )));
    }

    // Parse collaborators response to find the token owner
    let collaborators: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub API response: {e}")))?;

    // Get the authenticated user's login to find their permissions
    let user_output = Command::new("wget")
        .args([
            "-q", "-O", "-",
            "--header", "Accept: application/vnd.github+json", 
            "--header", &format!("Authorization: Bearer {token}"),
            "https://api.github.com/user"
        ])
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get user info: {e}")))?;

    if !user_output.status.success() {
        let error_msg = String::from_utf8_lossy(&user_output.stderr);
        return Err(AppError::BadRequest(format!(
            "Failed to get user info: {error_msg}"
        )));
    }

    let user_info: serde_json::Value = serde_json::from_slice(&user_output.stdout)
        .map_err(|e| AppError::Internal(format!("Failed to parse user info: {e}")))?;

    let username = user_info["login"]
        .as_str()
        .ok_or_else(|| AppError::Internal("No login found in user info".to_string()))?;

    // Find the user in collaborators and check permissions
    if let Some(collaborators_array) = collaborators.as_array() {
        for collaborator in collaborators_array {
            if let Some(login) = collaborator["login"].as_str() {
                if login == username {
                    let permissions = &collaborator["permissions"];
                    let can_push = permissions["push"].as_bool().unwrap_or(false);

                    if can_push {
                        info!("User '{username}' has push permissions to {owner}/{repo}");
                        return Ok(());
                    } else {
                        return Err(AppError::BadRequest(format!(
                            "User '{username}' does not have push permissions to repository {owner}/{repo}. Required permissions: push=true"
                        )));
                    }
                }
            }
        }
    }

    Err(AppError::BadRequest(format!(
        "User '{username}' is not a collaborator on repository {owner}/{repo}"
    )))
}

/// Extract owner and repository name from GitHub URL
fn extract_repo_info(url: &str) -> Result<(String, String), AppError> {
    // Handle both https://github.com/owner/repo and git@github.com:owner/repo.git formats
    let url = url.trim_end_matches(".git");

    // Find github.com in the URL
    if let Some(github_pos) = url.find("github.com") {
        let after_github = &url[github_pos + "github.com".len()..];

        // Skip the separator (: or /)
        let path = if let Some(stripped) = after_github.strip_prefix(':') {
            stripped
        } else if let Some(stripped) = after_github.strip_prefix('/') {
            stripped
        } else {
            return Err(AppError::BadRequest(format!(
                "Invalid GitHub repository URL format: {url}"
            )));
        };

        // Split by / to get owner and repo
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();
            Ok((owner, repo))
        } else {
            Err(AppError::BadRequest(format!(
                "Invalid GitHub repository URL - missing owner or repo: {url}"
            )))
        }
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid GitHub repository URL - must contain github.com: {url}"
        )))
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

    // Validate GitHub repository permissions if repository is configured
    if let Some(ref repository) = request.repository {
        if let Some(ref auth) = repository.auth {
            if matches!(
                auth.auth_type,
                orchestrator_common::models::pm_task::RepositoryAuthType::Token
            ) {
                info!("Validating GitHub permissions for task {}", request.id);
                if let Err(e) = validate_github_permissions(
                    &state.k8s_client,
                    &state.namespace,
                    &repository.url,
                    &auth.secret_name,
                    &auth.secret_key,
                )
                .await
                {
                    let error_msg = match &e {
                        AppError::BadRequest(msg) => msg.clone(),
                        AppError::Conflict(msg) => msg.clone(),
                        AppError::Internal(msg) => msg.clone(),
                    };
                    error!(
                        "GitHub permission validation failed for task {}: {}",
                        request.id, e
                    );
                    return Err((
                        StatusCode::from(e),
                        Json(ApiResponse::error(&format!(
                            "GitHub permission validation failed: {error_msg}"
                        ))),
                    ));
                }
            }
        }
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
            model: request.model.clone(),
            context_version: 1,
            markdown_files,
            agent_tools,
            repository: request
                .repository
                .map(|repo| crate::crds::taskrun::RepositorySpec {
                    url: repo.url,
                    branch: repo.branch,
                    path: repo.path,
                    auth: repo.auth.map(|auth| crate::crds::taskrun::RepositoryAuth {
                        auth_type: match auth.auth_type {
                            orchestrator_common::models::pm_task::RepositoryAuthType::Token => {
                                crate::crds::taskrun::RepositoryAuthType::Token
                            }
                            orchestrator_common::models::pm_task::RepositoryAuthType::SshKey => {
                                crate::crds::taskrun::RepositoryAuthType::SshKey
                            }
                            orchestrator_common::models::pm_task::RepositoryAuthType::BasicAuth => {
                                crate::crds::taskrun::RepositoryAuthType::BasicAuth
                            }
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

    #[test]
    fn test_extract_repo_info_https() {
        let url = "https://github.com/owner/repo";
        let result = extract_repo_info(url).unwrap();
        assert_eq!(result, ("owner".to_string(), "repo".to_string()));
    }

    #[test]
    fn test_extract_repo_info_https_with_git() {
        let url = "https://github.com/owner/repo.git";
        let result = extract_repo_info(url).unwrap();
        assert_eq!(result, ("owner".to_string(), "repo".to_string()));
    }

    #[test]
    fn test_extract_repo_info_ssh() {
        let url = "git@github.com:owner/repo.git";
        let result = extract_repo_info(url).unwrap();
        assert_eq!(result, ("owner".to_string(), "repo".to_string()));
    }

    #[test]
    fn test_extract_repo_info_invalid_url() {
        let url = "https://gitlab.com/owner/repo";
        let result = extract_repo_info(url);
        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("must contain github.com"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_extract_repo_info_missing_parts() {
        let url = "https://github.com/owner";
        let result = extract_repo_info(url);
        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("missing owner or repo"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_app_error_display() {
        let error = AppError::BadRequest("test message".to_string());
        assert_eq!(format!("{error}"), "Bad Request: test message");

        let error = AppError::Conflict("conflict message".to_string());
        assert_eq!(format!("{error}"), "Conflict: conflict message");

        let error = AppError::Internal("internal message".to_string());
        assert_eq!(format!("{error}"), "Internal Error: internal message");
    }
}
