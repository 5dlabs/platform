mod orchestrator_tools;

use anyhow::Result;
use rmcp::{
    transport::io::stdio,
    ServiceExt,
    ServerHandler,
    Error as McpError,
    model::{
        CallToolResult,
        Content,
        ServerInfo,
        ServerCapabilities,
        ProtocolVersion,
        Implementation,
        PaginatedRequestParam,
        ListResourcesResult,
        ListResourceTemplatesResult,
        ListPromptsResult,
        ReadResourceRequestParam,
        ReadResourceResult,
        GetPromptRequestParam,
        GetPromptResult,
    },
    service::{RequestContext, RoleServer},
    tool,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone)]
struct OrchestratorService;

#[derive(Debug, Deserialize, JsonSchema)]
struct InitDocsArgs {
    #[schemars(description = "Claude model to use ('sonnet' or 'opus', default: 'opus')")]
    model: Option<String>,
    #[schemars(description = "Working directory containing .taskmaster folder (auto-detected from TASKMASTER_ROOT env var if not specified)")]
    working_directory: Option<String>,
    #[schemars(description = "Overwrite existing documentation (default: false)")]
    force: Option<bool>,
    #[schemars(description = "Generate docs for specific task only (default: generates for all tasks)")]
    task_id: Option<u32>,
}

#[tool(tool_box)]
impl OrchestratorService {
    #[tool(description = "Initialize documentation for Task Master tasks using Claude\n\nExamples:\n- Generate docs for all tasks: init_docs({})\n- Use specific model: init_docs({model: 'opus'})\n- Target specific task: init_docs({task_id: 5})\n- Override directory: init_docs({working_directory: '/path/to/project'})\n- Force overwrite: init_docs({force: true})\n\nParameters (all optional):\n- model: 'opus' (default) | 'sonnet'\n- working_directory: auto-detected from TASKMASTER_ROOT env var\n- force: false (default) - set true to overwrite existing docs\n- task_id: null (default) - generates docs for all tasks")]
    async fn init_docs(
        &self,
        #[tool(aggr)] args: InitDocsArgs,
    ) -> Result<CallToolResult, McpError> {
        // Validate model parameter
        let model = args.model.as_deref().unwrap_or("opus");
        if !["sonnet", "opus"].contains(&model) {
            return Err(McpError::invalid_params(
                format!("Invalid model '{}' - must be 'sonnet' or 'opus'", model),
                None
            ));
        }

        let working_directory = args.working_directory.as_deref();
        let force = args.force.unwrap_or(false);
        let task_id = args.task_id;

        match orchestrator_tools::init_docs(model, working_directory, force, task_id) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => {
                // Provide more specific error messages
                let error_msg = if e.to_string().contains("No such file or directory") {
                    format!("Directory not found: {}. Please check the working_directory path or TASKMASTER_ROOT env var.", e)
                } else if e.to_string().contains("tasks.json") {
                    format!("Task Master tasks.json not found: {}. Please ensure you're in a valid Task Master project directory.", e)
                } else if e.to_string().contains("orchestrator command") {
                    format!("Orchestrator CLI not found: {}. Please ensure the orchestrator CLI is installed and in PATH.", e)
                } else {
                    format!("Documentation generation failed: {}", e)
                };

                Err(McpError::internal_error(error_msg, None))
            }
        }
    }

    #[tool(description = "Test MCP server connectivity and configuration\n\nReturns server status, environment info, and validates orchestrator CLI availability.")]
    async fn ping(
        &self,
    ) -> Result<CallToolResult, McpError> {
        match orchestrator_tools::ping_test() {
            Ok(status) => Ok(CallToolResult::success(vec![Content::text(status)])),
            Err(e) => Err(McpError::internal_error(format!("Ping failed: {}", e), None)),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for OrchestratorService {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .build();

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities,
            server_info: Implementation {
                name: "Orchestrator MCP Server".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("This server provides tools to initialize documentation for Task Master tasks using Claude.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::resource_not_found("No resources available".to_string(), None))
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    async fn get_prompt(
        &self,
        _request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params("No prompts available".to_string(), None))
    }

    async fn list_resource_templates(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = OrchestratorService;

    // Start the server and get a handle
    let server_handle = service.serve(stdio()).await?;

    // Wait for the server to complete (keeps it running)
    server_handle.waiting().await?;

    Ok(())
}