# Task 5: Implement Docs Agent Tool Discovery

## Overview
Implement functionality in the docs agent to discover available MCP tools by reading the Toolman ConfigMap and generate optimal tool configurations for projects. This is a critical component of the zero-hardcoding architecture where tool discovery happens dynamically.

## Context
The docs agent is responsible for analyzing projects and determining which tools would be most beneficial. Instead of hardcoding tool lists, it reads the Toolman ConfigMap to discover what's actually available in the platform, then matches project needs with available tools.

## Objectives
1. Implement ConfigMap reading to discover available tools
2. Create project analysis logic to identify tool requirements
3. Generate optimal tool configurations based on project needs
4. Save configurations for code agents to consume
5. Ensure no hardcoded tool names in the implementation

## Architecture

### Discovery Flow
```
┌─────────────────────────────────────────────────┐
│                    Docs Agent Flow                      │
├─────────────────────────────────────────────────┤
│                                                         │
│  1. Read Toolman ConfigMap                              │
│     │                                                   │
│     └─▶ Extract available tool names                    │
│                                                         │
│  2. Analyze Project Files                               │
│     │                                                   │
│     ├─▶ Detect Kubernetes manifests                     │
│     ├─▶ Find database configurations                   │
│     ├─▶ Identify CI/CD pipelines                       │
│     └─▶ Discover API integrations                      │
│                                                         │
│  3. Match Needs with Available Tools                    │
│     │                                                   │
│     └─▶ Generate optimal configuration                 │
│                                                         │
│  4. Save Configuration                                  │
│     │                                                   │
│     └─▶ Store for code agents to use                   │
│                                                         │
└─────────────────────────────────────────────────┘
```

### Key Design Principles
1. **Dynamic Discovery**: No hardcoded tool lists
2. **Pattern Matching**: Use patterns, not specific tool names
3. **Best Effort**: Recommend tools that exist, skip those that don't
4. **Project-Aware**: Tailor recommendations to project needs
5. **Single Source of Truth**: ConfigMap defines what's available

## Implementation Details

### Architecture Split

Task 5 involves two separate components:

1. **Orchestrator Controller** (this repository):
   - Mounts the `toolman-tool-catalog` ConfigMap into agent containers
   - Validates requested tools exist (see Task 11)
   - No discovery or matching logic

2. **Docs Agent** (runs inside container):
   - Reads the mounted ConfigMap from `/etc/tool-catalog/tool-catalog.json`
   - Analyzes the project files
   - Matches tools based on catalog metadata
   - Outputs configuration for code agents

### Orchestrator Changes

The orchestrator only needs to mount the ConfigMap:

```rust
// In orchestrator/core/src/controllers/task_controller/resources.rs
// Add to build_job_spec function:

// Tool catalog ConfigMap (for tool discovery)
volumes.push(json!({
    "name": "tool-catalog",
    "configMap": {
        "name": "toolman-tool-catalog",
        "optional": true  // Don't fail if it doesn't exist yet
    }
}));
volume_mounts.push(json!({
    "name": "tool-catalog",
    "mountPath": "/etc/tool-catalog",
    "readOnly": true
}));
```

### Docs Agent Implementation

The actual tool discovery logic runs in the docs agent container:

### Toolman Service Information
- **Namespace**: `mcp` (where Toolman is deployed)
- **Service URL**: `http://toolman.mcp.svc.cluster.local:3000`
- **ConfigMap**: `toolman-config` in the `mcp` namespace
- **ConfigMap Key**: `servers-config.json`

### 1. Core Data Structures
```rust
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ToolmanConfig {
    servers: HashMap<String, ServerConfig>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    name: String,
    description: String,
    transport: String,
    // Other fields as needed
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ProjectToolConfig {
    local: Vec<String>,
    remote: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProjectConfig {
    tools: ProjectToolConfig,
    generated_at: String,
    project_analysis: ProjectAnalysis,
}

#[derive(Debug, Serialize)]
struct ProjectAnalysis {
    has_kubernetes: bool,
    has_database: bool,
    has_ci_cd: bool,
    detected_languages: Vec<String>,
    detected_frameworks: Vec<String>,
}
```

