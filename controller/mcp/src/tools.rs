use serde_json::{json, Value};
use crate::agents::AgentsConfig;

/// Get all tool schemas with descriptions and parameter definitions
pub fn get_all_tool_schemas() -> Value {
    json!({
        "tools": [
            get_init_docs_schema(),
            get_submit_implementation_task_schema()
        ]
    })
}

/// Get tool schemas enhanced with current agents team info
pub fn get_enhanced_tool_schemas(agents_config: &AgentsConfig) -> Value {
    json!({
        "tools": [
            get_enhanced_docs_schema(agents_config),
            get_enhanced_task_schema(agents_config)
        ]
    })
}

fn get_init_docs_schema() -> Value {
    json!({
        "name": "docs",
        "description": "Initialize documentation for Task Master tasks using Claude",
        "inputSchema": {
            "type": "object",
            "properties": {
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like '_projects/simple-api'."
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to Helm configuration value)"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (optional if FDL_DEFAULT_DOCS_USER environment variable is set, which takes precedence)"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_submit_implementation_task_schema() -> Value {
    json!({
        "name": "task",
        "description": "Submit a Task Master task for implementation using Claude with persistent workspace",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "REQUIRED: Task ID to implement from task files",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "REQUIRED: Target service name (creates workspace-{service} PVC) - can be overridden by FDL_DEFAULT_SERVICE environment variable",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository": {
                    "type": "string",
                    "description": "REQUIRED: Target repository in format 'org/repo' or 'user/repo' (e.g., '5dlabs/platform')"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "REQUIRED: Documentation repository in format 'org/repo' or 'user/repo' where Task Master definitions are stored"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "REQUIRED: Project directory within docs repository (e.g., '_projects/simple-api', use '.' for repo root)"
                },
                "agent": {
                    "type": "string",
                    "description": "AI agent name to use for this task (optional, defaults to rex for code tasks)"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (optional, maintained for backward compatibility)"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (optional, defaults to '.' for repo root)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to Helm configuration value)"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (optional, defaults to false)"
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
            "required": ["task_id", "service", "repository", "docs_repository", "docs_project_directory"]
        }
    })
}

fn get_enhanced_docs_schema(agents_config: &AgentsConfig) -> Value {
    let team_info = agents_config.get_team_description();
    let description = format!(
        "Initialize documentation for Task Master tasks using Claude. {}",
        team_info
    );
    
    json!({
        "name": "docs",
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": {
                "working_directory": {
                    "type": "string",
                    "description": "Working directory containing .taskmaster folder (required). Use relative paths like '_projects/simple-api'."
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to agent configuration)"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (deprecated - use agent names instead)"
                },
                "github_app": {
                    "type": "string", 
                    "description": "GitHub App name or agent name (e.g., 'Morgan', '5DLabs-Morgan')"
                }
            },
            "required": ["working_directory"]
        }
    })
}

fn get_enhanced_task_schema(agents_config: &AgentsConfig) -> Value {
    let team_info = agents_config.get_team_description();
    let description = format!(
        "Submit a Task Master task for implementation using Claude with persistent workspace. {}",
        team_info
    );
    
    json!({
        "name": "task",
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "REQUIRED: Task ID to implement from task files",
                    "minimum": 1
                },
                "service": {
                    "type": "string",
                    "description": "REQUIRED: Target service name (creates workspace-{service} PVC)",
                    "pattern": "^[a-z0-9-]+$"
                },
                "repository": {
                    "type": "string",
                    "description": "REQUIRED: Target repository in format 'org/repo' or 'user/repo' (e.g., '5dlabs/platform')"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "REQUIRED: Documentation repository in format 'org/repo' or 'user/repo' where Task Master definitions are stored"
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "REQUIRED: Project directory within docs repository (e.g., '_projects/simple-api', use '.' for repo root)"
                },
                "github_user": {
                    "type": "string",
                    "description": "GitHub username for authentication (deprecated - use agent names instead)"
                },
                "github_app": {
                    "type": "string",
                    "description": "GitHub App name or agent name (e.g., 'Rex', 'Blaze', '5DLabs-Rex')"
                },
                "working_directory": {
                    "type": "string",
                    "description": "Working directory within target repository (optional, defaults to '.' for repo root)"
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use (optional, defaults to agent configuration)"
                },
                "continue_session": {
                    "type": "boolean",
                    "description": "Whether to continue a previous session (optional, defaults to false)"
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
            "required": ["task_id", "service", "repository", "docs_repository", "docs_project_directory"]
        }
    })
}