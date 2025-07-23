use anyhow::Result;
use orchestrator_common::models::{CodeRequest, DocsRequest};

use crate::api::ApiClient;
use crate::docs_generator::DocsGenerator;
use crate::output::OutputManager;

/// Handle task command routing
pub async fn handle_task_command(
    command: crate::TaskCommands,
    api_url: &str,
    _output_format: &str,
) -> Result<()> {
    let api_client = ApiClient::new(api_url.to_string());
    let output = OutputManager::new();

    match command {
        crate::TaskCommands::Docs {
            working_directory,
            model,
        } => {
            handle_docs_command(
                &api_client,
                &output,
                working_directory.as_deref(),
                &model,
            ).await
        }
        crate::TaskCommands::Code {
            task_id,
            service,
            repository_url,
            platform_repository_url,
            platform_project_directory,
            branch,
            github_user,
            working_directory,
            model,
            local_tools,
            remote_tools,
            tool_config,
        } => {
            handle_code_command(
                &api_client,
                &output,
                task_id,
                &service,
                repository_url.as_deref(),
                platform_repository_url.as_deref(),
                platform_project_directory.as_deref(),
                &branch,
                github_user.as_deref(),
                working_directory.as_deref(),
                &model,
                local_tools.as_deref(),
                remote_tools.as_deref(),
                &tool_config,
            ).await
        }
    }
}

/// Handle docs command - does local file prep then submits docs generation job
async fn handle_docs_command(
    api_client: &ApiClient,
    output: &OutputManager,
    working_directory: Option<&str>,
    model: &str,
) -> Result<()> {
    output.info("Initializing documentation generator...")?;

    // Do local file preparation and get git info
    let (repo_url, working_dir, source_branch, _generated_docs_branch) =
        DocsGenerator::prepare_for_submission(working_directory)?;

    // Auto-detect GitHub user from git config (simplified for now)
    let github_user = get_github_user().unwrap_or_else(|_| "claude-agent-1".to_string());

    // Create documentation generation request
    let request = DocsRequest {
        repository_url: repo_url.clone(),
        working_directory: working_dir.clone(),
        source_branch: source_branch.clone(),
        model: model.to_string(),
        github_user,
    };

    output.info("Submitting documentation generation job...")?;

    match api_client.submit_docs_generation(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message)?;

                if let Some(data) = response.data {
                    if let Some(taskrun_name) = data.get("taskrun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("TaskRun name: {taskrun_name}"))?;
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"))?;
                        output.info("You can monitor the job with:")?;
                        output.info(&format!("  kubectl -n {namespace} get taskrun"))?;
                    }
                }
            } else {
                output.error(&response.message)?;
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!("Failed to submit documentation generation job: {e}"))?;
            return Err(e);
        }
    }

    Ok(())
}

/// Handle code command - submits code task directly
#[allow(clippy::too_many_arguments)]
async fn handle_code_command(
    api_client: &ApiClient,
    output: &OutputManager,
    task_id: u32,
    service: &str,
    repository_url: Option<&str>,
    platform_repository_url: Option<&str>,
    platform_project_directory: Option<&str>,
    branch: &str,
    github_user: Option<&str>,
    working_directory: Option<&str>,
    model: &str,
    local_tools: Option<&str>,
    remote_tools: Option<&str>,
    tool_config: &str,
) -> Result<()> {
    output.info(&format!("Submitting code task {task_id} for service '{service}'..."))?;

    // Auto-detect target repository URL if not provided
    let repo_url = match repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?,
    };

    // Auto-detect platform repository URL if not provided
    let platform_repo_url = match platform_repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?, // TODO: This should be configurable
    };

    // Auto-detect GitHub user from git config if not provided
    let github_user_name = match github_user {
        Some(user) => user.to_string(),
        None => get_github_user().unwrap_or_else(|_| "claude-agent-1".to_string()),
    };

    // Auto-detect working directory if not provided
    let working_dir = match working_directory {
        Some(wd) => wd.to_string(),
        None => get_working_directory()?,
    };

    // Create code task request
    let request = CodeRequest {
        task_id,
        service: service.to_string(),
        repository_url: repo_url.clone(),
        platform_repository_url: platform_repo_url.clone(),
        platform_project_directory: platform_project_directory.map(|s| s.to_string()),
        branch: branch.to_string(),
        github_user: github_user_name.clone(),
        working_directory: Some(working_dir.clone()),
        model: model.to_string(),
        local_tools: local_tools.map(|s| s.to_string()),
        remote_tools: remote_tools.map(|s| s.to_string()),
        tool_config: tool_config.to_string(),
    };

    output.info(&format!("Target repository: {repo_url}"))?;
    output.info(&format!("Platform repository: {platform_repo_url}"))?;
    output.info(&format!("Branch: {branch}"))?;
    output.info(&format!("Working directory: {working_dir}"))?;
    output.info(&format!("GitHub user: {github_user_name}"))?;

    match api_client.submit_code_task(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message)?;

                if let Some(data) = response.data {
                    if let Some(coderun_name) = data.get("coderun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("CodeRun name: {coderun_name}"))?;
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"))?;
                        output.info("You can monitor the job with:")?;
                        output.info(&format!("  kubectl -n {namespace} get coderun"))?;
                    }
                }
            } else {
                output.error(&response.message)?;
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!("Failed to submit code task: {e}"))?;
            return Err(e);
        }
    }

    Ok(())
}

/// Helper functions for git operations
fn get_git_remote_url() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git remote URL");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[allow(dead_code)]
fn get_current_branch() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Ok("main".to_string());
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn get_working_directory() -> Result<String> {
    use std::process::Command;

    let current_dir = std::env::current_dir()?;
    let repo_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?
        .stdout;
    let repo_root_string = String::from_utf8(repo_root)?;
    let repo_root = repo_root_string.trim();

    let rel_path = current_dir
        .strip_prefix(repo_root)?
        .to_string_lossy()
        .to_string();

    Ok(if rel_path.is_empty() {
        ".".to_string()
    } else {
        rel_path
    })
}

fn get_github_user() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["config", "user.name"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git user.name");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}