use anyhow::Result;
use common::models::{CodeRequest, DocsRequest};

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
            repository_url,
            source_branch,
            github_user,
        } => {
            handle_docs_command(
                &api_client,
                &output,
                working_directory.as_deref(),
                model.as_deref(),
                repository_url.as_deref(),
                source_branch.as_deref(),
                &github_user,
            )
            .await
        }
        crate::TaskCommands::Code {
            task_id,
            service,
            repository_url,
            docs_repository_url,
            docs_project_directory,
            github_user,
            working_directory,
            model,
            continue_session,
            env,
            env_from_secrets,
        } => {
            handle_code_command(
                &api_client,
                &output,
                task_id,
                &service,
                repository_url.as_deref(),
                docs_repository_url.as_deref(),
                docs_project_directory.as_deref(),
                &github_user,
                working_directory.as_deref(),
                model.as_deref(),
                continue_session,
                env.as_deref(),
                env_from_secrets.as_deref(),
            )
            .await
        }
    }
}

/// Handle docs command - does local file prep then submits docs generation job
async fn handle_docs_command(
    api_client: &ApiClient,
    output: &OutputManager,
    working_directory: Option<&str>,
    model: Option<&str>,
    repository_url: Option<&str>,
    source_branch: Option<&str>,
    github_user: &str,
) -> Result<()> {
    output.info("Initializing documentation generator...");

    // Do local file preparation and get git info (used as fallbacks)
    let (detected_repo_url, detected_working_dir, detected_source_branch, _generated_docs_branch) =
        DocsGenerator::prepare_for_submission(working_directory)?;

    // Use provided parameters or fall back to auto-detected values
    let final_repo_url = repository_url.unwrap_or(&detected_repo_url);
    let final_working_dir = working_directory.unwrap_or(&detected_working_dir);
    let final_source_branch = source_branch.unwrap_or(&detected_source_branch);

    // Create documentation generation request
    let request = DocsRequest {
        repository_url: final_repo_url.to_string(),
        working_directory: final_working_dir.to_string(),
        source_branch: final_source_branch.to_string(),
        model: model.map(|s| s.to_string()),
        github_user: github_user.to_string(),
    };

    output.info("Submitting documentation generation job...");

    match api_client.submit_docs_generation(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message);

                if let Some(data) = response.data {
                    if let Some(taskrun_name) = data.get("taskrun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("TaskRun name: {taskrun_name}"));
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"));
                        output.info("You can monitor the job with:");
                        output.info(&format!("  kubectl -n {namespace} get taskrun"));
                    }
                }
            } else {
                output.error(&response.message);
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!(
                "Failed to submit documentation generation job: {e}"
            ));
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
    docs_repository_url: Option<&str>,
    docs_project_directory: Option<&str>,
    github_user: &str,
    working_directory: Option<&str>,
    model: Option<&str>,
    continue_session: bool,
    env: Option<&str>,
    env_from_secrets: Option<&str>,
) -> Result<()> {
    output.info(&format!(
        "Submitting code task {task_id} for service '{service}'..."
    ));

    // Auto-detect target repository URL if not provided
    let repo_url = match repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?,
    };

    // Auto-detect docs repository URL if not provided
    let docs_repo_url = match docs_repository_url {
        Some(url) => url.to_string(),
        None => get_git_remote_url()?, // TODO: This should be configurable
    };

    // Use provided GitHub user (now required)
    let github_user_name = github_user.to_string();

    // Working directory is now required
    let working_dir = working_directory
        .ok_or_else(|| anyhow::anyhow!("working_directory is required"))?
        .to_string();
        
    // Docs project directory is now required
    let docs_proj_dir = docs_project_directory
        .ok_or_else(|| anyhow::anyhow!("docs_project_directory is required"))?
        .to_string();

    // Auto-detect context version (always 1 for new tasks for now)
    // TODO: Query existing CodeRuns for this task+service and increment
    let context_version = 1u32;
    
    // Set overwrite_memory to false (not user-configurable)
    let overwrite_memory = false;
    
    // Auto-detect current git branch for docs
    let docs_branch = get_git_current_branch()?;

    // Parse environment variables
    let env_map = parse_env_vars(env)?;
    let env_from_secrets_vec = parse_env_from_secrets(env_from_secrets)?;

    // Create code task request
    let request = CodeRequest {
        task_id,
        service: service.to_string(),
        repository_url: repo_url.clone(),
        docs_repository_url: docs_repo_url.clone(),
        docs_project_directory: Some(docs_proj_dir.clone()),
        working_directory: Some(working_dir.clone()),
        model: model.map(|s| s.to_string()),
        github_user: github_user_name.clone(),
        context_version,
        docs_branch: docs_branch.to_string(),
        continue_session,
        overwrite_memory,
        env: env_map,
        env_from_secrets: env_from_secrets_vec,
    };

    output.info(&format!("Target repository: {repo_url}"));
    output.info(&format!("Docs repository: {docs_repo_url}"));
    output.info(&format!("Docs project directory: {docs_proj_dir}"));
    output.info(&format!("Docs branch: {docs_branch} (auto-detected)"));
    output.info(&format!("Working directory: {working_dir}"));
    output.info(&format!("Context version: {context_version} (auto-detected)"));
    output.info(&format!("GitHub user: {github_user_name}"));

    match api_client.submit_code_task(&request).await {
        Ok(response) => {
            if response.success {
                output.success(&response.message);

                if let Some(data) = response.data {
                    if let Some(coderun_name) = data.get("coderun_name").and_then(|n| n.as_str()) {
                        output.info(&format!("CodeRun name: {coderun_name}"));
                    }
                    if let Some(namespace) = data.get("namespace").and_then(|n| n.as_str()) {
                        output.info(&format!("Namespace: {namespace}"));
                        output.info("You can monitor the job with:");
                        output.info(&format!("  kubectl -n {namespace} get coderun"));
                    }
                }
            } else {
                output.error(&response.message);
                anyhow::bail!(response.message);
            }
        }
        Err(e) => {
            output.error(&format!("Failed to submit code task: {e}"));
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

/// Get current git branch
fn get_git_current_branch() -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to get current git branch");
    }

    let branch = String::from_utf8(output.stdout)?.trim().to_string();
    if branch.is_empty() {
        Ok("main".to_string()) // fallback to main if no branch (detached HEAD)
    } else {
        Ok(branch)
    }
}

