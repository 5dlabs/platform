# Autonomous Agent Prompt: Implement Docs Agent Tool Discovery

## Implementation Status (July 26, 2025)

### Completed ✅
- **Toolman RBAC**: Role and RoleBinding templates added for ConfigMap permissions
- **Tool Discovery Fixed**: Deployed image `main-5724488` - now discovers 48 tools
- **ConfigMap Creation**: Implemented in Toolman (deployed as `main-adfad50`)
- **Orchestrator Mounting**: ConfigMap mounted to `/etc/tool-catalog` in agents
- **Local Tools Discovery**: Dynamic discovery from local MCP servers (no hardcoding)
- **Server-side Apply**: Fixed conflicts with `.force()` for ConfigMap updates

### Current State 🎯
- **Task 5 is COMPLETE**: All orchestrator and Toolman components implemented
- **Toolman creates**: `toolman-tool-catalog` ConfigMap with full tool metadata
- **Local tools**: Discovered dynamically from `toolman-local-tools` ConfigMap
- **Remote tools**: Discovered from Toolman-proxied MCP servers

### Pending (Separate Work) 🔄
- **Docs Agent Implementation**: Read mounted catalog and implement matching logic
- This is separate from orchestrator work and runs inside the agent container

---

## Context
You are implementing the tool discovery functionality for the docs agent. This is a critical component that reads the Toolman ConfigMap to discover available MCP tools and generates optimal tool configurations based on project analysis. This implementation must follow the zero-hardcoding principle - no tool names should be hardcoded.

**Important**: The primary use case is **greenfield projects** where the docs agent must determine needed tools based on task requirements, not by scanning existing code.

## Your Mission
Implement the complete tool discovery and recommendation system in the docs agent, ensuring it dynamically discovers tools and makes intelligent recommendations based on task requirements.

## Key Requirements

### 1. Tool Catalog ConfigMap
- **Create** a new ConfigMap called `toolman-tool-catalog` in the orchestrator namespace
- **Populate** with comprehensive tool information:
  - Local tools (filesystem, git) with descriptions and use cases
  - Remote tools discovered from MCP servers
  - Tool metadata: categories, descriptions, use cases, schemas
- **Update** the catalog whenever Toolman starts up
- **RBAC**: Ensure Toolman has permissions to create/update this ConfigMap

### 2. RBAC Configuration
Add RBAC resources to the Toolman Helm chart:
- Create `role.yaml` with permissions to:
  - Read all ConfigMaps (for discovery)
  - Create/update the `toolman-tool-catalog` ConfigMap
- Create `rolebinding.yaml` to bind the role to Toolman's ServiceAccount
- Update `values.yaml` to include `rbac.create: true`

## Toolman Service Information
- **Deployment Namespace**: `orchestrator`
- **Service URL**: `http://toolman.orchestrator.svc.cluster.local:3000`
- **ConfigMap Name**: `toolman-config`
- **ConfigMap Key**: `servers-config.json`
- **Service Type**: ClusterIP on port 3000

## Implementation Requirements

### 1. Set Up the Foundation
```rust
// Add required dependencies to Cargo.toml
[dependencies]
k8s-openapi = { version = "0.20", features = ["v1_29"] }
kube = { version = "0.87", features = ["runtime", "derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tokio = { version = "1", features = ["full"] }
glob = "0.3"
chrono = "0.4"
log = "0.4"
```

### 2. Implement Core Data Structures
```rust
// In docs_handler.rs or similar
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use anyhow::{anyhow, Result};

#[derive(Debug, Deserialize)]
struct ToolmanConfig {
    servers: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ProjectToolConfig {
    pub local: Vec<String>,
    pub remote: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProjectConfig {
    tools: ProjectToolConfig,
    generated_at: String,
    project_analysis: ProjectAnalysis,
    docs_run_id: String,
}

#[derive(Debug, Serialize, Default)]
struct ProjectAnalysis {
    has_kubernetes: bool,
    has_database: bool,
    has_ci_cd: bool,
    has_terraform: bool,
    detected_languages: Vec<String>,
    detected_frameworks: Vec<String>,
    file_patterns_found: Vec<String>,
}
```

