//! Integration tests for Kubernetes client operations
//! 
//! These tests require a Kubernetes cluster to be available.
//! Set SKIP_INTEGRATION_TESTS=1 to skip these tests.

use orchestrator_core::k8s::{K8sClient, K8sError};
use std::collections::BTreeMap;
use std::env;
use std::time::Duration;

#[tokio::test]
async fn test_k8s_client_initialization() {
    if should_skip_tests() {
        return;
    }

    println!("🧪 Testing K8s client initialization...");

    // Test default initialization
    let client = K8sClient::new().await;
    assert!(client.is_ok(), "Failed to create K8s client: {:?}", client);

    let client = client.unwrap();
    assert_eq!(client.namespace(), "default");

    // Test with specific namespace
    let client = K8sClient::with_namespace("test-namespace".to_string()).await;
    assert!(
        client.is_ok(),
        "Failed to create K8s client with namespace: {:?}",
        client
    );

    let client = client.unwrap();
    assert_eq!(client.namespace(), "test-namespace");

    println!("✅ K8s client initialization works");
}

#[tokio::test]
async fn test_configmap_operations() {
    if should_skip_tests() {
        return;
    }

    println!("🧪 Testing ConfigMap operations...");

    let client = K8sClient::new()
        .await
        .expect("Failed to create K8s client");

    let name = format!("test-configmap-{}", uuid::Uuid::new_v4());
    let mut data = BTreeMap::new();
    data.insert("key1".to_string(), "value1".to_string());
    data.insert("key2".to_string(), "value2".to_string());

    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator-test".to_string());

    // Create ConfigMap
    let cm = client
        .create_configmap(&name, data.clone(), labels.clone())
        .await
        .expect("Failed to create ConfigMap");

    assert_eq!(cm.metadata.name, Some(name.clone()));
    assert_eq!(cm.data, Some(data));

    // Get ConfigMap
    let cm = client
        .get_configmap(&name)
        .await
        .expect("Failed to get ConfigMap");

    assert_eq!(cm.metadata.name, Some(name.clone()));
    assert_eq!(cm.metadata.labels, Some(labels));

    // Delete ConfigMap
    client
        .delete_configmap(&name)
        .await
        .expect("Failed to delete ConfigMap");

    // Verify deletion
    let result = client.get_configmap(&name).await;
    assert!(matches!(result, Err(K8sError::NotFound(_))));

    println!("✅ ConfigMap operations work");
}

#[tokio::test]
async fn test_job_operations() {
    if should_skip_tests() {
        return;
    }

    println!("🧪 Testing Job operations...");

    let client = K8sClient::new()
        .await
        .expect("Failed to create K8s client");

    let job_name = format!("test-job-{}", uuid::Uuid::new_v4());
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator-test".to_string());
    labels.insert("test".to_string(), "integration".to_string());

    // Build a simple container
    let container = client.build_container(
        "test-container",
        "busybox:latest",
        Some(vec!["sh".to_string()]),
        Some(vec!["-c".to_string(), "echo 'Hello from test job!' && sleep 2".to_string()]),
        vec![],
        vec![],
        &orchestrator_common::models::ResourceLimits::default(),
    );

    // Create Job
    let job = client
        .create_job(
            &job_name,
            orchestrator_common::models::job::JobType::Execute,
            container,
            vec![],
            labels.clone(),
        )
        .await
        .expect("Failed to create Job");

    assert_eq!(job.metadata.name, Some(job_name.clone()));

    // Get Job
    let job = client.get_job(&job_name).await.expect("Failed to get Job");
    assert_eq!(job.metadata.name, Some(job_name.clone()));

    // List Jobs with label selector
    let jobs = client
        .list_jobs(Some("app=orchestrator-test"))
        .await
        .expect("Failed to list Jobs");
    assert!(!jobs.is_empty());
    assert!(jobs.iter().any(|j| j.metadata.name == Some(job_name.clone())));

    // Wait for completion
    let completed_job = client
        .wait_for_job_completion(&job_name, Duration::from_secs(30))
        .await
        .expect("Job did not complete in time");

    // Check if job succeeded
    if let Some(status) = completed_job.status {
        assert!(
            status.succeeded == Some(1),
            "Job did not succeed: {:?}",
            status
        );
    }

    // Get logs
    let logs = client
        .get_job_logs(&job_name, false, Some(10))
        .await
        .expect("Failed to get job logs");
    assert!(logs.contains("Hello from test job!"));

    // Delete Job
    client
        .delete_job(&job_name)
        .await
        .expect("Failed to delete Job");

    println!("✅ Job operations work");
}

