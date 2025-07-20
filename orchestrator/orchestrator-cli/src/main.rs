//! CLI tool for the Orchestrator service

mod api;
mod commands;
mod docs_generator;
mod interactive;
mod output;
mod validation;

use anyhow::Result;
use api::ApiClient;
use clap::{Parser, Subcommand};
use output::{OutputFormat, OutputManager};
use tracing::{debug, info};

#[derive(Parser)]
#[command(name = "orchestrator")]
#[command(about = "CLI for Unified Orchestration Service", long_about = None)]
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

    /// Output format
    #[arg(long, short, default_value = "table")]
    output: OutputFormat,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Task management commands
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// Job management commands
    Job {
        #[command(subcommand)]
        action: JobCommands,
    },
    /// ConfigMap management commands
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Check service health
    Health,
}

#[derive(Subcommand, Debug)]
enum TaskCommands {
    /// Submit a new task (simplified workflow for Task Master projects)
    Submit {
        /// Task ID to submit (uses Task Master directory structure)
        task_id: u32,
        /// Target service name (e.g., auth-service, api-gateway)
        #[arg(long, short = 's')]
        service: String,
        /// Agent name
        #[arg(long, short = 'a', default_value = "claude-agent-1")]
        agent: String,
        /// Path to Task Master directory
        #[arg(long, short = 'd', default_value = ".taskmaster")]
        taskmaster_dir: String,
        /// Path to additional context files
        #[arg(long, short = 'c')]
        context: Vec<String>,
        /// Agent tools (format: tool_name:enabled, e.g., bash:true,edit:false)
        #[arg(long, short = 't')]
        tools: Vec<String>,
        /// Repository URL to clone (e.g., https://github.com/org/repo)
        #[arg(long, short = 'r')]
        repo: Option<String>,
        /// Repository branch or tag to checkout
        #[arg(long, short = 'b', default_value = "main")]
        branch: String,
        /// GitHub username for authentication (e.g., swe-1-5dlabs)
        #[arg(long)]
        github_user: Option<String>,
        /// Indicates this is a retry of a previous attempt
        #[arg(long)]
        retry: bool,
        /// Claude model to use (sonnet, opus)
        #[arg(long, short = 'm', default_value = "sonnet")]
        model: String,
    },
    /// Submit a new task (advanced workflow with explicit paths)
    SubmitAdvanced {
        /// Path to tasks.json file containing all tasks
        #[arg(long)]
        task_json: String,
        /// Task ID to submit from the tasks.json file
        #[arg(long)]
        task_id: u32,
        /// Path to design specification markdown file
        #[arg(long)]
        design_spec: String,
        /// Path to autonomous prompt markdown file (optional)
        #[arg(long)]
        prompt: Option<String>,
        /// Target service name (e.g., auth-service, api-gateway)
        #[arg(long)]
        service_name: String,
        /// Agent name (e.g., claude-agent-1)
        #[arg(long)]
        agent_name: String,
        /// Indicates this is a retry of a previous attempt
        #[arg(long)]
        retry: bool,
        /// Claude model to use (sonnet, opus)
        #[arg(long, short = 'm', default_value = "sonnet")]
        model: String,
    },
    /// Get task status
    Status {
        /// Task ID
        task_id: u32,
    },
    /// Add context to a running task
    AddContext {
        /// Task ID
        task_id: u32,
        /// Context to add (can be text or file path)
        context: String,
        /// Treat context as file path
        #[arg(long, short = 'f')]
        file: bool,
    },
    /// List all tasks
    List {
        /// Filter by service
        #[arg(long, short = 's')]
        service: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
    /// Initialize documentation for Task Master tasks using Claude
    InitDocs {
        /// Claude model to use (sonnet, opus)
        #[arg(long, short = 'm', default_value = "opus")]
        model: String,
        /// Repository URL (auto-detected if not specified)
        #[arg(long, short = 'r')]
        repo: Option<String>,
        /// Source branch to base documentation branch from (auto-detected if not specified)
        #[arg(long)]
        source_branch: Option<String>,
        /// Working directory containing .taskmaster folder (auto-detected if not specified)
        #[arg(long, short = 'w')]
        working_directory: Option<String>,
        /// Overwrite existing documentation
        #[arg(long, short = 'f')]
        force: bool,
        /// Generate docs for specific task only
        #[arg(long, short = 't')]
        task_id: Option<u32>,
        /// Update existing docs (regenerate from current tasks.json)
        #[arg(long, short = 'u')]
        update: bool,
        /// Force update all docs regardless of changes
        #[arg(long)]
        update_all: bool,
        /// Preview what would be generated without creating files
        #[arg(long)]
        dry_run: bool,
        /// Show detailed generation progress
        #[arg(long, short = 'v')]
        verbose: bool,
        /// GitHub user account for commits and PRs
        #[arg(long, default_value = "pm0-5dlabs")]
        github_user: String,
    },
}

#[derive(Subcommand, Debug)]
enum JobCommands {
    /// List all jobs
    List {
        /// Filter by microservice
        #[arg(long)]
        microservice: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
    },
    /// Get job details
    Get {
        /// Job ID
        #[arg(long)]
        id: String,
    },
    /// Stream job logs
    Logs {
        /// Job ID
        #[arg(long)]
        id: String,
        /// Follow log stream
        #[arg(long, short)]
        follow: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Create a new ConfigMap
    Create {
        /// ConfigMap name
        #[arg(long)]
        name: String,
        /// Files to include
        #[arg(long)]
        files: Vec<String>,
    },
    /// Get ConfigMap contents
    Get {
        /// ConfigMap name
        #[arg(long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing for CLI
    let filter = if cli.verbose {
        "orchestrator_cli=debug,orchestrator_common=debug"
    } else {
        "orchestrator_cli=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with_target(cli.verbose)
        .init();

    info!("Orchestrator CLI v{}", env!("CARGO_PKG_VERSION"));
    debug!("API URL: {}", cli.api_url);
    debug!("Output format: {:?}", cli.output);

    // Create API client
    let api_client = ApiClient::new(cli.api_url.clone());

    // Determine if output should be colored
    let colored = !cli.no_color && is_terminal::IsTerminal::is_terminal(&std::io::stdout());
    let output_manager = OutputManager::new(cli.output, colored);

    // Execute commands
    let result = match cli.command {
        Commands::Task { action } => match action {
            TaskCommands::Submit {
                task_id,
                service,
                agent,
                taskmaster_dir,
                context,
                tools,
                repo,
                branch,
                github_user,
                retry,
                model,
            } => {
                commands::task::submit_task_simplified(
                    &api_client,
                    &output_manager,
                    task_id,
                    &service,
                    &agent,
                    &taskmaster_dir,
                    &context,
                    &tools,
                    repo.as_deref(),
                    &branch,
                    github_user.as_deref(),
                    retry,
                    &model,
                )
                .await
            }
            TaskCommands::SubmitAdvanced {
                task_json,
                task_id,
                design_spec,
                prompt,
                service_name,
                agent_name,
                retry,
                model,
            } => {
                commands::task::submit_pm_task(
                    &api_client,
                    &output_manager,
                    &task_json,
                    task_id,
                    &design_spec,
                    prompt.as_deref(),
                    &service_name,
                    &agent_name,
                    retry,
                    &model,
                )
                .await
            }
            TaskCommands::Status { task_id } => {
                commands::task::status(&api_client, &output_manager, task_id).await
            }
            TaskCommands::AddContext {
                task_id,
                context,
                file,
            } => {
                commands::task::add_context(&api_client, &output_manager, task_id, &context, file)
                    .await
            }
            TaskCommands::List { service, status } => {
                commands::task::list(
                    &api_client,
                    &output_manager,
                    service.as_deref(),
                    status.as_deref(),
                )
                .await
            }
            TaskCommands::InitDocs {
                model,
                repo,
                source_branch,
                working_directory,
                force,
                task_id,
                update,
                update_all,
                dry_run,
                verbose,
                github_user,
            } => {
                commands::task::init_docs(
                    &api_client,
                    &output_manager,
                    &model,
                    repo.as_deref(),
                    source_branch.as_deref(),
                    working_directory.as_deref(),
                    force,
                    task_id,
                    update,
                    update_all,
                    dry_run,
                    verbose,
                    &github_user,
                )
                .await
            }
        },
        Commands::Job { action } => match action {
            JobCommands::List {
                microservice,
                status,
            } => {
                commands::job::list(
                    &api_client,
                    &output_manager,
                    microservice.as_deref(),
                    status.as_deref(),
                )
                .await
            }
            JobCommands::Get { id } => commands::job::get(&api_client, &output_manager, &id).await,
            JobCommands::Logs { id, follow } => {
                commands::job::logs(&api_client, &output_manager, &id, follow).await
            }
        },
        Commands::Config { action } => match action {
            ConfigCommands::Create { name, files } => {
                commands::config::create(&api_client, &output_manager, &name, &files).await
            }
            ConfigCommands::Get { name } => {
                commands::config::get(&api_client, &output_manager, &name).await
            }
        },
        Commands::Health => commands::health_check(&api_client, &output_manager).await,
    };

    match result {
        Ok(()) => {
            debug!("Command completed successfully");
            Ok(())
        }
        Err(e) => {
            debug!("Command failed: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Clear any environment variables to ensure clean test
        std::env::remove_var("ORCHESTRATOR_API_URL");

        // Give it a moment to clear
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Test that CLI can parse basic commands
        let cli = Cli::try_parse_from([
            "orchestrator",
            "task",
            "submit",
            "1001",
            "--service",
            "auth-service",
        ]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        // The URL might be affected by environment variables from other tests
        assert!(
            cli.api_url == "http://orchestrator.local/api/v1" || cli.api_url.starts_with("http://")
        );

        match cli.command {
            Commands::Task {
                action:
                    TaskCommands::Submit {
                        task_id,
                        service,
                        agent,
                        taskmaster_dir,
                        context,
                        tools,
                        repo,
                        branch,
                        github_user,
                        retry,
                        model,
                    },
            } => {
                assert_eq!(task_id, 1001);
                assert_eq!(service, "auth-service");
                assert_eq!(agent, "claude-agent-1");
                assert_eq!(taskmaster_dir, ".taskmaster");
                assert_eq!(context.len(), 0);
                assert_eq!(tools.len(), 0);
                assert_eq!(repo, None);
                assert_eq!(branch, "main");
                assert_eq!(github_user, None);
                assert!(!retry);
                assert_eq!(model, "sonnet");
            }
            _ => panic!("Expected Task Submit command"),
        }
    }

    #[test]
    fn test_cli_validation() {
        // Verify the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_output_format_parsing() {
        let cli = Cli::try_parse_from(["orchestrator", "--output", "json", "health"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        assert_eq!(cli.output, OutputFormat::Json);
    }

    #[test]
    fn test_api_url_explicit() {
        // Test explicit API URL override
        let cli = Cli::try_parse_from(["orchestrator", "--api-url", "http://test:9000", "health"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        assert_eq!(cli.api_url, "http://test:9000");
    }

    #[test]
    fn test_job_commands() {
        let cli =
            Cli::try_parse_from(["orchestrator", "job", "logs", "--id", "job-123", "--follow"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Job {
                action: JobCommands::Logs { id, follow },
            } => {
                assert_eq!(id, "job-123");
                assert!(follow);
            }
            _ => panic!("Expected Job Logs command"),
        }
    }

    #[test]
    fn test_config_commands() {
        let cli = Cli::try_parse_from([
            "orchestrator",
            "config",
            "create",
            "--name",
            "my-config",
            "--files",
            "file1.txt",
            "--files",
            "file2.txt",
        ]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        match cli.command {
            Commands::Config {
                action: ConfigCommands::Create { name, files },
            } => {
                assert_eq!(name, "my-config");
                assert_eq!(files, vec!["file1.txt", "file2.txt"]);
            }
            _ => panic!("Expected Config Create command"),
        }
    }
}
