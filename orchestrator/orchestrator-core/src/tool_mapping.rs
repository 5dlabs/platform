//! Tool mapping configuration for dynamic tool selection
//!
//! This module provides utilities for mapping tool categories and patterns
//! to specific MCP server tools, enabling easier tool selection via CLI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{anyhow, Result};

/// Tool mapping configuration loaded from tool-mappings.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMappingConfig {
    pub description: String,
    pub version: String,
    pub tool_categories: HashMap<String, ToolCategory>,
    pub preset_configurations: HashMap<String, PresetConfig>,
    pub common_patterns: HashMap<String, CommonPattern>,
    pub tool_compatibility: ToolCompatibility,
}

/// A category of tools (e.g., "web_search", "github")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCategory {
    pub description: String,
    pub tools: Vec<String>,
    pub mcp_servers: Vec<String>,
}

/// A preset configuration (minimal, default, advanced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub description: String,
    pub local_tools: Vec<String>,
    pub remote_tools: Vec<String>,
    pub mcp_servers: Vec<String>,
}

/// Common usage patterns for tool selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonPattern {
    pub description: String,
    pub categories: Vec<String>,
    pub additional_tools: Vec<String>,
}

/// Tool compatibility and dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCompatibility {
    pub description: String,
    pub notes: HashMap<String, String>,
    pub conflicts: Vec<String>,
    pub dependencies: HashMap<String, Vec<String>>,
}

impl ToolMappingConfig {
    /// Load tool mapping configuration from embedded JSON
    pub fn load() -> Result<Self> {
        let config_json = include_str!("../templates/tool-mappings.json");
        let config: ToolMappingConfig = serde_json::from_str(config_json)
            .map_err(|e| anyhow!("Failed to parse tool mapping configuration: {}", e))?;
        Ok(config)
    }

    /// Get preset configuration by name
    pub fn get_preset(&self, preset_name: &str) -> Result<&PresetConfig> {
        self.preset_configurations
            .get(preset_name)
            .ok_or_else(|| anyhow!("Unknown preset configuration: {}", preset_name))
    }

    /// Expand tool categories to individual tool names
    pub fn expand_categories(&self, categories: &[String]) -> Result<Vec<String>> {
        let mut tools = Vec::new();
        for category in categories {
            if let Some(cat) = self.tool_categories.get(category) {
                tools.extend(cat.tools.iter().cloned());
            } else {
                return Err(anyhow!("Unknown tool category: {}", category));
            }
        }
        Ok(tools)
    }

    /// Get required MCP servers for a list of tools
    pub fn get_required_mcp_servers(&self, tools: &[String]) -> Vec<String> {
        let mut servers = std::collections::HashSet::new();
        
        for tool in tools {
            // Find which category this tool belongs to
            for category in self.tool_categories.values() {
                if category.tools.contains(tool) {
                    servers.extend(category.mcp_servers.iter().cloned());
                }
            }
        }
        
        servers.into_iter().collect()
    }

    /// Validate tool selection and provide warnings
    pub fn validate_tools(&self, tools: &[String]) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        for tool in tools {
            // Check if tool exists in any category
            let mut found = false;
            for category in self.tool_categories.values() {
                if category.tools.contains(tool) {
                    found = true;
                    break;
                }
            }
            
            if !found {
                warnings.push(format!("Unknown tool '{tool}' - may not be available"));
            }
            
            // Check dependencies
            for (dep_pattern, deps) in &self.tool_compatibility.dependencies {
                if tool.starts_with(&dep_pattern.replace("*", "")) {
                    warnings.push(format!(
                        "Tool '{}' requires: {}", 
                        tool, 
                        deps.join(", ")
                    ));
                }
            }
        }
        
        Ok(warnings)
    }

    /// Get tools by common pattern
    pub fn get_pattern_tools(&self, pattern_name: &str) -> Result<Vec<String>> {
        let pattern = self.common_patterns
            .get(pattern_name)
            .ok_or_else(|| anyhow!("Unknown pattern: {}", pattern_name))?;
        
        let mut tools = self.expand_categories(&pattern.categories)?;
        tools.extend(pattern.additional_tools.iter().cloned());
        
        Ok(tools)
    }

    /// Generate MCP server list for a given tool configuration
    pub fn generate_mcp_servers(&self, _local_tools: &[String], remote_tools: &[String]) -> Vec<String> {
        let mut servers = std::collections::HashSet::new();
        
        // Add servers for remote tools
        servers.extend(self.get_required_mcp_servers(remote_tools));
        
        // For local tools, we don't need MCP servers (they're built into Claude Code)
        // but we might need to validate they exist
        
        servers.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = ToolMappingConfig::load().unwrap();
        assert!(!config.tool_categories.is_empty());
        assert!(!config.preset_configurations.is_empty());
    }

    #[test]
    fn test_get_preset() {
        let config = ToolMappingConfig::load().unwrap();
        let default_preset = config.get_preset("default").unwrap();
        assert!(!default_preset.local_tools.is_empty());
        assert!(!default_preset.remote_tools.is_empty());
    }

    #[test]
    fn test_expand_categories() {
        let config = ToolMappingConfig::load().unwrap();
        let tools = config.expand_categories(&["web_search".to_string()]).unwrap();
        assert!(tools.contains(&"brave-search_brave_web_search".to_string()));
    }

    #[test]
    fn test_get_required_mcp_servers() {
        let config = ToolMappingConfig::load().unwrap();
        let servers = config.get_required_mcp_servers(&["brave-search_brave_web_search".to_string()]);
        assert!(servers.contains(&"toolman".to_string()));
    }

    #[test]
    fn test_validate_tools() {
        let config = ToolMappingConfig::load().unwrap();
        let warnings = config.validate_tools(&["brave-search_brave_web_search".to_string()]).unwrap();
        // Should have no warnings for valid tools
        assert!(warnings.is_empty());
        
        let warnings = config.validate_tools(&["nonexistent_tool".to_string()]).unwrap();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_get_pattern_tools() {
        let config = ToolMappingConfig::load().unwrap();
        let tools = config.get_pattern_tools("rust_development").unwrap();
        assert!(!tools.is_empty());
        assert!(tools.contains(&"rustdocs_query_rust_docs".to_string()));
    }
}