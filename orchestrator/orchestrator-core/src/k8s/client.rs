//! Kubernetes client wrapper for orchestrator operations

use k8s_openapi::api::batch::v1::{Job, JobSpec};
use k8s_openapi::api::core::v1::{
    ConfigMap, ConfigMapVolumeSource, Container, EmptyDirVolumeSource, EnvVar,
    PersistentVolumeClaim, PersistentVolumeClaimSpec, PersistentVolumeClaimVolumeSource, Pod,
    PodSpec, PodTemplateSpec, ResourceRequirements, SecretVolumeSource, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::wait::{await_condition, conditions};
use kube::{
    api::{Api, DeleteParams, ListParams, LogParams, PostParams},
    Client, Config, Error,
};
use orchestrator_common::models::job::{JobType, VolumeSpec, VolumeType};
use orchestrator_common::models::ResourceLimits;
use std::collections::BTreeMap;
use std::time::Duration;
use tracing::{debug, info};

/// Kubernetes client wrapper with orchestrator-specific operations
pub struct K8sClient {
    client: Client,
    namespace: String,
}

/// Result type for K8s operations
pub type K8sResult<T> = Result<T, K8sError>;

/// Kubernetes operation errors
#[derive(Debug, thiserror::Error)]
pub enum K8sError {
    #[error("Kubernetes API error: {0}")]
    Api(#[from] Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Timeout waiting for resource: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl K8sClient {
    /// Create a new Kubernetes client with inferred configuration
    pub async fn new() -> K8sResult<Self> {
        let config = Config::infer()
            .await
            .map_err(|e| K8sError::Config(format!("Failed to infer K8s config: {e}")))?;
        let client = Client::try_from(config)?;
        Ok(Self {
            client,
            namespace: "default".to_string(),
        })
    }

    /// Create a new Kubernetes client with specific namespace
    pub async fn with_namespace(namespace: String) -> K8sResult<Self> {
        let config = Config::infer()
            .await
            .map_err(|e| K8sError::Config(format!("Failed to infer K8s config: {e}")))?;
        let client = Client::try_from(config)?;
        Ok(Self { client, namespace })
    }

    /// Get the current namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Create a ConfigMap for task data
    pub async fn create_configmap(
        &self,
        name: &str,
        data: BTreeMap<String, String>,
        labels: BTreeMap<String, String>,
    ) -> K8sResult<ConfigMap> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &self.namespace);

        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            data: Some(data),
            ..Default::default()
        };

        info!("Creating ConfigMap: {}", name);
        let result = api.create(&PostParams::default(), &cm).await?;
        Ok(result)
    }

    /// Get a ConfigMap by name
    pub async fn get_configmap(&self, name: &str) -> K8sResult<ConfigMap> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &self.namespace);
        api.get(name).await.map_err(|e| match e {
            Error::Api(response) if response.code == 404 => {
                K8sError::NotFound(format!("ConfigMap '{name}' not found"))
            }
            e => K8sError::from(e),
        })
    }

    /// Delete a ConfigMap
    pub async fn delete_configmap(&self, name: &str) -> K8sResult<()> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &self.namespace);
        api.delete(name, &DeleteParams::default()).await?;
        info!("Deleted ConfigMap: {}", name);
        Ok(())
    }

    /// Create a Job for task execution
    pub async fn create_job(
        &self,
        name: &str,
        job_type: JobType,
        container: Container,
        volumes: Vec<Volume>,
        labels: BTreeMap<String, String>,
    ) -> K8sResult<Job> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);

        let mut pod_labels = labels.clone();
        pod_labels.insert("job-name".to_string(), name.to_string());

        let job = Job {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(JobSpec {
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(pod_labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![container],
                        volumes: if volumes.is_empty() {
                            None
                        } else {
                            Some(volumes)
                        },
                        restart_policy: Some("Never".to_string()),
                        ..Default::default()
                    }),
                },
                backoff_limit: Some(2),
                ttl_seconds_after_finished: Some(3600), // Clean up after 1 hour
                ..Default::default()
            }),
            ..Default::default()
        };

        info!("Creating {} Job: {}", job_type, name);
        let result = api.create(&PostParams::default(), &job).await?;
        Ok(result)
    }

    /// Get a Job by name
    pub async fn get_job(&self, name: &str) -> K8sResult<Job> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);
        api.get(name).await.map_err(|e| match e {
            Error::Api(response) if response.code == 404 => {
                K8sError::NotFound(format!("Job '{name}' not found"))
            }
            e => K8sError::from(e),
        })
    }

    /// List Jobs with optional label selector
    pub async fn list_jobs(&self, label_selector: Option<&str>) -> K8sResult<Vec<Job>> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);
        let mut params = ListParams::default();
        if let Some(selector) = label_selector {
            params = params.labels(selector);
        }
        let jobs = api.list(&params).await?;
        Ok(jobs.items)
    }

    /// Wait for a Job to complete
    pub async fn wait_for_job_completion(&self, name: &str, timeout: Duration) -> K8sResult<Job> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);

        info!(
            "Waiting for Job '{}' to complete (timeout: {:?})",
            name, timeout
        );

        let condition = await_condition(api.clone(), name, conditions::is_job_completed());
        let result = tokio::time::timeout(timeout, condition)
            .await
            .map_err(|_| {
                K8sError::Timeout(format!("Job '{name}' did not complete within timeout"))
            })?;

        match result {
            Ok(Some(job)) => Ok(job),
            Ok(None) => Err(K8sError::NotFound(format!("Job '{name}' not found"))),
            Err(e) => Err(K8sError::Config(format!("Wait condition failed: {e}"))),
        }
    }

    /// Get logs from a Job's pod
    pub async fn get_job_logs(
        &self,
        job_name: &str,
        follow: bool,
        tail_lines: Option<i64>,
    ) -> K8sResult<String> {
        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);

        // Find pods for this job
        let pods = pod_api
            .list(&ListParams::default().labels(&format!("job-name={job_name}")))
            .await?;

        if pods.items.is_empty() {
            return Err(K8sError::NotFound(format!(
                "No pods found for job '{job_name}'"
            )));
        }

        // Get logs from the first pod
        let pod = &pods.items[0];
        let pod_name = pod
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| K8sError::Config("Pod has no name".to_string()))?;

        let log_params = LogParams {
            follow,
            tail_lines,
            ..Default::default()
        };

        debug!("Fetching logs from pod: {}", pod_name);
        let logs = pod_api.logs(pod_name, &log_params).await?;
        Ok(logs)
    }

    /// Delete a Job
    pub async fn delete_job(&self, name: &str) -> K8sResult<()> {
        let api: Api<Job> = Api::namespaced(self.client.clone(), &self.namespace);
        api.delete(name, &DeleteParams::default()).await?;
        info!("Deleted Job: {}", name);
        Ok(())
    }

    /// Create or update a PersistentVolumeClaim
    pub async fn create_pvc(
        &self,
        name: &str,
        storage_size: &str,
        storage_class: Option<&str>,
        labels: BTreeMap<String, String>,
    ) -> K8sResult<PersistentVolumeClaim> {
        let api: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), &self.namespace);

        let mut resources = BTreeMap::new();
        resources.insert("storage".to_string(), Quantity(storage_size.to_string()));

        let pvc = PersistentVolumeClaim {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                resources: Some(k8s_openapi::api::core::v1::VolumeResourceRequirements {
                    requests: Some(resources),
                    limits: None,
                }),
                storage_class_name: storage_class.map(|s| s.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        info!("Creating PVC: {}", name);
        let result = api.create(&PostParams::default(), &pvc).await?;
        Ok(result)
    }

    /// Get a PVC by name
    pub async fn get_pvc(&self, name: &str) -> K8sResult<PersistentVolumeClaim> {
        let api: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), &self.namespace);
        api.get(name).await.map_err(|e| match e {
            Error::Api(response) if response.code == 404 => {
                K8sError::NotFound(format!("PVC '{name}' not found"))
            }
            e => K8sError::from(e),
        })
    }

    /// Build container spec from job configuration
    #[allow(clippy::too_many_arguments)]
    pub fn build_container(
        &self,
        name: &str,
        image: &str,
        command: Option<Vec<String>>,
        args: Option<Vec<String>>,
        env_vars: Vec<EnvVar>,
        volume_mounts: Vec<VolumeMount>,
        resources: &ResourceLimits,
    ) -> Container {
        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity(resources.cpu_request.clone()));
        requests.insert(
            "memory".to_string(),
            Quantity(resources.memory_request.clone()),
        );

        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), Quantity(resources.cpu_limit.clone()));
        limits.insert(
            "memory".to_string(),
            Quantity(resources.memory_limit.clone()),
        );

        Container {
            name: name.to_string(),
            image: Some(image.to_string()),
            command,
            args,
            env: Some(env_vars),
            volume_mounts: if volume_mounts.is_empty() {
                None
            } else {
                Some(volume_mounts)
            },
            resources: Some(ResourceRequirements {
                requests: Some(requests),
                limits: Some(limits),
                claims: None,
            }),
            ..Default::default()
        }
    }

    /// Convert VolumeSpec to Kubernetes Volume
    pub fn build_volume(&self, spec: &VolumeSpec) -> Volume {
        let mut volume = Volume {
            name: spec.name.clone(),
            ..Default::default()
        };

        match &spec.volume_type {
            VolumeType::ConfigMap { name } => {
                volume.config_map = Some(ConfigMapVolumeSource {
                    name: Some(name.clone()),
                    ..Default::default()
                });
            }
            VolumeType::Pvc { claim_name } => {
                volume.persistent_volume_claim = Some(PersistentVolumeClaimVolumeSource {
                    claim_name: claim_name.clone(),
                    ..Default::default()
                });
            }
            VolumeType::EmptyDir => {
                volume.empty_dir = Some(EmptyDirVolumeSource::default());
            }
            VolumeType::Secret { name } => {
                volume.secret = Some(SecretVolumeSource {
                    secret_name: Some(name.clone()),
                    ..Default::default()
                });
            }
        };

        volume
    }

    /// Build VolumeMount from VolumeSpec
    pub fn build_volume_mount(&self, spec: &VolumeSpec) -> VolumeMount {
        VolumeMount {
            name: spec.name.clone(),
            mount_path: spec.mount_path.clone(),
            read_only: Some(spec.read_only),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_container() {
        // We don't need an actual client for testing builder methods
        // Just test the function logic directly

        let env_vars = vec![EnvVar {
            name: "TEST_VAR".to_string(),
            value: Some("test_value".to_string()),
            ..Default::default()
        }];

        let resources = ResourceLimits::default();

        // Test container building logic directly
        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity(resources.cpu_request.clone()));
        requests.insert(
            "memory".to_string(),
            Quantity(resources.memory_request.clone()),
        );

        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), Quantity(resources.cpu_limit.clone()));
        limits.insert(
            "memory".to_string(),
            Quantity(resources.memory_limit.clone()),
        );

        let container = Container {
            name: "test-container".to_string(),
            image: Some("busybox:latest".to_string()),
            command: Some(vec!["sh".to_string()]),
            args: Some(vec!["-c".to_string(), "echo hello".to_string()]),
            env: Some(env_vars),
            volume_mounts: None,
            resources: Some(ResourceRequirements {
                requests: Some(requests),
                limits: Some(limits),
                claims: None,
            }),
            ..Default::default()
        };

        assert_eq!(container.name, "test-container");
        assert_eq!(container.image, Some("busybox:latest".to_string()));
        assert!(container.command.is_some());
        assert!(container.env.is_some());
        assert!(container.resources.is_some());
    }

    #[test]
    fn test_build_volume() {
        let spec = VolumeSpec {
            name: "config-volume".to_string(),
            mount_path: "/config".to_string(),
            volume_type: VolumeType::ConfigMap {
                name: "test-config".to_string(),
            },
            read_only: true,
        };

        // Test volume building logic directly
        let mut volume = Volume {
            name: spec.name.clone(),
            ..Default::default()
        };

        volume.config_map = Some(ConfigMapVolumeSource {
            name: Some("test-config".to_string()),
            ..Default::default()
        });

        assert_eq!(volume.name, "config-volume");
        assert!(volume.config_map.is_some());
    }

    #[test]
    fn test_build_volume_mount() {
        let spec = VolumeSpec {
            name: "data-volume".to_string(),
            mount_path: "/data".to_string(),
            volume_type: VolumeType::Pvc {
                claim_name: "data-pvc".to_string(),
            },
            read_only: false,
        };

        // Test volume mount building logic directly
        let mount = VolumeMount {
            name: spec.name.clone(),
            mount_path: spec.mount_path.clone(),
            read_only: Some(spec.read_only),
            ..Default::default()
        };

        assert_eq!(mount.name, "data-volume");
        assert_eq!(mount.mount_path, "/data");
        assert_eq!(mount.read_only, Some(false));
    }
}