### 3. Implement ConfigMap Discovery
```rust
pub struct DocsHandler {
    k8s_client: Client,
    namespace: String,
}

impl DocsHandler {
    pub fn new(k8s_client: Client) -> Self {
        Self {
            k8s_client,
            namespace: "orchestrator".to_string(),
        }
    }

    /// Discover available tools from Toolman ConfigMap
    pub async fn discover_available_tools(&self) -> Result<Vec<String>> {
        log::info!("Discovering available MCP tools from ConfigMap");

        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            "orchestrator"  // Toolman is deployed in orchestrator namespace
        );

        // Read toolman-config ConfigMap
        let cm = configmaps.get("toolman-config").await?;

        // Extract and parse servers-config.json
        let config_json = cm.data
            .and_then(|d| d.get("servers-config.json"))
            .ok_or("Missing servers-config.json")?;

        let config: ToolmanConfig = serde_json::from_str(config_json)?;
        Ok(config.servers.keys().cloned().collect())
    }
}
```

### 4. Implement Project Analysis
```rust
impl DocsHandler {
    /// Analyze task content to understand tool requirements (greenfield focus)
    pub async fn analyze_task(&self, task_content: &str) -> Result<TaskAnalysis> {
        log::info!("Analyzing task requirements for tool selection");

        let mut analysis = TaskAnalysis::default();

        // Check for technology keywords in task content
        if task_content.to_lowercase().contains("kubernetes") ||
           task_content.to_lowercase().contains("k8s") ||
           task_content.to_lowercase().contains("deployment") {
            analysis.needs_kubernetes = true;
            analysis.technologies_mentioned.push("kubernetes".to_string());
        }

        // Check for infrastructure keywords
        if task_content.to_lowercase().contains("terraform") ||
           task_content.to_lowercase().contains("infrastructure") {
            analysis.needs_terraform = true;
            analysis.technologies_mentioned.push("terraform".to_string());
        }

        // Check for database keywords
        if task_content.to_lowercase().contains("database") ||
           task_content.to_lowercase().contains("postgres") ||
           task_content.to_lowercase().contains("sql") {
            analysis.needs_database = true;
            analysis.technologies_mentioned.push("database".to_string());
        }

        // Check for programming language mentions
        if task_content.to_lowercase().contains("rust") {
            analysis.languages_mentioned.push("rust".to_string());
        }

        log::info!("Task analysis complete: {:?}", analysis);
        Ok(analysis)
    }
}
```

### 5. Implement Tool Matching Logic
```rust
impl DocsHandler {
    /// Match task requirements with available tools
    pub fn match_tools_to_task(
        &self,
        analysis: &TaskAnalysis,
        available_tools: &[String]
    ) -> ProjectToolConfig {
        let mut config = ProjectToolConfig::default();

        // Always include filesystem for local file operations
        config.local.push("filesystem".to_string());

        // Match based on task requirements
        if analysis.needs_kubernetes {
            for tool in available_tools {
                if tool.contains("kubernetes") || tool.contains("k8s") {
                    config.remote.push(tool.clone());
                }
            }
        }

        if analysis.needs_terraform {
            for tool in available_tools {
                if tool.contains("terraform") {
                    config.remote.push(tool.clone());
                }
            }
        }

        if analysis.languages_mentioned.contains(&"rust".to_string()) {
            for tool in available_tools {
                if tool.contains("rustdocs") {
                    config.remote.push(tool.clone());
                }
            }
        }

        // Remove duplicates and sort
        config.remote.sort();
        config.remote.dedup();

        log::info!("Matched tools - Local: {:?}, Remote: {:?}",
                   config.local, config.remote);

        config
    }
}
```