### 2. Tool Discovery Implementation
```rust
impl DocsHandler {
    /// Discover available tools from Toolman ConfigMap
    async fn discover_available_tools(&self) -> Result<Vec<String>> {
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            "mcp"
        );

        let cm = configmaps
            .get("toolman-config")
            .await
            .map_err(|e| anyhow!("Failed to read ConfigMap: {}", e))?;

        let config_json = cm.data
            .as_ref()
            .and_then(|d| d.get("servers-config.json"))
            .ok_or_else(|| anyhow!("Missing servers-config.json in ConfigMap"))?;

        let config: ToolmanConfig = serde_json::from_str(config_json)
            .map_err(|e| anyhow!("Failed to parse ConfigMap JSON: {}", e))?;

        let tools: Vec<String> = config.servers.keys().cloned().collect();

        info!("Discovered {} available MCP tools from Toolman", tools.len());
        Ok(tools)
    }
}
```

### 3. Project Analysis Logic
```rust
impl DocsHandler {
    /// Analyze project to understand tool requirements
    async fn analyze_project(&self, project_path: &Path) -> Result<ProjectAnalysis> {
        let mut analysis = ProjectAnalysis {
            has_kubernetes: false,
            has_database: false,
            has_ci_cd: false,
            detected_languages: Vec::new(),
            detected_frameworks: Vec::new(),
        };

        // Check for Kubernetes files
        let k8s_patterns = vec![
            "**/*.yaml",
            "**/*.yml",
            "**/k8s/**/*",
            "**/kubernetes/**/*",
            "**/helm/**/*",
        ];

        for pattern in k8s_patterns {
            let files = glob::glob(&project_path.join(pattern).to_string_lossy())?;
            for file in files {
                if let Ok(content) = tokio::fs::read_to_string(file?).await {
                    if content.contains("apiVersion:") && content.contains("kind:") {
                        analysis.has_kubernetes = true;
                        break;
                    }
                }
            }
        }

        // Check for database configurations
        let db_patterns = vec![
            "**/database.yml",
            "**/database.yaml",
            "**/*.sql",
            "**/migrations/**/*",
            "**/schema.rb",
            "**/alembic/**/*",
        ];

        for pattern in db_patterns {
            if glob::glob(&project_path.join(pattern).to_string_lossy())?
                .next()
                .is_some()
            {
                analysis.has_database = true;
                break;
            }
        }

        // Check for CI/CD
        let ci_patterns = vec![
            ".github/workflows/**/*",
            ".gitlab-ci.yml",
            "Jenkinsfile",
            ".circleci/config.yml",
            "azure-pipelines.yml",
        ];

        for pattern in ci_patterns {
            if project_path.join(pattern).exists() {
                analysis.has_ci_cd = true;
                break;
            }
        }

        // Detect languages (simplified)
        if project_path.join("package.json").exists() {
            analysis.detected_languages.push("javascript".to_string());
        }
        if project_path.join("requirements.txt").exists() ||
           project_path.join("setup.py").exists() {
            analysis.detected_languages.push("python".to_string());
        }
        if project_path.join("go.mod").exists() {
            analysis.detected_languages.push("go".to_string());
        }
        if project_path.join("Cargo.toml").exists() {
            analysis.detected_languages.push("rust".to_string());
        }

        Ok(analysis)
    }
}
```

### 4. Tool Recommendation Engine
```rust
impl DocsHandler {
    /// Match project needs with available tools
    fn match_tools_to_project(
        &self,
        analysis: &ProjectAnalysis,
        available_tools: &[String]
    ) -> ProjectToolConfig {
        let mut config = ProjectToolConfig::default();

        // Always include filesystem for local access
        config.local.push("filesystem".to_string());

        // Add git if version control is needed
        if analysis.has_ci_cd || analysis.detected_languages.len() > 0 {
            config.local.push("git".to_string());
        }

        // Match remote tools based on patterns (no hardcoding!)
        for tool in available_tools {
            // Kubernetes-related tools
            if analysis.has_kubernetes &&
               (tool.contains("kubernetes") ||
                tool.contains("k8s") ||
                tool.contains("helm")) {
                config.remote.push(tool.clone());
            }

            // Database tools
            if analysis.has_database {
                if tool.contains("postgres") ||
                   tool.contains("mysql") ||
                   tool.contains("mongo") ||
                   tool.contains("redis") {
                    config.remote.push(tool.clone());
                }
            }

            // CI/CD tools
            if analysis.has_ci_cd {
                if tool.contains("github") ||
                   tool.contains("gitlab") ||
                   tool.contains("jenkins") {
                    config.remote.push(tool.clone());
                }
            }

            // Language-specific tools
            for lang in &analysis.detected_languages {
                if tool.to_lowercase().contains(lang) {
                    config.remote.push(tool.clone());
                }
            }

            // Search tools (useful for most projects)
            if tool.contains("search") || tool.contains("brave") {
                config.remote.push(tool.clone());
            }
        }

        // Remove duplicates
        config.remote.sort();
        config.remote.dedup();

        info!("Recommended tools - Local: {:?}, Remote: {:?}",
              config.local, config.remote);

        config
    }
}
```

