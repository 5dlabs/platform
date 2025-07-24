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
#[allow(clippy::large_enum_variant)]
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

        /// Documentation repository URL (where Task Master definitions come from)
        #[arg(long)]
        docs_repository_url: Option<String>,

        /// Project directory within docs repository (e.g. "_projects/simple-api")
        #[arg(long)]
        docs_project_directory: Option<String>,



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

        /// Context version for retry attempts (incremented on each retry)
        #[arg(long, default_value = "1")]
        context_version: u32,

        /// Additional context for retry attempts
        #[arg(long)]
        prompt_modification: Option<String>,

        /// Docs branch to use (e.g., "main", "feature/branch")
        #[arg(long, default_value = "main")]
        docs_branch: String,

        /// Whether to continue a previous session
        #[arg(long)]
        continue_session: bool,

        /// Whether to overwrite memory before starting
        #[arg(long)]
        overwrite_memory: bool,

        /// Environment variables (format: KEY=value,KEY2=value2)
        #[arg(long)]
        env: Option<String>,

        /// Environment variables from secrets (format: name:secretName:secretKey,...)
        #[arg(long)]
        env_from_secrets: Option<String>,
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
