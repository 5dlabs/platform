# Toolman Integration - Architecture & Implementation Plan

## Overview

Enable users to specify which MCP tools Claude should use for specific tasks. This document describes the technical implementation to support interactions like:

> "Hey Claude, start task 6 with local tool filesystem and remote tools Brave, Kubernetes, and Helm"

## Architecture Approach

### Core Concept: Hybrid Tool Architecture
- **Local Tools**: Run directly in agent containers (filesystem, git)
- **Remote Tools**: Accessed via toolman HTTP proxy (web search, Kubernetes, databases, etc.)
- **User Control**: Users specify exactly which tools Claude should have access to

### Key Innovation: ConfigMap as Single Source of Truth

The elegance of this design is that toolman's ConfigMap becomes the authoritative source for available MCP tools:

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐      ┌─────────────────────┐     │
│  │ Toolman ConfigMap   │◄─────│   Docs Agent        │     │
│  │                     │ read │                     │     │
│  │ servers-config.json:│      │ "What tools exist?" │     │
│  │  - github          │      │ "Project needs X"   │     │
│  │  - kubernetes      │      │ "Generate config"   │     │
│  │  - postgres        │      └─────────┬───────────┘     │
│  │  - NEW_TOOL ✨     │                │                   │
│  └──────────┬──────────┘                │ generates        │
│             │                           ▼                   │
│             │ mounted          ┌─────────────────────┐     │
│             ▼                  │   Code Agent        │     │
│  ┌─────────────────────┐      │                     │     │
│  │   Toolman Pod       │      │ "Use this config"   │     │
│  │                     │      │ (No discovery!)     │     │
│  │ "I proxy what's in  │      └─────────────────────┘     │
│  │  my ConfigMap"      │                                   │
│  └─────────────────────┘      ┌─────────────────────┐     │
│                               │   Platform Admin    │     │
│                               │                     │     │
│                               │ kubectl edit cm ... │     │
│                               └─────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### The Critical Flow
1. **Admin** maintains ONE ConfigMap with all available MCP servers
2. **Toolman** reads this ConfigMap and proxies those servers
3. **Docs Agent** reads the SAME ConfigMap to know what's available
4. **Docs Agent** generates optimal tool configuration for the project
5. **Code Agent** just uses the configuration it's given (no discovery!)
6. **Users** can override with specific tools if needed

## CRITICAL: Zero Hardcoding Architecture

### What We're NOT Doing (Ever)

```rust
// ❌ NEVER hardcode tool lists
const GITHUB_TOOLS: &[&str] = &["create_issue", "list_prs"];
const AVAILABLE_SERVERS: &[&str] = &["github", "kubernetes", "postgres"];

// ❌ NEVER embed tool knowledge in code
match tool_name {
    "github" => enable_github_tools(),
    "kubernetes" => enable_k8s_tools(),
    _ => return Err("Unknown tool")
}

// ❌ NEVER maintain static tool mappings
fn get_tool_capabilities() -> HashMap<String, Vec<String>> {
    // NO! This would require code changes for new tools
}
```

### What We ARE Doing (Always)

```rust
// ✅ ALWAYS discover dynamically (in DOCS agent)
let available_tools = read_toolman_configmap().await?;

// ✅ ALWAYS validate against ConfigMap (when user overrides)
if !available_tools.contains(&requested_tool) {
    return Err("Tool not found in toolman config");
}

// ✅ Code agent just uses what it's given
let config = load_generated_config();  // From docs phase
```

## What We're NOT Changing

### Toolman Remains As-Is
- Toolman continues to be a simple HTTP aggregator
- No authentication or filtering logic added
- No architectural changes to toolman
- We use it exactly as designed

### Why This Matters
- Toolman's simplicity is its strength
- Client-side filtering aligns with toolman's philosophy
- No need to maintain a fork or custom version

## Platform Changes (What We're Building)

### 1. Deploy Toolman Service

**File**: `infra/charts/orchestrator/templates/toolman-deployment.yaml`
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: toolman
  namespace: orchestrator