/// Parse environment variables from comma-separated key=value string
fn parse_env_vars(env_str: Option<&str>) -> Result<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;

    let mut env_map = HashMap::new();

    if let Some(env_str) = env_str {
        for pair in env_str.split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let mut parts = pair.splitn(2, '=');
            let key = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid env format: {}", pair))?;
            let value = parts
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid env format: {}", pair))?;

            env_map.insert(key.to_string(), value.to_string());
        }
    }

    Ok(env_map)
}

/// Parse environment variables from secrets in format: name:secretName:secretKey,...
fn parse_env_from_secrets(
    env_secrets_str: Option<&str>,
) -> Result<Vec<common::models::code_request::SecretEnvVar>> {
    use common::models::code_request::SecretEnvVar;

    let mut secrets = Vec::new();

    if let Some(secrets_str) = env_secrets_str {
        for secret_spec in secrets_str.split(',') {
            let secret_spec = secret_spec.trim();
            if secret_spec.is_empty() {
                continue;
            }

            let parts: Vec<&str> = secret_spec.split(':').collect();
            if parts.len() != 3 {
                anyhow::bail!(
                    "Invalid secret env format: {}. Expected name:secretName:secretKey",
                    secret_spec
                );
            }

            secrets.push(SecretEnvVar {
                name: parts[0].to_string(),
                secret_name: parts[1].to_string(),
                secret_key: parts[2].to_string(),
            });
        }
    }

    Ok(secrets)
}
