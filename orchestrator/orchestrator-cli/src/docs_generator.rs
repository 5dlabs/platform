//! Documentation generator for Task Master tasks using Claude CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::Write;
use tracing::debug;
use crate::output::OutputManager;

/// Task structure from Task Master
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub dependencies: Vec<u32>,
    #[serde(default)]
    pub priority: String,
    #[serde(rename = "testStrategy", default)]
    pub test_strategy: String,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u32,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub dependencies: Vec<u32>,
    #[serde(rename = "testStrategy", default)]
    pub test_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TasksFile {
    master: TaskContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskContext {
    tasks: Vec<Task>,
}

/// Document types that will be generated
#[derive(Debug, Clone, Copy)]
enum DocType {
    Task,
    Prompt,
    DesignSpec,
    AcceptanceCriteria,
}

impl DocType {
    fn filename(&self) -> &'static str {
        match self {
            DocType::Task => "task.md",
            DocType::Prompt => "prompt.md",
            DocType::DesignSpec => "design-spec.md",
            DocType::AcceptanceCriteria => "acceptance-criteria.md",
        }
    }

    fn get_prompt_filename(&self) -> &'static str {
        match self {
            DocType::Task => "task_md_prompt.txt",
            DocType::Prompt => "prompt_md_prompt.txt",
            DocType::DesignSpec => "design_spec_prompt.txt",
            DocType::AcceptanceCriteria => "acceptance_criteria_prompt.txt",
        }
    }
}

/// Documentation generator using Claude CLI
pub struct DocsGenerator<'a> {
    taskmaster_path: PathBuf,
    model: String,
    output: &'a OutputManager,
}

impl<'a> DocsGenerator<'a> {
    pub fn new(taskmaster_dir: &str, model: &str, output: &'a OutputManager) -> Result<Self> {
        let taskmaster_path = PathBuf::from(taskmaster_dir);
        if !taskmaster_path.exists() {
            anyhow::bail!("Task Master directory not found: {}", taskmaster_dir);
        }

        Ok(Self {
            taskmaster_path,
            model: model.to_string(),
            output,
        })
    }

    /// Generate documentation for all tasks
    pub async fn generate_all_docs(&self, force: bool, dry_run: bool, verbose: bool) -> Result<()> {
        let tasks = self.load_tasks()?;
        
        self.output.info(&format!("Found {} tasks to process", tasks.len()))?;

        for task in tasks {
            self.generate_task_docs(task.id, force, dry_run, verbose).await?;
        }

        Ok(())
    }

