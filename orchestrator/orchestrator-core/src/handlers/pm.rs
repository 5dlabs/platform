//! PM task submission handler

use crate::k8s::K8sClient;
use crate::services::helm_client::HelmClient;
use axum::{extract::State, http::StatusCode, response::Json};
use orchestrator_common::models::pm_task::PmTaskRequest;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{error, info};

/// Handle PM task submission
pub async fn pm_task_handler(
    State(k8s_client): State<Arc<K8sClient>>,
    State(helm_client): State<Arc<HelmClient>>,
    Json(request): Json<PmTaskRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!(
        "Received PM task submission: task_id={}, service={}",
        request.id, request.service_name
    );

    // Generate a unique name for this task deployment
    let release_name = format!("{}-task-{}", request.service_name, request.id);
    let config_map_name = format!("{}-task-{}-files", request.service_name, request.id);

    // Create ConfigMap with task files
    match create_task_config_map(&k8s_client, &config_map_name, &request).await {
        Ok(_) => info!("Created ConfigMap: {}", config_map_name),
        Err(e) => {
            error!("Failed to create ConfigMap: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Deploy using Helm
    match helm_client.deploy_task(&release_name, &request).await {
        Ok(deployment_info) => {
            info!("Successfully deployed task via Helm: {}", release_name);
            Ok(Json(json!({
                "success": true,
                "release": release_name,
                "config_map": config_map_name,
                "deployment": deployment_info
            })))
        }
        Err(e) => {
            error!("Failed to deploy task via Helm: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create ConfigMap with task files
async fn create_task_config_map(
    k8s_client: &K8sClient,
    name: &str,
    request: &PmTaskRequest,
) -> Result<(), anyhow::Error> {
    let mut data = BTreeMap::new();

    // Add all markdown files to the ConfigMap
    for file in &request.markdown_files {
        data.insert(file.filename.clone(), file.content.clone());
    }

    // Create labels for the ConfigMap
    let mut labels = BTreeMap::new();
    labels.insert("task-id".to_string(), request.id.to_string());
    labels.insert("service".to_string(), request.service_name.clone());
    labels.insert("managed-by".to_string(), "orchestrator".to_string());

    k8s_client
        .create_configmap(name, data, labels)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create ConfigMap: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_map_name_generation() {
        let service_name = "auth-service";
        let task_id = 1001;
        let expected = "auth-service-task-1001-files";
        let actual = format!("{service_name}-task-{task_id}-files");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_release_name_generation() {
        let service_name = "payment-service";
        let task_id = 2001;
        let expected = "payment-service-task-2001";
        let actual = format!("{service_name}-task-{task_id}");
        assert_eq!(actual, expected);
    }
}
