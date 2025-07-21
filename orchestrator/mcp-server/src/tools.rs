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
        "description": "Initialize documentation for Task Master tasks using Claude\n\n**Recommended Usage**: For most cases, call without parameters to use auto-detection from current directory.\n\n**Examples**:\n- All tasks with defaults: init_docs()\n- Specific model: init_docs({model: 'opus'})\n- Specific task: init_docs({task_id: 5})\n- Custom directory: init_docs({working_directory: '-projects/trader'})\n- Force overwrite: init_docs({force: true})\n- Full specification: init_docs({model: 'opus', working_directory: '-projects/trader', force: true, task_id: 5})\n\n**Parameters (all optional with robust defaults)**:\n- model: 'opus' (default) | 'sonnet' - Claude model to use\n- working_directory: auto-detected from current directory (default) | 'relative/path' - relative path from current directory\n- force: false (default) | true - set true to overwrite existing docs\n- task_id: null (default, generates docs for all tasks) | number - generate docs for specific task only\n\n**Robust Defaults Applied**:\n- No parameters uses: model='opus', force=false, auto-detect working_directory, all tasks\n- Missing parameters are automatically filled with safe defaults\n- All parameter combinations are supported\n\n**Common Errors & Fixes**:\n- If working_directory fails: Ensure path exists and has no trailing slash\n- If auto-detection fails: Specify working_directory parameter explicitly\n- Invalid model: Must be 'sonnet' or 'opus'\n- Directory not found: Verify the path is accessible and contains a .taskmaster folder",
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
        "description": "Submit a Task Master task for implementation using Claude\n\n**Purpose**: Starts Claude-powered implementation of specific tasks with persistent workspace and checkpoint system.\n\n**Examples**:\n- Basic task: submit_implementation_task({task_id: 5, service: 'auth-service'})\n- With custom repo: submit_implementation_task({task_id: 3, service: 'api-gateway', repository_url: 'https://github.com/org/repo'})\n- Retry attempt: submit_implementation_task({task_id: 7, service: 'user-service', retry: true})\n- Custom model: submit_implementation_task({task_id: 2, service: 'payment', model: 'opus'})\n\n**Key Features**:\n- **Persistent Workspace**: Uses PVC for multi-attempt implementations\n- **Checkpoint System**: Automatically saves progress for resumability\n- **Incremental Commits**: Commits work progressively during implementation\n- **Retry Support**: Can resume from previous attempts using checkpoint data\n- **Auto PVC Creation**: Creates service workspace PVC if it doesn't exist\n\n**Parameters**:\n- task_id: REQUIRED - ID of the task to implement from tasks.json\n- service: REQUIRED - Target service name (creates workspace-{service} PVC)\n- working_directory: auto-detected (default) | path - Task Master project directory\n- repository_url: auto-detected (default) | URL - Git repository to clone/update\n- branch: 'main' (default) | string - Git branch to work on\n- model: 'sonnet' (default) | 'opus' - Claude model for implementation\n- agent: 'claude-agent-1' (default) - Agent identifier\n- retry: false (default) | true - Is this a retry of previous attempt?\n- github_user: auto-detected (default) | string - GitHub username for auth\n\n**Workspace Structure**:\n- PVC: workspace-{service} (auto-created if needed)\n- Path: /workspace/{service}/ (persistent across attempts)\n- Checkpoints: /workspace/{service}/.orchestrator/task-{id}-checkpoint.json\n- Progress: /workspace/{service}/.orchestrator/task-{id}-progress.log\n\n**Implementation Process**:\n1. Create/validate service PVC\n2. Clone/update repository in persistent workspace\n3. Load/create checkpoint for task\n4. Execute Claude with implementation prompt and checkpoint context\n5. Save progress incrementally with checkpoint system\n6. Commit work progressively during implementation\n\n**Retry Behavior**:\n- Loads existing checkpoint and progress log\n- Provides Claude with previous attempt context\n- Resumes from last successful checkpoint\n- Preserves all previous commits and progress\n\n**Common Errors & Fixes**:\n- Task not found: Verify task_id exists in tasks.json\n- No .taskmaster folder: Ensure working_directory contains Task Master project\n- PVC creation failed: Check Kubernetes permissions and storage class\n- Repository access denied: Verify GitHub authentication and repository URL\n- Checkpoint corruption: Use retry=false to start fresh (previous work preserved in git)",
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
                    "description": "Task Master project directory (auto-detected if not specified)"
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
                }
            },
            "required": ["task_id", "service"]
        }
    })
}