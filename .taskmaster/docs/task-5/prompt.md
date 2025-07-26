# Autonomous Agent Prompt: Implement Docs Agent Tool Discovery

## Context
You are implementing the tool discovery functionality for the docs agent. This is a critical component that reads the Toolman ConfigMap to discover available MCP tools and generates optimal tool configurations based on project analysis. This implementation must follow the zero-hardcoding principle - no tool names should be hardcoded.

## Your Mission
Implement the complete tool discovery and recommendation system in the docs agent, ensuring it dynamically discovers tools and makes intelligent recommendations based on project characteristics.

## Key Requirements

### 1. Tool Catalog ConfigMap
- **Create** a new ConfigMap called `toolman-tool-catalog` in the mcp namespace
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
- **Deployment Namespace**: `mcp`
- **Service URL**: `http://toolman.mcp.svc.cluster.local:3000`
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
            "mcp"  // Toolman is deployed in mcp namespace
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
    /// Analyze project to understand tool requirements
    pub async fn analyze_project(&self, project_path: &Path) -> Result<ProjectAnalysis> {
        log::info!("Analyzing project at: {}", project_path.display());

        let mut analysis = ProjectAnalysis::default();

        // Check for Kubernetes files
        if self.check_kubernetes_files(project_path).await? {
            analysis.has_kubernetes = true;
            analysis.file_patterns_found.push("kubernetes".to_string());
        }

        // Check for database files
        if self.check_database_files(project_path).await? {
            analysis.has_database = true;
            analysis.file_patterns_found.push("database".to_string());
        }

        // Check for CI/CD
        if self.check_ci_cd_files(project_path).await? {
            analysis.has_ci_cd = true;
            analysis.file_patterns_found.push("ci/cd".to_string());
        }

        // Check for Terraform
        if self.check_terraform_files(project_path).await? {
            analysis.has_terraform = true;
            analysis.file_patterns_found.push("terraform".to_string());
        }

        // Detect programming languages
        analysis.detected_languages = self.detect_languages(project_path).await?;

        log::info!("Project analysis complete: {:?}", analysis);
        Ok(analysis)
    }

    async fn check_kubernetes_files(&self, project_path: &Path) -> Result<bool> {
        let patterns = vec![
            "**/k8s/**/*.yaml",
            "**/k8s/**/*.yml",
            "**/kubernetes/**/*.yaml",
            "**/kubernetes/**/*.yml",
            "**/helm/**/*",
            "**/*deployment*.yaml",
            "**/*deployment*.yml",
        ];

        for pattern in patterns {
            let full_pattern = project_path.join(pattern);
            for entry in glob::glob(&full_pattern.to_string_lossy())? {
                if let Ok(path) = entry {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if content.contains("apiVersion:") &&
                           (content.contains("kind: Deployment") ||
                            content.contains("kind: Service") ||
                            content.contains("kind: ConfigMap")) {
                            return Ok(true);
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    async fn check_database_files(&self, project_path: &Path) -> Result<bool> {
        let indicators = vec![
            "**/database.yml",
            "**/database.yaml",
            "**/*.sql",
            "**/migrations/**/*",
            "**/db/migrate/**/*",
            "**/alembic/**/*",
            "**/schema.sql",
            "**/schema.rb",
            "**/*postgres*.conf",
            "**/*mysql*.conf",
            "**/*mongo*.conf",
        ];

        for pattern in indicators {
            let full_pattern = project_path.join(pattern);
            if glob::glob(&full_pattern.to_string_lossy())?.next().is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn check_ci_cd_files(&self, project_path: &Path) -> Result<bool> {
        let ci_files = vec![
            ".github/workflows",
            ".gitlab-ci.yml",
            "Jenkinsfile",
            ".circleci/config.yml",
            "azure-pipelines.yml",
            ".travis.yml",
            "bitbucket-pipelines.yml",
        ];

        for file in ci_files {
            if project_path.join(file).exists() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn check_terraform_files(&self, project_path: &Path) -> Result<bool> {
        let patterns = vec!["**/*.tf", "**/*.tfvars", "**/*.hcl"];

        for pattern in patterns {
            let full_pattern = project_path.join(pattern);
            if glob::glob(&full_pattern.to_string_lossy())?.next().is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn detect_languages(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut languages = Vec::new();

        let language_indicators = vec![
            ("package.json", "javascript"),
            ("tsconfig.json", "typescript"),
            ("requirements.txt", "python"),
            ("setup.py", "python"),
            ("Pipfile", "python"),
            ("go.mod", "go"),
            ("Cargo.toml", "rust"),
            ("pom.xml", "java"),
            ("build.gradle", "java"),
            ("*.csproj", "csharp"),
            ("composer.json", "php"),
            ("Gemfile", "ruby"),
        ];

        for (indicator, language) in language_indicators {
            if indicator.contains('*') {
                let pattern = project_path.join(indicator);
                if glob::glob(&pattern.to_string_lossy())?.next().is_some() {
                    languages.push(language.to_string());
                }
            } else if project_path.join(indicator).exists() {
                languages.push(language.to_string());
            }
        }

        languages.sort();
        languages.dedup();
        Ok(languages)
    }
}
```

### 5. Implement Tool Matching Logic
```rust
impl DocsHandler {
    /// Match project needs with available tools using pattern matching
    pub fn match_tools_to_project(
        &self,
        analysis: &ProjectAnalysis,
        available_tools: &[String]
    ) -> ProjectToolConfig {
        let mut config = ProjectToolConfig::default();
        let available_set: HashSet<&str> = available_tools.iter().map(|s| s.as_str()).collect();

        // Local tools
        config.local.push("filesystem".to_string());
        if analysis.has_ci_cd || !analysis.detected_languages.is_empty() {
            config.local.push("git".to_string());
        }

        // Remote tools - use pattern matching, no hardcoding!
        let mut matched_tools = HashSet::new();

        // Kubernetes tools
        if analysis.has_kubernetes {
            for tool in available_tools {
                if tool.contains("kubernetes") ||
                   tool.contains("k8s") ||
                   tool.contains("helm") ||
                   tool.contains("kubectl") {
                    matched_tools.insert(tool.clone());
                }
            }
        }

        // Database tools
        if analysis.has_database {
            for tool in available_tools {
                if tool.contains("postgres") ||
                   tool.contains("mysql") ||
                   tool.contains("mongo") ||
                   tool.contains("redis") ||
                   tool.contains("sqlite") ||
                   tool.contains("database") ||
                   tool.contains("sql") {
                    matched_tools.insert(tool.clone());
                }
            }
        }

        // CI/CD tools
        if analysis.has_ci_cd {
            for tool in available_tools {
                if tool.contains("github") ||
                   tool.contains("gitlab") ||
                   tool.contains("jenkins") ||
                   tool.contains("circleci") ||
                   tool.contains("travis") {
                    matched_tools.insert(tool.clone());
                }
            }
        }

        // Terraform/IaC tools
        if analysis.has_terraform {
            for tool in available_tools {
                if tool.contains("terraform") ||
                   tool.contains("aws") ||
                   tool.contains("azure") ||
                   tool.contains("gcp") {
                    matched_tools.insert(tool.clone());
                }
            }
        }

        // Language-specific tools
        for lang in &analysis.detected_languages {
            for tool in available_tools {
                if tool.to_lowercase().contains(lang) {
                    matched_tools.insert(tool.clone());
                }
            }
        }

        // Universal tools (search, etc)
        for tool in available_tools {
            if tool.contains("search") ||
               tool.contains("brave") ||
               tool.contains("web") {
                matched_tools.insert(tool.clone());
            }
        }

        // Convert to sorted vector
        config.remote = matched_tools.into_iter().collect();
        config.remote.sort();

        log::info!("Matched tools - Local: {:?}, Remote: {:?}",
                   config.local, config.remote);

        config
    }
}
```

### 6. Implement Configuration Storage
```rust
impl DocsHandler {
    /// Save project configuration for code agents
    pub async fn save_project_config(
        &self,
        project_id: &str,
        config: ProjectConfig
    ) -> Result<()> {
        log::info!("Saving project configuration for: {}", project_id);

        let config_json = serde_json::to_string_pretty(&config)?;

        let cm = ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{}-project-config", project_id)),
                namespace: Some(self.namespace.clone()),
                labels: Some([
                    ("app.kubernetes.io/name".to_string(), "project-config".to_string()),
                    ("app.kubernetes.io/instance".to_string(), project_id.to_string()),
                    ("app.kubernetes.io/component".to_string(), "tool-config".to_string()),
                ].into()),
                ..Default::default()
            },
            data: Some([
                ("config.json".to_string(), config_json)
            ].into()),
            ..Default::default()
        };

        let configmaps: Api<ConfigMap> = Api::namespaced(
            self.k8s_client.clone(),
            &self.namespace
        );

        // Try to create, update if exists
        match configmaps.create(&Default::default(), &cm).await {
            Ok(_) => {
                log::info!("Created new project configuration ConfigMap");
            }
            Err(kube::Error::Api(err)) if err.code == 409 => {
                // Already exists, update it
                log::info!("Updating existing project configuration ConfigMap");
                configmaps.replace(
                    &format!("{}-project-config", project_id),
                    &Default::default(),
                    &cm
                ).await?;
            }
            Err(e) => return Err(anyhow!("Failed to save project config: {}", e)),
        }

        Ok(())
    }
}
```

### 7. Main Workflow Implementation
```rust
impl DocsHandler {
    /// Complete tool discovery and configuration workflow
    pub async fn generate_project_configuration(
        &self,
        project_path: &Path,
        project_id: &str,
        docs_run_id: &str
    ) -> Result<ProjectToolConfig> {
        log::info!("Starting tool discovery and configuration for project: {}", project_id);

        // Step 1: Discover available tools
        let available_tools = self.discover_available_tools().await?;
        if available_tools.is_empty() {
            log::warn!("No tools discovered from ConfigMap, using minimal defaults");
            return Ok(ProjectToolConfig {
                local: vec!["filesystem".to_string()],
                remote: vec![],
            });
        }

        // Step 2: Analyze project
        let analysis = self.analyze_project(project_path).await?;

        // Step 3: Match tools to project
        let tool_config = self.match_tools_to_project(&analysis, &available_tools);

        // Step 4: Save configuration
        let project_config = ProjectConfig {
            tools: tool_config.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            project_analysis: analysis,
            docs_run_id: docs_run_id.to_string(),
        };

        self.save_project_config(project_id, project_config).await?;

        log::info!("Tool discovery and configuration complete for project: {}", project_id);
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