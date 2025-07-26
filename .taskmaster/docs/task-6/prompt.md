# Autonomous Agent Prompt: Generate and Save Tool Configuration

## Context
You are extending the docs agent to generate and save comprehensive tool configurations during the documentation phase. This configuration will be stored as a Kubernetes ConfigMap and consumed by code agents, eliminating their need to perform tool discovery.

## Your Mission
Implement the complete configuration generation and storage system, ensuring it creates rich, informative configurations that code agents can reliably consume.

## Implementation Steps

### 1. Define Comprehensive Configuration Schema

Create a rich configuration structure that includes:

```rust
// In src/models/config.rs or similar
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Selected tools for the project
    pub tools: ProjectToolConfig,
    
    /// Timestamp of configuration generation
    pub generated_at: DateTime<Utc>,
    
    /// ID of the docs run that created this
    pub docs_run_id: String,
    
    /// Detailed project analysis results
    pub project_analysis: ProjectAnalysis,
    
    /// Schema version for compatibility
    pub version: String,
    
    /// Rich metadata about the configuration
    pub metadata: ConfigMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigMetadata {
    pub project_id: String,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub total_tools_available: usize,
    pub total_tools_recommended: usize,
    pub confidence_score: f32,
    pub generation_duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectToolConfig {
    pub local: Vec<String>,
    pub remote: Vec<String>,
    /// Tool-specific configurations
    pub tool_configs: HashMap<String, serde_json::Value>,
}
```

### 2. Implement Configuration Generation

```rust
impl DocsHandler {
    /// Generate a complete project configuration
    pub async fn generate_project_config(
        &self,
        project_path: &Path,
        project_id: &str,
        docs_run_id: &str,
        available_tools: &[String],
        analysis: &ProjectAnalysis,
    ) -> Result<ProjectConfig> {
        let start_time = std::time::Instant::now();
        
        info!("Generating project configuration for: {}", project_id);
        
        // Generate tool recommendations
        let mut tool_config = self.match_tools_to_project(analysis, available_tools);
        
        // Add tool-specific configurations
        tool_config.tool_configs = self.generate_tool_configs(&tool_config, analysis);
        
        // Calculate metrics
        let confidence_score = self.calculate_confidence_score(
            analysis,
            &tool_config,
            available_tools
        );
        
        // Get git information
        let (git_commit, git_branch) = self.get_git_info(project_path).await
            .unwrap_or((None, None));
        
        let generation_duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Build configuration
        let config = ProjectConfig {
            tools: tool_config,
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
                generation_duration_ms,
            },
        };
        
        info!(
            "Generated configuration with confidence {:.2} in {}ms",
            confidence_score,
            generation_duration_ms
        );
        
        Ok(config)
    }
    
    /// Generate tool-specific configurations
    fn generate_tool_configs(
        &self,
        tool_config: &ProjectToolConfig,
        analysis: &ProjectAnalysis,
    ) -> HashMap<String, serde_json::Value> {
        let mut configs = HashMap::new();
        
        // Filesystem configuration
        if tool_config.local.contains(&"filesystem".to_string()) {
            let fs_config = json!({
                "readonly": self.should_filesystem_be_readonly(analysis),
                "allowed_directories": ["/workspace"],
                "max_file_size_mb": 100,
                "excluded_patterns": [
                    "**/*.log",
                    "**/node_modules/**",
                    "**/.git/**"
                ]
            });
            configs.insert("filesystem".to_string(), fs_config);
        }
        
        // Git configuration
        if tool_config.local.contains(&"git".to_string()) {
            let git_config = json!({
                "allow_push": false,  // Safety default
                "allow_force_push": false,
                "default_branch": "main",
                "fetch_depth": 50
            });
            configs.insert("git".to_string(), git_config);
        }
        
        // Kubernetes configuration
        if tool_config.remote.iter().any(|t| t.contains("kubernetes")) {
            let k8s_config = json!({
                "namespaces": ["default", "orchestrator"],
                "read_only": false,
                "resource_types": ["pods", "services", "configmaps", "deployments"]
            });
            configs.insert("kubernetes".to_string(), k8s_config);
        }
        
        configs
    }
    
    /// Calculate confidence score (0.0 to 1.0)
    fn calculate_confidence_score(
        &self,
        analysis: &ProjectAnalysis,
        tool_config: &ProjectToolConfig,
        available_tools: &[String],
    ) -> f32 {
        let mut score = 0.5; // Base score
        let mut matches = 0;
        let mut expected = 0;
        
        // Check expected matches
        if analysis.has_kubernetes {
            expected += 1;
            if tool_config.remote.iter().any(|t| t.contains("kubernetes") || t.contains("k8s")) {
                matches += 1;
            }
        }
        
        if analysis.has_database {
            expected += 1;
            if tool_config.remote.iter().any(|t| 
                t.contains("postgres") || t.contains("mysql") || 
                t.contains("mongo") || t.contains("redis")
            ) {
                matches += 1;
            }
        }
        
        if analysis.has_ci_cd {
            expected += 1;
            if tool_config.remote.iter().any(|t| 
                t.contains("github") || t.contains("gitlab") || t.contains("jenkins")
            ) {
                matches += 1;
            }
        }
        
        // Calculate match ratio
        if expected > 0 {
            let match_ratio = matches as f32 / expected as f32;
            score = 0.3 + (0.5 * match_ratio);
        }
        
        // Bonus for rich tool ecosystem
        if available_tools.len() > 20 {
            score += 0.1;
        }
        
        // Bonus for comprehensive analysis
        if !analysis.detected_languages.is_empty() {
            score += 0.1;
        }
        
        score.min(1.0).max(0.0)
    }
}
```

