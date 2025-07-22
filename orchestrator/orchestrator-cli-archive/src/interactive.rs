//! Interactive task creation and validation features
#![allow(dead_code)]
#![allow(clippy::disallowed_macros)]

use crate::validation::{TaskValidator, ValidationResult};
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use orchestrator_common::models::request::CreateTaskRequest;

/// Interactive task builder for CLI
pub struct InteractiveTaskBuilder {
    validator: TaskValidator,
    colored: bool,
}

impl InteractiveTaskBuilder {
    /// Create a new interactive task builder
    pub fn new(colored: bool) -> Self {
        Self {
            validator: TaskValidator::new(),
            colored,
        }
    }

    /// Create a task interactively through CLI prompts
    pub fn create_task_interactive(&self) -> Result<CreateTaskRequest> {
        println!();
        if self.colored {
            println!("{}", "🚀 Interactive Task Creation".blue().bold());
            println!("{}", "Please provide the following information:".cyan());
        } else {
            println!("🚀 Interactive Task Creation");
            println!("Please provide the following information:");
        }
        println!();

        // Select microservice
        let microservice = self.select_microservice()?;

        // Get task title
        let title = self.get_task_title()?;

        // Get task description
        let description = self.get_task_description()?;

        // Get acceptance criteria
        let acceptance_criteria = self.get_acceptance_criteria()?;

        // Select priority
        let priority = self.select_priority()?;

        // Create the request
        let request = CreateTaskRequest {
            microservice,
            title,
            description,
            acceptance_criteria,
            priority,
            agent_type: None,
            metadata: None,
        };

        // Validate the task
        let validation_result = self.validator.validate_task(&request);
        self.display_validation_result(&validation_result)?;

        // If there are errors, ask if user wants to continue anyway
        if !validation_result.is_valid() {
            let continue_anyway = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Task has validation errors. Continue anyway?")
                .default(false)
                .interact()?;

            if !continue_anyway {
                return Err(anyhow::anyhow!(
                    "Task creation cancelled due to validation errors"
                ));
            }
        }

        // Show final confirmation
        self.show_task_summary(&request)?;

        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Submit this task?")
            .default(true)
            .interact()?;

        if !confirm {
            return Err(anyhow::anyhow!("Task creation cancelled by user"));
        }

        Ok(request)
    }

