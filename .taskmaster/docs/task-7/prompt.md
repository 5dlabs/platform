# Autonomous Agent Prompt: Update Code Agent Configuration Loading

## Context
You are updating the code agent to load tool configurations that were generated and saved by the docs agent. This eliminates the need for code agents to perform tool discovery, implementing a clean separation of concerns where docs agents configure and code agents consume.

## Your Mission
Implement a robust configuration loading system in the code agent that follows clear precedence rules: user overrides > saved configurations > minimal defaults. Ensure graceful handling of all edge cases.

## Implementation Steps

### 1. Set Up Configuration Models

```rust
// In src/models/agent.rs or similar
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Tool configuration for this agent
    pub tools: ProjectToolConfig,
    
    /// Project identifier
    pub project_id: String,
    
    /// Code run identifier
    pub run_id: String,
    
    /// Source of the configuration
    pub config_source: ConfigSource,
    
    /// Additional metadata
    pub metadata: AgentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    /// User explicitly specified tools in CodeRun
    UserOverride {
        specified_at: DateTime<Utc>,
    },
    
    /// Configuration loaded from docs phase
    SavedConfiguration {
        generated_at: DateTime<Utc>,
        docs_run_id: String,
        confidence_score: f32,
    },
    
    /// Fallback minimal defaults
    DefaultFallback {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    pub config_load_time_ms: u64,
    pub warnings: Vec<String>,
    pub tool_count: usize,
}
```

### 2. Implement Configuration Loading

```rust
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{Api, Client};
use anyhow::{anyhow, Result, Context};

pub struct CodeHandler {
    k8s_client: Client,
    namespace: String,
}

impl CodeHandler {
    pub fn new(k8s_client: Client) -> Self {
        Self {
            k8s_client,
            namespace: "orchestrator".to_string(),
        }
    }

    /// Load project configuration from Kubernetes ConfigMap
    pub async fn load_project_config(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectConfig>> {
        let start = std::time::Instant::now();
        let config_name = format!("{}-project-config", project_id);
        
        log::info!("Attempting to load configuration: {}", config_name);
        
        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );
        
        match configmaps.get(&config_name).await {
            Ok(cm) => {
                let load_time = start.elapsed().as_millis();
                log::info!("ConfigMap loaded in {}ms", load_time);
                
                // Extract and parse configuration
                self.parse_config_from_configmap(cm)
            }
            Err(kube::Error::Api(err)) if err.code == 404 => {
                log::info!("No saved configuration found for project: {}", project_id);
                Ok(None)
            }
            Err(e) => {
                log::warn!("Failed to load ConfigMap: {}. Will use defaults.", e);
                Ok(None)
            }
        }
    }
    
    /// Parse configuration from ConfigMap data
    fn parse_config_from_configmap(&self, cm: ConfigMap) -> Result<Option<ProjectConfig>> {
        let data = cm.data.as_ref()
            .ok_or_else(|| anyhow!("ConfigMap has no data"))?;
        
        // Try to get config.json
        let config_json = data.get("config.json")
            .ok_or_else(|| anyhow!("Missing config.json in ConfigMap"))?;
        
        // Parse the configuration
        match serde_json::from_str::<ProjectConfig>(config_json) {
            Ok(config) => {
                // Log configuration details
                log::info!(
                    "Loaded configuration generated at {} by {}",
                    config.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    config.docs_run_id
                );
                
                log::info!(
                    "Configuration contains {} local and {} remote tools (confidence: {:.0}%)",
                    config.tools.local.len(),
                    config.tools.remote.len(),
                    config.metadata.confidence_score * 100.0
                );
                
                // Validate the configuration
                if let Err(e) = self.validate_config(&config) {
                    log::error!("Configuration validation failed: {}", e);
                    Ok(None)
                } else {
                    Ok(Some(config))
                }
            }
            Err(e) => {
                log::error!("Failed to parse configuration JSON: {}", e);
                
                // Try to provide more context
                if let Ok(pretty) = serde_json::from_str::<serde_json::Value>(config_json) {
                    log::debug!("Configuration structure: {:#?}", pretty);
                }
                
                Ok(None)
            }
        }
    }
    
    /// Validate loaded configuration
    fn validate_config(&self, config: &ProjectConfig) -> Result<()> {
        // Check version compatibility
        if !config.version.starts_with("1.") {
            return Err(anyhow!(
                "Incompatible configuration version: {}. Expected 1.x",
                config.version
            ));
        }
        
        // Ensure at least one tool is configured
        if config.tools.local.is_empty() && config.tools.remote.is_empty() {
            return Err(anyhow!("Configuration contains no tools"));
        }
        
        // Check for duplicates
        let mut all_tools = HashSet::new();
        for tool in &config.tools.local {
            if !all_tools.insert(tool.clone()) {
                return Err(anyhow!("Duplicate tool found: {}", tool));
            }
        }
        for tool in &config.tools.remote {
            if !all_tools.insert(tool.clone()) {
                return Err(anyhow!("Duplicate tool found: {}", tool));
            }
        }
        
        // Validate timestamp is reasonable (not future, not too old)
        let now = Utc::now();
        if config.generated_at > now {
            return Err(anyhow!("Configuration has future timestamp"));
        }
        if (now - config.generated_at).num_days() > 30 {
            log::warn!("Configuration is over 30 days old");
        }
        
        Ok(())
    }
}
```

