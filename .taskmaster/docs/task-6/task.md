# Task 6: Generate and Save Tool Configuration in Docs Phase

## Overview
Extend the docs agent to generate and save optimal tool configurations during the documentation phase. This saved configuration will be consumed by code agents, eliminating the need for them to perform tool discovery.

## Context
Building on Task 5's tool discovery implementation, this task focuses on the configuration generation and persistence aspect. The docs agent will create a comprehensive configuration that includes tool selections, metadata, and project analysis results, storing it for future use by code agents.

## Objectives
1. Design comprehensive configuration schema
2. Implement configuration generation logic
3. Create reliable storage mechanism
4. Ensure compatibility with code agent expectations
5. Include metadata for traceability and debugging

## Architecture

### Configuration Flow
```
┌─────────────────────────────────────────────────┐
│              Configuration Generation & Storage         │
├─────────────────────────────────────────────────┤
│                                                         │
│  Tool Discovery ──▶ Project Analysis ──▶ Tool Matching  │
│       │                    │                  │        │
│       ▼                    ▼                  ▼        │
│  Available Tools      Project Needs      Recommendations│
│                            │                            │
│                            ▼                            │
│                   Configuration Object                  │
│                   ─────────────────────                │
│                   - Tool selections                     │
│                   - Metadata                            │
│                   - Analysis results                    │
│                   - Timestamps                          │
│                            │                            │
│                            ▼                            │
│                    Storage (ConfigMap)                  │
│                            │                            │
│                            ▼                            │
│                    Code Agents Read                     │
│                                                         │
└─────────────────────────────────────────────────┘
```

### Storage Strategy
- **Primary Storage**: Kubernetes ConfigMap in orchestrator namespace
- **Naming Convention**: `{project-id}-project-config`
- **Format**: JSON for easy parsing
- **Lifecycle**: Created during docs phase, consumed by code agents

## Implementation Details

### 1. Configuration Schema Design
```rust
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Tool configuration
    pub tools: ProjectToolConfig,
    
    /// When this configuration was generated
    pub generated_at: DateTime<Utc>,
    
    /// ID of the docs run that generated this
    pub docs_run_id: String,
    
    /// Project analysis results
    pub project_analysis: ProjectAnalysis,
    
    /// Version for future compatibility
    pub version: String,
    
    /// Additional metadata
    pub metadata: ConfigMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigMetadata {
    /// Project identifier
    pub project_id: String,
    
    /// Git commit hash at time of analysis
    pub git_commit: Option<String>,
    
    /// Branch name
    pub git_branch: Option<String>,
    
    /// Number of tools discovered
    pub total_tools_available: usize,
    
    /// Number of tools recommended
    pub total_tools_recommended: usize,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectToolConfig {
    /// Local tools (filesystem, git)
    pub local: Vec<String>,
    
    /// Remote tools via toolman
    pub remote: Vec<String>,
    
    /// Tool-specific configurations (future extension)
    pub tool_configs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAnalysis {
    pub has_kubernetes: bool,
    pub has_database: bool,
    pub has_ci_cd: bool,
    pub has_terraform: bool,
    pub detected_languages: Vec<String>,
    pub detected_frameworks: Vec<String>,
    pub file_patterns_found: Vec<String>,
    pub project_size: ProjectSize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectSize {
    Small,    // < 100 files
    Medium,   // 100-1000 files
    Large,    // > 1000 files
}
```

