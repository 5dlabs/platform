//! Output formatting and display utilities

use anyhow::Result;
use colored::*;
use orchestrator_common::models::{
    job::JobStatus,
    response::{JobResponse, TaskResponse},
    task::TaskStatus,
};
use prettytable::{Cell, Row, Table};
use serde_json::Value;

/// Output format options
#[derive(Debug, Clone, PartialEq, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            _ => Err(anyhow::anyhow!("Invalid output format: {}", s)),
        }
    }
}

/// Output manager for handling different output formats and styling
pub struct OutputManager {
    format: OutputFormat,
    colored: bool,
}

impl OutputManager {
    /// Create a new output manager
    pub fn new(format: OutputFormat, colored: bool) -> Self {
        Self { format, colored }
    }

    /// Print a success message
    #[allow(clippy::disallowed_macros)]
    pub fn success(&self, message: &str) -> Result<()> {
        if self.colored {
            println!("{}", message.green().bold());
        } else {
            println!("SUCCESS: {message}");
        }
        Ok(())
    }

    /// Print an info message
    #[allow(clippy::disallowed_macros)]
    pub fn info(&self, message: &str) -> Result<()> {
        if self.colored {
            println!("{}", message.cyan());
        } else {
            println!("INFO: {message}");
        }
        Ok(())
    }

    /// Print a warning message
    #[allow(dead_code, clippy::disallowed_macros)]
    pub fn warning(&self, message: &str) -> Result<()> {
        if self.colored {
            eprintln!("{}", message.yellow().bold());
        } else {
            eprintln!("WARNING: {message}");
        }
        Ok(())
    }

    /// Print an error message
    #[allow(clippy::disallowed_macros)]
    pub fn error(&self, message: &str) -> Result<()> {
        if self.colored {
            eprintln!("{}", message.red().bold());
        } else {
            eprintln!("ERROR: {message}");
        }
        Ok(())
    }