spec:
  replicas: 2
  selector:
    matchLabels:
      app: toolman
  template:
    metadata:
      labels:
        app: toolman
    spec:
      containers:
      - name: toolman
        image: ghcr.io/5dlabs/toolman:latest
        command: ["toolman-http"]
        args: ["--config=/config/servers-config.json"]
        ports:
        - containerPort: 3000
        volumeMounts:
        - name: config
          mountPath: /config
      volumes:
      - name: config
        configMap:
          name: toolman-servers-config
---
apiVersion: v1
kind: Service
metadata:
  name: toolman-service
  namespace: orchestrator
spec:
  selector:
    app: toolman
  ports:
  - port: 3000
    targetPort: 3000
```

**ConfigMap**: `toolman-servers-config`
```json
{
  "servers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {"GITHUB_TOKEN": "${GITHUB_TOKEN}"}
    },
    "kubernetes": {
      "command": "docker",
      "args": ["run", "--rm", "-i", "-v", "/home/appuser/.kube:/home/appuser/.kube:ro", "ginnux/k8s-mcp-server:latest"]
    },
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {"BRAVE_API_KEY": "${BRAVE_API_KEY}"}
    },
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {"POSTGRES_URL": "${POSTGRES_URL}"}
    }
    // Admin adds new servers here - instantly discoverable
    // NO CODE CHANGES NEEDED ANYWHERE!
  }
}
```

### 2. Docs Agent Tool Discovery & Configuration Generation

**Implementation in Docs Handler**:
```rust
// orchestrator/core/src/handlers/docs_handler.rs
impl DocsHandler {
    // DOCS AGENT: Discovers tools and generates configuration
    async fn generate_project_configuration(&self, project_analysis: &ProjectAnalysis) -> Result<ProjectConfig> {
        // Read toolman's ConfigMap - THE ONLY SOURCE OF TRUTH
        let available_tools = self.discover_available_tools().await?;

        // Match project needs with ACTUALLY available tools
        let recommended_tools = self.match_tools_to_project(project_analysis, available_tools);

        // Generate the configuration that CODE AGENT will use
        let config = ProjectConfig {
            remote_tools: recommended_tools,
            local_tools: vec!["filesystem", "git"], // Standard local tools
        };

        // Save this configuration for code agents to use
        self.save_project_config(&config).await?;

        Ok(config)
    }

    async fn discover_available_tools(&self) -> Result<Vec<String>> {
        // Read toolman's ConfigMap
        let configmap = self.k8s_client
            .api::<ConfigMap>()
            .get("toolman-servers-config")
            .await?;

        let servers_json = configmap.data.get("servers-config.json")
            .ok_or("No servers config found")?;

        let servers: Value = serde_json::from_str(servers_json)?;

        // Extract server names dynamically
        let available_tools = servers["servers"]
            .as_object()
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(available_tools)
    }

