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
    // Find the Task Master root
    let taskmaster_root = find_taskmaster_root(working_directory)?;
    
    // Build command arguments
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
    
    // Execute the command
    execute_orchestrator_command(&args, Some(&taskmaster_root))
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