### 3. Implement Robust Storage

```rust
impl DocsHandler {
    /// Save configuration to Kubernetes ConfigMap with retry logic
    pub async fn save_project_config(
        &self,
        config: &ProjectConfig,
    ) -> Result<()> {
        let config_name = format!("{}-project-config", config.metadata.project_id);
        
        // Retry logic for transient failures
        let mut attempts = 0;
        let max_attempts = 3;
        let mut delay = Duration::from_millis(100);
        
        loop {
            attempts += 1;
            match self.save_config_attempt(&config_name, config).await {
                Ok(_) => {
                    info!("Successfully saved project configuration");
                    return Ok(());
                }
                Err(e) if attempts < max_attempts => {
                    warn!(
                        "Failed to save config (attempt {}/{}): {}", 
                        attempts, max_attempts, e
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => {
                    error!("Failed to save config after {} attempts: {}", max_attempts, e);
                    return Err(e);
                }
            }
        }
    }
    
    /// Single attempt to save configuration
    async fn save_config_attempt(
        &self,
        config_name: &str,
        config: &ProjectConfig,
    ) -> Result<()> {
        // Serialize all parts
        let config_json = serde_json::to_string_pretty(config)?;
        let analysis_json = serde_json::to_string_pretty(&config.project_analysis)?;
        let tools_json = serde_json::to_string_pretty(&config.tools)?;
        
        // Create comprehensive ConfigMap
        let cm = ConfigMap {
            metadata: ObjectMeta {
                name: Some(config_name.to_string()),
                namespace: Some(self.namespace.clone()),
                labels: Some(self.generate_labels(config)),
                annotations: Some(self.generate_annotations(config)),
                ..Default::default()
            },
            data: Some([
                ("config.json".to_string(), config_json),
                ("analysis.json".to_string(), analysis_json),
                ("tools.json".to_string(), tools_json),
                // Add a summary for quick reference
                ("summary.txt".to_string(), self.generate_summary(config)),
            ].into()),
            ..Default::default()
        };
        
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        // Use server-side apply for safer updates
        let patch_params = PatchParams::apply("docs-agent").force();
        
        configmaps.patch(
            config_name,
            &patch_params,
            &Patch::Apply(&cm)
        ).await?;
        
        // Verify the save
        self.verify_saved_config(config_name, config).await?;
        
        Ok(())
    }
    
    /// Generate labels for the ConfigMap
    fn generate_labels(&self, config: &ProjectConfig) -> BTreeMap<String, String> {
        [
            ("app.kubernetes.io/name".to_string(), "project-config".to_string()),
            ("app.kubernetes.io/instance".to_string(), config.metadata.project_id.clone()),
            ("app.kubernetes.io/component".to_string(), "tool-config".to_string()),
            ("app.kubernetes.io/managed-by".to_string(), "docs-agent".to_string()),
            ("app.kubernetes.io/version".to_string(), config.version.clone()),
            ("generated-by".to_string(), config.docs_run_id.clone()),
        ].into()
    }
    
    /// Generate annotations for the ConfigMap
    fn generate_annotations(&self, config: &ProjectConfig) -> BTreeMap<String, String> {
        let mut annotations = BTreeMap::new();
        
        annotations.insert(
            "generated-at".to_string(),
            config.generated_at.to_rfc3339()
        );
        annotations.insert(
            "confidence-score".to_string(),
            format!("{:.2}", config.metadata.confidence_score)
        );
        annotations.insert(
            "total-tools".to_string(),
            config.metadata.total_tools_recommended.to_string()
        );
        
        if let Some(ref commit) = config.metadata.git_commit {
            annotations.insert("git-commit".to_string(), commit.clone());
        }
        if let Some(ref branch) = config.metadata.git_branch {
            annotations.insert("git-branch".to_string(), branch.clone());
        }
        
        annotations
    }
    
    /// Generate human-readable summary
    fn generate_summary(&self, config: &ProjectConfig) -> String {
        format!(
            "Project Configuration Summary\n\
             ============================\n\
             Project ID: {}\n\
             Generated: {}\n\
             Confidence: {:.0}%\n\
             \n\
             Local Tools ({}):\n\
             {}\n\
             \n\
             Remote Tools ({}):\n\
             {}\n\
             \n\
             Project Characteristics:\n\
             - Languages: {}\n\
             - Has Kubernetes: {}\n\
             - Has Database: {}\n\
             - Has CI/CD: {}\n\
             - Project Size: {:?}\n",
            config.metadata.project_id,
            config.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            config.metadata.confidence_score * 100.0,
            config.tools.local.len(),
            config.tools.local.join(", "),
            config.tools.remote.len(),
            config.tools.remote.join(", "),
            config.project_analysis.detected_languages.join(", "),
            config.project_analysis.has_kubernetes,
            config.project_analysis.has_database,
            config.project_analysis.has_ci_cd,
            config.project_analysis.project_size,
        )
    }
    
    /// Verify the configuration was saved correctly
    async fn verify_saved_config(
        &self,
        config_name: &str,
        expected_config: &ProjectConfig,
    ) -> Result<()> {
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        let cm = configmaps.get(config_name).await?;
        
        if let Some(data) = &cm.data {
            // Verify all expected keys exist
            let required_keys = vec!["config.json", "analysis.json", "tools.json", "summary.txt"];
            for key in &required_keys {
                if !data.contains_key(*key) {
                    return Err(anyhow!("Missing key in saved ConfigMap: {}", key));
                }
            }
            
            // Verify content integrity
            if let Some(config_json) = data.get("config.json") {
                let saved_config: ProjectConfig = serde_json::from_str(config_json)?;
                if saved_config.metadata.project_id != expected_config.metadata.project_id {
                    return Err(anyhow!("Project ID mismatch in saved config"));
                }
            }
            
            Ok(())
        } else {
            Err(anyhow!("ConfigMap has no data after save"))
        }
    }
}
```

