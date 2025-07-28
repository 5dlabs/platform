use super::config::ControllerConfig;
use super::types::{Context, Result, ssh_secret_name, github_token_secret_name};
use crate::crds::DocsRun;
use k8s_openapi::api::{
    batch::v1::Job,
    core::v1::ConfigMap,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use kube::runtime::controller::Action;
use kube::{ResourceExt};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

pub struct DocsResourceManager<'a> {
    pub jobs: &'a Api<Job>,
    pub configmaps: &'a Api<ConfigMap>,
    pub config: &'a Arc<ControllerConfig>,
    pub ctx: &'a Arc<Context>,
}

impl<'a> DocsResourceManager<'a> {
    pub fn new(
        jobs: &'a Api<Job>, 
        configmaps: &'a Api<ConfigMap>,
        config: &'a Arc<ControllerConfig>,
        ctx: &'a Arc<Context>,
    ) -> Self {
        Self { jobs, configmaps, config, ctx }
    }

    pub async fn reconcile_create_or_update(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!("Creating/updating docs resources for: {}", name);

        // Clean up older versions
        self.cleanup_old_jobs(docs_run).await?;
        self.cleanup_old_configmaps(docs_run).await?;

        // Create ConfigMap FIRST (without owner reference) so Job can mount it
        let cm_name = self.generate_configmap_name(docs_run);
        let configmap = self.create_configmap(docs_run, &cm_name, None)?;

        match self.configmaps.create(&PostParams::default(), &configmap).await {
            Ok(_) => info!("Created ConfigMap: {}", cm_name),
            Err(kube::Error::Api(ae)) if ae.code == 409 => {
                info!("ConfigMap already exists: {}", cm_name);
            }
            Err(e) => return Err(e.into()),
        }

        // Create Job SECOND (now it can successfully mount the existing ConfigMap)
        let job_ref = self.create_job(docs_run, &cm_name).await?;

        // Update ConfigMap with Job as owner (for automatic cleanup on job deletion)
        if let Some(owner_ref) = job_ref {
            self.update_configmap_owner(docs_run, &cm_name, owner_ref).await?;
        }

        Ok(Action::await_change())
    }

    pub async fn cleanup_resources(&self, docs_run: &Arc<DocsRun>) -> Result<Action> {
        let name = docs_run.name_any();
        info!("Cleaning up docs resources for: {}", name);

        // Clean up any remaining jobs and configmaps
        self.cleanup_old_jobs(docs_run).await?;
        self.cleanup_old_configmaps(docs_run).await?;

        Ok(Action::await_change())
    }

    fn generate_configmap_name(&self, _docs_run: &DocsRun) -> String {
        let service_name = "docs-generator".replace('_', "-");
        let context_version = 1; // Docs don't have context versions, always 1
        format!("{service_name}-docs-v{context_version}-files")
    }

