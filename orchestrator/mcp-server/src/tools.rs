use serde_json::{json, Value};

/// Get all tool schemas with descriptions and parameter definitions
pub fn get_all_tool_schemas() -> Value {
    json!({
        "tools": [
            get_init_docs_schema(),
            get_ping_schema()
        ]
    })
}

fn get_init_docs_schema() -> Value {
    json!({
        "name": "init_docs",
        "description": "Initialize documentation for Task Master tasks using Claude\n\n**Recommended Usage**: For most cases, call without parameters to use auto-detection from current directory.\n\n**Examples**:\n- All tasks with defaults: init_docs()\n- Specific model: init_docs({model: 'opus'})\n- Specific task: init_docs({task_id: 5})\n- Custom directory: init_docs({working_directory: '/absolute/path/to/project'})\n- Force overwrite: init_docs({force: true})\n- Full specification: init_docs({model: 'opus', working_directory: '/path/to/project', force: true, task_id: 5})\n\n**Parameters (all optional with robust defaults)**:\n- model: 'opus' (default) | 'sonnet' - Claude model to use\n- working_directory: auto-detected from current directory (default) | '/absolute/path' - must be absolute path if provided\n- force: false (default) | true - set true to overwrite existing docs\n- task_id: null (default, generates docs for all tasks) | number - generate docs for specific task only\n\n**Robust Defaults Applied**:\n- No parameters uses: model='opus', force=false, auto-detect working_directory, all tasks\n- Missing parameters are automatically filled with safe defaults\n- All parameter combinations are supported\n\n**Common Errors & Fixes**:\n- If working_directory fails: Ensure path exists, is absolute, and has no trailing slash\n- If auto-detection fails: Specify working_directory parameter explicitly\n- Invalid model: Must be 'sonnet' or 'opus'\n- Directory not found: Verify the path is accessible and contains a .taskmaster folder",
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
                    "description": "Working directory containing .taskmaster folder (auto-detected from current directory if not specified)"
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

fn get_ping_schema() -> Value {
    json!({
        "name": "ping",
        "description": "Test MCP server connectivity and configuration\n\nReturns server status, environment info, and validates orchestrator CLI availability.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "random_string": {
                    "type": "string",
                    "description": "Dummy parameter for no-parameter tools"
                }
            }
        }
    })
}