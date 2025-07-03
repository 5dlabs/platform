//! Command handlers for the CLI

use crate::api::ApiClient;
use crate::output::OutputManager;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::time::Duration;
use tracing::error;

/// Task command handlers
pub mod task {
    use super::*;
    use orchestrator_common::models::pm_task::{MarkdownPayload, PmTaskRequest, TaskMasterFile};
    use std::fs;

    /// Submit a PM task with design specification and autonomous prompt
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_pm_task(
        api_client: &ApiClient,
        output: &OutputManager,
        task_json_path: &str,
        task_id: u32,
        design_spec_path: &str,
        prompt_path: Option<&str>,
        service_name: &str,
        agent_name: &str,
        _retry: bool,
    ) -> Result<()> {
        output.info("Reading task files...")?;

        // Read Task Master JSON file
        let tasks_json = fs::read_to_string(task_json_path)
            .with_context(|| format!("Failed to read task JSON file: {task_json_path}"))?;

        let tasks_file: TaskMasterFile = serde_json::from_str(&tasks_json)
            .with_context(|| "Failed to parse Task Master JSON file")?;

        // Extract the specific task by ID
        let task = tasks_file
            .master
            .tasks
            .into_iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task ID {task_id} not found in tasks.json"))?;

        output.info(&format!("Found task: {}", task.title))?;

        // Read design specification markdown
        let design_spec = fs::read_to_string(design_spec_path)
            .with_context(|| format!("Failed to read design spec file: {design_spec_path}"))?;

        // Prepare markdown files
        let mut markdown_files = vec![
            MarkdownPayload {
                content: task_to_markdown(&task),
                filename: "task.md".to_string(),
                file_type: "task".to_string(),
            },
            MarkdownPayload {
                content: design_spec,
                filename: "design-spec.md".to_string(),
                file_type: "design-spec".to_string(),
            },
        ];

        // Read autonomous prompt if provided
        if let Some(prompt_path) = prompt_path {
            let prompt = fs::read_to_string(prompt_path)
                .with_context(|| format!("Failed to read prompt file: {prompt_path}"))?;

            markdown_files.push(MarkdownPayload {
                content: prompt,
                filename: "prompt.md".to_string(),
                file_type: "prompt".to_string(),
            });
        }

        // Extract acceptance criteria from test strategy
        let acceptance_criteria = extract_acceptance_criteria(&task.test_strategy);
        if !acceptance_criteria.is_empty() {
            markdown_files.push(MarkdownPayload {
                content: acceptance_criteria,
                filename: "acceptance-criteria.md".to_string(),
                file_type: "acceptance-criteria".to_string(),
            });
        }

        // Create PM request
        let pm_request = PmTaskRequest::new(
            task,
            service_name.to_string(),
            agent_name.to_string(),
            markdown_files,
        );

        // Submit the task
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Submitting PM task...");
        pb.enable_steady_tick(Duration::from_millis(100));

        // Submit to PM endpoint
        let result = api_client.submit_pm_task(&pm_request).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(data) = response.data {
                    output.success(&format!(
                        "Task submitted successfully! Release: {}",
                        serde_json::to_string(&data).unwrap_or_else(|_| "unknown".to_string())
                    ))?;
                    output.info(&format!("Service: {service_name}"))?;
                    output.info(&format!("Task ID: {}", pm_request.id))?;
                    output.info(&format!("Title: {}", pm_request.title))?;
                    output.info(&format!("Priority: {}", pm_request.priority))?;
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to submit task: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to submit task: {e}"))?;
                Err(e)
            }
        }
    }

    /// Convert task to markdown format
    fn task_to_markdown(task: &orchestrator_common::models::pm_task::Task) -> String {
        let deps = if task.dependencies.is_empty() {
            "None".to_string()
        } else {
            task.dependencies
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let subtasks = if task.subtasks.is_empty() {
            "No subtasks defined.".to_string()
        } else {
            task.subtasks.iter()
                .map(|st| {
                    format!(
                        "### Subtask {}: {}\n**Description:** {}\n**Dependencies:** {}\n**Details:** {}\n**Test Strategy:** {}\n",
                        st.id,
                        st.title,
                        st.description,
                        if st.dependencies.is_empty() { "None".to_string() } else { 
                            st.dependencies.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")
                        },
                        st.details,
                        st.test_strategy
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            r#"# Task {}: {}

**Priority:** {}  
**Status:** {}  
**Dependencies:** {}

## Description
{}

## Implementation Details
{}

## Test Strategy
{}

## Subtasks
{}
"#,
            task.id,
            task.title,
            task.priority,
            task.status,
            deps,
            task.description,
            task.details,
            task.test_strategy,
            subtasks
        )
    }

    /// Extract acceptance criteria from test strategy
    fn extract_acceptance_criteria(test_strategy: &str) -> String {
        // Simple extraction - look for "ACCEPTANCE CRITERIA:" section
        if let Some(start) = test_strategy.find("ACCEPTANCE CRITERIA:") {
            let criteria = &test_strategy[start..];
            // Take until the next major section or end
            if let Some(end) = criteria.find("\n\n") {
                criteria[..end].to_string()
            } else {
                criteria.to_string()
            }
        } else {
            // If no explicit acceptance criteria section, use the whole test strategy
            format!("# Acceptance Criteria\n\n{test_strategy}")
        }
    }

    pub async fn status(
        api_client: &ApiClient,
        output: &OutputManager,
        task_id: &str,
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Getting task status...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.get_task(task_id).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(task) = response.data {
                    output.print_task(&task)?;
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to get task: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to get task: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn list(
        api_client: &ApiClient,
        output: &OutputManager,
        microservice: Option<&str>,
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Listing tasks...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.list_tasks(microservice).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(tasks) = response.data {
                    if tasks.is_empty() {
                        output.info("No tasks found")?;
                    } else {
                        output.info(&format!("Found {} task(s)", tasks.len()))?;
                        output.print_task_list(&tasks)?;
                    }
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to list tasks: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to list tasks: {e}"))?;
                Err(e)
            }
        }
    }
}

/// Job command handlers
pub mod job {
    use super::*;

    pub async fn list(
        api_client: &ApiClient,
        output: &OutputManager,
        microservice: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Listing jobs...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.list_jobs(microservice, status_filter).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(jobs) = response.data {
                    if jobs.is_empty() {
                        output.info("No jobs found")?;
                    } else {
                        output.info(&format!("Found {} job(s)", jobs.len()))?;
                        output.print_job_list(&jobs)?;
                    }
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to list jobs: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to list jobs: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn get(api_client: &ApiClient, output: &OutputManager, job_id: &str) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Getting job details...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.get_job(job_id).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(job) = response.data {
                    output.print_job(&job)?;
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to get job: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to get job: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn logs(
        api_client: &ApiClient,
        output: &OutputManager,
        job_id: &str,
        follow: bool,
    ) -> Result<()> {
        if follow {
            output.info("Following logs... (Press Ctrl+C to stop)")?;

            loop {
                match api_client.get_job_logs(job_id, true).await {
                    Ok(logs) => {
                        if !logs.is_empty() {
                            print!("{logs}");
                            io::stdout().flush().context("Failed to flush stdout")?;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(e) => {
                        error!("Error getting logs: {e}");
                        break;
                    }
                }
            }
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .expect("Failed to set progress bar template"),
            );
            pb.set_message("Getting job logs...");
            pb.enable_steady_tick(Duration::from_millis(100));

            let result = api_client.get_job_logs(job_id, false).await;
            pb.finish_and_clear();

            match result {
                Ok(logs) => {
                    if logs.is_empty() {
                        output.info("No logs available")?;
                    } else {
                        #[allow(clippy::disallowed_macros)]
                        {
                            println!("{logs}");
                        }
                    }
                }
                Err(e) => {
                    output.error(&format!("Failed to get logs: {e}"))?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

/// Config command handlers
pub mod config {
    use super::*;

    pub async fn create(
        api_client: &ApiClient,
        output: &OutputManager,
        name: &str,
        files: &[String],
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Creating ConfigMap...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.create_configmap(name, files).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if response.data.is_some() {
                    output.success(&format!("ConfigMap '{name}' created successfully!"))?;
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to create ConfigMap: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to create ConfigMap: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn get(api_client: &ApiClient, output: &OutputManager, name: &str) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Getting ConfigMap...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.get_configmap(name).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(data) = response.data {
                    output.print_json(&data)?;
                } else if let Some(error) = response.error {
                    output.error(&format!("Failed to get ConfigMap: {}", error.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to get ConfigMap: {e}"))?;
                Err(e)
            }
        }
    }
}

/// Health check command
pub async fn health_check(api_client: &ApiClient, output: &OutputManager) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("Failed to set progress bar template"),
    );
    pb.set_message("Checking service health...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let result = api_client.health_check().await;
    pb.finish_and_clear();

    match result {
        Ok(response) => {
            if let Some(health) = response.data {
                output.success("Service is healthy!")?;
                output.print_json(&health)?;
            } else if let Some(error) = response.error {
                output.error(&format!("Service health check failed: {}", error.message))?;
            }
            Ok(())
        }
        Err(e) => {
            output.error(&format!("Failed to check service health: {e}"))?;
            Err(e)
        }
    }
}