### 3. Implement Agent Initialization Logic

```rust
impl CodeHandler {
    /// Initialize agent with appropriate tool configuration
    pub async fn initialize_agent(
        &self,
        ctx: &CodeContext,
    ) -> Result<AgentConfig> {
        let start = std::time::Instant::now();
        let mut warnings = Vec::new();
        
        log::info!(
            "Initializing code agent for project: {} (run: {})",
            ctx.project_id,
            ctx.run_id
        );
        
        // Determine configuration based on precedence
        let (tools, config_source) = self.determine_configuration(ctx, &mut warnings).await?;
        
        // Calculate metadata
        let tool_count = tools.local.len() + tools.remote.len();
        let config_load_time_ms = start.elapsed().as_millis() as u64;
        
        // Build agent configuration
        let agent_config = AgentConfig {
            tools,
            project_id: ctx.project_id.clone(),
            run_id: ctx.run_id.clone(),
            config_source,
            metadata: AgentMetadata {
                config_load_time_ms,
                warnings,
                tool_count,
            },
        };
        
        // Log configuration summary
        self.log_configuration_summary(&agent_config);
        
        Ok(agent_config)
    }
    
    /// Determine configuration based on precedence rules
    async fn determine_configuration(
        &self,
        ctx: &CodeContext,
        warnings: &mut Vec<String>,
    ) -> Result<(ProjectToolConfig, ConfigSource)> {
        // Priority 1: User override
        if let Some(user_tools) = &ctx.run_spec.tools {
            log::info!("User specified tools explicitly - using override");
            
            // Validate user-specified tools
            if let Err(e) = self.validate_user_tools(user_tools).await {
                warnings.push(format!("Tool validation warning: {}", e));
            }
            
            return Ok((
                user_tools.clone(),
                ConfigSource::UserOverride {
                    specified_at: Utc::now(),
                },
            ));
        }
        
        // Priority 2: Saved configuration
        match self.load_project_config(&ctx.project_id).await {
            Ok(Some(config)) => {
                log::info!("Using saved configuration from docs phase");
                
                return Ok((
                    config.tools.clone(),
                    ConfigSource::SavedConfiguration {
                        generated_at: config.generated_at,
                        docs_run_id: config.docs_run_id,
                        confidence_score: config.metadata.confidence_score,
                    },
                ));
            }
            Ok(None) => {
                log::info!("No saved configuration found");
            }
            Err(e) => {
                warnings.push(format!("Failed to load configuration: {}", e));
            }
        }
        
        // Priority 3: Minimal defaults
        log::warn!(
            "No configuration available for project {} - using minimal defaults",
            ctx.project_id
        );
        warnings.push("Using minimal default configuration".to_string());
        
        Ok((
            self.get_minimal_defaults(),
            ConfigSource::DefaultFallback {
                reason: "No saved configuration and no user override".to_string(),
            },
        ))
    }
    
    /// Get minimal default tool configuration
    fn get_minimal_defaults(&self) -> ProjectToolConfig {
        ProjectToolConfig {
            local: vec!["filesystem".to_string()],
            remote: vec![],
            tool_configs: HashMap::new(),
        }
    }
    
    /// Validate user-specified tools
    async fn validate_user_tools(&self, tools: &ProjectToolConfig) -> Result<()> {
        // Validate local tools against allowed set
        const VALID_LOCAL_TOOLS: &[&str] = &["filesystem", "git"];
        
        for tool in &tools.local {
            if !VALID_LOCAL_TOOLS.contains(&tool.as_str()) {
                return Err(anyhow!("Invalid local tool: {}", tool));
            }
        }
        
        // Remote tools will be validated against toolman ConfigMap
        // This is a placeholder - implement based on task 11
        if !tools.remote.is_empty() {
            log::info!("Remote tools will be validated at runtime");
        }
        
        Ok(())
    }
    
    /// Log configuration summary
    fn log_configuration_summary(&self, config: &AgentConfig) {
        let source_desc = match &config.config_source {
            ConfigSource::UserOverride { specified_at } => {
                format!(
                    "User override (specified at {})",
                    specified_at.format("%H:%M:%S")
                )
            }
            ConfigSource::SavedConfiguration {
                generated_at,
                docs_run_id,
                confidence_score,
            } => {
                format!(
                    "Saved configuration from {} (run: {}, confidence: {:.0}%)",
                    generated_at.format("%Y-%m-%d %H:%M"),
                    docs_run_id,
                    confidence_score * 100.0
                )
            }
            ConfigSource::DefaultFallback { reason } => {
                format!("Default fallback: {}", reason)
            }
        };
        
        log::info!("Configuration source: {}", source_desc);
        log::info!(
            "Tools configured: {} local, {} remote (total: {})",
            config.tools.local.len(),
            config.tools.remote.len(),
            config.metadata.tool_count
        );
        
        if !config.metadata.warnings.is_empty() {
            log::warn!(
                "Configuration warnings: {}",
                config.metadata.warnings.join("; ")
            );
        }
        
        log::debug!("Local tools: {:?}", config.tools.local);
        log::debug!("Remote tools: {:?}", config.tools.remote);
        
        if !config.tools.tool_configs.is_empty() {
            log::debug!(
                "Tool-specific configurations: {}",
                config.tools.tool_configs.keys().cloned().collect::<Vec<_>>().join(", ")
            );
        }
    }
}
```