### 2. Configuration Generation Logic
```rust
impl DocsHandler {
    /// Generate complete project configuration
    pub async fn generate_project_config(
        &self,
        project_path: &Path,
        project_id: &str,
        docs_run_id: &str,
        available_tools: &[String],
        analysis: &ProjectAnalysis,
    ) -> Result<ProjectConfig> {
        // Generate tool recommendations
        let mut tool_config = self.match_tools_to_project(analysis, available_tools);
        
        // Calculate confidence score based on matches
        let confidence_score = self.calculate_confidence_score(
            analysis,
            &tool_config,
            available_tools.len()
        );
        
        // Get git information if available
        let (git_commit, git_branch) = self.get_git_info(project_path).await
            .unwrap_or((None, None));
        
        // Add any tool-specific configurations
        let mut tool_configs = HashMap::new();
        
        // Example: Configure filesystem tool for read-only in certain cases
        if tool_config.local.contains(&"filesystem".to_string()) {
            if self.should_be_readonly(analysis) {
                tool_configs.insert(
                    "filesystem".to_string(),
                    json!({
                        "readonly": true,
                        "allowed_paths": ["/workspace"]
                    })
                );
            }
        }
        
        tool_config.tool_configs = tool_configs;
        
        // Build complete configuration
        let config = ProjectConfig {
            tools: tool_config.clone(),
            generated_at: Utc::now(),
            docs_run_id: docs_run_id.to_string(),
            project_analysis: analysis.clone(),
            version: "1.0.0".to_string(),
            metadata: ConfigMetadata {
                project_id: project_id.to_string(),
                git_commit,
                git_branch,
                total_tools_available: available_tools.len(),
                total_tools_recommended: tool_config.local.len() + tool_config.remote.len(),
                confidence_score,
            },
        };
        
        info!(
            "Generated project configuration for {} with {} local and {} remote tools",
            project_id,
            config.tools.local.len(),
            config.tools.remote.len()
        );
        
        Ok(config)
    }
    
    /// Calculate confidence score for recommendations
    fn calculate_confidence_score(
        &self,
        analysis: &ProjectAnalysis,
        tool_config: &ProjectToolConfig,
        total_available: usize,
    ) -> f32 {
        let mut score = 0.5; // Base score
        
        // Increase confidence based on clear indicators
        if analysis.has_kubernetes && 
           tool_config.remote.iter().any(|t| t.contains("kubernetes")) {
            score += 0.1;
        }
        
        if analysis.has_database &&
           tool_config.remote.iter().any(|t| 
               t.contains("postgres") || t.contains("mysql") || t.contains("mongo")
           ) {
            score += 0.1;
        }
        
        // Adjust based on tool availability
        if total_available > 20 {
            score += 0.1; // Rich tool ecosystem
        }
        
        // Cap at 1.0
        score.min(1.0)
    }
    
    /// Determine if filesystem should be read-only
    fn should_be_readonly(&self, analysis: &ProjectAnalysis) -> bool {
        // Read-only for documentation-heavy projects
        analysis.file_patterns_found.contains(&"documentation".to_string()) &&
        !analysis.has_ci_cd
    }
    
    /// Get git information from project
    async fn get_git_info(&self, project_path: &Path) -> Result<(Option<String>, Option<String>)> {
        let git_dir = project_path.join(".git");
        if !git_dir.exists() {
            return Ok((None, None));
        }
        
        // Get current commit
        let commit = tokio::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
            .await
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });
        
        // Get current branch
        let branch = tokio::process::Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
            .await
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });
        
        Ok((commit, branch))
    }
}
```

