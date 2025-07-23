//! Orchestrator CLI - Simplified with just docs and code task submission

mod api;
mod commands;
mod docs_generator;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orchestrator")]
#[command(about = "CLI for Orchestrator Service", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// API endpoint URL
    #[arg(
        long,
        env = "ORCHESTRATOR_API_URL",
        default_value = "http://orchestrator.orchestrator.svc.cluster.local/api/v1"
    )]
    api_url: String,

    /// Output format (table, json, yaml)
    #[arg(long, short, default_value = "table")]
    output: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Task operations
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// Generate documentation for Task Master tasks
    Docs {
        /// Working directory containing .taskmaster folder
        #[arg(long, short = 'w')]
        working_directory: Option<String>,

        /// Claude model to use (full model name like 'claude-3-5-sonnet-20241022')
        #[arg(long, default_value = "claude-opus-4-20250514")]
        model: String,
    },

    /// Submit implementation task to orchestrator
    Code {
        /// Task ID to implement
        task_id: u32,

        /// Target service name
        #[arg(long, short = 's')]
        service: String,

        /// Target project repository URL (where implementation work happens)
        #[arg(long)]
        repository_url: Option<String>,

        /// Platform repository URL (where Task Master definitions come from)
        #[arg(long)]
        platform_repository_url: Option<String>,

        /// Project directory within platform repository (e.g. "_projects/simple-api")
        #[arg(long)]
        platform_project_directory: Option<String>,

        /// Git branch to work on
        #[arg(long, default_value = "main")]
        branch: String,

        /// GitHub username for authentication
        #[arg(long)]
        github_user: Option<String>,

        /// Working directory within target repository
        #[arg(long, short = 'w')]
        working_directory: Option<String>,

        /// Claude model to use (full model name like 'claude-3-5-sonnet-20241022')
        #[arg(long, default_value = "claude-3-5-sonnet-20241022")]
        model: String,

        /// Local MCP tools to enable (comma-separated)
        #[arg(long)]
        local_tools: Option<String>,

        /// Remote MCP tools to enable (comma-separated)
        #[arg(long)]
        remote_tools: Option<String>,

        /// Tool configuration preset: 'default', 'minimal', 'advanced'
        #[arg(long, default_value = "default")]
        tool_config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Task { command } => {
            commands::handle_task_command(command, &cli.api_url, &cli.output).await?;
        }
    }

    Ok(())
}