### 4. Integrate with Code Execution Flow

```rust
impl CodeHandler {
    /// Main entry point for code execution
    pub async fn execute_code_task(
        &self,
        ctx: CodeContext,
    ) -> Result<()> {
        log::info!(
            "Starting code task for project: {} (task: {})",
            ctx.project_id,
            ctx.task_id.as_deref().unwrap_or("none")
        );
        
        // Initialize agent with configuration
        let agent_config = match self.initialize_agent(&ctx).await {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to initialize agent: {}", e);
                return Err(e);
            }
        };
        
        // Record configuration metrics
        self.record_config_metrics(&agent_config);
        
        // Build template context with configuration
        let mut template_context = self.build_base_context(&ctx)?;
        
        // Add tool configuration to context
        template_context.insert("tools", &agent_config.tools);
        template_context.insert("config_source", &agent_config.config_source);
        template_context.insert("has_remote_tools", &!agent_config.tools.remote.is_empty());
        
        // Apply tool-specific configurations if any
        if !agent_config.tools.tool_configs.is_empty() {
            self.apply_tool_configurations(&agent_config.tools.tool_configs);
        }
        
        // Render configurations using templates
        let rendered_configs = self.render_configurations(&template_context).await?;
        
        // Execute with configuration
        self.execute_with_tools(
            &ctx,
            &agent_config,
            &rendered_configs
        ).await?;
        
        log::info!("Code task completed successfully");
        Ok(())
    }
    
    /// Apply tool-specific configurations
    fn apply_tool_configurations(
        &self,
        configs: &HashMap<String, serde_json::Value>,
    ) {
        for (tool, config) in configs {
            log::info!("Applying configuration for tool: {}", tool);
            
            // Handle filesystem read-only mode
            if tool == "filesystem" {
                if let Some(readonly) = config.get("readonly").and_then(|v| v.as_bool()) {
                    if readonly {
                        log::info!("Filesystem tool configured as read-only");
                        // Apply read-only configuration
                    }
                }
            }
            
            // Handle other tool-specific configurations
            log::debug!("Tool {} configuration: {}", tool, config);
        }
    }
    
    /// Record configuration metrics for monitoring
    fn record_config_metrics(&self, config: &AgentConfig) {
        // Log metrics that could be collected by monitoring systems
        log::info!(
            "config_metrics source={:?} load_time_ms={} tool_count={} warnings={}",
            match &config.config_source {
                ConfigSource::UserOverride { .. } => "user_override",
                ConfigSource::SavedConfiguration { .. } => "saved_config",
                ConfigSource::DefaultFallback { .. } => "default_fallback",
            },
            config.metadata.config_load_time_ms,
            config.metadata.tool_count,
            config.metadata.warnings.len()
        );
    }
}
```