### 5. Configuration Storage
```rust
impl DocsHandler {
    /// Save project configuration for code agents
    async fn save_project_config(
        &self,
        project_id: &str,
        config: ProjectConfig
    ) -> Result<()> {
        // Option 1: Save to ConfigMap
        let config_json = serde_json::to_string_pretty(&config)?;

        let mut cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(format!("{}-project-config", project_id)),
                namespace: Some("orchestrator".to_string()),
                ..Default::default()
            },
            data: Some([
                ("config.json".to_string(), config_json)
            ].into()),
            ..Default::default()
        };

        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            "orchestrator"
        );

        configmaps.create(&PostParams::default(), &cm).await?;

        info!("Saved project configuration for {}", project_id);
        Ok(())
    }
}
```

### 6. Complete Workflow Integration
```rust
impl DocsHandler {
    /// Main entry point for tool discovery and configuration
    pub async fn generate_project_configuration(
        &self,
        project_path: &Path,
        project_id: &str
    ) -> Result<ProjectToolConfig> {
        // Step 1: Discover available tools
        let available_tools = self.discover_available_tools().await?;

        // Step 2: Analyze project
        let analysis = self.analyze_project(project_path).await?;

        // Step 3: Generate recommendations
        let tool_config = self.match_tools_to_project(&analysis, &available_tools);

        // Step 4: Save configuration
        let project_config = ProjectConfig {
            tools: tool_config.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            project_analysis: analysis,
        };

        self.save_project_config(project_id, project_config).await?;

        Ok(tool_config)
    }
}
```

## Testing Strategy

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_discovery() {
        let mock_configmap = create_mock_configmap(vec![
            "github", "kubernetes", "postgres", "brave-search"
        ]);

        let handler = DocsHandler::new_with_mock(mock_configmap);
        let tools = handler.discover_available_tools().await.unwrap();

        assert_eq!(tools.len(), 4);
        assert!(tools.contains(&"kubernetes".to_string()));
    }

    #[tokio::test]
    async fn test_project_analysis() {
        let temp_dir = tempdir::TempDir::new("test-project").unwrap();

        // Create test files
        std::fs::create_dir_all(temp_dir.path().join("k8s")).unwrap();
        std::fs::write(
            temp_dir.path().join("k8s/deployment.yaml"),
            "apiVersion: apps/v1\nkind: Deployment"
        ).unwrap();

        let handler = DocsHandler::new();
        let analysis = handler.analyze_project(temp_dir.path()).await.unwrap();

        assert!(analysis.has_kubernetes);
    }

    #[tokio::test]
    async fn test_tool_matching() {
        let analysis = ProjectAnalysis {
            has_kubernetes: true,
            has_database: true,
            has_ci_cd: false,
            detected_languages: vec![],
            detected_frameworks: vec![],
        };

        let available = vec![
            "kubernetes".to_string(),
            "postgres".to_string(),
            "github".to_string(),
            "unrelated-tool".to_string(),
        ];

        let handler = DocsHandler::new();
        let config = handler.match_tools_to_project(&analysis, &available);

        assert!(config.remote.contains(&"kubernetes".to_string()));
        assert!(config.remote.contains(&"postgres".to_string()));
        assert!(!config.remote.contains(&"github".to_string())); // No CI/CD
    }
}
```

### 2. Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_discovery() {
    // Setup test cluster with ConfigMap
    let test_env = setup_test_k8s_env().await;
    create_toolman_configmap(&test_env, vec![
        ("postgres", json!({"transport": "stdio"})),
        ("kubernetes", json!({"transport": "stdio"})),
    ]).await;

    // Create test project
    let project = create_test_project_with_k8s_files().await;

    // Run discovery
    let handler = DocsHandler::new(test_env.client);
    let config = handler.generate_project_configuration(
        &project.path,
        "test-project"
    ).await.unwrap();

    // Verify recommendations
    assert!(config.local.contains(&"filesystem".to_string()));
    assert!(config.remote.contains(&"kubernetes".to_string()));

    // Verify saved configuration
    let saved = load_project_config(&test_env, "test-project").await.unwrap();
    assert_eq!(saved.tools, config);
}
```