### 3. Storage Implementation
```rust
impl DocsHandler {
    /// Save project configuration to Kubernetes ConfigMap
    pub async fn save_project_config(
        &self,
        config: &ProjectConfig,
    ) -> Result<()> {
        let config_name = format!("{}-project-config", config.metadata.project_id);
        
        info!("Saving project configuration to ConfigMap: {}", config_name);
        
        // Serialize configuration
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        // Create ConfigMap object
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(config_name.clone()),
                namespace: Some(self.namespace.clone()),
                labels: Some([
                    ("app.kubernetes.io/name".to_string(), "project-config".to_string()),
                    ("app.kubernetes.io/instance".to_string(), config.metadata.project_id.clone()),
                    ("app.kubernetes.io/component".to_string(), "tool-config".to_string()),
                    ("app.kubernetes.io/managed-by".to_string(), "docs-agent".to_string()),
                    ("generated-by".to_string(), config.docs_run_id.clone()),
                ].into()),
                annotations: Some([
                    ("generated-at".to_string(), config.generated_at.to_rfc3339()),
                    ("config-version".to_string(), config.version.clone()),
                    ("confidence-score".to_string(), config.metadata.confidence_score.to_string()),
                ].into()),
                ..Default::default()
            },
            data: Some([
                ("config.json".to_string(), config_json.clone()),
                // Store analysis separately for easy access
                ("analysis.json".to_string(), 
                 serde_json::to_string_pretty(&config.project_analysis)?),
                // Store just tools for quick reference
                ("tools.json".to_string(),
                 serde_json::to_string_pretty(&config.tools)?),
            ].into()),
            binary_data: None,
            immutable: Some(false),
        };
        
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        // Try to create, update if exists
        match configmaps.create(&PostParams::default(), &cm).await {
            Ok(_) => {
                info!("Created new project configuration ConfigMap: {}", config_name);
            }
            Err(kube::Error::Api(err)) if err.code == 409 => {
                // ConfigMap exists, update it
                info!("Updating existing project configuration ConfigMap: {}", config_name);
                
                // Use server-side apply for safer updates
                let patch_params = PatchParams::apply("docs-agent")
                    .force();
                
                configmaps.patch(
                    &config_name,
                    &patch_params,
                    &Patch::Apply(&cm)
                ).await
                .map_err(|e| anyhow!("Failed to update ConfigMap: {}", e))?;
            }
            Err(e) => {
                return Err(anyhow!("Failed to create ConfigMap: {}", e));
            }
        }
        
        // Verify storage
        self.verify_config_storage(&config_name).await?;
        
        Ok(())
    }
    
    /// Verify configuration was stored correctly
    async fn verify_config_storage(&self, config_name: &str) -> Result<()> {
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        let cm = configmaps.get(config_name).await
            .map_err(|e| anyhow!("Failed to verify ConfigMap: {}", e))?;
        
        // Verify all expected keys exist
        if let Some(data) = cm.data {
            let required_keys = vec!["config.json", "analysis.json", "tools.json"];
            for key in required_keys {
                if !data.contains_key(key) {
                    return Err(anyhow!("Missing required key in ConfigMap: {}", key));
                }
            }
            info!("Configuration storage verified successfully");
            Ok(())
        } else {
            Err(anyhow!("ConfigMap has no data"))
        }
    }
}
```

### 4. Integration with Docs Generation Flow
```rust
impl DocsHandler {
    /// Main docs generation entry point with tool configuration
    pub async fn generate_docs(
        &self,
        ctx: DocsContext,
    ) -> Result<()> {
        info!("Starting docs generation for project: {}", ctx.project_id);
        
        // Phase 1: Tool Discovery and Configuration
        let tool_config = self.generate_and_save_tool_config(&ctx).await?;
        
        info!(
            "Tool configuration complete. Local tools: {:?}, Remote tools: {:?}",
            tool_config.local,
            tool_config.remote
        );
        
        // Phase 2: Regular docs generation
        // ... existing docs generation logic ...
        
        Ok(())
    }
    
    /// Generate and save tool configuration
    async fn generate_and_save_tool_config(
        &self,
        ctx: &DocsContext,
    ) -> Result<ProjectToolConfig> {
        // Discover available tools
        let available_tools = self.discover_available_tools().await?;
        
        if available_tools.is_empty() {
            warn!("No tools discovered, using minimal configuration");
            return Ok(ProjectToolConfig {
                local: vec!["filesystem".to_string()],
                remote: vec![],
                tool_configs: HashMap::new(),
            });
        }
        
        // Analyze project
        let analysis = self.analyze_project(&ctx.project_path).await?;
        
        // Generate configuration
        let config = self.generate_project_config(
            &ctx.project_path,
            &ctx.project_id,
            &ctx.run_id,
            &available_tools,
            &analysis,
        ).await?;
        
        // Save configuration
        self.save_project_config(&config).await?;
        
        // Return just the tool config for immediate use
        Ok(config.tools)
    }
}
```

