//! Minimal documentation generator for preparing local files before submission

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::info;
use serde_json::Value;

/// Simple docs generator that handles local file preparation
pub struct DocsGenerator;

impl DocsGenerator {
    /// Prepare documentation files and return git info for submission
    pub fn prepare_for_submission(
        working_directory: Option<&str>,
    ) -> Result<(String, String, String, String)> {
        info!("Preparing documentation files for submission...");

        // Auto-detect git repository URL
        let repo_url = Self::get_git_remote_url()?;

        // Auto-detect working directory (relative path from repo root to current dir)
        let working_dir = Self::get_working_directory(working_directory)?;

        // Auto-detect source branch
        let source_branch = Self::get_current_branch()?;

        // Generate unique target branch name with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let target_branch = format!("docs-generation-{}", timestamp);

        info!("Repository: {}", repo_url);
        info!("Working directory: {}", working_dir);
        info!("Source branch: {}", source_branch);
        info!("Target branch: {}", target_branch);

        // Check and commit .taskmaster changes if needed
        Self::check_and_commit_taskmaster_changes(&working_dir, &source_branch)?;

        // Create documentation directory structure and copy task files
        Self::create_docs_structure(&working_dir)?;

        Ok((repo_url, working_dir, source_branch, target_branch))
    }

    fn get_git_remote_url() -> Result<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .context("Failed to get git remote URL")?;

        if !output.status.success() {
            anyhow::bail!("Failed to detect git repository URL. Please specify with --repository-url");
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_working_directory(working_directory: Option<&str>) -> Result<String> {
        if let Some(wd) = working_directory {
            return Ok(wd.to_string());
        }

        let current_dir = std::env::current_dir()?;
        let repo_root = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get git repo root")?
            .stdout;
        let repo_root_string = String::from_utf8(repo_root)?;
        let repo_root = repo_root_string.trim();

        let rel_path = current_dir
            .strip_prefix(repo_root)
            .context("Current directory is not in repo")?
            .to_string_lossy()
            .to_string();

        Ok(if rel_path.is_empty() {
            ".".to_string()
        } else {
            rel_path
        })
    }

    fn get_current_branch() -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get current git branch")?;

        if !output.status.success() {
            return Ok("main".to_string());
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn check_and_commit_taskmaster_changes(working_dir: &str, source_branch: &str) -> Result<()> {
        let taskmaster_path = format!("{}/.taskmaster", working_dir);

        if !Path::new(&taskmaster_path).exists() {
            anyhow::bail!("No .taskmaster directory found in {}", working_dir);
        }

        info!("Checking for uncommitted .taskmaster changes...");

        let status_output = Command::new("git")
            .args(["status", "--porcelain", &taskmaster_path])
            .output()
            .context("Failed to check git status")?;

        if !status_output.stdout.is_empty() {
            info!("Found uncommitted changes in .taskmaster directory");

            Command::new("git")
                .args(["add", &taskmaster_path])
                .status()
                .context("Failed to add .taskmaster files")?;

            Command::new("git")
                .args(["commit", "-m", "chore: auto-commit .taskmaster directory for documentation generation"])
                .status()
                .context("Failed to commit .taskmaster files")?;

            info!("Pushing commit to remote...");
            let push_result = Command::new("git")
                .args(["push", "origin", source_branch])
                .status()
                .context("Failed to push commits")?;

            if !push_result.success() {
                anyhow::bail!("Failed to push .taskmaster commit");
            }

            info!("✓ Auto-committed and pushed .taskmaster directory");
        } else {
            info!("No uncommitted changes in .taskmaster directory");
        }

        Ok(())
    }

    fn create_docs_structure(working_dir: &str) -> Result<()> {
        info!("Creating documentation directory structure...");

        let taskmaster_path = format!("{}/.taskmaster", working_dir);
        let tasks_json_path = format!("{}/tasks/tasks.json", taskmaster_path);

        if !Path::new(&tasks_json_path).exists() {
            anyhow::bail!("No tasks.json found at {}", tasks_json_path);
        }

        let content = fs::read_to_string(&tasks_json_path)
            .context("Failed to read tasks.json")?;

        let json: Value = serde_json::from_str(&content)
            .context("Failed to parse tasks.json")?;

        let tasks = json
            .get("master")
            .and_then(|m| m.get("tasks"))
            .and_then(|t| t.as_array())
            .context("No tasks found in tasks.json")?;

        let docs_dir = format!("{}/docs", taskmaster_path);
        fs::create_dir_all(&docs_dir)
            .context("Failed to create docs directory")?;

        let mut created_count = 0;
        for task in tasks {
            if let Some(task_id) = task.get("id").and_then(|id| id.as_u64()) {
                if let Some(title) = task.get("title").and_then(|t| t.as_str()) {
                    let task_dir = format!("{}/task-{}", docs_dir, task_id);
                    fs::create_dir_all(&task_dir)
                        .context(format!("Failed to create directory for task {}", task_id))?;

                    let source_file = format!("{}/tasks/task_{:03}.txt", taskmaster_path, task_id);
                    let dest_file = format!("{}/task.txt", task_dir);

                    if Path::new(&source_file).exists() {
                        fs::copy(&source_file, &dest_file)
                            .context(format!("Failed to copy task file for task {}", task_id))?;
                        info!("✓ Copied task file for task {}: {}", task_id, title);
                    } else {
                        info!("⚠ No task file found for task {} (expected: {})", task_id, source_file);
                    }

                    created_count += 1;
                }
            }
        }

        info!("✓ Created documentation structure for {} tasks", created_count);
        Ok(())
    }
}