### 5. Implement Error Recovery

```rust
impl CodeHandler {
    /// Load configuration with comprehensive error handling
    pub async fn load_config_safe(
        &self,
        project_id: &str,
    ) -> ProjectToolConfig {
        // Try multiple times with exponential backoff
        let mut attempts = 0;
        let max_attempts = 3;
        let mut delay = Duration::from_millis(100);
        
        while attempts < max_attempts {
            attempts += 1;
            
            match self.load_project_config(project_id).await {
                Ok(Some(config)) => {
                    if self.validate_config(&config).is_ok() {
                        return config.tools;
                    }
                    log::warn!("Configuration validation failed, attempt {}", attempts);
                }
                Ok(None) => {
                    log::info!("No configuration found (attempt {})", attempts);
                    break; // No point retrying for 404
                }
                Err(e) => {
                    log::warn!(
                        "Configuration load failed (attempt {}/{}): {}",
                        attempts, max_attempts, e
                    );
                }
            }
            
            if attempts < max_attempts {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
        
        log::warn!("All configuration load attempts failed, using defaults");
        self.get_minimal_defaults()
    }
}
```

### 6. Write Comprehensive Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_configuration_precedence() {
        let handler = create_test_handler();
        
        // Setup: Save a configuration
        let saved_config = create_test_config();
        save_test_configmap(&saved_config).await;
        
        // Test 1: User override takes precedence
        let ctx_override = CodeContext {
            run_spec: CodeRunSpec {
                tools: Some(ProjectToolConfig {
                    local: vec!["filesystem".to_string()],
                    remote: vec!["custom".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..create_base_context()
        };
        
        let config = handler.initialize_agent(&ctx_override).await.unwrap();
        assert!(matches!(config.config_source, ConfigSource::UserOverride { .. }));
        assert_eq!(config.tools.remote, vec!["custom"]);
        
        // Test 2: Saved config used when no override
        let ctx_saved = CodeContext {
            run_spec: CodeRunSpec {
                tools: None,
                ..Default::default()
            },
            ..create_base_context()
        };
        
        let config = handler.initialize_agent(&ctx_saved).await.unwrap();
        assert!(matches!(config.config_source, ConfigSource::SavedConfiguration { .. }));
        
        // Test 3: Defaults when nothing available
        let ctx_default = CodeContext {
            project_id: "no-such-project".to_string(),
            run_spec: CodeRunSpec {
                tools: None,
                ..Default::default()
            },
            ..create_base_context()
        };
        
        let config = handler.initialize_agent(&ctx_default).await.unwrap();
        assert!(matches!(config.config_source, ConfigSource::DefaultFallback { .. }));
        assert_eq!(config.tools.local, vec!["filesystem"]);
    }
    
    #[tokio::test]
    async fn test_configuration_validation() {
        let handler = create_test_handler();
        
        // Test various invalid configurations
        let invalid_configs = vec![
            // No tools
            ProjectConfig {
                tools: ProjectToolConfig {
                    local: vec![],
                    remote: vec![],
                    ..Default::default()
                },
                ..create_base_config()
            },
            // Duplicate tools
            ProjectConfig {
                tools: ProjectToolConfig {
                    local: vec!["filesystem".to_string(), "filesystem".to_string()],
                    remote: vec![],
                    ..Default::default()
                },
                ..create_base_config()
            },
            // Future timestamp
            ProjectConfig {
                generated_at: Utc::now() + chrono::Duration::days(1),
                ..create_base_config()
            },
        ];
        
        for config in invalid_configs {
            assert!(handler.validate_config(&config).is_err());
        }
    }
}
```

## Key Implementation Points

1. **Clear Precedence**: User > Saved > Default
2. **Graceful Degradation**: Never fail completely
3. **Comprehensive Logging**: Track decision process
4. **Performance**: Async operations, minimal overhead
5. **Validation**: Ensure loaded configs are usable
6. **Metrics**: Enable monitoring of config sources

## Success Checklist

- [ ] Loads configurations from ConfigMap
- [ ] Implements proper precedence rules
- [ ] Validates configurations thoroughly
- [ ] Handles all error cases gracefully
- [ ] Logs configuration decisions clearly
- [ ] Maintains backward compatibility
- [ ] Includes comprehensive tests
- [ ] Documents configuration flow
- [ ] Tracks metrics for monitoring

Implement this configuration loading system to complete the tool configuration flow, ensuring code agents can reliably use the intelligent configurations generated by docs agents.