    fn create_configmap(
        &self,
        docs_run: &DocsRun,
        name: &str,
        owner_ref: Option<OwnerReference>,
    ) -> Result<ConfigMap> {
        let mut data = BTreeMap::new();

        // Generate all templates for docs
        let templates = super::docs_templates::DocsTemplateGenerator::generate_all_templates(docs_run, self.config)?;
        for (filename, content) in templates {
            data.insert(filename, content);
        }

        let labels = self.create_task_labels(docs_run);
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

    async fn create_job(&self, docs_run: &DocsRun, cm_name: &str) -> Result<Option<OwnerReference>> {
        let job_name = self.generate_job_name(docs_run);
        let job = self.build_job_spec(docs_run, &job_name, cm_name)?;

        match self.jobs.create(&PostParams::default(), &job).await {
            Ok(created_job) => {
                info!("Created docs job: {}", job_name);
                // Update status
                super::docs_status::DocsStatusManager::update_job_started(&Arc::new(docs_run.clone()), self.ctx, &job_name, cm_name).await?;

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

    fn generate_job_name(&self, docs_run: &DocsRun) -> String {
        let resource_name = docs_run.name_any().replace(['_', '.'], "-");
        format!("docs-gen-{resource_name}")
    }

    fn build_job_spec(&self, docs_run: &DocsRun, job_name: &str, cm_name: &str) -> Result<Job> {
        let labels = self.create_task_labels(docs_run);

        // Build volumes for docs (emptyDir, no PVCs)
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

        // EmptyDir workspace volume for docs (no persistence needed)
        volumes.push(json!({
            "name": "workspace",
            "emptyDir": {}
        }));
        volume_mounts.push(json!({
            "name": "workspace", 
            "mountPath": "/workspace"
        }));

        // SSH volumes
        let ssh_volumes = self.generate_ssh_volumes(docs_run);
        volumes.extend(ssh_volumes.volumes);
        volume_mounts.extend(ssh_volumes.volume_mounts);

        let image = "ghcr.io/anthropics/claude-3-5-sonnet-20241022:latest";
        let job_spec = json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_name,
                "labels": labels
            },
            "spec": {
                "backoffLimit": 0,
                "template": {
                    "metadata": {
                        "labels": labels
                    },
                    "spec": {
                        "restartPolicy": "Never",
                        "containers": [{
                            "name": "claude-docs",
                            "image": image,
                            "env": [
                                {
                                    "name": "GITHUB_TOKEN",
                                    "valueFrom": {
                                        "secretKeyRef": {
                                            "name": github_token_secret_name(&docs_run.spec.github_user),
                                            "key": "token"
                                        }
                                    }
                                }
                            ],
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

    fn create_task_labels(&self, docs_run: &DocsRun) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();

        labels.insert("app".to_string(), "orchestrator".to_string());
        labels.insert("component".to_string(), "docs-generator".to_string());
        labels.insert("github-user".to_string(), self.sanitize_label_value(&docs_run.spec.github_user));
        labels.insert("context-version".to_string(), "1".to_string()); // Docs always version 1

        // Docs-specific labels
        labels.insert("task-type".to_string(), "docs".to_string());
        labels.insert("repository".to_string(), self.sanitize_label_value(&docs_run.spec.repository_url));

        labels
    }

    fn generate_ssh_volumes(&self, docs_run: &DocsRun) -> SshVolumes {
        let ssh_secret = ssh_secret_name(&docs_run.spec.github_user);

        let volumes = vec![
            json!({
                "name": "ssh-key",
                "secret": {
                    "secretName": ssh_secret,
                    "defaultMode": 0o600,
                    "items": [{
                        "key": "ssh-privatekey",
                        "path": "id_rsa"
                    }]
                }
            }),
            json!({
                "name": "ssh-config",
                "configMap": {
                    "name": "ssh-config",
                    "defaultMode": 0o644
                }
            })
        ];

        let volume_mounts = vec![
            json!({
                "name": "ssh-key",
                "mountPath": "/home/claude/.ssh",
                "readOnly": true
            }),
            json!({
                "name": "ssh-config", 
                "mountPath": "/etc/ssh",
                "readOnly": true
            })
        ];

        SshVolumes { volumes, volume_mounts }
    }

    async fn update_configmap_owner(
        &self,
        _docs_run: &DocsRun,
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

    async fn cleanup_old_jobs(&self, docs_run: &DocsRun) -> Result<()> {
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-user={}",
            self.sanitize_label_value(&docs_run.spec.github_user)
        ));

        let jobs = self.jobs.list(&list_params).await?;
        
        for job in jobs {
            if let Some(job_name) = job.metadata.name {
                info!("Deleting old docs job: {}", job_name);
                let _ = self.jobs.delete(&job_name, &DeleteParams::default()).await;
            }
        }

        Ok(())
    }

    async fn cleanup_old_configmaps(&self, docs_run: &DocsRun) -> Result<()> {
        let list_params = ListParams::default().labels(&format!(
            "app=orchestrator,component=docs-generator,github-user={}",
            self.sanitize_label_value(&docs_run.spec.github_user)
        ));

        let configmaps = self.configmaps.list(&list_params).await?;
        
        for cm in configmaps {
            if let Some(cm_name) = cm.metadata.name {
                info!("Deleting old docs ConfigMap: {}", cm_name);
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