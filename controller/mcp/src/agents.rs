use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Agent {
    pub name: String,
    #[serde(rename = "githubApp")]
    pub github_app: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    pub role: String,
    pub model: String,
    pub expertise: Vec<String>,
    pub description: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Defaults {
    #[serde(rename = "docsAgent")]
    pub docs_agent: String,
    #[serde(rename = "codeAgent")]
    pub code_agent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentsConfig {
    pub agents: HashMap<String, Agent>,
    pub defaults: Defaults,
}

impl AgentsConfig {
    /// Load agents configuration from mounted ConfigMap or fallback to defaults
    pub fn load() -> Result<Self> {
        // Try to read from mounted ConfigMap first
        let paths = vec![
            "/agents/agents.yaml",           // Mounted ConfigMap path
            "./controller/config/agents.yaml", // Local development fallback
        ];
        
        for path in paths {
            if let Ok(contents) = fs::read_to_string(path) {
                match serde_yaml::from_str(&contents) {
                    Ok(config) => {
                        eprintln!("✅ Loaded agents config from {}", path);
                        return Ok(config);
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse agents config from {}: {}", path, e);
                    }
                }
            }
        }
        
        eprintln!("⚠️  No agents config found, using defaults");
        Ok(Self::default())
    }
    
    /// Get the default docs agent
    pub fn get_docs_agent(&self) -> Option<&Agent> {
        self.agents.get(&self.defaults.docs_agent)
    }

    
    /// Resolve an agent by human-friendly name, GitHub App name, or key
    /// Examples: "Morgan", "morgan", "5DLabs-Morgan" all resolve to Morgan
    pub fn resolve_agent(&self, input: &str) -> Option<&Agent> {
        let normalized = input.to_lowercase();
        
        // First, try exact key match (case-insensitive)
        if let Some(agent) = self.agents.get(&normalized) {
            return Some(agent);
        }
        
        // Then try to find by agent name (case-insensitive)
        for (_, agent) in &self.agents {
            if agent.name.to_lowercase() == normalized {
                return Some(agent);
            }
        }
        
        // Finally, try to match GitHub App name
        for (_, agent) in &self.agents {
            if agent.github_app.to_lowercase() == normalized {
                return Some(agent);
            }
        }
        
        None
    }
    
    /// Generate a team description for tool descriptions
    pub fn get_team_description(&self) -> String {
        if self.agents.is_empty() {
            return "No agents configured.".to_string();
        }
        
        let mut descriptions = Vec::new();
        for (_key, agent) in &self.agents {
            descriptions.push(format!("{} ({})", agent.name, agent.role));
        }
        
        format!(
            "Available agents: {}. You can assign tasks by saying 'send to {}' or 'get {} to handle this'.",
            descriptions.join(", "),
            self.agents.keys().next().unwrap_or(&"morgan".to_string()),
            self.agents.values().next().map(|a| a.name.as_str()).unwrap_or("Morgan")
        )
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        let mut agents = HashMap::new();
        
        // Default Morgan configuration
        agents.insert("morgan".to_string(), Agent {
            name: "Morgan".to_string(),
            github_app: "5DLabs-Morgan".to_string(),
            app_id: "1723711".to_string(),
            client_id: "Iv23liXbJaNAQELWXIYD".to_string(),
            role: "Product Manager & Documentation Specialist".to_string(),
            model: "claude-opus-4-20250514".to_string(),
            expertise: vec!["documentation".to_string(), "requirements".to_string(), "planning".to_string()],
            description: "AI Documentation Specialist".to_string(),
            system_prompt: "You are Morgan, a meticulous AI Product Manager and Documentation Specialist.".to_string(),
        });
        
        Self {
            agents,
            defaults: Defaults {
                docs_agent: "morgan".to_string(),
                code_agent: "morgan".to_string(), // Temporarily using Morgan until Rex is provisioned
            },
        }
    }
}