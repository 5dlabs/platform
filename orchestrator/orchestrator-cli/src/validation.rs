//! Task validation framework and rules
#![allow(dead_code)]
use colored::*;
use orchestrator_common::models::request::CreateTaskRequest;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Validation errors for task submission
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    MissingField { field: String },

    #[error("Invalid field format: {field} - {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Field value out of range: {field} - {reason}")]
    OutOfRange { field: String, reason: String },

    #[error("Business rule violation: {rule} - {reason}")]
    BusinessRule { rule: String, reason: String },

    #[error("File system error: {path} - {reason}")]
    FileSystem { path: String, reason: String },

    #[error("Multiple validation errors occurred")]
    #[allow(dead_code)]
    Multiple { errors: Vec<ValidationError> },
}

/// Validation result containing errors and warnings
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Merge another validation result into this one
    #[allow(dead_code)]
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }

    /// Format errors and warnings for display
    pub fn format_messages(&self, colored: bool) -> String {
        let mut messages = Vec::new();

        if !self.errors.is_empty() {
            let error_header = if colored {
                "❌ Validation Errors:".red().bold().to_string()
            } else {
                "❌ Validation Errors:".to_string()
            };
            messages.push(error_header);

            for (i, error) in self.errors.iter().enumerate() {
                let formatted = if colored {
                    format!("  {}. {}", i + 1, error.to_string().red())
                } else {
                    format!("  {}. {}", i + 1, error)
                };
                messages.push(formatted);
            }
        }

        if !self.warnings.is_empty() {
            if !messages.is_empty() {
                messages.push(String::new()); // Empty line separator
            }

            let warning_header = if colored {
                "⚠️  Validation Warnings:".yellow().bold().to_string()
            } else {
                "⚠️  Validation Warnings:".to_string()
            };
            messages.push(warning_header);

            for (i, warning) in self.warnings.iter().enumerate() {
                let formatted = if colored {
                    format!("  {}. {}", i + 1, warning.yellow())
                } else {
                    format!("  {}. {}", i + 1, warning)
                };
                messages.push(formatted);
            }
        }

        messages.join("\n")
    }
}

/// Task validator with configurable rules
pub struct TaskValidator {
    microservice_patterns: HashMap<String, Regex>,
    max_title_length: usize,
    max_description_length: usize,
    required_microservices: Vec<String>,
}

impl TaskValidator {
    /// Create a new task validator with default rules
    pub fn new() -> Self {
        let mut microservice_patterns = HashMap::new();

        // Define patterns for known microservices
        microservice_patterns.insert(
            "auth".to_string(),
            Regex::new(r"^(auth|authentication|login|jwt|oauth|security)").unwrap(),
        );
        microservice_patterns.insert(
            "api".to_string(),
            Regex::new(r"^(api|endpoint|rest|graphql|service)").unwrap(),
        );
        microservice_patterns.insert(
            "database".to_string(),
            Regex::new(r"^(db|database|sql|postgres|mysql|migration)").unwrap(),
        );
        microservice_patterns.insert(
            "frontend".to_string(),
            Regex::new(r"^(ui|frontend|react|vue|angular|component)").unwrap(),
        );

        Self {
            microservice_patterns,
            max_title_length: 200,
            max_description_length: 10000,
            required_microservices: vec![
                "auth".to_string(),
                "api".to_string(),
                "database".to_string(),
                "frontend".to_string(),
                "orchestrator".to_string(),
            ],
        }
    }

    /// Validate a task submission request
    pub fn validate_task(&self, request: &CreateTaskRequest) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate required fields
        self.validate_required_fields(request, &mut result);

        // Validate field formats
        self.validate_field_formats(request, &mut result);

        // Validate business rules
        self.validate_business_rules(request, &mut result);

        // Add contextual warnings
        self.add_contextual_warnings(request, &mut result);