### 6. Implement Configuration Storage
```rust
impl DocsHandler {
    /// Save tool configuration in task folder
    pub async fn save_task_tool_config(
        &self,
        task_id: &str,
        config: ProjectToolConfig
    ) -> Result<()> {
        log::info!("Saving tool configuration for task: {}", task_id);

        let tool_config = ToolConfig {
            tools: config,
            reasoning: "Based on task requirements".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        let config_json = serde_json::to_string_pretty(&tool_config)?;

        // Save to task folder
        let config_path = format!(".taskmaster/docs/task-{}/tools.json", task_id);
        tokio::fs::create_dir_all(format!(".taskmaster/docs/task-{}", task_id)).await?;
        tokio::fs::write(&config_path, config_json).await?;

        log::info!("Tool configuration saved to: {}", config_path);
        Ok(())
    }
}
```

### 7. Main Workflow Implementation
```rust
impl DocsHandler {
    /// Complete tool discovery and configuration workflow for greenfield projects
    pub async fn generate_task_tool_configuration(
        &self,
        task_id: &str,
        task_content: &str
    ) -> Result<ProjectToolConfig> {
        log::info!("Starting tool discovery for task: {}", task_id);

        // Step 1: Discover available tools from catalog
        let available_tools = self.discover_available_tools().await?;
        if available_tools.is_empty() {
            log::warn!("No tools discovered from ConfigMap, using minimal defaults");
            return Ok(ProjectToolConfig {
                local: vec!["filesystem".to_string()],
                remote: vec![],
            });
        }

        // Step 2: Analyze task content
        let analysis = self.analyze_task(task_content).await?;

        // Step 3: Match tools to task requirements
        let tool_config = self.match_tools_to_task(&analysis, &available_tools);

        // Step 4: Save configuration to task folder
        self.save_task_tool_config(task_id, tool_config.clone()).await?;

        log::info!("Tool discovery complete for task: {}", task_id);
        Ok(tool_config)
    }
}
```

### 8. Testing Your Implementation

Create comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_pattern_based_matching() {
        let analysis = ProjectAnalysis {
            has_kubernetes: true,
            has_database: true,
            ..Default::default()
        };

        let available = vec![
            "kubernetes-client".to_string(),
            "k8s-tools".to_string(),
            "postgresql-client".to_string(),
            "mysql-connector".to_string(),
            "unrelated-tool".to_string(),
        ];

        let handler = DocsHandler::new(Client::try_default().await.unwrap());
        let config = handler.match_tools_to_project(&analysis, &available);

        // Should match based on patterns
        assert!(config.remote.iter().any(|t| t.contains("kubernetes")));
        assert!(config.remote.iter().any(|t| t.contains("k8s")));
        assert!(config.remote.iter().any(|t| t.contains("postgres")));
        assert!(config.remote.iter().any(|t| t.contains("mysql")));
        assert!(!config.remote.contains(&"unrelated-tool".to_string()));
    }
}
```

## Key Implementation Points

1. **No Hardcoding**: Use pattern matching, not exact tool names
2. **Best Effort**: Handle failures gracefully, continue with what works
3. **Logging**: Log all important steps for debugging
4. **Error Handling**: Don't fail the entire process for partial errors
5. **Performance**: Use efficient algorithms and early exits

## Success Criteria

- [ ] Reads ConfigMap successfully
- [ ] Discovers all available tools
- [ ] Analyzes project files correctly
- [ ] Matches tools based on patterns
- [ ] Saves configuration for code agents
- [ ] No hardcoded tool names
- [ ] Handles errors gracefully
- [ ] Well-tested implementation

Proceed with implementing this tool discovery system, ensuring it's robust, maintainable, and follows the zero-hardcoding principle throughout.
Proceed with implementing this tool discovery system, ensuring it's robust, maintainable, and follows the zero-hardcoding principle throughout.