## Success Criteria
1. ✅ Discovers all tools from ConfigMap dynamically
2. ✅ Analyzes projects to identify tool needs
3. ✅ Generates appropriate tool recommendations
4. ✅ Saves configuration for code agents
5. ✅ No hardcoded tool names anywhere
6. ✅ Handles missing tools gracefully

## Error Handling

1. **ConfigMap Not Found**: Log warning, return empty tool list
2. **Invalid JSON**: Log error details, return empty tool list
3. **Project Analysis Failure**: Log warning, use minimal defaults
4. **Save Failure**: Retry with exponential backoff
5. **Partial Failures**: Continue with best effort

## Performance Considerations

1. **Caching**: Cache ConfigMap reads for 5 minutes
2. **Parallel Analysis**: Analyze file patterns concurrently
3. **Early Exit**: Stop searching once patterns found
4. **Efficient Matching**: Use HashSet for O(1) lookups

## Related Tasks
- Task 3: ConfigMap structure that we're reading
- Task 6: Configuration storage for code agents
- Task 7: Code agents consuming saved configs
- Task 11: Validation against the same ConfigMap

### 2. Tool Catalog ConfigMap Creation

The docs agent needs detailed tool information beyond just server names. Toolman should create and maintain a `toolman-tool-catalog` ConfigMap with comprehensive tool metadata:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct ToolCatalog {
    last_updated: String,
    local: HashMap<String, LocalServerInfo>,
    remote: HashMap<String, RemoteServerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalServerInfo {
    description: String,
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RemoteServerInfo {
    description: String,
    endpoint: String,
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolInfo {
    name: String,
    description: String,
    category: String,
    use_cases: Vec<String>,
    input_schema: Option<serde_json::Value>,
}

async fn populate_tool_catalog(client: Client) -> Result<(), Box<dyn Error>> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), "mcp");

    // Build catalog from discovered tools
    let catalog = ToolCatalog {
        last_updated: chrono::Utc::now().to_rfc3339(),
        local: get_local_tool_definitions(),
        remote: discover_remote_tools(&client).await?,
    };

    // Create or update the catalog ConfigMap
    let catalog_json = serde_json::to_string_pretty(&catalog)?;
    let cm = ConfigMap {
        metadata: ObjectMeta {
            name: Some("toolman-tool-catalog".to_string()),
            namespace: Some("mcp".to_string()),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("tool-catalog.json".to_string(), catalog_json),
        ])),
        ..Default::default()
    };

    // Try to update, create if doesn't exist
    match configmaps.patch("toolman-tool-catalog", &PatchParams::apply("toolman"), &Patch::Apply(cm)).await {
        Ok(_) => info!("Tool catalog updated successfully"),
        Err(e) if e.to_string().contains("not found") => {
            configmaps.create(&PostParams::default(), &cm).await?;
            info!("Tool catalog created successfully");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
```

### 3. RBAC Requirements

**Important**: Toolman currently has no RBAC permissions. To enable ConfigMap management, add:

1. **Role Template** (`toolman/charts/toolman/templates/role.yaml`):
```yaml
{{- if .Values.rbac.create -}}
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: {{ include "toolman.fullname" . }}
  namespace: {{ .Release.Namespace }}
rules:
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list"]  # Read all ConfigMaps
  - apiGroups: [""]
    resources: ["configmaps"]
    resourceNames: ["toolman-tool-catalog"]
    verbs: ["update", "patch"]  # Update specific ConfigMap
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["create"]  # Create if doesn't exist
{{- end }}
```

2. **RoleBinding Template** (`toolman/charts/toolman/templates/rolebinding.yaml`):
```yaml
{{- if .Values.rbac.create -}}
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: {{ include "toolman.fullname" . }}
  namespace: {{ .Release.Namespace }}
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: {{ include "toolman.fullname" . }}
subjects:
  - kind: ServiceAccount
    name: {{ include "toolman.serviceAccountName" . }}
    namespace: {{ .Release.Namespace }}
{{- end }}
```

3. **Values Configuration**:
```yaml
rbac:
  create: true  # Enable RBAC resources
```

### 4. Local Tool Definitions

Define local tools that are built into the platform:

```rust
fn get_local_tool_definitions() -> HashMap<String, LocalServerInfo> {
    let mut local = HashMap::new();

    local.insert("filesystem".to_string(), LocalServerInfo {
        description: "File system operations for reading, writing, and managing files".to_string(),
        tools: vec![
            ToolInfo {
                name: "read_file".to_string(),
                description: "Read contents of a file".to_string(),
                category: "file-operations".to_string(),
                use_cases: vec![
                    "reading config files".to_string(),
                    "analyzing code".to_string(),
                    "viewing documentation".to_string(),
                ],
                input_schema: None, // Could add if needed
            },
            ToolInfo {
                name: "write_file".to_string(),
                description: "Write or update file contents".to_string(),
                category: "file-operations".to_string(),
                use_cases: vec![
                    "generating code".to_string(),
                    "updating configs".to_string(),
                    "creating documentation".to_string(),
                ],
                input_schema: None,
            },
            ToolInfo {
                name: "list_directory".to_string(),
                description: "List directory contents".to_string(),
                category: "file-operations".to_string(),
                use_cases: vec![
                    "exploring project structure".to_string(),
                    "finding files".to_string(),
                ],
                input_schema: None,
            },
        ],
    });

    local.insert("git".to_string(), LocalServerInfo {
        description: "Git version control operations".to_string(),
        tools: vec![
            ToolInfo {
                name: "git_status".to_string(),
                description: "Check repository status".to_string(),
                category: "version-control".to_string(),
                use_cases: vec![
                    "checking changes".to_string(),
                    "review before commit".to_string(),
                ],
                input_schema: None,
            },
            ToolInfo {
                name: "git_log".to_string(),
                description: "View commit history".to_string(),
                category: "version-control".to_string(),
                use_cases: vec![
                    "reviewing changes".to_string(),
                    "understanding project history".to_string(),
                ],
                input_schema: None,
            },
        ],
    });

    local
}
```

### 5. Updated Tool Discovery for Docs Agent

The docs agent should now read from the tool catalog instead of the server config:

```rust
async fn discover_available_tools(client: Client) -> Result<ToolCatalog, Box<dyn Error>> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client, "mcp");

    // Read from the tool catalog ConfigMap
    let cm = configmaps.get("toolman-tool-catalog").await?;

    let catalog_json = cm.data
        .and_then(|d| d.get("tool-catalog.json"))
        .ok_or("tool-catalog.json not found in toolman-tool-catalog")?;

    let catalog: ToolCatalog = serde_json::from_str(catalog_json)?;

    Ok(catalog)
}

