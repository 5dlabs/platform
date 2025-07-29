use super::config::ControllerConfig;
use super::types::{Context, Result, ssh_secret_name, github_token_secret_name};
use crate::crds::CodeRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::{ConfigMap, PersistentVolumeClaim},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::{ResourceExt};
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
        Self { jobs, configmaps, pvcs, config, ctx }
    }

    pub async fn reconcile_create_or_update(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
        let name = code_run.name_any();
        error!("🚀 CODE DEBUG: Creating/updating code resources for: {}", name);

        // Ensure PVC exists for code tasks (persistent workspace)
        let service_name = &code_run.spec.service;
        let pvc_name = format!("workspace-{service_name}");
        error!("📦 CODE DEBUG: Ensuring PVC exists: {}", pvc_name);
        self.ensure_pvc_exists(&pvc_name, service_name).await?;
        error!("✅ CODE DEBUG: PVC check completed");

        // Don't cleanup resources at start - let idempotent creation handle it
        error!("🔄 CODE DEBUG: Using idempotent resource creation (no aggressive cleanup)");

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(code_run);
        error!("📄 CODE DEBUG: Generated ConfigMap name: {}", cm_name);
        
        error!("🔧 CODE DEBUG: Creating ConfigMap template data...");
        let configmap = self.create_configmap(code_run, &cm_name, None)?;
        error!("✅ CODE DEBUG: ConfigMap template created successfully");

        // Always create or update ConfigMap to ensure latest template content
        error!("📤 CODE DEBUG: Attempting to create ConfigMap: {}", cm_name);
        match self.configmaps.create(&PostParams::default(), &configmap).await {
            Ok(_) => {
                error!("✅ CODE DEBUG: Created ConfigMap: {}", cm_name);
            },
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                // ConfigMap exists, update it with latest content
                error!("📝 CODE DEBUG: ConfigMap exists, updating with latest content: {}", cm_name);
                match self.configmaps.replace(&cm_name, &PostParams::default(), &configmap).await {
                    Ok(_) => {
                        error!("✅ CODE DEBUG: Updated ConfigMap: {}", cm_name);
                    },
                    Err(e) => {
                        error!("❌ CODE DEBUG: Failed to update ConfigMap {}: {}", cm_name, e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                error!("❌ CODE DEBUG: Failed to create ConfigMap {}: {}", cm_name, e);
                return Err(e.into());
            }
        }

        // Create Job using idempotent creation (now it can successfully mount the existing ConfigMap)
        error!("🚀 CODE DEBUG: Creating job with ConfigMap: {}", cm_name);
        let job_ref = self.create_or_get_job(code_run, &cm_name).await?;
        error!("✅ CODE DEBUG: Job creation completed");

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            error!("🔗 CODE DEBUG: Updating ConfigMap owner reference");
            self.update_configmap_owner(code_run, &cm_name, owner_ref).await?;
            error!("✅ CODE DEBUG: ConfigMap owner reference updated");
        } else {
            error!("⚠️ CODE DEBUG: No job owner reference to set");
        }

        error!("🎉 CODE DEBUG: Reconciliation completed successfully for: {}", name);
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
        let task_id = code_run.spec.task_id;
        let service_name = code_run.spec.service.replace('_', "-");
        let context_version = code_run.spec.context_version;
        format!("{service_name}-task{task_id}-v{context_version}-files")
    }

    fn create_configmap(
        &self,
        code_run: &CodeRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for code
        let templates = super::code_templates::CodeTemplateGenerator::generate_all_templates(code_run, self.config)?;
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
    async fn create_or_get_job(&self, code_run: &CodeRun, cm_name: &str) -> Result<Option<OwnerReference>> {
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

    async fn create_job(&self, code_run: &CodeRun, cm_name: &str) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(code_run);
        let job = self.build_job_spec(code_run, &job_name, cm_name)?;

        match self.jobs.create(&PostParams::default(), &job).await {
            Ok(created_job) => {
                info!("Created code job: {}", job_name);
                // Update status
                super::code_status::CodeStatusManager::update_job_started(&Arc::new(code_run.clone()), self.ctx, &job_name, cm_name).await?;

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
                    Ok(None)
                }
            }
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!("Job already exists: {}", job_name);
                // Try to get existing job for owner reference
                match self.jobs.get(&job_name).await {
                    Ok(existing_job) => {
                        if let (Some(uid), Some(name)) = (existing_job.metadata.uid, existing_job.metadata.name) {
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
        let uid_suffix = code_run.metadata.uid.as_deref()
            .map(|uid| &uid[..8]) // Use first 8 chars of UID for uniqueness
            .unwrap_or("nouid");
        let task_id = code_run.spec.task_id;
        let context_version = code_run.spec.context_version;
        
        format!("code-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}")
            .replace(['_', '.'], "-")
            .to_lowercase()
    }

    fn build_job_spec(&self, code_run: &CodeRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(code_run);

        // Create owner reference to CodeRun for proper event handling
        let owner_ref = OwnerReference {
            api_version: "orchestrator.platform/v1".to_string(),
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

        // SSH volumes
        let ssh_volumes = self.generate_ssh_volumes(code_run);
        volumes.extend(ssh_volumes.volumes);
        volume_mounts.extend(ssh_volumes.volume_mounts);

        let image = format!("{}:{}", self.config.agent.image.repository, self.config.agent.image.tag);
        
        // Build environment variables for code tasks
        let mut env_vars = vec![
            json!({
                "name": "GITHUB_TOKEN",
                "valueFrom": {
                    "secretKeyRef": {
                        "name": github_token_secret_name(&code_run.spec.github_user),
                        "key": "token"
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
            })
        ];

        // Add code-specific environment variables based on spec
        if let Some(local_tools) = &code_run.spec.local_tools {
            env_vars.push(json!({
                "name": "LOCAL_TOOLS",
                "value": local_tools
            }));
        }

        if let Some(remote_tools) = &code_run.spec.remote_tools {
            env_vars.push(json!({
                "name": "REMOTE_TOOLS", 
                "value": remote_tools
            }));
        }

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
        labels.insert("github-user".to_string(), self.sanitize_label_value(&code_run.spec.github_user));
        labels.insert("context-version".to_string(), code_run.spec.context_version.to_string());

        // Code-specific labels
        labels.insert("task-type".to_string(), "code".to_string());
        labels.insert("task-id".to_string(), code_run.spec.task_id.to_string());
        labels.insert("service".to_string(), self.sanitize_label_value(&code_run.spec.service));

        labels
    }

    fn generate_ssh_volumes(&self, code_run: &CodeRun) -> SshVolumes {
        let ssh_secret = ssh_secret_name(&code_run.spec.github_user);

        let volumes = vec![
            json!({
                "name": "ssh-key",
                "secret": {
                    "secretName": ssh_secret,
                    "defaultMode": 0o644,
                    "items": [{
                        "key": "ssh-privatekey",
                        "path": "id_ed25519"
                    }]
                }
            })
        ];

        let volume_mounts = vec![
            json!({
                "name": "ssh-key",
                "mountPath": "/workspace/.ssh",
                "readOnly": true
            })
        ];

        SshVolumes { volumes, volume_mounts }
    }

    async fn update_configmap_owner(
        &self,
        _code_run: &CodeRun,
        cm_name: &str,
        owner_ref: OwnerReference,
    ) -> Result<()> {
        let mut existing_cm = self.configmaps.get(cm_name).await?;
        
        // Add owner reference
        let owner_refs = existing_cm.metadata.owner_references.get_or_insert_with(Vec::new);
        owner_refs.push(owner_ref);
        
        // Update the ConfigMap
        self.configmaps.replace(cm_name, &PostParams::default(), &existing_cm).await?;
        info!("Updated ConfigMap {} with owner reference", cm_name);
        
        Ok(())
    }

    // Legacy cleanup method for backward compatibility
    async fn cleanup_old_jobs(&self, code_run: &CodeRun) -> Result<()> {
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(&code_run.spec.github_user),
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
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=code-runner,github-user={},service={}",
            self.sanitize_label_value(&code_run.spec.github_user),
            self.sanitize_label_value(&code_run.spec.service)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;
        
        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                info!("Deleting old code ConfigMap: {}", cm_name);
                let _ = self.configmaps.delete(&cm_name, &DeleteParams::default()).await;
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
        let end = chars.iter().rposition(|c| c.is_alphanumeric()).unwrap_or(chars.len().saturating_sub(1));

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