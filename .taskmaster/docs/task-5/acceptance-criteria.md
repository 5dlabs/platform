# Acceptance Criteria: Task 5 - Implement Docs Agent Tool Discovery

## Overview
This document defines the acceptance criteria for implementing tool discovery functionality in the docs agent, enabling dynamic discovery of available MCP tools and intelligent project-based recommendations.

## Core Requirements

### 1. Tool Discovery
- [ ] **ConfigMap Reading**: Successfully reads toolman-servers-config
- [ ] **JSON Parsing**: Correctly parses servers configuration
- [ ] **Tool Extraction**: Extracts all available tool names
- [ ] **Error Handling**: Gracefully handles missing/invalid ConfigMap
- [ ] **Logging**: Logs discovery process and results

### 2. Project Analysis
- [ ] **File Detection**: Identifies relevant project files
- [ ] **Pattern Matching**: Uses glob patterns effectively
- [ ] **Language Detection**: Recognizes programming languages
- [ ] **Framework Detection**: Identifies frameworks in use
- [ ] **Comprehensive Analysis**: Covers K8s, DB, CI/CD, IaC

### 3. Tool Matching
- [ ] **Pattern-Based**: Uses patterns, not hardcoded names
- [ ] **Contextual**: Matches based on project needs
- [ ] **No Hardcoding**: Zero hardcoded tool names
- [ ] **Deduplication**: Removes duplicate recommendations
- [ ] **Sorting**: Returns sorted tool lists

### 4. Configuration Storage
- [ ] **ConfigMap Creation**: Creates project-specific ConfigMap
- [ ] **JSON Format**: Stores configuration as JSON
- [ ] **Metadata**: Includes timestamps and analysis
- [ ] **Update Support**: Can update existing configs
- [ ] **Error Recovery**: Handles storage failures

## Technical Specifications

### 1. Data Flow
```
ConfigMap Read -> Tool Discovery -> Project Analysis -> Tool Matching -> Config Storage
```

### 2. Expected ConfigMap Format
```json
{
  "servers": {
    "github": { "transport": "stdio", ... },
    "kubernetes": { "transport": "stdio", ... },
    "postgres": { "transport": "stdio", ... }
  }
}
```

### 3. Output Configuration Format
```json
{
  "tools": {
    "local": ["filesystem", "git"],
    "remote": ["github", "kubernetes", "postgres"]
  },
  "generated_at": "2024-01-20T10:30:00Z",
  "project_analysis": {
    "has_kubernetes": true,
    "has_database": true,
    "has_ci_cd": true,
    "detected_languages": ["go", "python"],
    "file_patterns_found": ["kubernetes", "database"]
  },
  "docs_run_id": "docs-run-123"
}
```

## Test Cases

### Test Case 1: ConfigMap Discovery
```rust
#[tokio::test]
async fn test_configmap_discovery() {
    // Setup mock ConfigMap
    let mock_cm = create_mock_configmap(json!({
        "servers": {
            "github": {},
            "kubernetes": {},
            "postgres": {}
        }
    }));
    
    let handler = DocsHandler::new_with_mock(mock_cm);
    let tools = handler.discover_available_tools().await.unwrap();
    
    assert_eq!(tools.len(), 3);
    assert!(tools.contains(&"github".to_string()));
    assert!(tools.contains(&"kubernetes".to_string()));
    assert!(tools.contains(&"postgres".to_string()));
}
```

### Test Case 2: Project Analysis - Kubernetes
```rust
#[tokio::test]
async fn test_kubernetes_detection() {
    let temp_dir = TempDir::new("k8s-project").unwrap();
    
    // Create K8s files
    create_dir_all(temp_dir.path().join("k8s")).unwrap();
    write(
        temp_dir.path().join("k8s/deployment.yaml"),
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: test"
    ).unwrap();
    
    let handler = DocsHandler::new();
    let analysis = handler.analyze_project(temp_dir.path()).await.unwrap();
    
    assert!(analysis.has_kubernetes);
    assert!(analysis.file_patterns_found.contains(&"kubernetes".to_string()));
}
```

### Test Case 3: Pattern-Based Tool Matching
```rust
#[tokio::test]
async fn test_pattern_matching_no_hardcoding() {
    let analysis = ProjectAnalysis {
        has_kubernetes: true,
        has_database: true,
        has_ci_cd: true,
        ..Default::default()
    };
    
    // Tools with various naming patterns
    let available = vec![
        "k8s-manager".to_string(),
        "kubernetes-client".to_string(),
        "postgresql-connector".to_string(),
        "mysql-db".to_string(),
        "github-actions".to_string(),
        "gitlab-runner".to_string(),
        "unmatched-tool".to_string(),
    ];
    
    let handler = DocsHandler::new();
    let config = handler.match_tools_to_project(&analysis, &available);
    
    // Should match based on patterns
    assert!(config.remote.iter().any(|t| t.contains("k8s")));
    assert!(config.remote.iter().any(|t| t.contains("kubernetes")));
    assert!(config.remote.iter().any(|t| t.contains("postgres")));
    assert!(config.remote.iter().any(|t| t.contains("mysql")));
    assert!(config.remote.iter().any(|t| t.contains("github")));
    assert!(config.remote.iter().any(|t| t.contains("gitlab")));
    assert!(!config.remote.contains(&"unmatched-tool".to_string()));
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