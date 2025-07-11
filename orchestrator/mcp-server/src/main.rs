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
use serde_json::json;

#[derive(Clone)]
struct OrchestratorService;

#[derive(Debug, Deserialize, JsonSchema)]
struct InitDocsArgs {
    #[schemars(description = "Claude model to use (sonnet, opus)")]
    model: Option<String>,
    #[schemars(description = "Working directory containing .taskmaster folder (auto-detected if not specified)")]
    working_directory: Option<String>,
    #[schemars(description = "Overwrite existing documentation")]
    force: Option<bool>,
    #[schemars(description = "Generate docs for specific task only")]
    task_id: Option<u32>,
}

#[tool(tool_box)]
impl OrchestratorService {
    #[tool(description = "Initialize documentation for Task Master tasks using Claude")]
    async fn init_docs(
        &self,
        #[tool(aggr)] args: InitDocsArgs,
    ) -> Result<CallToolResult, McpError> {
        let model = args.model.as_deref().unwrap_or("opus");
        let working_directory = args.working_directory.as_deref();
        let force = args.force.unwrap_or(false);
        let task_id = args.task_id;

        match orchestrator_tools::init_docs(model, working_directory, force, task_id) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Err(McpError::internal_error(format!("Failed: {}", e), None)),
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