use anyhow::Result;
use common::models::{CodeRequest, DocsRequest};
use std::path::PathBuf;

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
            local_tools,
            remote_tools,
            context_version,
            prompt_modification,
            docs_branch,
            continue_session,
            overwrite_memory,
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
                local_tools.as_deref(),
                remote_tools.as_deref(),
                context_version,
                prompt_modification.as_deref(),
                &docs_branch,
                continue_session,
                overwrite_memory,
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

    // Debug: Log the received working_directory
    eprintln!("CLI: Received working_directory: {:?}", working_directory);

    // Do local file preparation and get git info (used as fallbacks)
    let (detected_repo_url, detected_working_dir, detected_source_branch, _generated_docs_branch) =
        DocsGenerator::prepare_for_submission(working_directory)?;

    // Debug: Log what was detected
    eprintln!("CLI: Detected working_dir: '{}'", detected_working_dir);

    // Use provided parameters or fall back to auto-detected values
    let final_repo_url = repository_url.unwrap_or(&detected_repo_url);
    let final_working_dir = working_directory.unwrap_or(&detected_working_dir);
    let final_source_branch = source_branch.unwrap_or(&detected_source_branch);

    // Debug: Log the final value being sent to server
    eprintln!("CLI: Final working_dir to send to server: '{}'", final_working_dir);

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
    local_tools: Option<&str>,
    remote_tools: Option<&str>,
    context_version: u32,
    prompt_modification: Option<&str>,
    docs_branch: &str,
    continue_session: bool,
    overwrite_memory: bool,
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

    // Auto-detect working directory if not provided
    let working_dir = match working_directory {
        Some(wd) => wd.to_string(),
        None => get_working_directory()?,
    };

    // Parse environment variables
    let env_map = parse_env_vars(env)?;
    let env_from_secrets_vec = parse_env_from_secrets(env_from_secrets)?;

    // Create code task request
    let request = CodeRequest {
        task_id,
        service: service.to_string(),
        repository_url: repo_url.clone(),
        docs_repository_url: docs_repo_url.clone(),
        docs_project_directory: docs_project_directory.map(std::string::ToString::to_string),
        working_directory: Some(working_dir.clone()),
        model: model.map(|s| s.to_string()),
        github_user: github_user_name.clone(),
        local_tools: local_tools.map(std::string::ToString::to_string),
        remote_tools: remote_tools.map(std::string::ToString::to_string),
        context_version,
        prompt_modification: prompt_modification.map(std::string::ToString::to_string),
        docs_branch: docs_branch.to_string(),
        continue_session,
        overwrite_memory,
        env: env_map,
        env_from_secrets: env_from_secrets_vec,
    };

    output.info(&format!("Target repository: {repo_url}"));
    output.info(&format!("Docs repository: {docs_repo_url}"));
    output.info(&format!("Docs branch: {docs_branch}"));
    output.info(&format!("Working directory: {working_dir}"));
    output.info(&format!("Context version: {context_version}"));
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

/// Handle analyze command
pub fn handle_analyze_command(
    output: String,
    format: String,
    working_directory: Option<String>,
    include_source: bool,
) -> Result<()> {
    use crate::analyzer::CodebaseAnalyzer;

    let work_dir = working_directory
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let analyzer = CodebaseAnalyzer::new(work_dir, include_source);
    let analysis = analyzer.analyze()?;

    match format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&analysis)?;
            std::fs::write(&output, json_output)?;
            println!("✅ Codebase analysis written to: {} (JSON format)", output);
        }
        "single" => {
            let markdown_output = analyzer.generate_single_markdown(&analysis)?;
            std::fs::write(&output, markdown_output)?;
            println!("✅ Codebase analysis written to: {} (Single Markdown)", output);
        }
        "modular" | _ => {
            analyzer.generate_modular_markdown(&analysis, &output)?;
            println!("✅ Modular codebase analysis written to: {}/", output);
        }
    }

    Ok(())
}