        result
    }

    /// Validate that required fields are present and non-empty
    fn validate_required_fields(&self, request: &CreateTaskRequest, result: &mut ValidationResult) {
        if request.microservice.trim().is_empty() {
            result.add_error(ValidationError::MissingField {
                field: "microservice".to_string(),
            });
        }

        if request.title.trim().is_empty() {
            result.add_error(ValidationError::MissingField {
                field: "title".to_string(),
            });
        }

        if request.description.trim().is_empty() {
            result.add_error(ValidationError::MissingField {
                field: "description".to_string(),
            });
        }
    }

    /// Validate field formats and constraints
    fn validate_field_formats(&self, request: &CreateTaskRequest, result: &mut ValidationResult) {
        // Validate microservice name format
        if !request.microservice.trim().is_empty() {
            let microservice_regex = Regex::new(r"^[a-z][a-z0-9-]*[a-z0-9]$").unwrap();
            if !microservice_regex.is_match(&request.microservice) {
                result.add_error(ValidationError::InvalidFormat {
                    field: "microservice".to_string(),
                    reason: "Must be lowercase, start with letter, contain only letters, numbers, and hyphens".to_string(),
                });
            }
        }

        // Validate title length and format
        if !request.title.trim().is_empty() {
            if request.title.len() > self.max_title_length {
                result.add_error(ValidationError::OutOfRange {
                    field: "title".to_string(),
                    reason: format!("Maximum length is {} characters", self.max_title_length),
                });
            }

            // Check for reasonable title format
            if request
                .title
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace())
            {
                result.add_warning(
                    "Title appears to be all uppercase - consider using sentence case".to_string(),
                );
            }
        }

        // Validate description length
        if !request.description.trim().is_empty() {
            if request.description.len() > self.max_description_length {
                result.add_error(ValidationError::OutOfRange {
                    field: "description".to_string(),
                    reason: format!(
                        "Maximum length is {} characters",
                        self.max_description_length
                    ),
                });
            }

            if request.description.len() < 10 {
                result.add_warning(
                    "Description is very short - consider adding more details".to_string(),
                );
            }
        }

        // Validate priority if provided
        if let Some(priority) = &request.priority {
            let valid_priorities = ["low", "medium", "high", "critical"];
            if !valid_priorities.contains(&priority.to_lowercase().as_str()) {
                result.add_error(ValidationError::InvalidFormat {
                    field: "priority".to_string(),
                    reason: format!("Must be one of: {}", valid_priorities.join(", ")),
                });
            }
        }
    }

    /// Validate business rules
    fn validate_business_rules(&self, request: &CreateTaskRequest, result: &mut ValidationResult) {
        // Check if microservice is in the known list
        if !self.required_microservices.contains(&request.microservice) {
            result.add_warning(format!(
                "Microservice '{}' is not in the standard list: {}",
                request.microservice,
                self.required_microservices.join(", ")
            ));
        }

        // Validate that task description matches microservice context
        if let Some(pattern) = self.microservice_patterns.get(&request.microservice) {
            let combined_text = format!("{} {}", request.title, request.description).to_lowercase();
            if !pattern.is_match(&combined_text) {
                result.add_warning(format!(
                    "Task content doesn't seem to match microservice '{}' - verify this is the correct microservice",
                    request.microservice
                ));
            }
        }

        // Check for acceptance criteria
        if request.acceptance_criteria.is_empty() {
            result.add_warning(
                "No acceptance criteria provided - consider adding clear success criteria"
                    .to_string(),
            );
        } else {
            // Validate acceptance criteria format
            for (i, criteria) in request.acceptance_criteria.iter().enumerate() {
                if criteria.trim().is_empty() {
                    result.add_error(ValidationError::InvalidFormat {
                        field: format!("acceptance_criteria[{i}]"),
                        reason: "Acceptance criteria cannot be empty".to_string(),
                    });
                }
            }
        }

        // Business rule: High/Critical priority tasks should have acceptance criteria
        if let Some(priority) = &request.priority {
            if ["high", "critical"].contains(&priority.to_lowercase().as_str())
                && request.acceptance_criteria.is_empty()
            {
                result.add_error(ValidationError::BusinessRule {
                    rule: "high_priority_criteria".to_string(),
                    reason: "High and critical priority tasks must include acceptance criteria"
                        .to_string(),
                });
            }
        }
    }

    /// Add contextual warnings based on task content
    fn add_contextual_warnings(&self, request: &CreateTaskRequest, result: &mut ValidationResult) {
        let combined_text = format!("{} {}", request.title, request.description).to_lowercase();

        // Check for security-related tasks
        if (combined_text.contains("password")
            || combined_text.contains("security")
            || combined_text.contains("auth"))
            && request.microservice != "auth"
        {
            result.add_warning(
                "Task appears security-related - consider using 'auth' microservice".to_string(),
            );
        }

        // Check for database-related tasks
        if (combined_text.contains("database")
            || combined_text.contains("migration")
            || combined_text.contains("sql"))
            && request.microservice != "database"
        {
            result.add_warning(
                "Task appears database-related - consider using 'database' microservice"
                    .to_string(),
            );
        }

        // Check for UI-related tasks
        if (combined_text.contains("frontend")
            || combined_text.contains("ui")
            || combined_text.contains("component"))
            && request.microservice != "frontend"
        {
            result.add_warning(
                "Task appears UI-related - consider using 'frontend' microservice".to_string(),
            );
        }

        // Check task size indicators
        if combined_text.contains("refactor")
            || combined_text.contains("rewrite")
            || combined_text.contains("migrate")
        {
            result.add_warning(
                "Task appears to be large scope - consider breaking into smaller tasks".to_string(),
            );
        }
    }

    /// Validate a task file exists and is readable
    pub fn validate_task_file(&self, file_path: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let path = Path::new(file_path);

        // Check if file exists
        if !path.exists() {
            result.add_error(ValidationError::FileSystem {
                path: file_path.to_string(),
                reason: "File does not exist".to_string(),
            });
            return result;
        }

        // Check if it's a file (not directory)
        if !path.is_file() {
            result.add_error(ValidationError::FileSystem {
                path: file_path.to_string(),
                reason: "Path is not a file".to_string(),
            });
            return result;
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            if !["md", "txt", "rst"].contains(&ext.as_str()) {
                result.add_warning(format!(
                    "File extension '{ext}' is not a typical task format (md, txt, rst)"
                ));
            }
        } else {
            result.add_warning(
                "File has no extension - consider using .md for Markdown format".to_string(),
            );
        }

        // Check if file is readable (basic check)
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    result.add_error(ValidationError::FileSystem {
                        path: file_path.to_string(),
                        reason: "File is empty".to_string(),
                    });
                } else if metadata.len() > 1_000_000 {
                    result.add_warning(
                        "File is very large (>1MB) - consider splitting into smaller tasks"
                            .to_string(),
                    );
                }
            }
            Err(e) => {
                result.add_error(ValidationError::FileSystem {
                    path: file_path.to_string(),
                    reason: format!("Cannot read file metadata: {e}"),
                });
            }
        }

        result
    }
}

