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
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (defaults to service name)"
                },
                "platform_repository_url": {
                    "type": "string",
                    "description": "Platform repository URL for documentation access (auto-detected from current git repo if not specified)"
                },
                "repository_url": {
                    "type": "string",
                    "description": "Git repository URL (auto-detected from .git/config if not specified)"
                },
                "branch": {
                    "type": "string",
                    "description": "Git branch to work on (default: 'main')",
                    "default": "main"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (default: 'sonnet')",
                    "enum": ["sonnet", "opus"],
                    "default": "sonnet"
                },
                "agent": {
                    "type": "string",
                    "description": "Agent identifier (default: 'claude-agent-1')",
                    "default": "claude-agent-1"
                },
                "retry": {
                    "type": "boolean",
                    "description": "Is this a retry of a previous attempt? (default: false)",
                    "default": false
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (auto-detected if not specified)"
                },
                "prompt_modification": {
                    "type": "string",
                    "description": "Additional prompt instructions for retry attempts (used when retry=true)"
                },
                "prompt_mode": {
                    "type": "string",
                    "description": "How to apply prompt_modification: 'append' (add to existing prompt) or 'replace' (replace system prompt)",
                    "enum": ["append", "replace"],
                    "default": "append"
                }
            },
            "required": ["task_id", "service"]
        }
    })
}