use serde_json::{json, Value};

/// Get tool schemas for MCP protocol with rich descriptions
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_docs_schema(),
            get_task_schema(),
            get_export_schema()
        ]
    })
}


fn get_docs_schema() -> Value {
    json!({
        "name": "docs",
        "description": "Initialize documentation for Task Master tasks using Claude",
        "inputSchema": {
            "type": "object",
            "properties": {
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like 'projects/market-research'."
                },
                "agent": {
                    "type": "string",
                    "description": "Agent name for task assignment (optional, uses workflow default if not specified)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "include_codebase": {
                    "type": "boolean",
                    "description": "Include existing codebase as markdown context (optional, defaults to false)"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_task_schema() -> Value {
    json!({
        "name": "task",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "Target service name (creates workspace-{service} PVC)",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., https://github.com/5dlabs/cto)"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g., projects/market-research)"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "Documentation repository URL (optional, defaults to configured value)"
                },
                "agent": {
                    "type": "string",
                    "description": "Agent name for task assignment (e.g., morgan, rex, blaze, cipher)"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (optional, defaults to '.')"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to configuration)"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (optional, defaults to false)"
                },
                "overwrite_memory": {
                    "type": "boolean",
                    "description": "Whether to overwrite CLAUDE.md memory file (optional, defaults to false)"
                },
                "env": {
                    "type": "object",
                    "description": "Environment variables to set in the container (optional)",
                    "additionalProperties": {
                        "type": "string"
                    }
                },
                "env_from_secrets": {
                    "type": "array",
                    "description": "Environment variables from secrets (optional)",
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
            "required": ["task_id", "service", "repository", "docs_project_directory"]
        }
    })
}

fn get_export_schema() -> Value {
    json!({
        "name": "export",
        "description": "Export Rust codebase to markdown for documentation context",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    })
}