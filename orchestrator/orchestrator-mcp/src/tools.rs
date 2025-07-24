use serde_json::{json, Value};

/// Get all tool schemas with descriptions and parameter definitions
pub fn get_all_tool_schemas() -> Value {
    json!({
        "tools": [
            get_init_docs_schema(),
            get_submit_implementation_task_schema()
        ]
    })
}

fn get_init_docs_schema() -> Value {
    json!({
        "name": "init_docs",
        "description": "Initialize documentation for Task Master tasks using Claude",
        "inputSchema": {
            "type": "object",
            "properties": {
                "model": {
                    "type": "string",
                    "description": "Claude model to use ('sonnet' or 'opus', default: 'opus')",
                    "enum": ["sonnet", "opus"],
                    "default": "opus"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (auto-detected from current directory if not specified). Use relative paths like '-projects/trader'."
                },
                "force": {
                    "type": "boolean",
                    "description": "Overwrite existing documentation (default: false)",
                    "default": false
                },
                "task_id": {
                    "type": "integer",
                    "description": "Generate docs for specific task only (default: generates for all tasks)",
                    "minimum": 0
                }
            }
        }
    })
}

fn get_submit_implementation_task_schema() -> Value {
    json!({
        "name": "submit_implementation_task",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "REQUIRED: Task ID to implement from tasks.json",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "REQUIRED: Target service name (creates workspace-{service} PVC)",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository_url": {
                    "type": "string",
                    "description": "Target project repository URL (where implementation work happens, auto-detected from .git/config if not specified)"
                },
                "docs_repository_url": {
                    "type": "string",
                    "description": "Documentation repository URL (where Task Master definitions come from, auto-detected from current git repo if not specified)"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g. '_projects/simple-api')"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (defaults to service name)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (default: 'claude-3-5-sonnet-20241022')",
                    "default": "claude-3-5-sonnet-20241022"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (auto-detected if not specified)"
                },
                "local_tools": {
                    "type": "string",
                    "description": "Comma-separated list of local MCP tools/servers to enable (e.g., 'mcp-server-git,taskmaster')"
                },
                "remote_tools": {
                    "type": "string",
                    "description": "Comma-separated list of remote MCP tools/servers to enable (e.g., 'api-docs-tool')"
                },
                "context_version": {
                    "type": "integer",
                    "description": "Context version for retry attempts (incremented on each retry, default: 1)",
                    "default": 1,
                    "minimum": 1
                },
                "prompt_modification": {
                    "type": "string",
                    "description": "Additional context for retry attempts"
                },
                "docs_branch": {
                    "type": "string",
                    "description": "Docs branch to use (e.g., 'main', 'feature/branch', default: 'main')",
                    "default": "main"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (auto-continue on retries or user-requested, default: false)",
                    "default": false
                },
                "overwrite_memory": {
                    "type": "boolean",
                    "description": "Whether to overwrite memory before starting (default: false)",
                    "default": false
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables to set in the container (key-value pairs)",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "env_from_secrets": {
                    "type": "array",
                    "description": "Environment variables from secrets",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Name of the environment variable"
                            },
                            "secretName": {
                                "type": "string",
                                "description": "Name of the secret"
                            },
                            "secretKey": {
                                "type": "string",
                                "description": "Key within the secret"
                            }
                        },
                        "required": ["name", "secretName", "secretKey"]
                    }
                }
            },
            "required": ["task_id", "service"]
        }
    })
}