    /// Print a single task
    #[allow(dead_code)]
    pub fn print_task(&self, task: &TaskResponse) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.print_task_table(task),
            OutputFormat::Json => self.print_json_value(&serde_json::to_value(task)?),
            OutputFormat::Yaml => self.print_yaml_value(&serde_json::to_value(task)?),
        }
    }

    /// Print a list of tasks
    #[allow(dead_code)]
    pub fn print_task_list(&self, tasks: &[TaskResponse]) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.print_task_list_table(tasks),
            OutputFormat::Json => self.print_json_value(&serde_json::to_value(tasks)?),
            OutputFormat::Yaml => self.print_yaml_value(&serde_json::to_value(tasks)?),
        }
    }

    /// Print a single job
    pub fn print_job(&self, job: &JobResponse) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.print_job_table(job),
            OutputFormat::Json => self.print_json_value(&serde_json::to_value(job)?),
            OutputFormat::Yaml => self.print_yaml_value(&serde_json::to_value(job)?),
        }
    }

    /// Print a list of jobs
    pub fn print_job_list(&self, jobs: &[JobResponse]) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.print_job_list_table(jobs),
            OutputFormat::Json => self.print_json_value(&serde_json::to_value(jobs)?),
            OutputFormat::Yaml => self.print_yaml_value(&serde_json::to_value(jobs)?),
        }
    }

    /// Print JSON data
    pub fn print_json(&self, value: &Value) -> Result<()> {
        match self.format {
            OutputFormat::Json => self.print_json_value(value),
            OutputFormat::Yaml => self.print_yaml_value(value),
            OutputFormat::Table => {
                // For table format, just pretty print the JSON
                self.print_json_value(value)
            }
        }
    }

    /// Print a single task in table format
    #[allow(dead_code)]
    fn print_task_table(&self, task: &TaskResponse) -> Result<()> {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Field").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));

        table.add_row(Row::new(vec![Cell::new("ID"), Cell::new(&task.id)]));

        table.add_row(Row::new(vec![Cell::new("Title"), Cell::new(&task.title)]));

        table.add_row(Row::new(vec![
            Cell::new("Description"),
            Cell::new(&task.description),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Status"),
            Cell::new(&self.format_task_status(&task.status)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Priority"),
            Cell::new(&format!("{}", task.priority)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Microservice"),
            Cell::new(&task.microservice),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Created"),
            Cell::new(&task.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Updated"),
            Cell::new(&task.updated_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));

        if !task.job_ids.is_empty() {
            table.add_row(Row::new(vec![
                Cell::new("Job IDs"),
                Cell::new(&task.job_ids.join(", ")),
            ]));
        }

        table.printstd();
        Ok(())
    }

    /// Print a list of tasks in table format
    #[allow(dead_code)]
    fn print_task_list_table(&self, tasks: &[TaskResponse]) -> Result<()> {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("ID").style_spec("Fb"),
            Cell::new("Title").style_spec("Fb"),
            Cell::new("Status").style_spec("Fb"),
            Cell::new("Priority").style_spec("Fb"),
            Cell::new("Microservice").style_spec("Fb"),
            Cell::new("Created").style_spec("Fb"),
        ]));

        for task in tasks {
            table.add_row(Row::new(vec![
                Cell::new(&task.id),
                Cell::new(&truncate_string(&task.title, 30)),
                Cell::new(&self.format_task_status(&task.status)),
                Cell::new(&format!("{}", task.priority)),
                Cell::new(&task.microservice),
                Cell::new(&task.created_at.format("%m-%d %H:%M").to_string()),
            ]));
        }

        table.printstd();
        Ok(())
    }

    /// Print a single job in table format
    fn print_job_table(&self, job: &JobResponse) -> Result<()> {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Field").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));

        table.add_row(Row::new(vec![Cell::new("ID"), Cell::new(&job.id)]));

        table.add_row(Row::new(vec![
            Cell::new("Task ID"),
            Cell::new(&job.task_id),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Type"),
            Cell::new(&format!("{}", job.job_type)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Status"),
            Cell::new(&self.format_job_status(&job.status)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("K8s Job Name"),
            Cell::new(&job.k8s_job_name),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Namespace"),
            Cell::new(&job.namespace),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Created"),
            Cell::new(&job.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
        ]));

        if let Some(started) = job.started_at {
            table.add_row(Row::new(vec![
                Cell::new("Started"),
                Cell::new(&started.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ]));
        }

        if let Some(completed) = job.completed_at {
            table.add_row(Row::new(vec![
                Cell::new("Completed"),
                Cell::new(&completed.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            ]));
        }

        if let Some(logs_url) = &job.logs_url {
            table.add_row(Row::new(vec![Cell::new("Logs URL"), Cell::new(logs_url)]));
        }

        table.printstd();
        Ok(())
    }

    /// Print a list of jobs in table format
    fn print_job_list_table(&self, jobs: &[JobResponse]) -> Result<()> {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("ID").style_spec("Fb"),
            Cell::new("Task ID").style_spec("Fb"),
            Cell::new("Type").style_spec("Fb"),
            Cell::new("Status").style_spec("Fb"),
            Cell::new("K8s Job").style_spec("Fb"),
            Cell::new("Created").style_spec("Fb"),
        ]));

        for job in jobs {
            table.add_row(Row::new(vec![
                Cell::new(&truncate_string(&job.id, 12)),
                Cell::new(&truncate_string(&job.task_id, 12)),
                Cell::new(&format!("{}", job.job_type)),
                Cell::new(&self.format_job_status(&job.status)),
                Cell::new(&truncate_string(&job.k8s_job_name, 20)),
                Cell::new(&job.created_at.format("%m-%d %H:%M").to_string()),
            ]));
        }

        table.printstd();
        Ok(())
    }

    /// Print JSON value
    #[allow(clippy::disallowed_macros)]
    fn print_json_value(&self, value: &Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)?;
        println!("{json_str}");
        Ok(())
    }

    /// Print YAML value
    #[allow(clippy::disallowed_macros)]
    fn print_yaml_value(&self, value: &Value) -> Result<()> {
        let yaml_str = serde_yaml::to_string(value)?;
        println!("{yaml_str}");
        Ok(())
    }

    /// Format task status with color
    #[allow(dead_code)]
    fn format_task_status(&self, status: &TaskStatus) -> String {
        let status_str = format!("{status}");
        if !self.colored {
            return status_str;
        }

        match status {
            TaskStatus::Pending => status_str.yellow().to_string(),
            TaskStatus::InProgress => status_str.blue().to_string(),
            TaskStatus::Completed => status_str.green().to_string(),
            TaskStatus::Failed => status_str.red().to_string(),
            TaskStatus::Cancelled => status_str.purple().to_string(),
            TaskStatus::Blocked => status_str.red().to_string(),
        }
    }

    /// Format job status with color
    fn format_job_status(&self, status: &JobStatus) -> String {
        let status_str = format!("{status}");
        if !self.colored {
            return status_str;
        }

        match status {
            JobStatus::Pending => status_str.yellow().to_string(),
            JobStatus::Running => status_str.blue().to_string(),
            JobStatus::Succeeded => status_str.green().to_string(),
            JobStatus::Failed => status_str.red().to_string(),
            JobStatus::Unknown => status_str.purple().to_string(),
        }
    }
}

/// Truncate a string to a maximum length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exact", 5), "exact");
    }

    #[test]
    fn test_output_format_parsing() {
        assert_eq!(
            "table".parse::<OutputFormat>().unwrap(),
            OutputFormat::Table
        );
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml);
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);

        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_output_manager_creation() {
        let manager = OutputManager::new(OutputFormat::Json, true);
        assert_eq!(manager.format, OutputFormat::Json);
        assert!(manager.colored);

        let manager = OutputManager::new(OutputFormat::Table, false);
        assert_eq!(manager.format, OutputFormat::Table);
        assert!(!manager.colored);
    }
}
