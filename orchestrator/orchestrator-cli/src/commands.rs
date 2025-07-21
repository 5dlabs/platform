//! Command handlers for the CLI

use crate::api::ApiClient;
use crate::output::OutputManager;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::time::Duration;
use tracing::{error, info};

/// Task command handlers
pub mod task {
    use super::*;
    use orchestrator_common::models::pm_task::{
        AgentToolSpec, MarkdownPayload, PmTaskRequest, RepositorySpec, TaskMasterFile,
    };
    use std::fs;
    use std::path::Path;

    /// Submit a task using simplified Task Master directory structure
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_task_simplified(
        api_client: &ApiClient,
        output: &OutputManager,
        task_id: u32,
        service_name: &str,
        agent_name: &str,
        taskmaster_dir: &str,
        context_files: &[String],
        tool_specs: &[String],
        repo_url: Option<&str>,
        branch: &str,
        github_user: Option<&str>,
        retry: bool,
        model: &str,
    ) -> Result<()> {
        output.info("Preparing task submission...")?;
        info!(
            "Task ID: {}, Service: {}, Model: {}",
            task_id, service_name, model
        );
        info!("Task Master directory: {}", taskmaster_dir);

        // Set up directory paths
        let task_docs_dir = Path::new(taskmaster_dir).join("docs").join(format!("task-{task_id}"));
        let root_docs_dir = Path::new(taskmaster_dir).join("docs");

        // Construct paths
        let tasks_json_path = Path::new(taskmaster_dir).join("tasks/tasks.json");

        // Read Task Master JSON file
        info!("Reading tasks JSON from: {}", tasks_json_path.display());
        let tasks_json = fs::read_to_string(&tasks_json_path).with_context(|| {
            format!(
                "Failed to read task JSON file: {}",
                tasks_json_path.display()
            )
        })?;
        info!("Successfully read tasks JSON file");

        let tasks_file: TaskMasterFile = serde_json::from_str(&tasks_json)
            .with_context(|| "Failed to parse Task Master JSON file")?;

        // Extract the specific task by ID
        let task = tasks_file
            .master
            .tasks
            .into_iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task ID {} not found in tasks.json", task_id))?;

        output.info(&format!("Found task: {}", task.title))?;

        // Prepare markdown files
        let mut markdown_files = vec![];

        // 1. Task-specific files (required to be in task directory)

        // task.md - Always from task directory or generated
        let task_md_path = task_docs_dir.join("task.md");
        if task_md_path.exists() {
            let task_md = fs::read_to_string(&task_md_path).with_context(|| {
                format!("Failed to read task.md: {}", task_md_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: task_md,
                filename: "task.md".to_string(),
                file_type: "task".to_string(),
            });
            output.info(&format!("Using task.md from: {}", task_md_path.display()))?;
        } else {
            // Generate from JSON if no task.md exists
            markdown_files.push(MarkdownPayload {
                content: task_to_markdown(&task),
                filename: "task.md".to_string(),
                file_type: "task".to_string(),
            });
            output.info("Generated task.md from tasks.json")?;
        }

        // prompt.md - Task-specific only
        let prompt_path = task_docs_dir.join("prompt.md");
        if prompt_path.exists() {
            let prompt = fs::read_to_string(&prompt_path)
                .with_context(|| format!("Failed to read prompt: {}", prompt_path.display()))?;
            markdown_files.push(MarkdownPayload {
                content: prompt,
                filename: "prompt.md".to_string(),
                file_type: "prompt".to_string(),
            });
            output.info(&format!("Using prompt.md from: {}", prompt_path.display()))?;
        }

        // acceptance-criteria.md - Task-specific only
        let acceptance_criteria_path = task_docs_dir.join("acceptance-criteria.md");
        if acceptance_criteria_path.exists() {
            let criteria = fs::read_to_string(&acceptance_criteria_path).with_context(|| {
                format!("Failed to read acceptance criteria: {}", acceptance_criteria_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: criteria,
                filename: "acceptance-criteria.md".to_string(),
                file_type: "acceptance-criteria".to_string(),
            });
            output.info(&format!("Using acceptance-criteria.md from: {}", acceptance_criteria_path.display()))?;
        }

        // 2. Files with fallback (check task-specific first, then root)

        // design-spec.md - Task-specific with root fallback
        let task_design_spec_path = task_docs_dir.join("design-spec.md");
        let root_design_spec_path = root_docs_dir.join("design-spec.md");

        if task_design_spec_path.exists() {
            let design_spec = fs::read_to_string(&task_design_spec_path).with_context(|| {
                format!("Failed to read task-specific design spec: {}", task_design_spec_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: design_spec,
                filename: "design-spec.md".to_string(),
                file_type: "design-spec".to_string(),
            });
            output.info(&format!("Using task-specific design-spec.md from: {}", task_design_spec_path.display()))?;
        } else if root_design_spec_path.exists() {
            let design_spec = fs::read_to_string(&root_design_spec_path).with_context(|| {
                format!("Failed to read root design spec: {}", root_design_spec_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: design_spec,
                filename: "design-spec.md".to_string(),
                file_type: "design-spec".to_string(),
            });
            output.info("Using root-level design-spec.md")?;
        }

        // 3. Always from root (project-wide files)

        // CLAUDE.md - Always from root
        let claude_md_path = root_docs_dir.join("CLAUDE.md");
        if claude_md_path.exists() {
            let claude_md = fs::read_to_string(&claude_md_path).with_context(|| {
                format!("Failed to read CLAUDE.md: {}", claude_md_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: claude_md,
                filename: "CLAUDE.md".to_string(),
                file_type: "claude".to_string(),
            });
        }

        // git-guidelines.md - Always from root
        let git_guidelines_path = root_docs_dir.join("git-guidelines.md");
        if git_guidelines_path.exists() {
            let git_guidelines = fs::read_to_string(&git_guidelines_path).with_context(|| {
                format!(
                    "Failed to read git-guidelines.md: {}",
                    git_guidelines_path.display()
                )
            })?;
            markdown_files.push(MarkdownPayload {
                content: git_guidelines,
                filename: "git-guidelines.md".to_string(),
                file_type: "git-guidelines".to_string(),
            });
        }

        // regression-testing.md - From root if exists (optional)
        let regression_testing_path = root_docs_dir.join("regression-testing.md");
        if regression_testing_path.exists() {
            let regression_guide = fs::read_to_string(&regression_testing_path).with_context(|| {
                format!("Failed to read regression testing guide: {}", regression_testing_path.display())
            })?;
            markdown_files.push(MarkdownPayload {
                content: regression_guide,
                filename: "regression-testing.md".to_string(),
                file_type: "context".to_string(),
            });
        }

        // Add any additional context files
        for (idx, context_file) in context_files.iter().enumerate() {
            let content = fs::read_to_string(context_file)
                .with_context(|| format!("Failed to read context file: {context_file}"))?;
            markdown_files.push(MarkdownPayload {
                content,
                filename: format!("context-{}.md", idx + 1),
                file_type: "context".to_string(),
            });
        }

        // Parse agent tools
        let agent_tools = parse_tool_specs(tool_specs)?;

        // Create repository specification if URL provided
        let repository = match (repo_url, github_user) {
            (Some(url), Some(username)) => Some(RepositorySpec {
                url: url.to_string(),
                branch: branch.to_string(),
                github_user: username.to_string(),
                token: None, // Reserved for future use
            }),
            (Some(_), None) => {
                error!("Repository URL provided but no GitHub user specified. Please provide --github-user");
                return Err(anyhow::anyhow!("Repository URL provided but no GitHub user specified. Please provide --github-user"));
            }
            _ => None,
        };

        // Create PM request with model selection
        let pm_request = PmTaskRequest::new_with_repository(
            task,
            service_name.to_string(),
            agent_name.to_string(),
            model.to_string(),
            markdown_files,
            agent_tools,
            repository,
        );

        // Debug: print the request JSON
        if let Ok(json) = serde_json::to_string_pretty(&pm_request) {
            info!("PM Request JSON:\n{}", json);
        }

        // Submit the task
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Submitting task...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.submit_pm_task(&pm_request).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if let Some(data) = response.data {
                    output.success(&format!("Task {task_id} submitted successfully!"))?;
                    output.info(&format!("Service: {service_name}"))?;
                    output.info(&format!("Agent: {agent_name}"))?;
                    if retry {
                        output.info("(Retry attempt)")?;
                    }
                    output.print_json(&data)?;
                } else {
                    output.error(&format!("Failed to submit task: {}", response.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to submit task: {e}"))?;
                Err(e)
            }
        }
    }

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
        model: &str,
    ) -> Result<()> {
        output.info("Reading task files...")?;
        info!("Using Claude model: {}", model);

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

        // Create PM request with model selection
        let pm_request = PmTaskRequest::new(
            task,
            service_name.to_string(),
            agent_name.to_string(),
            model.to_string(),
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
                } else {
                    output.error(&format!("Failed to submit task: {}", response.message))?;
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

    /// Add context to a running task
    pub async fn add_context(
        api_client: &ApiClient,
        output: &OutputManager,
        task_id: u32,
        context: &str,
        is_file: bool,
    ) -> Result<()> {
        let content = if is_file {
            fs::read_to_string(context)
                .with_context(|| format!("Failed to read context file: {context}"))?
        } else {
            context.to_string()
        };

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Adding context to task...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.add_context(task_id, &content).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if response.success {
                    output.success(&format!("Context added to task {task_id} successfully!"))?;
                    if let Some(data) = response.data {
                        output.print_json(&data)?;
                    }
                } else {
                    output.error(&format!("Failed to add context: {}", response.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to add context: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn status(
        api_client: &ApiClient,
        output: &OutputManager,
        task_id: u32,
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Getting task status...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.get_task_status(task_id).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        output.success(&format!("Task {task_id} status:"))?;
                        output.print_json(&data)?;
                    }
                } else {
                    output.error(&format!("Failed to get task status: {}", response.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to get task status: {e}"))?;
                Err(e)
            }
        }
    }

    pub async fn list(
        api_client: &ApiClient,
        output: &OutputManager,
        service: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set progress bar template"),
        );
        pb.set_message("Listing tasks...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let result = api_client.list_tasks(service, status_filter).await;
        pb.finish_and_clear();

        match result {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        output.print_json(&data)?;
                    } else {
                        output.info("No tasks found")?;
                    }
                } else {
                    output.error(&format!("Failed to list tasks: {}", response.message))?;
                }
                Ok(())
            }
            Err(e) => {
                output.error(&format!("Failed to list tasks: {e}"))?;
                Err(e)
            }
        }
    }

    /// Parse tool specifications from CLI arguments
    fn parse_tool_specs(tool_specs: &[String]) -> Result<Vec<AgentToolSpec>> {
        let mut tools = Vec::new();

        for spec in tool_specs {
            let parts: Vec<&str> = spec.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid tool spec format: '{}'. Expected format: 'tool_name:enabled' (e.g., 'bash:true')",
                    spec
                ));
            }

            let name = parts[0].to_string();
            let enabled = match parts[1].to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid enabled value: '{}'. Use true/false",
                        parts[1]
                    ))
                }
            };

            tools.push(AgentToolSpec {
                name,
                enabled,
                config: None,
                restrictions: Vec::new(),
            });
        }

        // If no tools specified, use defaults
        if tools.is_empty() {
            tools = vec![
                AgentToolSpec {
                    name: "bash".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
                AgentToolSpec {
                    name: "edit".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
                AgentToolSpec {
                    name: "read".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
                AgentToolSpec {
                    name: "write".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
                AgentToolSpec {
                    name: "glob".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
                AgentToolSpec {
                    name: "grep".to_string(),
                    enabled: true,
                    config: None,
                    restrictions: Vec::new(),
                },
            ];
        }

        Ok(tools)
    }

    /// Initialize documentation for Task Master tasks using Claude
    pub async fn init_docs(
        api_client: &crate::api::ApiClient,
        output: &OutputManager,
        model: &str,
        repo: Option<&str>,
        source_branch: Option<&str>,
        working_directory: Option<&str>,
        force: bool,
        task_id: Option<u32>,
        _update: bool,
        _update_all: bool,
        dry_run: bool,
        _verbose: bool,
        github_user: &str,
    ) -> Result<()> {
        use std::process::Command;

        output.info("Initializing documentation generator...")?;

        // Auto-detect git repository URL if not provided
        let repo_url = match repo {
            Some(url) => url.to_string(),
            None => {
                let output_bytes = Command::new("git")
                    .args(["remote", "get-url", "origin"])
                    .output()
                    .context("Failed to get git remote URL")?;

                if !output_bytes.status.success() {
                    anyhow::bail!("Failed to detect git repository URL. Please specify with --repo");
                }

                String::from_utf8(output_bytes.stdout)?
                    .trim()
                    .to_string()
            }
        };

        // Auto-detect working directory (relative path from repo root to current dir)
        let current_dir = std::env::current_dir()?;
        let repo_root = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get git repo root")?
            .stdout;
        let repo_root = String::from_utf8(repo_root)?.trim().to_string();

        let rel_path = current_dir.strip_prefix(&repo_root)
            .context("Current directory is not in repo")?
            .to_string_lossy()
            .to_string();

        let working_directory = match working_directory {
            Some(wd) => wd.to_string(),
            None => {
                if rel_path.is_empty() {
                    ".".to_string()
                } else {
                    rel_path
                }
            }
        };

        // Check if tasks.json needs to be committed
        let tasks_json_path = format!("{}/.taskmaster/tasks/tasks.json", working_directory);
        output.info(&format!("Checking status of {}", tasks_json_path))?;

        let status_output = Command::new("git")
            .args(["status", "--porcelain", &tasks_json_path])
            .output()
            .context("Failed to check git status")?;

        let status = String::from_utf8(status_output.stdout)?;
        if !status.is_empty() {
            output.info("tasks.json has changes; committing and pushing...")?;

            // Get current branch
            let current_branch_output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .context("Failed to get current branch")?;
            let current_branch = String::from_utf8(current_branch_output.stdout)?.trim().to_string();

            Command::new("git")
                .args(["add", &tasks_json_path])
                .status()
                .context("Failed to git add tasks.json")?;

            Command::new("git")
                .args(["commit", "-m", "Add/update tasks.json for documentation generation"])
                .status()
                .context("Failed to commit tasks.json")?;

            Command::new("git")
                .args(["push", "origin", &current_branch])
                .status()
                .context("Failed to push tasks.json")?;

            output.success("Successfully committed and pushed tasks.json")?;
        } else {
            output.info("tasks.json is already up to date")?;
        }

        // Auto-detect source branch if not provided
        let source_branch_name = match source_branch {
            Some(branch) => branch.to_string(),
            None => {
                let output_bytes = Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .output()
                    .context("Failed to get current git branch")?;

                if !output_bytes.status.success() {
                    output.warning("Failed to detect current git branch, defaulting to 'main'")?;
                    "main".to_string()
                } else {
                    let branch = String::from_utf8(output_bytes.stdout)?
                        .trim()
                        .to_string();
                    output.info(&format!("Auto-detected current branch: {branch}"))?;
                    branch
                }
            }
        };

        // Generate unique target branch name with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let target_branch_name = format!("docs-generation-{}", timestamp);

        output.info(&format!("Repository: {repo_url}"))?;
        output.info(&format!("Working directory: {working_directory}"))?;
        output.info(&format!("Source branch: {source_branch_name}"))?;
        output.info(&format!("Target branch: {target_branch_name}"))?;

        // Auto-commit .taskmaster directory if it exists and has uncommitted changes
        let taskmaster_path = format!("{}/.taskmaster", working_directory);

        // Check if .taskmaster directory exists
        if std::path::Path::new(&taskmaster_path).exists() {
            output.info("Checking for uncommitted .taskmaster changes...")?;

            // Check git status for .taskmaster directory
            let status_output = Command::new("git")
                .args(["status", "--porcelain", &taskmaster_path])
                .output()
                .context("Failed to check git status")?;

            if !status_output.stdout.is_empty() {
                output.info("Found uncommitted changes in .taskmaster directory")?;

                // Add all .taskmaster files
                let add_result = Command::new("git")
                    .args(["add", &taskmaster_path])
                    .status()
                    .context("Failed to add .taskmaster files")?;

                if !add_result.success() {
                    output.warning("Failed to add .taskmaster files to git")?;
                } else {
                    // Commit the changes
                    let commit_result = Command::new("git")
                        .args(["commit", "-m", "chore: auto-commit .taskmaster directory for documentation generation"])
                        .status()
                        .context("Failed to commit .taskmaster files")?;

                    if commit_result.success() {
                        output.success("Auto-committed .taskmaster directory")?;

                        // Push the commit to ensure Claude gets the latest version
                        output.info("Pushing commit to remote...")?;
                        let push_result = Command::new("git")
                            .args(["push", "origin", &source_branch_name])
                            .status()
                            .context("Failed to push commits")?;

                        if !push_result.success() {
                            return Err(anyhow::anyhow!("Failed to push .taskmaster commit. Claude won't have access to your local tasks.json"));
                        }
                        output.success("Pushed .taskmaster directory to remote")?;

                        // Verify the content matches what we expect
                        output.info("Verifying tasks.json content...")?;
                        let tasks_json_path = format!("{}/tasks/tasks.json", taskmaster_path);
                        if std::path::Path::new(&tasks_json_path).exists() {
                            let content = std::fs::read_to_string(&tasks_json_path)
                                .context("Failed to read tasks.json for verification")?;

                            // Quick content verification
                            if content.contains("Express TypeScript") || content.contains("Node.js") {
                                output.error("WARNING: tasks.json contains Node.js/Express content!")?;
                                output.error("This appears to be an old version. Documentation will be incorrect.")?;
                                output.error("Please update your tasks.json with the correct project tasks.")?;
                                return Err(anyhow::anyhow!("Outdated tasks.json detected"));
                            }

                            // Show first task for verification
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(first_task) = json.get("master")
                                    .and_then(|m| m.get("tasks"))
                                    .and_then(|t| t.get(0))
                                    .and_then(|t| t.get("title"))
                                    .and_then(|t| t.as_str()) {
                                    output.info(&format!("✓ First task verified: \"{}\"", first_task))?;
                                }
                            }
                        }
                    } else {
                        output.warning("No changes to commit in .taskmaster directory")?;
                    }
                }
            } else {
                // No uncommitted changes, but we should verify the committed version
                output.info("No uncommitted changes in .taskmaster directory")?;

                // Still verify the committed content
                let tasks_json_path = format!("{}/tasks/tasks.json", taskmaster_path);
                if std::path::Path::new(&tasks_json_path).exists() {
                    let content = std::fs::read_to_string(&tasks_json_path)
                        .context("Failed to read tasks.json for verification")?;

                    if content.contains("Express TypeScript") || content.contains("Node.js") {
                        output.error("WARNING: Your committed tasks.json contains old Node.js/Express content!")?;
                        output.error("Please update and commit the correct tasks for your project.")?;
                        return Err(anyhow::anyhow!("Outdated tasks.json in repository"));
                    }
                }
            }
        }

        // Create documentation directory structure and placeholder files
        output.info("Creating documentation directory structure...")?;

        // Read tasks.json to get all task IDs
        let tasks_json_path = format!("{}/tasks/tasks.json", taskmaster_path);
        if std::path::Path::new(&tasks_json_path).exists() {
            let content = std::fs::read_to_string(&tasks_json_path)
                .context("Failed to read tasks.json for directory creation")?;

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tasks) = json.get("master").and_then(|m| m.get("tasks")).and_then(|t| t.as_array()) {
                    let docs_dir = format!("{}/docs", taskmaster_path);
                    std::fs::create_dir_all(&docs_dir)
                        .context("Failed to create docs directory")?;

                    let mut created_count = 0;
                    for task in tasks {
                        if let Some(task_id) = task.get("id").and_then(|id| id.as_u64()) {
                            if let Some(title) = task.get("title").and_then(|t| t.as_str()) {
                                // Create task directory
                                let task_dir = format!("{}/task-{}", docs_dir, task_id);
                                std::fs::create_dir_all(&task_dir)
                                    .context(format!("Failed to create directory for task {}", task_id))?;

                                // Copy individual task file from tasks/task_XXX.txt to docs/task-{id}/task.txt
                                let source_file = format!("{}/tasks/task_{:03}.txt", taskmaster_path, task_id);
                                let dest_file = format!("{}/task.txt", task_dir);

                                if std::path::Path::new(&source_file).exists() {
                                    std::fs::copy(&source_file, &dest_file)
                                        .context(format!("Failed to copy task file for task {}", task_id))?;
                                    output.info(&format!("✓ Copied task file for task {}: {}", task_id, title))?;
                                } else {
                                    output.warning(&format!("⚠ No task file found for task {} (expected: {})", task_id, source_file))?;
                                }

                                created_count += 1;
                            }
                        }
                    }

                    output.success(&format!("Created documentation structure for {} tasks", created_count))?;
                    output.info("📋 Individual task files copied - Claude will generate all documentation in one commit")?;
                }
            }
        }

        // Submit documentation generation job to Kubernetes
        if dry_run {
            output.info("DRY RUN: Would submit documentation generation job with:")?;
            output.info(&format!("  Repository: {repo_url}"))?;
            output.info(&format!("  Working dir: {working_directory}"))?;
            output.info(&format!("  Source branch: {source_branch_name}"))?;
            output.info(&format!("  Target branch: {target_branch_name}"))?;
            output.info(&format!("  Model: {model}"))?;
            if let Some(id) = task_id {
                output.info(&format!("  Task ID: {id}"))?;
            }
            output.success("DRY RUN complete - no job submitted")?;
            return Ok(());
        }

        // Use provided GitHub user (passed as parameter with default "pm0-5dlabs")

        // Create documentation generation request
        use orchestrator_common::models::pm_task::DocsGenerationRequest;

        let request = DocsGenerationRequest {
            repository_url: repo_url.clone(),
            working_directory: working_directory.clone(),
            source_branch: source_branch_name.to_string(),
            target_branch: target_branch_name.to_string(),
            service_name: "docs-generator".to_string(),
            agent_name: format!("claude-docs-{model}"),
            model: model.to_string(),
            github_user: github_user.to_string(),
            task_id,
            force,
            dry_run: false, // The API dry_run is different from CLI dry_run
        };

        output.info("Submitting documentation generation job...")?;

        match api_client.submit_docs_generation(&request).await {
            Ok(response) => {
                if response.success {
                    output.success(&response.message)?;
                    let mut namespace = "orchestrator".to_string();

                    if let Some(data) = response.data {
                        if let Some(taskrun_name) = data.get("taskrun_name").and_then(|n| n.as_str()) {
                            output.info(&format!("TaskRun name: {taskrun_name}"))?;
                        }
                        if let Some(ns) = data.get("namespace").and_then(|n| n.as_str()) {
                            namespace = ns.to_string();
                            output.info(&format!("Namespace: {namespace}"))?;
                        }
                    }
                    output.info("You can monitor the job with:")?;
                    output.info(&format!("  kubectl -n {namespace} get taskrun"))?;
                } else {
                    output.error(&response.message)?;
                    return Err(anyhow::anyhow!(response.message));
                }
            }
            Err(e) => {
                output.error(&format!("Failed to submit documentation generation job: {e}"))?;
                return Err(e);
            }
        }
        Ok(())
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
