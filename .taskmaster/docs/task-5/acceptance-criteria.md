# Acceptance Criteria: Task 5 - Implement Docs Agent Tool Discovery

## Overview
This document defines the acceptance criteria for implementing tool discovery functionality in the docs agent, enabling dynamic discovery of available MCP tools and intelligent project-based recommendations.

## Implementation Note (July 26, 2025)
The orchestrator and Toolman components are **COMPLETE** ✅. The remaining work for the docs agent itself (sections 1, 4, 5, 6) runs inside the agent container and is separate from the platform orchestration work.

## Core Requirements

### 1. Tool Discovery
- [x] **ConfigMap Reading**: Successfully reads toolman-servers-config *(via Toolman)*
- [x] **JSON Parsing**: Correctly parses servers configuration *(via Toolman)*
- [x] **Tool Extraction**: Extracts all available tool names *(via Toolman)*
- [x] **Error Handling**: Gracefully handles missing/invalid ConfigMap *(implemented)*
- [x] **Logging**: Logs discovery process and results *(Toolman logs discovery)*

### 2. Tool Catalog ConfigMap
- [x] **Catalog Creation**: Creates `toolman-tool-catalog` ConfigMap in orchestrator namespace
- [x] **Local Tools**: Includes filesystem tool definitions (12 tools)
- [x] **Remote Tools**: Populates with discovered tool information (46 tools across 6 servers)
- [x] **Tool Metadata**: Includes descriptions, categories, and use cases
- [x] **Auto-Update**: Updates catalog on Toolman startup
- [x] **RBAC Permissions**: Has proper permissions to create/update ConfigMap
- [x] **Namespace Auto-Detection**: Dynamically detects namespace instead of hardcoding

### 3. RBAC Configuration
- [x] **Role Created**: Role with ConfigMap read/write permissions
- [x] **RoleBinding Created**: Binds role to Toolman ServiceAccount
- [x] **Helm Chart Updated**: Includes role.yaml and rolebinding.yaml templates
- [x] **Values Updated**: rbac.create flag added to values.yaml
- [x] **Least Privilege**: Only necessary permissions granted
- [x] **Namespace Isolation**: Works within deployed namespace only

### 4. Project Analysis *(Docs Agent - Pending)*
- [ ] **Task Analysis**: Analyzes task description for technology keywords
- [ ] **Pattern Matching**: Uses simple keyword matching effectively
- [ ] **Technology Detection**: Recognizes mentioned technologies (K8s, Terraform, etc.)
- [ ] **Language Detection**: Identifies programming languages mentioned in tasks
- [ ] **Greenfield Focus**: Optimized for new projects without existing code

### 5. Tool Matching *(Docs Agent - Pending)*
- [ ] **Task-Based**: Matches tools based on task requirements, not code scanning
- [ ] **Keyword Matching**: Simple but effective keyword-based matching
- [x] **No Hardcoding**: Zero hardcoded tool names *(achieved in Toolman)*
- [ ] **Always Filesystem**: Always includes filesystem for file operations
- [ ] **Minimal Set**: Returns only necessary tools, not all available

### 6. Configuration Storage *(Docs Agent - Pending)*
- [ ] **Task Folder Storage**: Saves tools.json in task documentation folder
- [ ] **Simple Format**: Minimal JSON with just tools and reasoning
- [ ] **No ConfigMap**: Direct file storage, no Kubernetes ConfigMap needed
- [ ] **Environment Variable**: Code agent gets path via TOOLS_CONFIG env var
- [ ] **Human Editable**: Simple enough for manual editing if needed

## Technical Specifications

### 1. Data Flow
```
ConfigMap Read -> Tool Discovery -> Project Analysis -> Tool Matching -> Config Storage
```

### 2. Tool Catalog Format (Implemented)
```json
{
  "last_updated": "2025-07-26T16:10:00Z",
  "local": {
    // Dynamically discovered from local MCP servers
    "filesystem": { "description": "...", "tools": [...] },
    "git": { "description": "...", "tools": [...] }
  },
  "remote": {
    // Dynamically discovered from Toolman-proxied servers
    "kubernetes": { "description": "...", "endpoint": "stdio", "tools": [...] },
    "postgres": { "description": "...", "endpoint": "stdio", "tools": [...] }
  }
}
```