// Helper to get just tool names for compatibility
async fn get_available_tool_names(client: Client) -> Result<Vec<String>, Box<dyn Error>> {
    let catalog = discover_available_tools(client).await?;

    let mut tool_names = Vec::new();

    // Collect local tool names
    for (server_name, server_info) in &catalog.local {
        for tool in &server_info.tools {
            tool_names.push(format!("{}_{}", server_name, tool.name));
        }
    }

    // Collect remote tool names
    for (server_name, server_info) in &catalog.remote {
        for tool in &server_info.tools {
            tool_names.push(format!("{}_{}", server_name, tool.name));
        }
    }

    Ok(tool_names)
}
```

### 6. Pattern Matching

## Completion Summary

### What We Actually Implemented ✅

After discussing with the user, we clarified the architecture and implemented the orchestrator-side changes:

1. **ConfigMap Mounting** (Task 5):
   - Updated `resources.rs` to mount `toolman-tool-catalog` ConfigMap
   - Mounts to `/etc/tool-catalog` in agent containers
   - Set as optional to not fail if ConfigMap doesn't exist

2. **CRD Structure Update**:
   - Updated `CodeRunSpec` to use structured `ToolConfig` instead of string fields
   - Maintained backward compatibility in API
   - Converts comma-separated strings to arrays

3. **Tool Validation** (Task 11):
   - Added `validate_tools` function in `common.rs`
   - Validates local tools against fixed set ["filesystem", "git"]
   - Validates remote tools exist in Toolman ConfigMap
   - Integrated validation into `code_handler`

4. **Test ConfigMap**:
   - Created and applied test `toolman-tool-catalog` with sample data
   - Verified mounting works correctly

### Architecture Clarification

The user helped clarify that:
- **Orchestrator**: Only mounts ConfigMaps and validates tools - no discovery logic
- **Toolman**: Creates and maintains the tool catalog ConfigMap
- **Docs Agent**: Reads mounted catalog and performs discovery/matching (runs in container)

### Pending Work

1. **Toolman Implementation**:
   - Needs to create/update the catalog on startup
   - Should query each MCP server for tool details
   - Already has RBAC permissions via Helm chart updates

2. **Docs Agent Implementation**:
   - Read mounted catalog from `/etc/tool-catalog/tool-catalog.json`
   - Implement project analysis logic
   - Match tools based on catalog metadata
   - Output configuration for code agents