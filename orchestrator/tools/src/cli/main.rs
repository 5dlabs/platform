/*
 * 5D Labs Agent Platform - CLI Tools for AI Coding Agents
 * Copyright (C) 2025 5D Labs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Orchestrator CLI - Simplified with just docs and code task submission

mod analyzer;
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
    /// Analyze codebase and generate documentation for Task Master PRD
    Analyze {
        /// Output directory (default: docs/codebase-analysis)
        #[arg(short, long, default_value = "docs/codebase-analysis")]
        output: String,

        /// Output format: modular, single, json (default: modular)
        #[arg(short, long, default_value = "modular")]
        format: String,

        /// Working directory to analyze (default: current directory)
        #[arg(short, long)]
        working_directory: Option<String>,

        /// Include full source code (default: true)
        #[arg(long, default_value = "true")]
        include_source: bool,
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

        /// Claude model to use (required) - e.g., 'claude-opus-4-20250514' or 'claude-sonnet-4-20250514'
        #[arg(long)]
        model: Option<String>,

        /// Documentation repository URL
        #[arg(long)]
        repository_url: Option<String>,

        /// Source branch to use
        #[arg(long)]
        source_branch: Option<String>,

        /// GitHub username for authentication (required)
        #[arg(long)]
        github_user: String,
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

        /// GitHub username for authentication (required)
        #[arg(long)]
        github_user: String,

        /// Working directory within target repository
        #[arg(long, short = 'w')]
        working_directory: Option<String>,

        /// Claude model to use (required) - e.g., 'claude-opus-4-20250514' or 'claude-sonnet-4-20250514'
        #[arg(long)]
        model: Option<String>,

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
        Commands::Analyze { output, format, working_directory, include_source } => {
            commands::handle_analyze_command(output, format, working_directory, include_source)?;
        }
    }

    Ok(())
}