### 5. Configuration Loading (for Testing)
```rust
impl DocsHandler {
    /// Load saved configuration (primarily for testing)
    pub async fn load_project_config(
        &self,
        project_id: &str,
    ) -> Result<ProjectConfig> {
        let config_name = format!("{}-project-config", project_id);
        
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        let cm = configmaps.get(&config_name).await
            .map_err(|e| anyhow!("Failed to load ConfigMap: {}", e))?;
        
        let config_json = cm.data
            .as_ref()
            .and_then(|d| d.get("config.json"))
            .ok_or_else(|| anyhow!("Missing config.json in ConfigMap"))?;
        
        let config: ProjectConfig = serde_json::from_str(config_json)
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;
        
        Ok(config)
    }
}
```

## Testing Strategy

### 1. Configuration Generation Tests
```rust
#[tokio::test]
async fn test_config_generation() {
    let handler = create_test_handler().await;
    let analysis = create_test_analysis();
    let available_tools = vec![
        "kubernetes".to_string(),
        "postgres".to_string(),
    ];
    
    let config = handler.generate_project_config(
        Path::new("/test"),
        "test-project",
        "docs-123",
        &available_tools,
        &analysis,
    ).await.unwrap();
    
    assert_eq!(config.metadata.project_id, "test-project");
    assert_eq!(config.docs_run_id, "docs-123");
    assert_eq!(config.version, "1.0.0");
    assert!(config.metadata.confidence_score > 0.0);
}
```

### 2. Storage and Retrieval Tests
```rust
#[tokio::test]
async fn test_config_storage_and_retrieval() {
    let handler = create_test_handler().await;
    let config = create_test_config();
    
    // Save
    handler.save_project_config(&config).await.unwrap();
    
    // Load
    let loaded = handler.load_project_config("test-project").await.unwrap();
    
    assert_eq!(loaded.tools, config.tools);
    assert_eq!(loaded.metadata.project_id, config.metadata.project_id);
}
```

### 3. Integration Tests
```rust
#[tokio::test]
async fn test_full_workflow() {
    let ctx = create_test_docs_context().await;
    let handler = create_test_handler().await;
    
    // Run workflow
    let tool_config = handler.generate_and_save_tool_config(&ctx).await.unwrap();
    
    // Verify configuration was saved
    let saved = handler.load_project_config(&ctx.project_id).await.unwrap();
    assert_eq!(saved.tools, tool_config);
    
    // Verify metadata
    assert_eq!(saved.docs_run_id, ctx.run_id);
    assert!(saved.generated_at < Utc::now());
}
```

## Success Criteria
1. ✅ Comprehensive configuration schema defined
2. ✅ Configuration includes all necessary metadata
3. ✅ Reliable storage to Kubernetes ConfigMap
4. ✅ Proper error handling and recovery
5. ✅ Integration with docs generation flow
6. ✅ Configuration loadable by code agents

## Error Handling

1. **Storage Failures**: Retry with exponential backoff
2. **Serialization Errors**: Log details, use defaults
3. **Git Info Failures**: Continue without git metadata
4. **Update Conflicts**: Use server-side apply
5. **Verification Failures**: Log warning but don't fail docs

## Performance Considerations

1. **Async Operations**: All I/O operations are async
2. **Efficient Serialization**: Use serde for fast JSON
3. **Minimal Storage**: Store only necessary data
4. **Quick Verification**: Simple key existence check

## Related Tasks
- Task 5: Provides tool discovery and analysis
- Task 7: Code agents consume this configuration
- Task 11: Uses same ConfigMap for validation
- Task 12: Orchestrator passes config to templates