### 3. Local Tools ConfigMap Format (Implemented)
```json
{
  "servers": {
    "filesystem": {
      "name": "Filesystem",
      "description": "File system operations",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem"],
      "workingDirectory": "project_root"
    },
    "git": {
      "name": "Git",
      "description": "Git version control operations",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"],
      "workingDirectory": "project_root"
    }
  }
}
```

### 3. Output Configuration Format
```json
{
  "tools": {
    "local": ["filesystem"],
    "remote": ["kubernetes", "terraform"]
  },
  "reasoning": "Based on task requirements",
  "generated_at": "2024-01-20T10:30:00Z"
}
```

## Test Cases

### Test Case 1: Task Analysis for Kubernetes
```rust
#[tokio::test]
async fn test_task_analysis_kubernetes() {
    let handler = DocsHandler::new();
    let task_content = "Create a Kubernetes deployment for the API service with 3 replicas";

    let analysis = handler.analyze_task(task_content).await.unwrap();

    assert!(analysis.needs_kubernetes);
    assert!(analysis.technologies_mentioned.contains(&"kubernetes".to_string()));
}
```

### Test Case 2: Task Analysis for Multiple Technologies
```rust
#[tokio::test]
async fn test_task_analysis_multiple() {
    let handler = DocsHandler::new();
    let task_content = "Set up Terraform infrastructure for deploying a Rust microservice to Kubernetes";

    let analysis = handler.analyze_task(task_content).await.unwrap();

    assert!(analysis.needs_kubernetes);
    assert!(analysis.needs_terraform);
    assert!(analysis.languages_mentioned.contains(&"rust".to_string()));
}
```

### Test Case 3: Tool Matching Based on Task
```rust
#[tokio::test]
async fn test_tool_matching_task_driven() {
    let handler = DocsHandler::new();

    let analysis = TaskAnalysis {
        needs_kubernetes: true,
        needs_terraform: false,
        needs_database: false,
        technologies_mentioned: vec!["kubernetes".to_string()],
        languages_mentioned: vec![],
    };

    let available = vec![
        "kubernetes".to_string(),
        "terraform".to_string(),
        "postgres".to_string(),
        "rustdocs".to_string(),
    ];

    let config = handler.match_tools_to_task(&analysis, &available);

    // Should always include filesystem
    assert_eq!(config.local, vec!["filesystem"]);
    // Should only include kubernetes, not other tools
    assert_eq!(config.remote, vec!["kubernetes"]);
}
```

### Test Case 4: Language Detection
```rust
#[tokio::test]
async fn test_language_detection() {
    let temp_dir = TempDir::new("multi-lang").unwrap();

    // Create language indicator files
    write(temp_dir.path().join("package.json"), "{}").unwrap();
    write(temp_dir.path().join("requirements.txt"), "flask==2.0").unwrap();
    write(temp_dir.path().join("go.mod"), "module test").unwrap();

    let handler = DocsHandler::new();
    let analysis = handler.analyze_project(temp_dir.path()).await.unwrap();

    assert!(analysis.detected_languages.contains(&"javascript".to_string()));
    assert!(analysis.detected_languages.contains(&"python".to_string()));
    assert!(analysis.detected_languages.contains(&"go".to_string()));
}
```

### Test Case 5: Configuration Storage
```rust
#[tokio::test]
async fn test_config_storage() {
    let handler = DocsHandler::new();
    let config = ProjectConfig {
        tools: ProjectToolConfig {
            local: vec!["filesystem".to_string()],
            remote: vec!["github".to_string()],
        },
        generated_at: "2024-01-20T10:00:00Z".to_string(),
        project_analysis: Default::default(),
        docs_run_id: "test-123".to_string(),
    };

    // Save config
    handler.save_project_config("test-project", config.clone()).await.unwrap();

    // Verify ConfigMap created
    let cm = get_configmap("test-project-project-config").await.unwrap();
    let stored_json = cm.data.unwrap().get("config.json").unwrap();
    let stored: ProjectConfig = serde_json::from_str(stored_json).unwrap();

    assert_eq!(stored.tools, config.tools);
}
```

