//! Integration tests for Kubernetes client
//!
//! These tests require a running Kubernetes cluster (kind, k3s, etc.)
//! They are ignored by default to avoid CI failures.

use orchestrator_common::models::{JobType, ResourceLimits};
use orchestrator_core::k8s::K8sClient;
use std::collections::BTreeMap;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires kubernetes cluster"]
async fn test_configmap_lifecycle() {
    let client = K8sClient::new().await.expect("Failed to create K8s client");

    let mut data = BTreeMap::new();
    data.insert(
        "task.md".to_string(),
        "# Test Task\nA test task".to_string(),
    );
    data.insert("claude.md".to_string(), "Claude instructions".to_string());

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert("test".to_string(), "true".to_string());

    // Create ConfigMap
    let cm_name = format!("test-cm-{}", chrono::Utc::now().timestamp());
    let cm = client
        .create_configmap(&cm_name, data.clone(), labels)
        .await
        .expect("Failed to create ConfigMap");

    assert_eq!(cm.metadata.name, Some(cm_name.clone()));
    assert_eq!(cm.data, Some(data));

    // Get ConfigMap
    let retrieved = client
        .get_configmap(&cm_name)
        .await
        .expect("Failed to get ConfigMap");
    assert_eq!(retrieved.metadata.name, Some(cm_name.clone()));

    // Delete ConfigMap
    client
        .delete_configmap(&cm_name)
        .await
        .expect("Failed to delete ConfigMap");

    // Verify deletion
    let result = client.get_configmap(&cm_name).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "requires kubernetes cluster"]
async fn test_job_lifecycle() {
    let client = K8sClient::new().await.expect("Failed to create K8s client");

    let job_name = format!("test-job-{}", chrono::Utc::now().timestamp());

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert("job-type".to_string(), "test".to_string());

    let container = client.build_container(
        "test",
        "busybox:latest",
        Some(vec!["sh".to_string()]),
        Some(vec![
            "-c".to_string(),
            "echo 'Hello from test job' && sleep 2".to_string(),
        ]),
        vec![],
        vec![],
        &ResourceLimits::default(),
    );

    // Create Job
    let job = client
        .create_job(&job_name, JobType::Execute, container, vec![], labels)
        .await
        .expect("Failed to create Job");

    assert_eq!(job.metadata.name, Some(job_name.clone()));

    // Wait for completion
    let completed = client
        .wait_for_job_completion(&job_name, Duration::from_secs(30))
        .await
        .expect("Job did not complete in time");

    assert!(completed.status.is_some());
    let status = completed.status.unwrap();
    assert_eq!(status.succeeded, Some(1));

    // Get logs
    let logs = client
        .get_job_logs(&job_name, false, Some(10))
        .await
        .expect("Failed to get job logs");
    assert!(logs.contains("Hello from test job"));

    // Delete Job
    client
        .delete_job(&job_name)
        .await
        .expect("Failed to delete Job");
}

#[tokio::test]
#[ignore = "requires kubernetes cluster"]
async fn test_pvc_creation() {
    let client = K8sClient::new().await.expect("Failed to create K8s client");

    let pvc_name = format!("test-pvc-{}", chrono::Utc::now().timestamp());

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert("microservice".to_string(), "test".to_string());

    // Create PVC
    let pvc = client
        .create_pvc(&pvc_name, "1Gi", Some("local-path"), labels)
        .await
        .expect("Failed to create PVC");

    assert_eq!(pvc.metadata.name, Some(pvc_name.clone()));

    // Get PVC
    let retrieved = client.get_pvc(&pvc_name).await.expect("Failed to get PVC");
    assert_eq!(retrieved.metadata.name, Some(pvc_name.clone()));

    // Note: PVC deletion is not implemented in this test as it may hang
    // if the PVC is not properly released
}

#[tokio::test]
#[ignore = "requires kubernetes cluster"]
async fn test_list_jobs_with_selector() {
    let client = K8sClient::new().await.expect("Failed to create K8s client");

    // Create a test job with specific labels
    let job_name = format!("test-list-job-{}", chrono::Utc::now().timestamp());

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator".to_string());
    labels.insert("test-group".to_string(), "list-test".to_string());

    let container = client.build_container(
        "test",
        "busybox:latest",
        Some(vec!["echo".to_string()]),
        Some(vec!["test".to_string()]),
        vec![],
        vec![],
        &ResourceLimits::default(),
    );

    client
        .create_job(&job_name, JobType::Execute, container, vec![], labels)
        .await
        .expect("Failed to create Job");

    // List jobs with label selector
    let jobs = client
        .list_jobs(Some("test-group=list-test"))
        .await
        .expect("Failed to list jobs");

    assert!(!jobs.is_empty());
    assert!(jobs
        .iter()
        .any(|j| j.metadata.name == Some(job_name.clone())));

    // Cleanup
    client
        .delete_job(&job_name)
        .await
        .expect("Failed to delete Job");
}