### 4. Integrate with Docs Generation Flow

```rust
impl DocsHandler {
    /// Enhanced docs generation with tool configuration
    pub async fn generate_docs(&self, ctx: DocsContext) -> Result<()> {
        info!(
            "Starting docs generation for project: {} (run: {})",
            ctx.project_id,
            ctx.run_id
        );
        
        // Phase 1: Tool configuration
        let tool_config = match self.generate_and_save_tool_config(&ctx).await {
            Ok(config) => {
                info!("Tool configuration phase completed successfully");
                config
            }
            Err(e) => {
                error!("Tool configuration failed: {}. Using defaults.", e);
                ProjectToolConfig {
                    local: vec!["filesystem".to_string()],
                    remote: vec![],
                    tool_configs: HashMap::new(),
                }
            }
        };
        
        // Log tool summary
        info!(
            "Proceeding with docs generation using {} local and {} remote tools",
            tool_config.local.len(),
            tool_config.remote.len()
        );
        
        // Phase 2: Document generation
        self.generate_documentation_with_tools(&ctx, &tool_config).await?;
        
        info!("Docs generation completed for project: {}", ctx.project_id);
        Ok(())
    }
    
    /// Generate and save tool configuration
    async fn generate_and_save_tool_config(
        &self,
        ctx: &DocsContext,
    ) -> Result<ProjectToolConfig> {
        // Step 1: Discover available tools
        let available_tools = self.discover_available_tools().await?;
        
        if available_tools.is_empty() {
            warn!("No tools discovered from ConfigMap");
        }
        
        // Step 2: Analyze project
        let analysis = self.analyze_project(&ctx.project_path).await?;
        
        // Step 3: Generate configuration
        let config = self.generate_project_config(
            &ctx.project_path,
            &ctx.project_id,
            &ctx.run_id,
            &available_tools,
            &analysis,
        ).await?;
        
        // Step 4: Save configuration with retry
        self.save_project_config(&config).await?;
        
        // Return the tool config for immediate use
        Ok(config.tools)
    }
}
```