    /// Generate documentation for a specific task
    pub async fn generate_task_docs(&self, task_id: u32, force: bool, dry_run: bool, verbose: bool) -> Result<()> {
        let tasks = self.load_tasks()?;
        let task = tasks.iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))?;

        let task_dir = self.taskmaster_path.join("docs").join(format!("task-{task_id}"));

        if task_dir.exists() && !force {
            self.output.warning(&format!("Documentation already exists for task {task_id}. Use --force to overwrite."))?;
            return Ok(());
        }

        if dry_run {
            self.output.info(&format!("DRY RUN: Would generate docs for task {}: {}", task_id, task.title))?;
            self.output.info(&format!("DRY RUN: Would create directory: {}", task_dir.display()))?;
            for doc_type in &[DocType::Task, DocType::Prompt, DocType::DesignSpec, DocType::AcceptanceCriteria] {
                self.output.info(&format!("DRY RUN: Would generate: {}/{}", task_dir.display(), doc_type.filename()))?;
            }
            return Ok(());
        }

        // Create task directory
        fs::create_dir_all(&task_dir)
            .with_context(|| format!("Failed to create directory: {}", task_dir.display()))?;

        // Generate each document type
        for doc_type in &[DocType::Task, DocType::Prompt, DocType::DesignSpec, DocType::AcceptanceCriteria] {
            if verbose {
                self.output.info(&format!("Generating {} for task {}...", doc_type.filename(), task_id))?;
            }
            let content = self.generate_document(task, *doc_type, verbose).await?;
            
            let file_path = task_dir.join(doc_type.filename());
            fs::write(&file_path, content)
                .with_context(|| format!("Failed to write {}", file_path.display()))?;
            
            if verbose {
                self.output.info(&format!("Created: {}", file_path.display()))?;
            }
        }

        self.output.success(&format!("✓ Generated documentation for task {task_id}"))?;
        Ok(())
    }

    /// Load tasks from tasks.json
    fn load_tasks(&self) -> Result<Vec<Task>> {
        let tasks_path = self.taskmaster_path.join("tasks").join("tasks.json");
        let content = fs::read_to_string(&tasks_path)
            .with_context(|| format!("Failed to read tasks.json from {}", tasks_path.display()))?;
        
        let tasks_file: TasksFile = serde_json::from_str(&content)
            .with_context(|| "Failed to parse tasks.json")?;
        
        Ok(tasks_file.master.tasks)
    }

    /// Generate a specific document using Claude
    async fn generate_document(&self, task: &Task, doc_type: DocType, verbose: bool) -> Result<String> {
        // Load prompt template
        let prompt_path = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?
            .join("prompts")
            .join(doc_type.get_prompt_filename());
        
        let prompt_template = if prompt_path.exists() {
            fs::read_to_string(&prompt_path)
                .with_context(|| format!("Failed to read prompt template: {}", prompt_path.display()))?
        } else {
            // Fallback to embedded prompts if files don't exist
            self.get_embedded_prompt(doc_type)
        };

        let task_json = serde_json::to_string_pretty(task)?;
        let prompt = format!(
            "Given this task definition from a Task Master project:\n{task_json}\n\n{prompt_template}"
        );

        if verbose {
            debug!("Sending prompt to Claude (model: {})", self.model);
            debug!("Prompt length: {} characters", prompt.len());
            self.output.info(&format!("Sending {} character prompt to Claude...", prompt.len()))?;
        }

        // Get API key from environment or fallback to fetching from Kubernetes
        let api_key = match std::env::var("ANTHROPIC_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                // Try to get from Kubernetes secret as fallback
                match Command::new("kubectl")
                    .args(&["get", "secret", "claude-api-key", "-n", "orchestrator", "-o", "jsonpath={.data.api-key}"])
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        let encoded = String::from_utf8_lossy(&output.stdout);
                        match Command::new("base64")
                            .arg("-d")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                if let Some(mut stdin) = child.stdin.take() {
                                    stdin.write_all(encoded.as_bytes())?;
                                }
                                child.wait_with_output()
                            })
                        {
                            Ok(output) if output.status.success() => {
                                String::from_utf8_lossy(&output.stdout).trim().to_string()
                            }
                            _ => return Err(anyhow::anyhow!("Failed to decode API key from Kubernetes secret"))
                        }
                    }
                    _ => return Err(anyhow::anyhow!("ANTHROPIC_API_KEY not set and unable to fetch from Kubernetes"))
                }
            }
        };
        
        // Write prompt to temp file and use shell redirection
        let temp_file = format!("/tmp/claude_prompt_{}.txt", std::process::id());
        fs::write(&temp_file, &prompt)
            .context("Failed to write prompt to temp file")?;
        
        // Add verbose flag for better feedback during generation
        let verbose_flag = if verbose { "--verbose" } else { "" };
        let shell_command = format!(
            "ANTHROPIC_API_KEY='{}' /Users/jonathonfritz/.claude/local/claude -p --model {} {} < {}",
            api_key,
            self.model,
            verbose_flag,
            temp_file
        );
        
        let output = if verbose {
            // In verbose mode, let stderr pass through for real-time feedback
            Command::new("bash")
                .arg("-c")
                .arg(&shell_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())  // Show Claude's progress messages
                .output()
                .context("Failed to execute Claude CLI via shell")?
        } else {
            Command::new("bash")
                .arg("-c")
                .arg(&shell_command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .context("Failed to execute Claude CLI via shell")?
        };
        
        // Clean up temp file
        let _ = fs::remove_file(&temp_file);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Claude failed with error: stderr={}, stdout={}", stderr, stdout);
        }

        let response = String::from_utf8(output.stdout)
            .context("Failed to parse Claude output as UTF-8")?;

        Ok(response.trim().to_string())
    }

    /// Get embedded prompt as fallback
    fn get_embedded_prompt(&self, doc_type: DocType) -> String {
        match doc_type {
            DocType::Task => r#"Generate a comprehensive task.md that:
1. Explains the task's purpose and its role in the larger project
2. Provides a narrative overview of what needs to be accomplished
3. Breaks down each subtask with context about why it's important
4. Shows how subtasks relate to and depend on each other
5. Includes relevant commands, code snippets, and examples
6. Maintains a helpful, instructional tone

Format the output as a well-structured Markdown document."#.to_string(),
            
            DocType::Prompt => r#"Generate a prompt.md with step-by-step implementation instructions that:
1. Provides exact commands to run for each subtask
2. Includes code examples and boilerplate where helpful
3. Explains common pitfalls and how to avoid them
4. Shows how to verify each step is completed correctly
5. Integrates with Task Master commands
6. Concludes with instructions for creating a pull request

The target audience is an AI agent (Claude) that will implement these instructions autonomously."#.to_string(),
            
            DocType::DesignSpec => r#"Generate a design-spec.md that covers:
1. Technical architecture for this component
2. Data models and structures
3. API contracts (if applicable)
4. File structure and organization
5. Integration points with other components
6. Technology choices and rationale
7. Performance and security considerations

Focus on providing clear technical guidance for implementation."#.to_string(),
            
            DocType::AcceptanceCriteria => r#"Generate an acceptance-criteria.md that includes:
1. Checklist of completion criteria
2. Specific test cases with expected outcomes
3. Manual testing steps with example commands
4. Documentation requirements
5. Git and PR requirements

Format as actionable checklists and test procedures. The final section should explicitly state that the task is ONLY complete when a pull request has been created."#.to_string(),
        }
    }
}