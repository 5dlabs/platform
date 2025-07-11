use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

/// Find the Task Master root directory using hybrid approach
/// Priority order:
/// 1. Provided working_directory parameter
/// 2. TASKMASTER_ROOT environment variable
/// 3. Auto-detect by walking up from current directory
/// 4. Default to current directory
pub fn find_taskmaster_root(working_directory: Option<&str>) -> Result<PathBuf> {
    // 1. First check if explicit working directory was provided
    if let Some(dir) = working_directory {
        let path = PathBuf::from(dir);
        if path.join(".taskmaster").exists() {
            return Ok(path);
        }
        // If provided path doesn't have .taskmaster, still use it as starting point for search
        return find_taskmaster_from_path(&path);
    }

    // 2. Check environment variable
    if let Ok(taskmaster_root) = env::var("TASKMASTER_ROOT") {
        let path = PathBuf::from(taskmaster_root);
        if path.join(".taskmaster").exists() {
            return Ok(path);
        }
    }

    // 3. Auto-detect from current directory
    let current = env::current_dir()?;
    find_taskmaster_from_path(&current)
}

/// Helper function to walk up directory tree looking for .taskmaster
fn find_taskmaster_from_path(start: &Path) -> Result<PathBuf> {
    let mut current = start;

    loop {
        let taskmaster_path = current.join(".taskmaster");
        if taskmaster_path.exists() && taskmaster_path.is_dir() {
            return Ok(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // 4. If not found anywhere, default to current directory
    // This allows the orchestrator command to fail with its own error message
    Ok(start.to_path_buf())
}

/// Execute orchestrator CLI command
pub fn execute_orchestrator_command(args: &[&str], working_dir: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("orchestrator");

    // Set working directory if provided
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Add arguments
    for arg in args {
        cmd.arg(arg);
    }

    // Execute command
    let output = cmd
        .output()
        .context("Failed to execute orchestrator command")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Orchestrator command failed: {}", stderr)
    }
}

/// Initialize documentation generation
pub fn init_docs(
    model: &str,
    working_directory: Option<&str>,
    force: bool,
    task_id: Option<u32>,
) -> Result<String> {
    // Validate model parameter early
    if !["sonnet", "opus"].contains(&model) {
        anyhow::bail!("Invalid model '{}' - must be 'sonnet' or 'opus'", model);
    }

    // Find the Task Master root with better error context
    let taskmaster_root = find_taskmaster_root(working_directory)
        .with_context(|| {
            match working_directory {
                Some(dir) => format!("Failed to find Task Master project in directory: {}", dir),
                None => "Failed to find Task Master project. Set TASKMASTER_ROOT env var or specify working_directory".to_string(),
            }
        })?;

    // Verify tasks.json exists before proceeding
    let tasks_json_path = taskmaster_root.join(".taskmaster/tasks/tasks.json");
    if !tasks_json_path.exists() {
        anyhow::bail!(
            "tasks.json not found at {}. Please ensure this is a valid Task Master project with initialized tasks.",
            tasks_json_path.display()
        );
    }

    // Build command arguments with logging
    let mut args = vec!["task", "init-docs", "--model", model];

    if force {
        args.push("--force");
    }

    let task_id_str;
    if let Some(id) = task_id {
        args.push("--task-id");
        task_id_str = id.to_string();
        args.push(&task_id_str);
    }

    // Log what we're about to execute
    eprintln!("INFO: Executing orchestrator command: {:?}", args);
    eprintln!("INFO: Working directory: {}", taskmaster_root.display());
    eprintln!("INFO: Model: {}", model);
    if let Some(id) = task_id {
        eprintln!("INFO: Target task ID: {}", id);
    } else {
        eprintln!("INFO: Target: All tasks");
    }
    eprintln!("INFO: Force overwrite: {}", force);

    // Execute the command with enhanced error context
    execute_orchestrator_command(&args, Some(&taskmaster_root))
        .with_context(|| {
            format!(
                "Failed to execute orchestrator command with args: {:?}. \
                 Ensure the 'orchestrator' CLI is installed and accessible in PATH.",
                args
            )
        })
}

/// Test MCP server connectivity and configuration
pub fn ping_test() -> Result<String> {
    use std::process::Command;

    let mut status_lines = vec![
        "🟢 Orchestrator MCP Server Status".to_string(),
        "".to_string(),
    ];

    // Check environment variables
    status_lines.push("📋 Environment Configuration:".to_string());

    if let Ok(taskmaster_root) = std::env::var("TASKMASTER_ROOT") {
        status_lines.push(format!("  ✅ TASKMASTER_ROOT: {}", taskmaster_root));

        // Check if the directory exists
        let path = std::path::Path::new(&taskmaster_root);
        if path.exists() {
            status_lines.push("  ✅ TASKMASTER_ROOT directory exists".to_string());

            // Check for .taskmaster directory
            let taskmaster_path = path.join(".taskmaster");
            if taskmaster_path.exists() {
                status_lines.push("  ✅ .taskmaster directory found".to_string());

                // Check for tasks.json
                let tasks_json = taskmaster_path.join("tasks/tasks.json");
                if tasks_json.exists() {
                    status_lines.push("  ✅ tasks.json found".to_string());
                } else {
                    status_lines.push("  ⚠️  tasks.json not found".to_string());
                }
            } else {
                status_lines.push("  ⚠️  .taskmaster directory not found".to_string());
            }
        } else {
            status_lines.push("  ❌ TASKMASTER_ROOT directory does not exist".to_string());
        }
    } else {
        status_lines.push("  ⚠️  TASKMASTER_ROOT not set (will auto-detect)".to_string());
    }

    if let Ok(api_url) = std::env::var("ORCHESTRATOR_API_URL") {
        status_lines.push(format!("  ✅ ORCHESTRATOR_API_URL: {}", api_url));
    } else {
        status_lines.push("  ⚠️  ORCHESTRATOR_API_URL not set".to_string());
    }

    status_lines.push("".to_string());

    // Test orchestrator CLI availability
    status_lines.push("🔧 Orchestrator CLI Check:".to_string());

    match Command::new("orchestrator").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            status_lines.push(format!("  ✅ Orchestrator CLI available: {}", version));
        }
        Ok(_) => {
            status_lines.push("  ❌ Orchestrator CLI found but returned error".to_string());
        }
        Err(_) => {
            status_lines.push("  ❌ Orchestrator CLI not found in PATH".to_string());
            status_lines.push("     Install with: cargo install --path orchestrator/orchestrator-cli".to_string());
        }
    }

    status_lines.push("".to_string());

    // Test current directory context
    status_lines.push("📂 Current Context:".to_string());

    if let Ok(current_dir) = std::env::current_dir() {
        status_lines.push(format!("  📁 Current directory: {}", current_dir.display()));

        // Try to find taskmaster root from current directory
        match find_taskmaster_root(None) {
            Ok(root) => {
                status_lines.push(format!("  ✅ Auto-detected Task Master root: {}", root.display()));
            }
            Err(e) => {
                status_lines.push(format!("  ⚠️  Could not auto-detect Task Master root: {}", e));
            }
        }
    }

    status_lines.push("".to_string());
    status_lines.push("🎯 MCP Server Ready for Commands!".to_string());
    status_lines.push("".to_string());
    status_lines.push("Usage examples:".to_string());
    status_lines.push("  init_docs({})                    # Generate docs for all tasks".to_string());
    status_lines.push("  init_docs({model: 'opus'})       # Use specific model".to_string());
    status_lines.push("  init_docs({task_id: 5})          # Generate docs for task 5 only".to_string());

    Ok(status_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_taskmaster_root() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("project");
        let taskmaster_dir = project_root.join(".taskmaster");
        let sub_dir = project_root.join("src").join("components");

        fs::create_dir_all(&taskmaster_dir).unwrap();
        fs::create_dir_all(&sub_dir).unwrap();

        // Test from subdirectory
        let found = find_taskmaster_root(Some(sub_dir.to_str().unwrap())).unwrap();
        assert_eq!(found, project_root);

        // Test from project root
        let found = find_taskmaster_root(Some(project_root.to_str().unwrap())).unwrap();
        assert_eq!(found, project_root);
    }
}