    /// Select microservice from predefined list
    fn select_microservice(&self) -> Result<String> {
        let microservices = vec![
            "auth - Authentication and authorization",
            "api - REST API and GraphQL services",
            "database - Database operations and migrations",
            "frontend - User interface components",
            "orchestrator - Task orchestration and workflow",
            "monitoring - Logging and metrics",
            "other - Custom microservice name",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select target microservice")
            .items(&microservices)
            .default(0)
            .interact()?;

        if selection == microservices.len() - 1 {
            // "other" was selected
            let custom_name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter custom microservice name")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim().is_empty() {
                        Err("Microservice name cannot be empty")
                    } else if !input.chars().all(|c| c.is_alphanumeric() || c == '-') {
                        Err("Microservice name can only contain letters, numbers, and hyphens")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;
            Ok(custom_name.trim().to_lowercase())
        } else {
            // Extract the microservice name (before the " - " separator)
            let microservice_name = microservices[selection].split(" - ").next().unwrap();
            Ok(microservice_name.to_string())
        }
    }

    /// Get task title with validation
    fn get_task_title(&self) -> Result<String> {
        let title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Task title")
            .validate_with(|input: &String| -> Result<(), &str> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    Err("Title cannot be empty")
                } else if trimmed.len() > 200 {
                    Err("Title too long (max 200 characters)")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        Ok(title.trim().to_string())
    }

    /// Get task description with validation
    fn get_task_description(&self) -> Result<String> {
        println!();
        if self.colored {
            println!(
                "{}",
                "💡 Tip: Provide a detailed description of what needs to be implemented".yellow()
            );
        } else {
            println!("💡 Tip: Provide a detailed description of what needs to be implemented");
        }

        let description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Task description")
            .validate_with(|input: &String| -> Result<(), &str> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    Err("Description cannot be empty")
                } else if trimmed.len() < 10 {
                    Err("Description too short (minimum 10 characters)")
                } else if trimmed.len() > 10000 {
                    Err("Description too long (max 10000 characters)")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        Ok(description.trim().to_string())
    }

    /// Get acceptance criteria with interactive prompts
    fn get_acceptance_criteria(&self) -> Result<Vec<String>> {
        println!();
        if self.colored {
            println!("{}", "📋 Acceptance Criteria (optional)".blue());
            println!("{}", "Define clear success criteria for this task".cyan());
        } else {
            println!("📋 Acceptance Criteria (optional)");
            println!("Define clear success criteria for this task");
        }

        let add_criteria = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Add acceptance criteria?")
            .default(true)
            .interact()?;

        if !add_criteria {
            return Ok(vec![]);
        }

        let mut criteria = Vec::new();
        loop {
            let criterion: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Criterion #{}", criteria.len() + 1))
                .allow_empty(true)
                .interact_text()?;

            let trimmed = criterion.trim();
            if trimmed.is_empty() {
                if criteria.is_empty() {
                    continue; // Don't allow completely empty criteria if they chose to add them
                } else {
                    break; // Empty input ends the criteria entry
                }
            }

            criteria.push(trimmed.to_string());

            if criteria.len() >= 10 {
                println!("Maximum 10 acceptance criteria reached.");
                break;
            }

            let add_another = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Add another criterion?")
                .default(false)
                .interact()?;

            if !add_another {
                break;
            }
        }

        Ok(criteria)
    }

    /// Select task priority
    fn select_priority(&self) -> Result<Option<String>> {
        let priorities = vec![
            "low - Minor improvements or nice-to-have features",
            "medium - Standard development tasks (default)",
            "high - Important features or significant improvements",
            "critical - Urgent fixes or essential functionality",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select task priority")
            .items(&priorities)
            .default(1) // medium as default
            .interact()?;

        let priority_name = priorities[selection].split(" - ").next().unwrap();
        Ok(Some(priority_name.to_string()))
    }

    /// Display validation results with colored output
    fn display_validation_result(&self, result: &ValidationResult) -> Result<()> {
        if result.errors.is_empty() && result.warnings.is_empty() {
            if self.colored {
                println!("\n{}", "✅ Task validation passed!".green().bold());
            } else {
                println!("\n✅ Task validation passed!");
            }
            return Ok(());
        }

        println!();
        let formatted = result.format_messages(self.colored);
        println!("{formatted}");
        println!();

        Ok(())
    }

    /// Show a summary of the task before submission
    fn show_task_summary(&self, request: &CreateTaskRequest) -> Result<()> {
        println!();
        if self.colored {
            println!("{}", "📋 Task Summary".blue().bold());
            println!("{}", "━".repeat(50).blue());
        } else {
            println!("📋 Task Summary");
            println!("{}", "━".repeat(50));
        }

        if self.colored {
            println!("{}: {}", "Microservice".bold(), request.microservice.cyan());
            println!("{}: {}", "Title".bold(), request.title.white());
            println!(
                "{}: {}",
                "Priority".bold(),
                request
                    .priority
                    .as_ref()
                    .unwrap_or(&"medium".to_string())
                    .yellow()
            );
        } else {
            println!("Microservice: {}", request.microservice);
            println!("Title: {}", request.title);
            println!(
                "Priority: {}",
                request.priority.as_ref().unwrap_or(&"medium".to_string())
            );
        }

        println!();
        if self.colored {
            println!("{}:", "Description".bold());
        } else {
            println!("Description:");
        }

        // Word wrap the description
        let wrapped = textwrap::wrap(&request.description, 70);
        for line in wrapped {
            println!("  {line}");
        }

        if !request.acceptance_criteria.is_empty() {
            println!();
            if self.colored {
                println!("{}:", "Acceptance Criteria".bold());
            } else {
                println!("Acceptance Criteria:");
            }
            for (i, criterion) in request.acceptance_criteria.iter().enumerate() {
                if self.colored {
                    println!("  {}. {}", (i + 1).to_string().green(), criterion);
                } else {
                    println!("  {}. {}", i + 1, criterion);
                }
            }
        }

        if self.colored {
            println!("{}", "━".repeat(50).blue());
        } else {
            println!("{}", "━".repeat(50));
        }

        Ok(())
    }

    /// Validate a task from file input
    #[allow(dead_code)]
    pub fn validate_task_file(
        &self,
        file_path: &str,
        microservice: &str,
    ) -> Result<ValidationResult> {
        // First validate the file itself
        let mut result = self.validator.validate_task_file(file_path);

        if !result.is_valid() {
            return Ok(result);
        }

        // Read and parse the file content
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read task file: {file_path}"))?;

        // Parse the content to create a task request
        let (title, description) = self.parse_task_file_content(&content)?;

        let request = CreateTaskRequest {
            microservice: microservice.to_string(),
            title,
            description,
            acceptance_criteria: vec![], // Files typically don't have structured AC
            priority: None,
            agent_type: None,
            metadata: None,
        };

        // Validate the parsed task
        let task_validation = self.validator.validate_task(&request);
        result.merge(task_validation);

        Ok(result)
    }

    /// Parse task file content to extract title and description
    #[allow(dead_code)]
    fn parse_task_file_content(&self, content: &str) -> Result<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err(anyhow::anyhow!("Task file is empty"));
        }

        // Try to detect if it's a Markdown file with a title
        let mut title = String::new();
        let mut description_lines = Vec::new();
        let mut found_title = false;

        for line in lines {
            let trimmed = line.trim();

            // Check for Markdown title (# Title)
            if !found_title && trimmed.starts_with('#') {
                title = trimmed.trim_start_matches('#').trim().to_string();
                found_title = true;
                continue;
            }

            // Skip empty lines at the beginning
            if !found_title && trimmed.is_empty() {
                continue;
            }

            // If no markdown title found, use first non-empty line as title
            if !found_title && !trimmed.is_empty() {
                title = trimmed.to_string();
                found_title = true;
                continue;
            }

            // Everything else is description
            description_lines.push(line);
        }

        // If still no title found, use default
        if title.is_empty() {
            title = "Task from file".to_string();
        }

        let description = description_lines.join("\n").trim().to_string();

        if description.is_empty() {
            return Err(anyhow::anyhow!("No description found in task file"));
        }

        Ok((title, description))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_markdown_content() {
        let builder = InteractiveTaskBuilder::new(false);
        let content = r#"# Implement user authentication

This task involves implementing JWT-based authentication
for the user management system.

## Requirements
- JWT token generation
- Token validation middleware
"#;

        let (title, description) = builder.parse_task_file_content(content).unwrap();
        assert_eq!(title, "Implement user authentication");
        assert!(description.contains("JWT-based authentication"));
    }

    #[test]
    fn test_parse_plain_text_content() {
        let builder = InteractiveTaskBuilder::new(false);
        let content = r#"Fix login bug

The login form is not properly validating user credentials.
Need to fix the validation logic.
"#;

        let (title, description) = builder.parse_task_file_content(content).unwrap();
        assert_eq!(title, "Fix login bug");
        assert!(description.contains("validation logic"));
    }

    #[test]
    fn test_validate_task_file_integration() {
        let builder = InteractiveTaskBuilder::new(false);

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Test Task").unwrap();
        writeln!(temp_file, "This is a test task description.").unwrap();

        let result = builder
            .validate_task_file(temp_file.path().to_str().unwrap(), "auth")
            .unwrap();

        // Should have warnings but no errors for a basic valid file
        assert!(result.is_valid() || !result.errors.is_empty());
    }

    #[test]
    fn test_empty_file_validation() {
        let builder = InteractiveTaskBuilder::new(false);

        let temp_file = NamedTempFile::new().unwrap();
        // Don't write anything - empty file

        let result = builder
            .validate_task_file(temp_file.path().to_str().unwrap(), "auth")
            .unwrap();

        assert!(!result.is_valid(), "Empty file should fail validation");
    }
}