impl Default for TaskValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_task() -> CreateTaskRequest {
        CreateTaskRequest {
            microservice: "auth".to_string(),
            title: "Implement JWT token validation".to_string(),
            description: "Add JWT token validation middleware to protect authenticated endpoints"
                .to_string(),
            acceptance_criteria: vec![
                "JWT tokens are validated on protected routes".to_string(),
                "Invalid tokens return 401 status".to_string(),
            ],
            priority: Some("high".to_string()),
            agent_type: None,
            metadata: None,
        }
    }

    #[test]
    fn test_valid_task_passes_validation() {
        let validator = TaskValidator::new();
        let task = create_valid_task();
        let result = validator.validate_task(&task);

        assert!(result.is_valid(), "Valid task should pass validation");
        assert!(result.errors.is_empty(), "Should have no errors");
    }

    #[test]
    fn test_missing_required_fields() {
        let validator = TaskValidator::new();
        let task = CreateTaskRequest {
            microservice: "".to_string(),
            title: "".to_string(),
            description: "".to_string(),
            acceptance_criteria: vec![],
            priority: None,
            agent_type: None,
            metadata: None,
        };

        let result = validator.validate_task(&task);
        assert!(!result.is_valid(), "Empty task should fail validation");
        assert_eq!(result.errors.len(), 3, "Should have 3 missing field errors");
    }

    #[test]
    fn test_invalid_microservice_format() {
        let validator = TaskValidator::new();
        let mut task = create_valid_task();
        task.microservice = "Auth-Service-1".to_string(); // Invalid: contains uppercase

        let result = validator.validate_task(&task);
        assert!(
            !result.is_valid(),
            "Invalid microservice format should fail"
        );
    }

    #[test]
    fn test_invalid_priority() {
        let validator = TaskValidator::new();
        let mut task = create_valid_task();
        task.priority = Some("urgent".to_string()); // Invalid priority

        let result = validator.validate_task(&task);
        assert!(
            !result.is_valid(),
            "Invalid priority should fail validation"
        );
    }

    #[test]
    fn test_high_priority_requires_acceptance_criteria() {
        let validator = TaskValidator::new();
        let mut task = create_valid_task();
        task.priority = Some("critical".to_string());
        task.acceptance_criteria = vec![]; // Remove acceptance criteria

        let result = validator.validate_task(&task);
        assert!(
            !result.is_valid(),
            "High priority without acceptance criteria should fail"
        );
    }

    #[test]
    fn test_title_length_validation() {
        let validator = TaskValidator::new();
        let mut task = create_valid_task();
        task.title = "x".repeat(250); // Exceeds max length

        let result = validator.validate_task(&task);
        assert!(
            !result.is_valid(),
            "Overly long title should fail validation"
        );
    }

    #[test]
    fn test_contextual_warnings() {
        let validator = TaskValidator::new();
        let mut task = create_valid_task();
        task.microservice = "api".to_string();
        task.title = "Fix authentication bug".to_string(); // Auth-related but wrong microservice

        let result = validator.validate_task(&task);
        assert!(
            !result.warnings.is_empty(),
            "Should have contextual warnings"
        );
    }

    #[test]
    fn test_validation_result_formatting() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingField {
            field: "title".to_string(),
        });
        result.add_warning("Test warning".to_string());

        let formatted = result.format_messages(false);
        assert!(formatted.contains("Validation Errors"));
        assert!(formatted.contains("Validation Warnings"));
        assert!(formatted.contains("title"));
        assert!(formatted.contains("Test warning"));
    }

    #[test]
    fn test_file_validation_nonexistent() {
        let validator = TaskValidator::new();
        let result = validator.validate_task_file("/nonexistent/file.md");

        assert!(
            !result.is_valid(),
            "Nonexistent file should fail validation"
        );
        assert!(!result.errors.is_empty(), "Should have file system errors");
    }
}