    fn match_tools_to_project(
        &self,
        analysis: &ProjectAnalysis,
        available_tools: Vec<String>
    ) -> Vec<String> {
        // Pattern-based matching (no hardcoded names!)
        available_tools.into_iter()
            .filter(|tool| {
                match tool.as_str() {
                    name if analysis.files.iter().any(|f| f.contains("postgres"))
                        && name.contains("postgres") => true,
                    name if analysis.has_k8s_files && name.contains("kubernetes") => true,
                    name if analysis.has_github_actions && name.contains("github") => true,
                    // More patterns...
                    _ => false
                }
            })
            .collect()
    }
}
```

### 3. Update CRD Schema

**File**: `orchestrator/core/src/crds/coderun.rs`
```rust
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct CodeRunSpec {
    // Existing fields...

    /// Optional tool specification from user (overrides docs-generated config)
    #[serde(default)]
    pub tools: Option<ToolSpecification>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ToolSpecification {
    /// Local tools to enable (e.g., ["filesystem", "git"])
    #[serde(default)]
    pub local: Vec<String>,

    /// Remote tools to enable (e.g., ["brave-search_brave_web_search", "kubernetes_*"])
    /// These are validated against toolman ConfigMap at runtime
    #[serde(default)]
    pub remote: Vec<String>,
}
```

### 4. Update Templates

**File**: `infra/charts/orchestrator/claude-templates/code/client-config.json.hbs`
```handlebars
{
  {{#if tools}}
  {{!-- User specified exact tools (override) --}}
  "remoteTools": [
    {{#each tools.remote}}
    "{{this}}"{{#unless @last}},{{/unless}}
    {{/each}}
  ],
  "localServers": {
    {{#if (includes tools.local "filesystem")}}
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "tools": ["read_file", "write_file", "list_directory", "create_directory"],
      "workingDirectory": "project_root"
    }{{#if (includes tools.local "git")}},{{/if}}
    {{/if}}
    {{#if (includes tools.local "git")}}
    "git": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"],
      "tools": ["*"],
      "workingDirectory": "project_root"
    }
    {{/if}}
  }
  {{else}}
  {{!-- Use configuration generated by DOCS AGENT --}}
  {{!-- Code agent just consumes this, no discovery! --}}
  {{#if project_config}}
  "remoteTools": [
    {{#each project_config.remote_tools}}
    "{{this}}"{{#unless @last}},{{/unless}}
    {{/each}}
  ],
  "localServers": {
    {{#each project_config.local_tools}}
    {{#if (eq this "filesystem")}}
    "filesystem": { /* ... */ }{{#unless @last}},{{/unless}}
    {{else if (eq this "git")}}
    "git": { /* ... */ }{{#unless @last}},{{/unless}}
    {{/if}}
    {{/each}}
  }
  {{else}}
  {{!-- Minimal fallback --}}
  "remoteTools": [],
  "localServers": {
    "filesystem": { /* basic file access */ }
  }
  {{/if}}
  {{/if}}
}
```

**File**: `infra/charts/orchestrator/claude-templates/code/mcp.json.hbs`
```handlebars
{
  "mcpServers": {
    {{!-- Include toolman if we have remote tools from either source --}}
    {{#if (or tools.remote project_config.remote_tools)}}
    "toolman": {
      "transport": "http",
      "url": "http://toolman-service.orchestrator.svc.cluster.local:3000/mcp"
    }
    {{/if}}
  }
}
```

### 5. Update Code Handler (Simplified!)

**File**: `orchestrator/core/src/handlers/code_handler.rs`
```rust
impl CodeHandler {
    fn prepare_template_context(
        &self,
        spec: &CodeRunSpec,
        context: &mut Context,
    ) -> Result<()> {
        // Existing context setup...

        // Check if user specified tools (override)
        if let Some(ref tools) = spec.tools {
            // Validate against ConfigMap ONLY if user specified tools
            self.validate_user_specified_tools(tools).await?;
            context.insert("tools", tools);
        } else {
            // Use the configuration that DOCS AGENT generated
            // CODE AGENT doesn't discover - it just uses!
            if let Some(config) = self.load_project_config(&spec.service).await? {
                context.insert("project_config", &config);
            }
        }

        Ok(())
    }

    // Only validate when user overrides the docs-generated config
    async fn validate_user_specified_tools(
        &self,
        tools: &ToolSpecification,
    ) -> Result<()> {
        // Read toolman ConfigMap to validate tool names
        let available = self.read_toolman_configmap().await?;

        for tool in &tools.remote {
            // Handle wildcards
            if tool.ends_with("*") {
                let prefix = &tool[..tool.len()-1];
                if !available.iter().any(|t| t.starts_with(prefix)) {
                    return Err(format!("No tools found matching pattern: {}", tool));
                }
            } else if !available.contains(tool) {
                return Err(format!("Unknown tool: {} (not in toolman ConfigMap)", tool));
            }
        }

        Ok(())
    }

    // Simple loader - no discovery!
    async fn load_project_config(&self, service: &str) -> Result<Option<ProjectConfig>> {
        // Load the configuration that docs agent generated
        // This is just reading a file/ConfigMap, not discovering tools
        self.k8s_client
            .api::<ConfigMap>()
            .get(&format!("{}-project-config", service))
            .await
            .ok()
            .and_then(|cm| cm.data.get("config.json"))
            .and_then(|json| serde_json::from_str(json).ok())
    }
}
```

## Implementation Flow

### 1. Initial Setup by Admin
```bash
# Admin configures toolman with available MCP servers
kubectl create configmap toolman-servers-config --from-file=servers-config.json

# Deploy toolman
kubectl apply -f toolman-deployment.yaml
```

### 2. Documentation Phase (Discovery Happens Here!)
```
User: "Generate docs for my project"
     ↓
Docs Agent:
- Analyzes project files
- Reads toolman ConfigMap to see what's available
- Matches project needs to available tools
- Generates optimal configuration
- Saves configuration for code agents
```

### 3. Code Implementation Phase (Just Consumption)
```
User: "Implement task 6"
     ↓
Code Agent:
- Loads the configuration from docs phase
- Uses exactly those tools
- No discovery needed!

OR

User: "Implement task 6 with tools X, Y, Z" (override)
     ↓
Code Agent:
- Validates X, Y, Z exist in toolman ConfigMap
- Uses exactly those tools
- Still no discovery by the agent itself!
```

### 4. Admin Adds New Tool
```bash
# Edit ConfigMap
kubectl edit configmap toolman-servers-config

# Add new server
"new-ai-tool": {
  "command": "npx",
  "args": ["-y", "@company/new-ai-mcp-server"],
  "env": {"API_KEY": "${NEW_AI_API_KEY}"}
}

# Restart toolman
kubectl rollout restart deployment toolman

# Next time docs agent runs, it will see and can recommend the new tool!
```

## Testing Plan

### Docs Agent Discovery Tests
```rust
#[test]
async fn test_docs_agent_discovers_and_configures() {
    // Add tool to ConfigMap
    add_tool_to_configmap("special-db-tool");

    // Create project with files that need this tool
    let project = create_test_project_with_files(vec!["special-db-schema.sql"]);

    // Docs agent should discover and include the tool
    let config = docs_handler.generate_project_configuration(&project).await.unwrap();
    assert!(config.remote_tools.contains(&"special-db-tool".to_string()));
}
```

### Code Agent Consumption Tests
```rust
#[test]
async fn test_code_agent_uses_generated_config() {
    // Setup: Docs agent has already generated config
    let project_config = ProjectConfig {
        remote_tools: vec!["postgres".to_string()],
        local_tools: vec!["filesystem".to_string()],
    };
    save_project_config("test-service", &project_config).await;

    // Code agent just loads and uses it
    let context = code_handler.prepare_template_context(&spec).await.unwrap();
    assert_eq!(context.get("project_config"), Some(&project_config));
    // No discovery calls made by code agent!
}
```

## Documentation for Users

### The Flow
```markdown
## How Tool Configuration Works

### 1. Documentation Phase (Automatic Discovery)
When you run docs generation:
- Docs agent analyzes your project
- Discovers what tools are available in the platform
- Generates optimal tool configuration for your project
- This becomes the default for all code agents

### 2. Code Implementation Phase (Just Works)
When you implement tasks:
- Code agents use the configuration from docs phase
- You get exactly the tools your project needs
- No manual configuration required!

### 3. Manual Override (When Needed)
If you want specific tools:
```
"Implement task 6 with filesystem and kubernetes tools"
```
- Platform validates these tools exist
- Code agent uses exactly what you specified

### Adding New Tools (Admin)
1. Edit ConfigMap: `kubectl edit configmap toolman-servers-config`
2. Add new server configuration
3. Restart toolman: `kubectl rollout restart deployment toolman`
4. Next docs run will automatically discover and can recommend it!
```

## What This Achieves

1. **Single Source of Truth**: One ConfigMap defines all available tools
2. **Smart Discovery**: Docs agent discovers and configures optimally
3. **Simple Consumption**: Code agents just use what they're given
4. **Easy Override**: Users can specify exact tools when needed
5. **Zero Maintenance**: New tools work automatically
6. **Clear Separation**: Discovery (docs) vs Consumption (code)

## What We're NOT Doing

- ❌ Code agents discovering tools
- ❌ Multiple discovery points
- ❌ Hardcoding tool names ANYWHERE
- ❌ Complex tool negotiation
- ❌ Redundant ConfigMap reads

## Conclusion

The flow is simple and elegant:
1. **Toolman ConfigMap** defines what exists
2. **Docs Agent** discovers and configures
3. **Code Agent** just consumes

This separation of concerns makes the system maintainable, efficient, and easy to understand. Code agents don't need to know about tool discovery - they just use what they're configured with!