#[tokio::test]
async fn test_pvc_operations() {
    if should_skip_tests() {
        return;
    }

    println!("🧪 Testing PVC operations...");

    let client = K8sClient::new()
        .await
        .expect("Failed to create K8s client");

    let pvc_name = format!("test-pvc-{}", uuid::Uuid::new_v4());
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "orchestrator-test".to_string());

    // Create PVC
    let pvc = client
        .create_pvc(&pvc_name, "1Gi", None, labels.clone())
        .await
        .expect("Failed to create PVC");

    assert_eq!(pvc.metadata.name, Some(pvc_name.clone()));

    // Get PVC
    let pvc = client.get_pvc(&pvc_name).await.expect("Failed to get PVC");
    assert_eq!(pvc.metadata.name, Some(pvc_name.clone()));
    assert_eq!(pvc.metadata.labels, Some(labels));

    // Note: We don't delete the PVC in tests as it might be bound and take time to clean up
    // In a real test environment, you would clean this up properly

    println!("✅ PVC operations work");
}

#[tokio::test]
async fn test_volume_builders() {
    println!("🧪 Testing volume builders...");

    let client = K8sClient::new()
        .await
        .expect("Failed to create K8s client");

    // Test ConfigMap volume
    let cm_spec = orchestrator_common::models::job::VolumeSpec {
        name: "config-volume".to_string(),
        mount_path: "/config".to_string(),
        volume_type: orchestrator_common::models::job::VolumeType::ConfigMap {
            name: "my-config".to_string(),
        },
        read_only: true,
    };

    let volume = client.build_volume(&cm_spec);
    assert_eq!(volume.name, "config-volume");
    assert!(volume.config_map.is_some());

    let mount = client.build_volume_mount(&cm_spec);
    assert_eq!(mount.mount_path, "/config");
    assert_eq!(mount.read_only, Some(true));

    // Test PVC volume
    let pvc_spec = orchestrator_common::models::job::VolumeSpec {
        name: "data-volume".to_string(),
        mount_path: "/data".to_string(),
        volume_type: orchestrator_common::models::job::VolumeType::Pvc {
            claim_name: "my-pvc".to_string(),
        },
        read_only: false,
    };

    let volume = client.build_volume(&pvc_spec);
    assert_eq!(volume.name, "data-volume");
    assert!(volume.persistent_volume_claim.is_some());

    // Test EmptyDir volume
    let empty_spec = orchestrator_common::models::job::VolumeSpec {
        name: "temp-volume".to_string(),
        mount_path: "/tmp".to_string(),
        volume_type: orchestrator_common::models::job::VolumeType::EmptyDir,
        read_only: false,
    };

    let volume = client.build_volume(&empty_spec);
    assert_eq!(volume.name, "temp-volume");
    assert!(volume.empty_dir.is_some());

    println!("✅ Volume builders work");
}

fn should_skip_tests() -> bool {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        println!("⚠️  Skipping integration tests due to SKIP_INTEGRATION_TESTS environment variable");
        return true;
    }

    // Also check if we're in CI without a cluster
    if env::var("CI").is_ok() && env::var("ENABLE_K8S_TESTS").is_err() {
        println!("⚠️  Skipping integration tests in CI (set ENABLE_K8S_TESTS=1 to run)");
        return true;
    }

    false
}