### 5. Implement Testing Utilities

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Helper to create test configuration
    fn create_test_config() -> ProjectConfig {
        ProjectConfig {
            tools: ProjectToolConfig {
                local: vec!["filesystem".to_string(), "git".to_string()],
                remote: vec!["github".to_string(), "kubernetes".to_string()],
                tool_configs: HashMap::new(),
            },
            generated_at: Utc::now(),
            docs_run_id: "test-docs-123".to_string(),
            project_analysis: ProjectAnalysis {
                has_kubernetes: true,
                has_database: false,
                has_ci_cd: true,
                has_terraform: false,
                detected_languages: vec!["rust".to_string()],
                detected_frameworks: vec![],
                file_patterns_found: vec!["kubernetes".to_string()],
                project_size: ProjectSize::Medium,
            },
            version: "1.0.0".to_string(),
            metadata: ConfigMetadata {
                project_id: "test-project".to_string(),
                git_commit: Some("abc123".to_string()),
                git_branch: Some("main".to_string()),
                total_tools_available: 10,
                total_tools_recommended: 4,
                confidence_score: 0.85,
                generation_duration_ms: 250,
            },
        }
    }
    
    #[tokio::test]
    async fn test_configuration_generation() {
        let handler = create_test_handler();
        // Test implementation
    }
    
    #[tokio::test]
    async fn test_storage_and_retrieval() {
        let handler = create_test_handler();
        let config = create_test_config();
        
        // Save
        handler.save_project_config(&config).await.unwrap();
        
        // Verify
        let loaded = handler.load_project_config("test-project").await.unwrap();
        assert_eq!(loaded.metadata.project_id, config.metadata.project_id);
    }
}
```

## Key Implementation Points

1. **Rich Metadata**: Include confidence scores, git info, timing
2. **Tool Configurations**: Provide sensible defaults for each tool
3. **Robust Storage**: Retry logic, verification, error recovery
4. **Human-Readable**: Include summary for easy inspection
5. **Comprehensive Testing**: Unit and integration tests

## Success Checklist

- [ ] Configuration schema includes all required fields
- [ ] Generation logic produces complete configurations
- [ ] Storage uses Kubernetes ConfigMap reliably
- [ ] Retry logic handles transient failures
- [ ] Verification ensures data integrity
- [ ] Integration with docs flow is seamless
- [ ] Error handling is comprehensive
- [ ] Testing covers all scenarios
- [ ] Documentation is complete

Proceed with implementing this comprehensive configuration generation and storage system. Focus on reliability, completeness, and ease of consumption by code agents.