### Test Case 6: Error Handling
```rust
#[tokio::test]
async fn test_missing_configmap_handling() {
    let handler = DocsHandler::new_with_missing_cm();

    // Should not panic, return empty list
    let tools = handler.discover_available_tools().await.unwrap();
    assert_eq!(tools.len(), 0);
}

#[tokio::test]
async fn test_invalid_json_handling() {
    let mock_cm = create_mock_configmap_with_invalid_json();
    let handler = DocsHandler::new_with_mock(mock_cm);

    // Should handle gracefully
    let tools = handler.discover_available_tools().await.unwrap();
    assert_eq!(tools.len(), 0);
}
```

### Test Case 7: End-to-End Workflow
```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Setup
    let temp_dir = setup_test_project_with_k8s_and_db().await;
    let handler = setup_handler_with_tools(vec![
        "kubernetes", "postgres", "github"
    ]).await;

    // Execute workflow
    let config = handler.generate_project_configuration(
        temp_dir.path(),
        "test-project",
        "docs-run-456"
    ).await.unwrap();

    // Verify results
    assert!(config.local.contains(&"filesystem".to_string()));
    assert!(config.remote.contains(&"kubernetes".to_string()));
    assert!(config.remote.contains(&"postgres".to_string()));

    // Verify saved configuration
    let saved = get_saved_config("test-project").await.unwrap();
    assert_eq!(saved.docs_run_id, "docs-run-456");
}
```

## Performance Criteria

### 1. Discovery Performance
- [ ] **ConfigMap Read**: < 500ms
- [ ] **JSON Parsing**: < 100ms for typical configs
- [ ] **Tool Extraction**: O(n) complexity

### 2. Analysis Performance
- [ ] **File Scanning**: < 5s for average project
- [ ] **Pattern Matching**: Efficient glob usage
- [ ] **Early Exit**: Stops when patterns found

### 3. Matching Performance
- [ ] **Tool Matching**: O(n*m) where n=tools, m=patterns
- [ ] **Deduplication**: Efficient using HashSet
- [ ] **Memory Usage**: Reasonable for large tool lists

## Security Requirements

### 1. Input Validation
- [ ] **ConfigMap Validation**: Safe JSON parsing
- [ ] **Path Traversal**: Prevented in file analysis
- [ ] **Pattern Safety**: No regex DoS vulnerabilities

### 2. Access Control
- [ ] **RBAC Compliance**: Respects K8s permissions
- [ ] **Namespace Isolation**: Stays within namespace
- [ ] **No Privilege Escalation**: Read-only operations

## Documentation Requirements

### 1. Code Documentation
- [ ] **Function Comments**: All public functions documented
- [ ] **Pattern Explanation**: Document matching logic
- [ ] **Error Handling**: Document failure modes

### 2. Integration Guide
- [ ] **Setup Instructions**: How to deploy
- [ ] **Configuration**: Required RBAC permissions
- [ ] **Troubleshooting**: Common issues and fixes

### 3. API Documentation
- [ ] **Public Interface**: All public methods documented
- [ ] **Data Structures**: Schema definitions
- [ ] **Examples**: Usage examples provided

## Definition of Done

✅ **Discovery Implementation**
- Reads ConfigMap successfully
- Extracts tool names correctly
- Handles errors gracefully
- No hardcoded tool names

✅ **Analysis Implementation**
- Detects all file patterns
- Identifies languages accurately
- Performs comprehensive analysis
- Efficient performance

✅ **Matching Implementation**
- Pattern-based matching works
- No hardcoded tool names
- Contextual recommendations
- Proper deduplication

✅ **Storage Implementation**
- Creates/updates ConfigMaps
- Stores complete metadata
- Handles conflicts properly
- Follows K8s best practices

✅ **Testing Complete**
- Unit tests passing
- Integration tests passing
- Error cases covered
- Performance validated

✅ **Documentation Delivered**
- Code fully documented
- Integration guide complete
- API reference ready
- Examples provided

## Sign-off Requirements

- [ ] **Code Review**: Implementation reviewed and approved
- [ ] **Security Review**: No security vulnerabilities
- [ ] **Performance Review**: Meets performance criteria
- [ ] **Integration Test**: Works with real ConfigMap
- [ ] **Documentation Review**: All docs complete and clear

## Notes
- This is a critical component of the zero-hardcoding architecture
- Tool discovery must remain dynamic and pattern-based
- Consider caching for performance optimization
- Ensure compatibility with future tool additions