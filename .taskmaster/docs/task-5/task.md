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

### 1. Core Data Structures
```rust
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ToolmanConfig {
    servers: HashMap<String, serde_json::Value>,
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
            "orchestrator"
        );
        
        let cm = configmaps
            .get("toolman-servers-config")
            .await
            .map_err(|e| anyhow!("Failed to read ConfigMap: {}", e))?;
        
        let config_json = cm.data
            .as_ref()
            .and_then(|d| d.get("servers-config.json"))
            .ok_or_else(|| anyhow!("Missing servers-config.json in ConfigMap"))?;
        
        let config: ToolmanConfig = serde_json::from_str(config_json)
            .map_err(|e| anyhow!("Failed to parse ConfigMap JSON: {}", e))?;
        
        let tools: Vec<String> = config.servers.keys().cloned().collect();
        
        info!("Discovered {} available MCP tools: {:?}", tools.len(), tools);
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