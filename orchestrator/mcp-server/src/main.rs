use anyhow::Result;
use clap::Parser;
use serde_json::{json, Value};
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use tracing::info;

mod orchestrator_tools;

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
                                "name": "init_docs",
                                "description": "Initialize documentation for Task Master tasks using Claude (orchestrator task init-docs)",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "model": {
                                            "type": "string",
                                            "description": "Claude model to use (sonnet, opus)",
                                            "default": "opus",
                                            "enum": ["sonnet", "opus"]
                                        },
                                        "working_directory": {
                                            "type": "string",
                                            "description": "Working directory containing .taskmaster folder (auto-detected if not specified)"
                                        },
                                        "force": {
                                            "type": "boolean",
                                            "description": "Overwrite existing documentation",
                                            "default": false
                                        },
                                        "task_id": {
                                            "type": "integer",
                                            "description": "Generate docs for specific task only"
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
                    "init_docs" => {
                        let model = arguments["model"]
                            .as_str()
                            .unwrap_or("opus");
                        let working_directory = arguments["working_directory"]
                            .as_str();
                        let force = arguments["force"]
                            .as_bool()
                            .unwrap_or(false);
                        let task_id = arguments["task_id"]
                            .as_u64()
                            .map(|id| id as u32);

                        info!("Initializing documentation with model: {}", model);
                        
                        // Log the working directory resolution
                        match orchestrator_tools::find_taskmaster_root(working_directory) {
                            Ok(root) => info!("Using Task Master root: {}", root.display()),
                            Err(e) => info!("Failed to find Task Master root: {}", e),
                        }

                        match orchestrator_tools::init_docs(model, working_directory, force, task_id) {
                            Ok(output) => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "content": [
                                            {
                                                "type": "text",
                                                "text": format!("Documentation generation initiated successfully!\n\n{}", output)
                                            }
                                        ]
                                    }
                                });
                                Ok(response)
                            }
                            Err(e) => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": {
                                        "content": [
                                            {
                                                "type": "text",
                                                "text": format!("Failed to initialize documentation: {}", e)
                                            }
                                        ],
                                        "isError": true
                                    }
                                });
                                Ok(response)
                            }
                        }
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