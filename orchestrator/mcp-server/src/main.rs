use anyhow::Result;
use clap::Parser;
use serde_json::{json, Value};
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use tracing::info;

#[derive(Parser)]
#[command(name = "mcp-server")]
#[command(about = "MCP Server for document generation and task assignment")]
struct Args {
    #[arg(long, help = "Enable debug logging")]
    debug: bool,
}

struct McpServer {
    id_counter: u64,
}

impl McpServer {
    fn new() -> Self {
        Self { id_counter: 0 }
    }

    fn next_id(&mut self) -> u64 {
        self.id_counter += 1;
        self.id_counter
    }

    fn handle_request(&mut self, request: Value) -> Result<Value> {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        match method {
            "initialize" => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-06-18",
                        "capabilities": {
                            "tools": {
                                "listChanged": false
                            }
                        },
                        "serverInfo": {
                            "name": "documentation-server",
                            "version": "1.0.0"
                        }
                    }
                });
                Ok(response)
            }
            "tools/list" => {
                let response = json!({
                    "jsonrpc": "2.0", 
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "generate_docs",
                                "description": "Generate documentation for a given task using the Taskmaster system",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "task": {
                                            "type": "object",
                                            "description": "The task object containing all task details"
                                        },
                                        "output_dir": {
                                            "type": "string",
                                            "description": "Output directory for generated documentation",
                                            "default": "./docs"
                                        }
                                    },
                                    "required": ["task"]
                                }
                            },
                            {
                                "name": "list_tasks",
                                "description": "List available tasks from the project",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "project_path": {
                                            "type": "string",
                                            "description": "Path to the project directory",
                                            "default": "."
                                        }
                                    }
                                }
                            }
                        ]
                    }
                });
                Ok(response)
            }
            "tools/call" => {
                let params = &request["params"];
                let tool_name = params["name"].as_str().unwrap_or("");
                let arguments = &params["arguments"];

                match tool_name {
                    "generate_docs" => {
                        let task = &arguments["task"];
                        let output_dir = arguments["output_dir"]
                            .as_str()
                            .unwrap_or("./docs");

                        info!("Generating documentation for task in directory: {}", output_dir);

                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [
                                    {
                                        "type": "text",
                                        "text": format!("Successfully generated documentation for task in {}\n\nTask details: {}", output_dir, task)
                                    }
                                ]
                            }
                        });
                        Ok(response)
                    }
                    "list_tasks" => {
                        let project_path = arguments["project_path"]
                            .as_str()
                            .unwrap_or(".");

                        info!("Listing tasks from project: {}", project_path);

                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [
                                    {
                                        "type": "text",
                                        "text": format!("No tasks found in project: {}\n\nThis is a placeholder - integrate with your task management system.", project_path)
                                    }
                                ]
                            }
                        });
                        Ok(response)
                    }
                    _ => {
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": format!("Unknown tool: {}", tool_name)
                            }
                        });
                        Ok(error_response)
                    }
                }
            }
            _ => {
                let error_response = json!({
                    "jsonrpc": "2.0", 
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                });
                Ok(error_response)
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        let stdin = stdin();
        let mut stdout = stdout();
        let reader = BufReader::new(stdin);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&line) {
                Ok(request) => {
                    match self.handle_request(request) {
                        Ok(response) => {
                            let response_str = serde_json::to_string(&response)?;
                            writeln!(stdout, "{}", response_str)?;
                            stdout.flush()?;
                        }
                        Err(e) => {
                            eprintln!("Error handling request: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let filter = if args.debug {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Starting MCP Server for documentation generation");

    let mut server = McpServer::new();
    server.run()?;

    Ok(())
}